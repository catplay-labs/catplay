#![allow(clippy::useless_conversion)]
use core::mem;
use core::ptr;

const AES_BLOCK: usize = 16;
const AFALG_ZERO_COPY_PAGES: usize = 16;
type RawFd = libc::c_int;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AfAlgError {
    Sys(i32),
    InvalidInput,
    ShortSend,
    ShortRead,
    UnalignedPartialSend,
}

type Result<T> = core::result::Result<T, AfAlgError>;

struct OwnedFd {
    fd: RawFd,
}

impl OwnedFd {
    #[inline]
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        Self { fd }
    }

    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}

impl Drop for OwnedFd {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            libc::close(self.fd);
        }
    }
}

// UAPI constants from linux/if_alg.h.
const SOL_ALG: libc::c_int = 279;
const ALG_SET_KEY: libc::c_int = 1;
const ALG_SET_IV: libc::c_int = 2;
const ALG_SET_OP: libc::c_int = 3;
const ALG_OP_ENCRYPT: u32 = 1;

// struct sockaddr_alg from linux/if_alg.h
#[repr(C)]
struct SockaddrAlg {
    salg_family: libc::sa_family_t,
    salg_type: [u8; 14],
    salg_feat: u32,
    salg_mask: u32,
    salg_name: [u8; 64],
}

// struct af_alg_iv from linux/if_alg.h:
//
// struct af_alg_iv {
//     __u32 ivlen;
//     __u8 iv[0];
// };
#[repr(C)]
struct AfAlgIv {
    ivlen: u32,
}

pub struct AfAlgCtrAes128 {
    _tfm_fd: OwnedFd,
    op_fd: OwnedFd,
    pipe_rd: Option<OwnedFd>,
    pipe_wr: Option<OwnedFd>,
    splice_chunk: usize,
    initial_iv: [u8; 16],
    stream_pos: u128,
    pending_keystream: [u8; AES_BLOCK],
    pending_len: usize,
}

impl AfAlgCtrAes128 {
    pub fn new(key: &[u8; 16], iv: &[u8; 16]) -> Self {
        Self::try_new(key, iv).expect("AF_ALG ctr(aes) init failed")
    }

    pub fn try_new(key: &[u8; 16], iv: &[u8; 16]) -> Result<Self> {
        let tfm_raw = unsafe { libc::socket(libc::AF_ALG, libc::SOCK_SEQPACKET | libc::SOCK_CLOEXEC, 0) };

        if tfm_raw < 0 {
            return Err(last_errno());
        }

        let tfm_fd = unsafe { OwnedFd::from_raw_fd(tfm_raw) };

        let mut sa = SockaddrAlg {
            salg_family: libc::AF_ALG as libc::sa_family_t,
            salg_type: [0u8; 14],
            salg_feat: 0,
            salg_mask: 0,
            salg_name: [0u8; 64],
        };

        copy_cstr_bytes(&mut sa.salg_type, b"skcipher")?;
        copy_cstr_bytes(&mut sa.salg_name, b"ctr(aes)")?;

        let bind_ret = unsafe {
            libc::bind(
                tfm_fd.as_raw_fd(),
                &sa as *const SockaddrAlg as *const libc::sockaddr,
                mem::size_of::<SockaddrAlg>() as libc::socklen_t,
            )
        };

        if bind_ret < 0 {
            return Err(last_errno());
        }

        let setkey_ret = unsafe {
            libc::setsockopt(
                tfm_fd.as_raw_fd(),
                SOL_ALG,
                ALG_SET_KEY,
                key.as_ptr() as *const libc::c_void,
                key.len() as libc::socklen_t,
            )
        };

        if setkey_ret < 0 {
            return Err(last_errno());
        }

        let op_raw = unsafe { libc::accept4(tfm_fd.as_raw_fd(), ptr::null_mut(), ptr::null_mut(), libc::SOCK_CLOEXEC) };

        if op_raw < 0 {
            return Err(last_errno());
        }

        let op_fd = unsafe { OwnedFd::from_raw_fd(op_raw) };
        set_nonblocking(op_fd.as_raw_fd())?;

        let splice_chunk = afalg_zero_copy_chunk_len();
        let (pipe_rd, pipe_wr) = create_splice_pipe();

        Ok(Self {
            _tfm_fd: tfm_fd,
            op_fd,
            pipe_rd,
            pipe_wr,
            splice_chunk,
            initial_iv: *iv,
            stream_pos: 0,
            pending_keystream: [0u8; AES_BLOCK],
            pending_len: 0,
        })
    }

