use std::{path::Path, time::Duration};

use log::{debug, trace};
use rusb::{Context, Device, DeviceHandle, Direction, Recipient, RequestType};
use std::io;

use crate::{GadgetError, GadgetResult};

#[derive(Debug, PartialEq, Eq)]
pub enum GadgetClient {
    Phone {
        bus_addr: (u8, u8),
        usb_device: Device<Context>,
        vid_pid: (u16, u16),
    },
    Accessory {
        bus_addr: (u8, u8),
        usb_device: Device<Context>,
        vid_pid: (u16, u16),

        iap2_in_out: (u8, u8),
        iap2_iface: u8,
        ncm_iface: Option<(u8, u8)>,
        usb_configuration: u8,
        syspath: String,
    },
}

impl GadgetClient {
    const ROLE_SWITCH_REQUEST: u8 = 0x51;
    const POWER_CAPABILITY_REQUEST: u8 = 0x40;
    const CAPABILITIES_REQUEST: u8 = 0x53;

    pub fn is_phone(&self) -> bool {
        matches!(self, GadgetClient::Phone { .. })
    }

    pub fn bus_addr(&self) -> (u8, u8) {
        match self {
            GadgetClient::Phone { bus_addr, .. } => *bus_addr,
            GadgetClient::Accessory { bus_addr, .. } => *bus_addr,
        }
    }

    pub fn device(&self) -> &Device<Context> {
        match self {
            GadgetClient::Phone { usb_device, .. } => usb_device,
            GadgetClient::Accessory { usb_device, .. } => usb_device,
        }
    }

    /// The /sys device path.
    pub fn syspath(&self) -> GadgetResult<&Path> {
        match self {
            GadgetClient::Phone { .. } => Err(GadgetError::Unsupported),
            GadgetClient::Accessory { syspath, .. } => Ok(Path::new(syspath)),
        }
    }

    /// The filename part of /sys device entry.
    pub fn devname(&self) -> GadgetResult<&str> {
        let path = self.syspath()?;
        Ok(path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| io::Error::other("invalid /sys path filename"))?)
    }

    pub fn is_active(&self) -> bool {
        self.open().is_ok()
    }

    pub fn open(&self) -> GadgetResult<DeviceHandle<Context>> {
        Ok(self
            .device()
            .open()
            .inspect_err(|e| trace!("Gadget USB device has disappeared: {e:?}"))?)
    }

    pub fn has_ncm(&self) -> bool {
        match self {
            GadgetClient::Accessory { ncm_iface, .. } => ncm_iface.is_some(),
            _ => false,
        }
    }

    pub fn role_switch(&self, is_carplay: bool, timeout: Duration) -> GadgetResult<usize> {
        debug!("Sending role switch");
        let request_type: u8 = rusb::request_type(Direction::Out, RequestType::Vendor, Recipient::Device);
        let handle = self.open()?;
        // handle.set_active_configuration(4)?;

        Ok(handle.write_control(
            request_type,
            Self::ROLE_SWITCH_REQUEST,
            if is_carplay { 1 } else { 0 },
            0,
            &[],
            timeout,
        )?)
    }

    pub fn offer_power_capability(&self, power_ma: u16, timeout: Duration) -> GadgetResult<usize> {
        debug!("Sending power capability: {power_ma}");
        let request_type: u8 = rusb::request_type(Direction::Out, RequestType::Vendor, Recipient::Device);
        let handle = self.open()?;

        Ok(handle.write_control(
            request_type,
            Self::POWER_CAPABILITY_REQUEST,
            power_ma.saturating_sub(500),
            power_ma.saturating_sub(500),
            &[],
            timeout,
        )?)
    }

    pub fn get_capabilities(&self, timeout: Duration) -> GadgetResult<[u8; 4]> {
        debug!("Sending capabilities request");
        let request_type: u8 = rusb::request_type(Direction::In, RequestType::Vendor, Recipient::Device);

        let handle = self.open()?;
        let mut buf = [0u8; 4];

        let len = handle.read_control(
            request_type,
            Self::CAPABILITIES_REQUEST,
            0, // wValue
            0, // wIndex
            &mut buf,
            timeout,
        )?;

        if len != buf.len() {
            return Err(format!("expected 4 bytes of capabilities, got {len}").into());
        }

        Ok(buf)
    }

    pub fn query_serial(&self) -> GadgetResult<Option<String>> {
        let device = self.open()?;
        let desc = device.device().device_descriptor()?;

        let Some(sn_index) = desc.serial_number_string_index() else {
            return Ok(None);
        };

        Ok(Some(device.read_string_descriptor_ascii(sn_index)?))
    }

    pub fn query_product(&self) -> GadgetResult<Option<String>> {
        let device = self.open()?;
        let desc = device.device().device_descriptor()?;

        let Some(sn_index) = desc.product_string_index() else {
            return Ok(None);
        };

        Ok(Some(device.read_string_descriptor_ascii(sn_index)?))
    }

    pub fn query_manufacturer(&self) -> GadgetResult<Option<String>> {
        let device = self.open()?;
        let desc = device.device().device_descriptor()?;

        let Some(sn_index) = desc.manufacturer_string_index() else {
            return Ok(None);
        };

        Ok(Some(device.read_string_descriptor_ascii(sn_index)?))
    }

    pub fn vid_pid(&self) -> (u16, u16) {
        match self {
            GadgetClient::Phone { vid_pid, .. } => *vid_pid,
            GadgetClient::Accessory { vid_pid, .. } => *vid_pid,
        }
    }
}
