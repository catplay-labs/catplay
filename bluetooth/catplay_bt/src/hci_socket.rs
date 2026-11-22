use std::io::{self, Write};
use std::mem;
use std::os::unix::io::RawFd;

use crate::{AF_BLUETOOTH, BTPROTO_HCI, HCI_CHANNEL_CONTROL, MGMT_INDEX_NONE, MgmtEv, MgmtOp, MgmtStatus, SockAddrHci};

fn send_mgmt_packet(fd: RawFd, opcode: u16, index: u16, payload: &[u8]) -> io::Result<()> {
    let mut packet = Vec::with_capacity(6 + payload.len());
    packet.extend_from_slice(&opcode.to_le_bytes());
    packet.extend_from_slice(&index.to_le_bytes());
    packet.extend_from_slice(&(payload.len() as u16).to_le_bytes());
    packet.extend_from_slice(payload);

    let written = unsafe { libc::write(fd, packet.as_ptr() as *const _, packet.len()) };
    if written < 0 {
        return Err(io::Error::last_os_error());
    }

    let mut buf = [0u8; 1024];
    loop {
        let len = unsafe { libc::read(fd, buf.as_mut_ptr() as *mut _, buf.len()) };
        if len <= 0 {
            return Err(io::Error::last_os_error());
        }

        if len < 6 {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "mgmt header too short"));
        }

        let evt_opcode = u16::from_le_bytes([buf[0], buf[1]]);
        let _evt_index = u16::from_le_bytes([buf[2], buf[3]]);
        let evt_len = u16::from_le_bytes([buf[4], buf[5]]) as usize;
        if (len as usize) < 6 + evt_len {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "mgmt payload too short"));
        }

        match MgmtEv::from(evt_opcode) {
            MgmtEv::CmdComplete => {
                if evt_len < 3 {
                    return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "mgmt cmd complete too short"));
                }

                let rsp_opcode = u16::from_le_bytes([buf[6], buf[7]]);
                let status = buf[8];
                if rsp_opcode != opcode {
                    continue;
                }
                if status != 0x00 {
                    return Err(io::Error::other(format!("MGMT failed (status={})", MgmtStatus::from(status))));
                }
                return Ok(());
            }

            MgmtEv::CmdStatus => {
                if evt_len < 3 {
                    return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "mgmt cmd status too short"));
                }

                let rsp_opcode = u16::from_le_bytes([buf[6], buf[7]]);
                let status = buf[8];
                if rsp_opcode != opcode {
                    continue;
                }
                if status == 0x00 {
                    continue;
                }
                return Err(io::Error::other(format!("MGMT failed early (status={})", MgmtStatus::from(status))));
            }
            _ => continue,
        }
    }
}

pub fn uuid_string_to_mgmt_le_bytes(uuid: &str) -> Result<[u8; 16], String> {
    let hex: String = uuid.chars().filter(|c| *c != '-').collect();
    if hex.len() != 32 {
        return Err("UUID must be 32 hex digits".into());
    }

    let mut bytes = [0u8; 16];
    for i in 0..16 {
        bytes[i] = u8::from_str_radix(&hex[2 * i..2 * i + 2], 16).map_err(|e| e.to_string())?;
    }

    bytes.reverse(); // little endian
    Ok(bytes)
}

/// Allows overriding EIR UUIDs and device class at runtime which otherwise - with modern BlueZ - lives in bluetoothd.conf.
pub struct HciSocket {
    fd: RawFd,
    adapter: u16,
}

impl HciSocket {
    /// `adapter` - for hci1, the value is `1`
    pub fn new(adapter: u16) -> io::Result<Self> {
        let fd = unsafe { libc::socket(AF_BLUETOOTH, libc::SOCK_RAW, BTPROTO_HCI) };
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }

        let addr = SockAddrHci {
            family: AF_BLUETOOTH as u16,
            dev: MGMT_INDEX_NONE,
            channel: HCI_CHANNEL_CONTROL,
        };

        let ret = unsafe {
            libc::bind(
                fd,
                &addr as *const SockAddrHci as *const libc::sockaddr,
                mem::size_of::<SockAddrHci>() as u32,
            )
        };

        if ret != 0 {
            let err = io::Error::last_os_error();
            unsafe { libc::close(fd) };
            return Err(err);
        }

        Ok(Self { fd, adapter })
    }

    /// Set HCI class 0x200408 = [0x20, 0x04, 0x00(forced by kernel)]
    pub fn set_class(&self, mut class: [u8; 2]) -> io::Result<()> {
        class.reverse(); // little endian
        send_mgmt_packet(self.fd, MgmtOp::SetDevClass as u16, self.adapter, &class)
    }

    /// Set the adapter's Powered setting.
    pub fn set_powered(&self, powered: bool) -> io::Result<()> {
        send_mgmt_packet(self.fd, MgmtOp::SetPowered as u16, self.adapter, &[powered as u8])
    }

    /// Set BR/EDR Secure Connections support. The kernel only accepts this while unpowered.
    pub fn set_secure_connections(&self, enabled: bool) -> io::Result<()> {
        send_mgmt_packet(self.fd, MgmtOp::SetSecureConn as u16, self.adapter, &[enabled as u8])
    }

    /// Add EIR advertisement UUID
    pub fn add_uuid(&self, uuid: &str) -> io::Result<()> {
        let uuid_le = uuid_string_to_mgmt_le_bytes(uuid).map_err(|e| io::Error::other(format!("invalid uuid: {}", e)))?;
        let mut payload = uuid_le.to_vec();
        payload.push(0x00); // null terminator
        send_mgmt_packet(self.fd, MgmtOp::AddUuid as u16, self.adapter, &payload)
    }
}

impl Drop for HciSocket {
    fn drop(&mut self) {
        unsafe { libc::close(self.fd) };
    }
}
