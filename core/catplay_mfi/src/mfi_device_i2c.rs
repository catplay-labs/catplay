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

fn read_transaction(file: &File, dev_addr: u8, read: &mut [u8]) -> io::Result<()> {
    let mut message = I2cMessage {
        addr: u16::from(dev_addr),
        flags: I2C_M_RD,
        len: checked_i2c_len(read.len())?,
        buf: read.as_mut_ptr(),
    };
    transfer(file.as_raw_fd(), std::slice::from_mut(&mut message))
}

fn read_i2c(i2c: &mut File, dev_addr: u8, addr: u8, n: usize) -> MfiResult<Vec<u8>> {
    let deadline = Instant::now() + DEADLINE;
    let mut buf = vec![0u8; n];
    let mut tries = 0;

    debug!("read_i2c 0x{addr:02X} n={n}");

    /* Select the register once. If the following read NACKs because the
     * coprocessor is not ready yet, retry only the read. */
    loop {
        tries += 1;
        match write_transaction(i2c, dev_addr, &[addr]) {
            Ok(_) => break,
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

    tries = 0;
    loop {
        tries += 1;
        match read_transaction(i2c, dev_addr, &mut buf) {
            Ok(_) => {
                debug!("read_i2c 0x{addr:02X} OK after {tries} read tries");
                return Ok(buf);
            }
            Err(e) => {
                if Instant::now() >= deadline {
                    debug!("read_i2c 0x{addr:02X} failed with deadline after {tries} read tries");
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

    certificate: Mutex<Vec<u8>>,
}

impl MfiDeviceI2C {
    pub fn new(bus_offset: u32, dev_addr: u8) -> MfiResult<Self> {
        let i2c_device = OpenOptions::new()
            .read(true)
            .write(true)
            .open(format!("/dev/i2c-{bus_offset}"))?;
        let s = Self {
            device: Mutex::new(i2c_device),
            // bus_offset,
            dev_addr,
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
            let len_bytes = read_i2c(&mut device, self.dev_addr, 0x30, 2)?;
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

        let cert = read_i2c(&mut device, self.dev_addr, 0x31, size)?;
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

        let status = read_i2c(&mut device, self.dev_addr, 0x10, 1)?;
        debug!("status {}", status[0]);
        if (status[0] & 0x80) != 0 {
            let err_code = read_i2c(&mut device, self.dev_addr, 0x05, 1)?[0];
            return Err(MfiI2cError::SigningError {
                status: status[0],
                code: err_code,
            });
        }

        let len_bytes = read_i2c(&mut device, self.dev_addr, 0x11, 2)?;
        let size = u16::from_be_bytes([len_bytes[0], len_bytes[1]]) as usize;
        debug!("Challenge response size is {}", size);

        if size == 0 || size > 0x80 {
            return Err(MfiI2cError::UnexpectedSize(size));
        }

        read_i2c(&mut device, self.dev_addr, 0x12, size)
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
