use log::debug;
use std::fs::{File, OpenOptions};
use std::io;
use std::os::fd::{AsRawFd, RawFd};
use std::sync::Mutex;
use std::thread::sleep;
use std::time::{Duration, Instant};

use crate::{MfiDevice, MfiI2cError, MfiResult};

const I2C_RDWR: libc::Ioctl = 0x0707;
const I2C_M_RD: u16 = 0x0001;
const RETRY_DELAY_US: u64 = 5_000; // 5 ms
const DEADLINE: Duration = Duration::from_secs(2);
// Empirical accommodation for the Mini Ultra's MFi device, not a universal Apple
// timing requirement: same-boot bus 0 / 0x10 certificate-length reads failed 3/3
// with a combined transfer or a split transfer without delay; a 4 ms gap returned
// the expected 908-byte length 3/3 times.
const SELECTOR_READ_DELAY: Duration = Duration::from_millis(4);

/// The firmware generates `[mfi.i2c]` per platform: the Ingenic/Mini Ultra
/// uses bus 0 / address 0x10 while Imx and V821 use bus 1 / address 0x11.
/// Scope the empirical STOP + delay path to the former; other targets keep the
/// upstream combined transfer.
fn split_selector_for(bus_offset: u32, dev_addr: u8) -> bool {
    bus_offset == 0 && dev_addr == 0x10
}

#[repr(C)]
struct I2cMessage {
    addr: u16,
    flags: u16,
    len: u16,
    buf: *mut u8,
}

#[repr(C)]
struct I2cRdwrData {
    msgs: *mut I2cMessage,
    nmsgs: u32,
}

fn checked_i2c_len(len: usize) -> io::Result<u16> {
    u16::try_from(len).map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "I2C message exceeds 65535 bytes"))
}

fn transfer(fd: RawFd, messages: &mut [I2cMessage]) -> io::Result<()> {
    let mut data = I2cRdwrData {
        msgs: messages.as_mut_ptr(),
        nmsgs: u32::try_from(messages.len()).map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "too many I2C messages"))?,
    };

    // SAFETY: data points to messages valid for this call, and every message buffer
    // remains alive and writable as indicated by its flags until ioctl returns.
    let result = unsafe { libc::ioctl(fd, I2C_RDWR, &mut data) };
    if result < 0 {
        return Err(io::Error::last_os_error());
    }
    if result as usize != messages.len() {
        return Err(io::Error::other(format!(
            "incomplete I2C transfer: {result}/{} messages",
            messages.len()
        )));
    }

    Ok(())
}

fn write_transaction(file: &File, dev_addr: u8, data: &[u8]) -> io::Result<()> {
    let mut message = I2cMessage {
        addr: u16::from(dev_addr),
        flags: 0,
        len: checked_i2c_len(data.len())?,
        buf: data.as_ptr().cast_mut(),
    };
    transfer(file.as_raw_fd(), std::slice::from_mut(&mut message))
}

fn write_read_transaction(file: &File, dev_addr: u8, write: &[u8], read: &mut [u8], split: bool) -> io::Result<()> {
    write_read_with(dev_addr, write, read, split, |messages| transfer(file.as_raw_fd(), messages), sleep)
}

fn write_read_with(
    dev_addr: u8,
    write: &[u8],
    read: &mut [u8],
    split: bool,
    mut transfer: impl FnMut(&mut [I2cMessage]) -> io::Result<()>,
    mut sleep: impl FnMut(Duration),
) -> io::Result<()> {
    // Validate both lengths before any bus activity or sleep.
    let mut messages = [
        I2cMessage {
            addr: u16::from(dev_addr),
            flags: 0,
            len: checked_i2c_len(write.len())?,
            buf: write.as_ptr().cast_mut(),
        },
        I2cMessage {
            addr: u16::from(dev_addr),
            flags: I2C_M_RD,
            len: checked_i2c_len(read.len())?,
            buf: read.as_mut_ptr(),
        },
    ];
    if split {
        // Separate single-message ioctls terminate the selector write with STOP.
        // The caller retains the device mutex across both transfers and the gap.
        transfer(&mut messages[..1])?;
        sleep(SELECTOR_READ_DELAY);
        transfer(&mut messages[1..])
    } else {
        transfer(&mut messages)
    }
}

fn read_i2c(i2c: &mut File, dev_addr: u8, addr: u8, n: usize, split: bool) -> MfiResult<Vec<u8>> {
    let deadline = Instant::now() + DEADLINE;
    let mut buf = vec![0u8; n];
    let mut tries = 0;

    debug!("read_i2c 0x{addr:02X} n={n}");

    loop {
        tries += 1;
        match write_read_transaction(i2c, dev_addr, &[addr], &mut buf, split) {
            Ok(_) => {
                debug!("read_i2c 0x{addr:02X} OK after {tries} tries");
                return Ok(buf);
            }
            Err(e) => {
                if Instant::now() >= deadline {
                    debug!("read_i2c 0x{addr:02X} failed with deadline after {tries} tries");
                    return Err(MfiI2cError::ReadTimeout {
                        reg: addr,
                        n,
                        tries,
                        status: e,
                    });
                }
                sleep(Duration::from_micros(RETRY_DELAY_US));
            }
        }
    }
}