    pub fn apply_keystream(&mut self, data: &mut [u8]) {
        self.try_apply_keystream(data)
            .expect("AF_ALG ctr(aes) encrypt failed");
    }

    pub fn try_apply_keystream(&mut self, data: &mut [u8]) -> Result<()> {
        if data.is_empty() {
            return Ok(());
        }

        let mut offset = 0usize;

        if self.pending_len > 0 {
            let take = self.pending_len.min(data.len());
            xor_with_keystream(&mut data[..take], &self.pending_keystream[..take]);
            offset += take;
            self.stream_pos = self
                .stream_pos
                .checked_add(take as u128)
                .expect("AES-CTR stream position overflow");

            if take < self.pending_len {
                self.pending_keystream
                    .copy_within(take..self.pending_len, 0);
                self.pending_len -= take;
                return Ok(());
            }

            self.pending_len = 0;
        }

        let remaining = &mut data[offset..];
        let full_len = remaining.len() / AES_BLOCK * AES_BLOCK;
        let tail_len = remaining.len() - full_len;

        if full_len > 0 {
            let block_index = self.stream_pos / AES_BLOCK as u128;
            let iv = iv_add_be128(self.initial_iv, block_index);

            self.crypt_in_place(iv, &mut remaining[..full_len])?;
            self.stream_pos = self
                .stream_pos
                .checked_add(full_len as u128)
                .expect("AES-CTR stream position overflow");
        }

        if tail_len > 0 {
            let block_index = self.stream_pos / AES_BLOCK as u128;
            let iv = iv_add_be128(self.initial_iv, block_index);
            let mut output = [0u8; AES_BLOCK];
            let zero_pad = [0u8; AES_BLOCK];
            let tail = &mut remaining[full_len..];

            self.crypt_slices(&iv, &[tail, &zero_pad[..AES_BLOCK - tail_len]], &mut output)?;

            tail.copy_from_slice(&output[..tail_len]);
            self.pending_keystream[..AES_BLOCK - tail_len].copy_from_slice(&output[tail_len..]);
            self.pending_len = AES_BLOCK - tail_len;

            self.stream_pos = self
                .stream_pos
                .checked_add(tail_len as u128)
                .expect("AES-CTR stream position overflow");
        }

        Ok(())
    }

    pub fn crypt_in_place(&mut self, iv: [u8; 16], data: &mut [u8]) -> Result<()> {
        if data.is_empty() {
            return Ok(());
        }

        let mut byte_offset = 0usize;
        let mut remaining = data;

        while !remaining.is_empty() {
            let remaining_before = remaining.len();
            let block_offset = (byte_offset / AES_BLOCK) as u128;
            let chunk_iv = iv_add_be128(iv, block_offset);
            let sent = self.send_once_wait_writable(&chunk_iv, remaining)?;

            let (sent_data, rest) = remaining.split_at_mut(sent);
            read_exact_fd(self.op_fd.as_raw_fd(), sent_data)?;
            byte_offset = byte_offset
                .checked_add(sent)
                .expect("AES-CTR byte offset overflow");
            remaining = rest;

            if sent < remaining_before && !sent.is_multiple_of(AES_BLOCK) {
                let partial_block_offset = byte_offset as u128 / AES_BLOCK as u128;
                let partial_block_iv = iv_add_be128(iv, partial_block_offset);
                let within_block = sent % AES_BLOCK;
                let partial_len = (AES_BLOCK - within_block).min(remaining.len());
                let mut keystream = [0u8; AES_BLOCK];
                let zero_block = [0u8; AES_BLOCK];

                self.crypt_slices(&partial_block_iv, &[&zero_block], &mut keystream)?;
                xor_with_keystream(&mut remaining[..partial_len], &keystream[within_block..within_block + partial_len]);
                byte_offset = byte_offset
                    .checked_add(partial_len)
                    .expect("AES-CTR byte offset overflow");
                remaining = &mut remaining[partial_len..];
            }
        }

        Ok(())
    }

