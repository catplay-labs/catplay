use std::{fs, path::Path};

use log::debug;
use rusb::{Context, Device, Direction, TransferType, UsbContext};

use crate::{
    GadgetResult,
    host::{GadgetClient, RusbHotplugWatcher},
};

pub struct GadgetHostHelper {}

pub struct GadgetHostEntry {
    sys_path: String,
    bus_addr: (u8, u8),
}

fn read_u32<P: AsRef<Path>>(path: P) -> Option<u32> {
    fs::read_to_string(path).ok()?.trim().parse().ok()
}

impl GadgetHostHelper {
    const APPLE_VENDOR_ID: u16 = 0x05AC; // Apple Inc.

    pub fn create_hotplug_watcher() -> GadgetResult<RusbHotplugWatcher> {
        Ok(RusbHotplugWatcher::new()?)
    }

    pub fn scan() -> GadgetResult<Vec<GadgetHostEntry>> {
        let mut out = Vec::new();
        let base = Path::new("/sys/bus/usb/devices");

        for entry in fs::read_dir(base)? {
            let entry = entry?;
            let path = entry.path();

            let bus = read_u32(path.join("busnum"));
            let addr = read_u32(path.join("devnum"));

            if let (Some(bus), Some(addr)) = (bus, addr) {
                let sys_path = path.to_string_lossy().into_owned();

                out.push(GadgetHostEntry {
                    sys_path,
                    bus_addr: (bus as u8, addr as u8),
                });
            }
        }

        Ok(out)
    }

    pub fn find_iphones() -> GadgetResult<Vec<GadgetClient>> {
        let context = rusb::Context::new()?;

        let phones: Vec<Device<Context>> = context
            .devices()
            .unwrap()
            .iter()
            .filter(|d| {
                d.device_descriptor()
                    .map(|dd| dd.vendor_id() == Self::APPLE_VENDOR_ID && dd.product_id() != 0x12FF)
                    .unwrap_or(false)
            })
            .collect();

        phones
            .into_iter()
            .map(|p| {
                let desc = p.device_descriptor()?;

                Ok(GadgetClient::Phone {
                    bus_addr: (p.bus_number(), p.address()),
                    usb_device: p,
                    vid_pid: (desc.vendor_id(), desc.product_id()),
                })
            })
            .collect()
    }

    pub fn find_accessories() -> GadgetResult<Vec<GadgetClient>> {
        let context = rusb::Context::new()?;

        let devices = context.devices()?;
        let device_desc = devices
            .iter()
            .filter_map(|device| {
                let desc = device.device_descriptor().ok()?;
                Some((device, desc))
            })
            .flat_map(|(device, desc)| {
                (0..desc.num_configurations()).filter_map(move |i| {
                    device
                        .config_descriptor(i)
                        .ok()
                        .map(|o| (device.clone(), o))
                })
            });

        let udev = Self::scan()?;
        let mut out: Vec<GadgetClient> = Vec::new();

        for (device, config) in device_desc {
            let bus_addr = (device.bus_number(), device.address());
            let already_added = out.iter().any(|g| g.bus_addr() == bus_addr);
            if already_added {
                continue;
            }

            let descriptors = || config.interfaces().flat_map(|iface| iface.descriptors());

            let iap2 = descriptors().find(|d| (d.class_code(), d.sub_class_code(), d.protocol_code()) == (0xFF, 0xF0, 0x00));
            let ncm_control = descriptors()
                .find(|d| (d.class_code(), d.sub_class_code()) == (0x02, 0x0D))
                .map(|n| n.interface_number());
            let ncm_data = descriptors()
                .find(|d| (d.class_code(), d.sub_class_code()) == (0x0A, 0x00))
                .map(|n| n.interface_number());

            let Some(iap2) = iap2 else {
                // debug!("Skipping incomplete USB gadget entry (no iAP2 found)");
                continue;
            };

            let (mut ep_out, mut ep_in) = (None, None);

            for ep in iap2.endpoint_descriptors() {
                match (ep.direction(), ep.transfer_type()) {
                    (Direction::Out, TransferType::Bulk) => ep_out = Some(ep.address()),
                    (Direction::In, TransferType::Bulk) => ep_in = Some(ep.address()),
                    _ => {}
                }
            }

            let syspath = udev
                .iter()
                .find(|u| u.bus_addr == bus_addr)
                .map(|u| u.sys_path.clone());
            let (Some(ep_out), Some(ep_in), Some(syspath)) = (ep_out, ep_in, syspath) else {
                // debug!("Skipping incomplete USB gadget entry");
                continue;
            };

            let ncm = match (ncm_control, ncm_data) {
                (Some(c), Some(d)) => Some((c, d)),
                _ => None,
            };

            let desc = device.device_descriptor()?;

            let gadget = GadgetClient::Accessory {
                bus_addr: (device.bus_number(), device.address()),
                iap2_in_out: (ep_in, ep_out),
                iap2_iface: iap2.interface_number(),
                ncm_iface: ncm,

                usb_configuration: config.number(),
                usb_device: device,
                syspath,

                vid_pid: (desc.vendor_id(), desc.product_id()),
            };

            debug!("Found gadget: {:?}", gadget);
            out.push(gadget);
        }

        Ok(out)
    }
}