fn write_i2c(i2c: &mut File, dev_addr: u8, addr: u8, data: &[u8]) -> MfiResult<()> {
    let mut tmp = Vec::with_capacity(1 + data.len());
    tmp.push(addr);
    tmp.extend_from_slice(data);

    let deadline = Instant::now() + DEADLINE;
    let mut tries = 0;
    loop {
        tries += 1;
        match write_transaction(i2c, dev_addr, &tmp) {
            Ok(_) => {
                debug!("write_i2c 0x{addr:02X} OK after {tries} tries");
                return Ok(());
            }
            Err(e) => {
                if Instant::now() >= deadline {
                    debug!("write_i2c 0x{addr:02X} failed with deadline after {tries} tries");
                    return Err(MfiI2cError::WriteTimeout {
                        reg: addr,
                        n: data.len(),
                        tries,
                        status: e,
                    });
                }
                sleep(Duration::from_micros(RETRY_DELAY_US));
            }
        }
    }
}

pub struct MfiDeviceI2C {
    device: Mutex<File>,
    // bus_offset: u32,
    dev_addr: u8,
    split_selector: bool,

    certificate: Mutex<Vec<u8>>,
}

impl MfiDeviceI2C {
    pub fn new(bus_offset: u32, dev_addr: u8) -> MfiResult<Self> {
        let i2c_device = OpenOptions::new().read(true).write(true).open(format!("/dev/i2c-{bus_offset}"))?;
        let s = Self {
            device: Mutex::new(i2c_device),
            // bus_offset,
            dev_addr,
            split_selector: split_selector_for(bus_offset, dev_addr),
            certificate: Mutex::new(Vec::new()),
        };

        // s.certificate = s.i2c_read_certificate()?;
        Ok(s)
    }

    pub fn i2c_read_certificate(&self) -> MfiResult<Vec<u8>> {
        let mut device = self.device.lock().unwrap();

        let mut cached_cert = self.certificate.lock().unwrap();
        if !cached_cert.is_empty() {
            return Ok(cached_cert.clone());
        }

        let mut retries = 0;
        let mut size;

        loop {
            let len_bytes = read_i2c(&mut device, self.dev_addr, 0x30, 2, self.split_selector)?;
            size = u16::from_be_bytes([len_bytes[0], len_bytes[1]]) as usize;
            retries += 1;

            if !(64..=1024).contains(&size) && retries < 100 {
                debug!("Certificate size is garbage {}, retry", size);
                sleep(Duration::from_millis(5));
                continue;
            }

            break;
        }

        debug!("Certificate size is {}", size);

        if !(64..=1024).contains(&size) {
            return Err(MfiI2cError::UnexpectedSize(size));
        }

        let cert = read_i2c(&mut device, self.dev_addr, 0x31, size, self.split_selector)?;
        *cached_cert = cert.clone();
        Ok(cert)
    }

    pub fn i2c_generate_challenge_response(&self, challenge: &[u8]) -> MfiResult<Vec<u8>> {
        let mut device = self.device.lock().unwrap();
        debug!("Challenge buf size: {}", challenge.len());

        // let mut buf = Vec::with_capacity(2 + challenge.len());
        // buf.extend_from_slice(&u16::to_be_bytes(challenge.len() as u16));
        // buf.extend_from_slice(challenge);

        // write_i2c(&mut device, self.dev_addr, 0x20, &buf)?;
        write_i2c(&mut device, self.dev_addr, 0x20, &u16::to_be_bytes(challenge.len() as u16))?;
        write_i2c(&mut device, self.dev_addr, 0x21, challenge)?;

        // write_i2c(&mut device, self.dev_addr, 0x11, &u16::to_be_bytes(0x80))?;
        write_i2c(&mut device, self.dev_addr, 0x10, &[0x01])?;

        sleep(Duration::from_millis(10));

        let status = read_i2c(&mut device, self.dev_addr, 0x10, 1, self.split_selector)?;
        debug!("status {}", status[0]);
        if (status[0] & 0x80) != 0 {
            let err_code = read_i2c(&mut device, self.dev_addr, 0x05, 1, self.split_selector)?[0];
            return Err(MfiI2cError::SigningError {
                status: status[0],
                code: err_code,
            });
        }

        let len_bytes = read_i2c(&mut device, self.dev_addr, 0x11, 2, self.split_selector)?;
        let size = u16::from_be_bytes([len_bytes[0], len_bytes[1]]) as usize;
        debug!("Challenge response size is {}", size);

        if size == 0 || size > 0x80 {
            return Err(MfiI2cError::UnexpectedSize(size));
        }

        read_i2c(&mut device, self.dev_addr, 0x12, size, self.split_selector)
    }
}