    fn send_once_wait_writable(&mut self, iv: &[u8; 16], data: &[u8]) -> Result<usize> {
        if let (Some(pipe_rd), Some(pipe_wr)) = (&self.pipe_rd, &self.pipe_wr) {
            loop {
                match unsafe {
                    send_afalg_skcipher_request_splice_once(
                        self.op_fd.as_raw_fd(),
                        pipe_rd.as_raw_fd(),
                        pipe_wr.as_raw_fd(),
                        iv,
                        data,
                        self.splice_chunk,
                    )
                } {
                    Err(AfAlgError::Sys(errno)) if would_block(errno) => wait_fd(self.op_fd.as_raw_fd(), FdInterest::Write)?,
                    result => return result,
                }
            }
        }

        loop {
            match unsafe { send_afalg_skcipher_request_vectored_once(self.op_fd.as_raw_fd(), iv, &[data]) } {
                Err(AfAlgError::Sys(errno)) if would_block(errno) => wait_fd(self.op_fd.as_raw_fd(), FdInterest::Write)?,
                result => return result,
            }
        }
    }

    pub fn crypt_slices(&mut self, iv: &[u8; 16], inputs: &[&[u8]], output: &mut [u8]) -> Result<()> {
        let input_len: usize = inputs.iter().map(|input| input.len()).sum();
        assert_eq!(input_len, output.len());

        if input_len == 0 {
            return Ok(());
        }

        unsafe {
            send_afalg_skcipher_request_vectored_exact(self.op_fd.as_raw_fd(), iv, inputs)?;
        }

        read_exact_fd(self.op_fd.as_raw_fd(), output)
    }
}

impl AfAlgCtrAes128 {
    fn _as_raw_fd(&self) -> RawFd {
        self.op_fd.as_raw_fd()
    }

    pub fn stream_pos(&self) -> u128 {
        self.stream_pos
    }
}

pub struct Aes128CtrKernelStream {
    inner: AfAlgCtrAes128,
}

impl Aes128CtrKernelStream {
    pub fn new(key: &[u8; 16], iv: &[u8; 16]) -> Result<Self> {
        Ok(Self {
            inner: AfAlgCtrAes128::try_new(key, iv)?,
        })
    }

    pub fn stream_pos(&self) -> u128 {
        self.inner.stream_pos()
    }

    pub fn apply_keystream(&mut self, data: &mut [u8]) -> Result<()> {
        self.inner.try_apply_keystream(data)
    }

    #[cfg(test)]
    fn set_splice_chunk_for_test(&mut self, splice_chunk: usize) {
        self.inner.splice_chunk = splice_chunk;
    }
}

fn xor_with_keystream(data: &mut [u8], keystream: &[u8]) {
    debug_assert_eq!(data.len(), keystream.len());

    for (dst, ks) in data.iter_mut().zip(keystream.iter()) {
        *dst ^= *ks;
    }
}

fn iv_add_be128(iv: [u8; 16], block_index: u128) -> [u8; 16] {
    u128::from_be_bytes(iv)
        .wrapping_add(block_index)
        .to_be_bytes()
}

fn copy_cstr_bytes(dst: &mut [u8], src: &[u8]) -> Result<()> {
    if src.len() >= dst.len() {
        return Err(AfAlgError::InvalidInput);
    }

    dst[..src.len()].copy_from_slice(src);
    dst[src.len()] = 0;
    Ok(())
}

fn afalg_zero_copy_chunk_len() -> usize {
    let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
    let page_size = if page_size > 0 { page_size as usize } else { 4096 };
    let len = page_size.saturating_mul(AFALG_ZERO_COPY_PAGES);

    (len / AES_BLOCK * AES_BLOCK).max(AES_BLOCK)
}

fn create_splice_pipe() -> (Option<OwnedFd>, Option<OwnedFd>) {
    let mut fds = [0 as RawFd; 2];
    let ret = unsafe { libc::pipe2(fds.as_mut_ptr(), libc::O_CLOEXEC | libc::O_NONBLOCK) };

    if ret < 0 {
        return (None, None);
    }

    unsafe {
        let rd = OwnedFd::from_raw_fd(fds[0]);
        let wr = OwnedFd::from_raw_fd(fds[1]);

        (Some(rd), Some(wr))
    }
}

unsafe fn send_afalg_skcipher_request_vectored_exact(fd: RawFd, iv: &[u8; 16], inputs: &[&[u8]]) -> Result<()> {
    let input_len: usize = inputs.iter().map(|input| input.len()).sum();

    loop {
        match unsafe { send_afalg_skcipher_request_vectored_once(fd, iv, inputs) } {
            Ok(sent) if sent == input_len => return Ok(()),
            Ok(_) => return Err(AfAlgError::ShortSend),
            Err(AfAlgError::Sys(errno)) if would_block(errno) => wait_fd(fd, FdInterest::Write)?,
            Err(err) => return Err(err),
        }
    }
}

unsafe fn send_afalg_skcipher_request_splice_once(
    fd: RawFd,
    pipe_rd: RawFd,
    pipe_wr: RawFd,
    iv: &[u8; 16],
    input: &[u8],
    splice_chunk: usize,
) -> Result<usize> {
    let len = input.len().min(splice_chunk);

    if len == 0 {
        return Err(AfAlgError::ShortSend);
    }

    unsafe {
        send_afalg_skcipher_control(fd, iv, libc::MSG_MORE)?;
    }

    let iov = libc::iovec {
        iov_base: input.as_ptr() as *mut libc::c_void,
        iov_len: len,
    };

    let piped = unsafe { libc::vmsplice(pipe_wr, &iov, 1, libc::SPLICE_F_NONBLOCK) };
    if piped < 0 {
        return Err(last_errno());
    }

    if piped == 0 {
        return Err(AfAlgError::ShortSend);
    }

    let piped = piped as usize;
    let mut drained = 0usize;

    while drained < piped {
        let ret = unsafe {
            libc::splice(
                pipe_rd,
                ptr::null_mut(),
                fd,
                ptr::null_mut(),
                piped - drained,
                libc::SPLICE_F_NONBLOCK,
            )
        };

        if ret < 0 {
            let err = errno();
            if err == libc::EINTR {
                continue;
            }
            if would_block(err) {
                wait_fd(fd, FdInterest::Write)?;
                continue;
            }
            return Err(AfAlgError::Sys(err));
        }

        if ret == 0 {
            return Err(AfAlgError::ShortSend);
        }

        drained += ret as usize;
    }

    Ok(drained)
}