impl MfiDevice for MfiDeviceI2C {
    fn read_certificate(&self) -> MfiResult<Vec<u8>> {
        self.i2c_read_certificate()
    }

    fn generate_challenge_response(&self, challenge: &[u8]) -> MfiResult<Vec<u8>> {
        self.i2c_generate_challenge_response(challenge)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    #[test]
    fn selector_stop_delay_then_read() {
        let events = RefCell::new(Vec::new());
        let selector = [0x30];
        let mut read = [0; 2];
        let read_ptr = read.as_mut_ptr();
        write_read_with(
            0x10,
            &selector,
            &mut read,
            true,
            |messages| {
                assert_eq!(messages.len(), 1, "each ioctl must contain exactly one message");
                let message = &messages[0];
                assert_eq!(message.addr, 0x10);
                if events.borrow().is_empty() {
                    assert_eq!(message.flags, 0);
                    assert_eq!(message.len, 1);
                    assert_eq!(message.buf, selector.as_ptr().cast_mut());
                    events.borrow_mut().push("selector STOP");
                } else {
                    assert_eq!(&*events.borrow(), &["selector STOP", "4 ms"]);
                    assert_eq!(message.flags, I2C_M_RD);
                    assert_eq!(message.len, 2);
                    assert_eq!(message.buf, read_ptr);
                    // SAFETY: the helper supplies the live, writable two-byte read buffer.
                    unsafe { std::ptr::copy_nonoverlapping([0x03, 0x8c].as_ptr(), message.buf, 2) };
                    events.borrow_mut().push("read");
                }
                Ok(())
            },
            |delay| {
                assert_eq!(delay, Duration::from_millis(4));
                assert_eq!(&*events.borrow(), &["selector STOP"]);
                events.borrow_mut().push("4 ms");
            },
        )
        .unwrap();
        assert_eq!(events.into_inner(), ["selector STOP", "4 ms", "read"]);
        assert_eq!(u16::from_be_bytes(read), 908);
    }

    #[test]
    fn selector_failure_skips_sleep_and_read() {
        let mut calls = 0;
        let error = write_read_with(
            0x10,
            &[0x30],
            &mut [0; 2],
            true,
            |messages| {
                calls += 1;
                assert_eq!(messages.len(), 1);
                assert_eq!(messages[0].flags, 0);
                Err(io::Error::from_raw_os_error(libc::EIO))
            },
            |_| panic!("must not sleep after selector failure"),
        )
        .unwrap_err();
        assert_eq!(calls, 1);
        assert_eq!(error.raw_os_error(), Some(libc::EIO));
    }

    #[test]
    fn read_failure_is_propagated() {
        let mut calls = 0;
        let mut sleeps = 0;
        let error = write_read_with(
            0x10,
            &[0x30],
            &mut [0; 2],
            true,
            |messages| {
                calls += 1;
                assert_eq!(messages.len(), 1);
                match calls {
                    1 => Ok(()),
                    2 => {
                        assert_eq!(messages[0].flags, I2C_M_RD);
                        Err(io::Error::from_raw_os_error(libc::EIO))
                    }
                    _ => panic!("unexpected transfer"),
                }
            },
            |delay| {
                assert_eq!(delay, Duration::from_millis(4));
                sleeps += 1;
            },
        )
        .unwrap_err();
        assert_eq!((calls, sleeps), (2, 1));
        assert_eq!(error.raw_os_error(), Some(libc::EIO));
    }

    #[test]
    fn oversized_lengths_fail_before_side_effects() {
        for (write_len, read_len) in [(65536, 2), (1, 65536)] {
            let error = write_read_with(
                0x10,
                &vec![0; write_len],
                &mut vec![0; read_len],
                true,
                |_| panic!("must validate both lengths before transfer"),
                |_| panic!("must validate both lengths before sleep"),
            )
            .unwrap_err();
            assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
        }
    }

    #[test]
    fn selector_delay_is_scoped_to_ingenic_config() {
        assert!(split_selector_for(0, 0x10));
        assert!(!split_selector_for(1, 0x11));
        assert!(!split_selector_for(0, 0x11));
        assert!(!split_selector_for(1, 0x10));
    }

    #[test]
    fn combined_transfer_keeps_upstream_behavior() {
        let mut calls = 0;
        let mut sleeps = 0;
        write_read_with(
            0x10,
            &[0x30],
            &mut [0; 2],
            false,
            |messages| {
                calls += 1;
                assert_eq!(messages.len(), 2, "combined mode is one two-message ioctl");
                Ok(())
            },
            |_| sleeps += 1,
        )
        .unwrap();
        assert_eq!((calls, sleeps), (1, 0));
    }
}