unsafe fn send_afalg_skcipher_control(fd: RawFd, iv: &[u8; 16], flags: libc::c_int) -> Result<()> {
    let op_cmsg_len = mem::size_of::<u32>();
    let iv_payload_len = mem::size_of::<AfAlgIv>() + iv.len();

    const CONTROL_CAPACITY: usize = 128;

    let control_len =
        unsafe { libc::CMSG_SPACE(op_cmsg_len as u32) } as usize + unsafe { libc::CMSG_SPACE(iv_payload_len as u32) } as usize;
    if control_len > CONTROL_CAPACITY {
        return Err(AfAlgError::InvalidInput);
    }

    let mut control = [0u8; CONTROL_CAPACITY];
    let mut msg: libc::msghdr = unsafe { mem::zeroed() };
    msg.msg_control = control.as_mut_ptr() as *mut libc::c_void;
    msg.msg_controllen = control_len
        .try_into()
        .map_err(|_| AfAlgError::InvalidInput)?;

    let cmsg1 = unsafe { libc::CMSG_FIRSTHDR(&msg) };
    if cmsg1.is_null() {
        return Err(AfAlgError::InvalidInput);
    }

    unsafe {
        (*cmsg1).cmsg_level = SOL_ALG;
        (*cmsg1).cmsg_type = ALG_SET_OP;
        (*cmsg1).cmsg_len = libc::CMSG_LEN(op_cmsg_len as u32)
            .try_into()
            .map_err(|_| AfAlgError::InvalidInput)?;
    }

    let op = ALG_OP_ENCRYPT.to_ne_bytes();
    unsafe {
        ptr::copy_nonoverlapping(op.as_ptr(), libc::CMSG_DATA(cmsg1), op.len());
    }

    let cmsg2 = unsafe { libc::CMSG_NXTHDR(&msg, cmsg1) };
    if cmsg2.is_null() {
        return Err(AfAlgError::InvalidInput);
    }

    unsafe {
        (*cmsg2).cmsg_level = SOL_ALG;
        (*cmsg2).cmsg_type = ALG_SET_IV;
        (*cmsg2).cmsg_len = libc::CMSG_LEN(iv_payload_len as u32)
            .try_into()
            .map_err(|_| AfAlgError::InvalidInput)?;
    }

    let iv_data = unsafe { libc::CMSG_DATA(cmsg2) };
    let ivlen = (iv.len() as u32).to_ne_bytes();
    unsafe {
        ptr::copy_nonoverlapping(ivlen.as_ptr(), iv_data, ivlen.len());
        ptr::copy_nonoverlapping(iv.as_ptr(), iv_data.add(mem::size_of::<AfAlgIv>()), iv.len());
    }

    let ret = unsafe { libc::sendmsg(fd, &msg, flags) };
    if ret < 0 {
        return Err(last_errno());
    }

    Ok(())
}

unsafe fn send_afalg_skcipher_request_vectored_once(fd: RawFd, iv: &[u8; 16], inputs: &[&[u8]]) -> Result<usize> {
    let op_cmsg_len = mem::size_of::<u32>();
    let iv_payload_len = mem::size_of::<AfAlgIv>() + iv.len();

    const MAX_IOVEC: usize = 2;
    const CONTROL_CAPACITY: usize = 128;

    debug_assert!(inputs.len() <= MAX_IOVEC);
    if inputs.len() > MAX_IOVEC {
        return Err(AfAlgError::InvalidInput);
    }

    let control_len =
        unsafe { libc::CMSG_SPACE(op_cmsg_len as u32) } as usize + unsafe { libc::CMSG_SPACE(iv_payload_len as u32) } as usize;
    if control_len > CONTROL_CAPACITY {
        return Err(AfAlgError::InvalidInput);
    }

    let mut control = [0u8; CONTROL_CAPACITY];
    let mut iovecs: [libc::iovec; MAX_IOVEC] = unsafe { mem::zeroed() };
    for (slot, input) in iovecs.iter_mut().zip(inputs.iter()) {
        *slot = libc::iovec {
            iov_base: input.as_ptr() as *mut libc::c_void,
            iov_len: input.len(),
        };
    }

    let mut msg: libc::msghdr = unsafe { mem::zeroed() };
    msg.msg_iov = iovecs.as_mut_ptr();
    msg.msg_iovlen = inputs
        .len()
        .try_into()
        .map_err(|_| AfAlgError::InvalidInput)?;
    msg.msg_control = control.as_mut_ptr() as *mut libc::c_void;
    msg.msg_controllen = control_len
        .try_into()
        .map_err(|_| AfAlgError::InvalidInput)?;

    let cmsg1 = unsafe { libc::CMSG_FIRSTHDR(&msg) };
    if cmsg1.is_null() {
        return Err(AfAlgError::InvalidInput);
    }

    unsafe {
        (*cmsg1).cmsg_level = SOL_ALG;
        (*cmsg1).cmsg_type = ALG_SET_OP;
        (*cmsg1).cmsg_len = libc::CMSG_LEN(op_cmsg_len as u32)
            .try_into()
            .map_err(|_| AfAlgError::InvalidInput)?;
    }

    let op = ALG_OP_ENCRYPT.to_ne_bytes();
    unsafe {
        ptr::copy_nonoverlapping(op.as_ptr(), libc::CMSG_DATA(cmsg1), op.len());
    }

    let cmsg2 = unsafe { libc::CMSG_NXTHDR(&msg, cmsg1) };
    if cmsg2.is_null() {
        return Err(AfAlgError::InvalidInput);
    }

    unsafe {
        (*cmsg2).cmsg_level = SOL_ALG;
        (*cmsg2).cmsg_type = ALG_SET_IV;
        (*cmsg2).cmsg_len = libc::CMSG_LEN(iv_payload_len as u32)
            .try_into()
            .map_err(|_| AfAlgError::InvalidInput)?;
    }

    let iv_data = unsafe { libc::CMSG_DATA(cmsg2) };

    let ivlen = (iv.len() as u32).to_ne_bytes();
    unsafe {
        ptr::copy_nonoverlapping(ivlen.as_ptr(), iv_data, ivlen.len());
        ptr::copy_nonoverlapping(iv.as_ptr(), iv_data.add(mem::size_of::<AfAlgIv>()), iv.len());
    }

    let sent = unsafe { libc::sendmsg(fd, &msg, 0) };
    if sent < 0 {
        return Err(last_errno());
    }

    if sent == 0 {
        return Err(AfAlgError::ShortSend);
    }

    Ok(sent as usize)
}

fn read_exact_fd(fd: RawFd, mut output: &mut [u8]) -> Result<()> {
    while !output.is_empty() {
        let ret = unsafe { libc::read(fd, output.as_mut_ptr() as *mut libc::c_void, output.len()) };

        if ret < 0 {
            let err = errno();

            if err == libc::EINTR {
                continue;
            }

            if would_block(err) {
                wait_fd(fd, FdInterest::Read)?;
                continue;
            }

            return Err(AfAlgError::Sys(err));
        }

        if ret == 0 {
            return Err(AfAlgError::ShortRead);
        }

        let n = ret as usize;
        output = &mut output[n..];
    }

    Ok(())
}

fn set_nonblocking(fd: RawFd) -> Result<()> {
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    if flags < 0 {
        return Err(last_errno());
    }

    let ret = unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) };
    if ret < 0 {
        return Err(last_errno());
    }

    Ok(())
}

enum FdInterest {
    Read,
    Write,
}

fn wait_fd(fd: RawFd, interest: FdInterest) -> Result<()> {
    loop {
        let mut readfds = unsafe { mem::zeroed::<libc::fd_set>() };
        let mut writefds = unsafe { mem::zeroed::<libc::fd_set>() };

        let (read_ptr, write_ptr) = unsafe {
            match interest {
                FdInterest::Read => {
                    libc::FD_ZERO(&mut readfds);
                    libc::FD_SET(fd, &mut readfds);
                    (&mut readfds as *mut libc::fd_set, ptr::null_mut())
                }
                FdInterest::Write => {
                    libc::FD_ZERO(&mut writefds);
                    libc::FD_SET(fd, &mut writefds);
                    (ptr::null_mut(), &mut writefds as *mut libc::fd_set)
                }
            }
        };

        let ret = unsafe { libc::select(fd + 1, read_ptr, write_ptr, ptr::null_mut(), ptr::null_mut()) };
        if ret > 0 {
            return Ok(());
        }

        if ret < 0 {
            let err = errno();
            if err == libc::EINTR {
                continue;
            }
            return Err(AfAlgError::Sys(err));
        }
    }
}

#[inline]
fn would_block(errno: i32) -> bool {
    errno == libc::EAGAIN || errno == libc::EWOULDBLOCK
}

#[inline]
fn last_errno() -> AfAlgError {
    AfAlgError::Sys(errno())
}

#[cfg(target_os = "linux")]
#[inline]
fn errno() -> i32 {
    unsafe { *libc::__errno_location() }
}

#[cfg(test)]
mod tests {
    use catplay_tracing::logger::setup_test_logger;

    use crate::aes::Aes128CtrSoft;

    use super::*;

    const KEY: [u8; 16] = [
        0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf, 0x4f, 0x3c,
    ];

    const IV: [u8; 16] = [
        0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe, 0xff,
    ];

    // NIST SP 800-38A F.5.1 AES-128 CTR test vector.
    const PLAINTEXT_64: [u8; 64] = [
        0x6b, 0xc1, 0xbe, 0xe2, 0x2e, 0x40, 0x9f, 0x96, 0xe9, 0x3d, 0x7e, 0x11, 0x73, 0x93, 0x17, 0x2a, 0xae, 0x2d, 0x8a, 0x57, 0x1e, 0x03,
        0xac, 0x9c, 0x9e, 0xb7, 0x6f, 0xac, 0x45, 0xaf, 0x8e, 0x51, 0x30, 0xc8, 0x1c, 0x46, 0xa3, 0x5c, 0xe4, 0x11, 0xe5, 0xfb, 0xc1, 0x19,
        0x1a, 0x0a, 0x52, 0xef, 0xf6, 0x9f, 0x24, 0x45, 0xdf, 0x4f, 0x9b, 0x17, 0xad, 0x2b, 0x41, 0x7b, 0xe6, 0x6c, 0x37, 0x10,
    ];

    const CIPHERTEXT_64: [u8; 64] = [
        0x87, 0x4d, 0x61, 0x91, 0xb6, 0x20, 0xe3, 0x26, 0x1b, 0xef, 0x68, 0x64, 0x99, 0x0d, 0xb6, 0xce, 0x98, 0x06, 0xf6, 0x6b, 0x79, 0x70,
        0xfd, 0xff, 0x86, 0x17, 0x18, 0x7b, 0xb9, 0xff, 0xfd, 0xff, 0x5a, 0xe4, 0xdf, 0x3e, 0xdb, 0xd5, 0xd3, 0x5e, 0x5b, 0x4f, 0x09, 0x02,
        0x0d, 0xb0, 0x3e, 0xab, 0x1e, 0x03, 0x1d, 0xda, 0x2f, 0xbe, 0x03, 0xd1, 0x79, 0x21, 0x70, 0xa0, 0xf3, 0x00, 0x9c, 0xee,
    ];

    #[test]
    fn ctr_matches_nist_vector_small() {
        let key = [
            0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf, 0x4f, 0x3c,
        ];
        let iv = [
            0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe, 0xff,
        ];
        let mut data = [
            0x6b, 0xc1, 0xbe, 0xe2, 0x2e, 0x40, 0x9f, 0x96, 0xe9, 0x3d, 0x7e, 0x11, 0x73, 0x93, 0x17, 0x2a,
        ];
        let expected = [
            0x87, 0x4d, 0x61, 0x91, 0xb6, 0x20, 0xe3, 0x26, 0x1b, 0xef, 0x68, 0x64, 0x99, 0x0d, 0xb6, 0xce,
        ];

        let mut cipher = Aes128CtrKernelStream::new(&key, &iv).expect("AF_ALG ctr(aes) init failed");
        cipher.apply_keystream(&mut data).unwrap();

        assert_eq!(data, expected);
    }

    #[test]
    fn afalg_ctr_matches_nist_vector_one_shot() {
        let mut data = PLAINTEXT_64;

        let mut ctr = Aes128CtrKernelStream::new(&KEY, &IV).expect("AF_ALG ctr(aes) init failed");

        ctr.apply_keystream(&mut data)
            .expect("AF_ALG ctr(aes) encrypt failed");

        assert_eq!(data, CIPHERTEXT_64);
        assert_eq!(ctr.stream_pos(), 64);
    }

    #[test]
    fn afalg_ctr_matches_nist_vector_split_weird_boundaries() {
        let mut data = PLAINTEXT_64;

        let mut ctr = Aes128CtrKernelStream::new(&KEY, &IV).expect("AF_ALG ctr(aes) init failed");

        let splits = [1usize, 3, 17, 5, 2, 19, 7, 10];

        let mut off = 0usize;
        for len in splits {
            ctr.apply_keystream(&mut data[off..off + len])
                .expect("AF_ALG ctr(aes) split encrypt failed");
            off += len;
        }

        assert_eq!(off, 64);
        assert_eq!(data, CIPHERTEXT_64);
        assert_eq!(ctr.stream_pos(), 64);
    }

    #[test]
    fn afalg_ctr_roundtrip_split() {
        let plaintext = *b"catplay-afalg-ctr-continuous-stream-test-12345";
        let mut encrypted = plaintext;

        let mut enc = Aes128CtrKernelStream::new(&[0x11u8; 16], &[0x22u8; 16]).expect("AF_ALG ctr(aes) init failed");

        enc.apply_keystream(&mut encrypted[..7]).unwrap();
        enc.apply_keystream(&mut encrypted[7..23]).unwrap();
        enc.apply_keystream(&mut encrypted[23..]).unwrap();

        assert_ne!(encrypted, plaintext);

        let mut decrypted = encrypted;

        let mut dec = Aes128CtrKernelStream::new(&[0x11u8; 16], &[0x22u8; 16]).expect("AF_ALG ctr(aes) init failed");

        dec.apply_keystream(&mut decrypted[..5]).unwrap();
        dec.apply_keystream(&mut decrypted[5..31]).unwrap();
        dec.apply_keystream(&mut decrypted[31..]).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn afalg_ctr_matches_soft_for_large_chunk() {
        const LARGE_LEN: usize = 1024 * 1024 + 13;

        let mut kernel_data = vec![0u8; LARGE_LEN];
        for (idx, byte) in kernel_data.iter_mut().enumerate() {
            *byte = idx.wrapping_mul(31).wrapping_add(7) as u8;
        }

        let mut soft_data = kernel_data.clone();

        let mut kernel = Aes128CtrKernelStream::new(&KEY, &IV).expect("AF_ALG ctr(aes) init failed");
        let mut soft = Aes128CtrSoft::new(&KEY, &IV);

        kernel
            .apply_keystream(&mut kernel_data)
            .expect("AF_ALG ctr(aes) large encrypt failed");
        soft.apply_keystream(&mut soft_data);

        assert_eq!(kernel_data, soft_data);
        assert_eq!(kernel.stream_pos(), LARGE_LEN as u128);
    }

    #[test]
    fn afalg_ctr_handles_random_streaming_chunks_with_unaligned_partial_sends() {
        let mut kernel = Aes128CtrKernelStream::new(&KEY, &IV).expect("AF_ALG ctr(aes) init failed");
        kernel.set_splice_chunk_for_test(17);

        let mut plain = vec![0u8; 512];
        for (idx, byte) in plain.iter_mut().enumerate() {
            *byte = idx.wrapping_mul(19).wrapping_add(7) as u8;
        }

        let mut expected = plain.clone();
        let mut soft = Aes128CtrSoft::new(&KEY, &IV);
        soft.apply_keystream(&mut expected);

        let mut rng = 0x9e37_79b9_7f4a_7c15u64;
        let mut off = 0usize;
        while off < plain.len() {
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;

            let max_chunk = (rng as usize % 64) + 1;
            let len = max_chunk.min(plain.len() - off);
            kernel
                .apply_keystream(&mut plain[off..off + len])
                .expect("AF_ALG ctr(aes) streaming encrypt failed");
            off += len;
        }

        assert_eq!(plain, expected);
        assert_eq!(kernel.stream_pos(), 512);
    }

    #[test]
    fn afalg_ctr_encrypts_1mib_without_deadlock() {
        setup_test_logger(false);
        const LARGE_LEN: usize = 1024 * 1024;

        let mut data = vec![0x5au8; LARGE_LEN];
        let mut kernel = Aes128CtrKernelStream::new(&KEY, &IV).expect("AF_ALG ctr(aes) init failed");

        kernel
            .apply_keystream(&mut data)
            .expect("AF_ALG ctr(aes) 1MiB encrypt failed");

        assert_eq!(kernel.stream_pos(), LARGE_LEN as u128);
        assert_ne!(data, vec![0x5au8; LARGE_LEN]);

        let mut kernel = Aes128CtrKernelStream::new(&KEY, &IV).expect("AF_ALG ctr(aes) init failed");

        kernel
            .apply_keystream(&mut data)
            .expect("AF_ALG ctr(aes) 1MiB decrypt failed");

        assert_eq!(data, vec![0x5au8; LARGE_LEN]);
    }
}
