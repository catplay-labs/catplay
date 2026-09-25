use std::{
    ffi::CString,
    fs::{self},
    io::{self, ErrorKind},
    mem::{self, zeroed},
    net::Ipv6Addr,
    os::fd::{FromRawFd, OwnedFd},
    path::Path,
    process::Command,
    thread::sleep,
    time::Duration,
};

use log::debug;
use macaddr::MacAddr6;

use crate::{GadgetError, GadgetResult};

pub struct NcmHelper(());

fn retry<T, E, F: Fn() -> Result<T, E>>(interval: Duration, retries: usize, callback: F) -> Result<T, E> {
    let mut tried = 0;
    let mut ret = callback();

    while ret.is_err() && tried < retries {
        tried += 1;
        ret = callback();
        sleep(interval);
    }

    ret
}

impl NcmHelper {
    pub const CDC_NCM_RETRY_INTERVAL: Duration = Duration::from_millis(10);
    pub const CDC_NCM_RETRIES: usize = 100;

    pub const LINK_LOCAL_IP_PHONE: &str = "fe80::1234:5678:9abc:def0/64";
    pub const LINK_LOCAL_IP_CAR: &str = "fe80::1234:5678:9abc:def1/64";

    pub const LINK_LOCAL_IP_PHONE_RAW: &str = "fe80::1234:5678:9abc:def0";
    pub const LINK_LOCAL_IP_CAR_RAW: &str = "fe80::1234:5678:9abc:def1";

    pub fn release_from_network_manager(interface: &str) -> GadgetResult<()> {
        let status = match Command::new("nmcli")
            .args(["device", "set", interface, "managed", "no"])
            .status()
        {
            Ok(status) => status,
            Err(err) if err.kind() == ErrorKind::NotFound => return Ok(()),
            Err(err) => return Err(err.into()),
        };

        if !status.success() {
            return Err(GadgetError::FailedIpLinkSetup(format!(
                "NetworkManager refused to release interface {interface}"
            )));
        }

        Ok(())
    }

    pub fn set_interface_ip(interface: &str, ip: &str) -> GadgetResult<()> {
        debug!("IP for {} is being set to {}", interface, ip);

        let status = Command::new("/sbin/ip")
            .args(["addr", "flush", "dev", interface])
            .status()?;
        if !status.success() {
            return Err(GadgetError::FailedIpLinkSetup("ip addr flush failed".into()));
        }

        let status = Command::new("/sbin/ip")
            .args(["addr", "replace", ip, "dev", interface])
            .status()?;
        if !status.success() {
            return Err(GadgetError::FailedIpLinkSetup("ip addr replace failed".into()));
        }

        let status = Command::new("/sbin/ip")
            .args(["link", "set", interface, "up"])
            .status()?;
        if !status.success() {
            return Err(GadgetError::FailedIpLinkSetup("ip link set up failed".into()));
        }

        Ok(())
    }

    pub fn disable_interface(interface: &str) -> GadgetResult<()> {
        debug!("NCM interface {} is being disabled", interface);

        let status = Command::new("/sbin/ip")
            .args(["link", "set", interface, "down"])
            .status()?;

        if !status.success() {
            return Err(GadgetError::FailedIpLinkSetup("ip link set down failed".into()));
        }

        Ok(())
    }

    pub fn set_ipv6_param(interface: &str, param_type: &str, param: &str, val: &str) -> GadgetResult<()> {
        if fs::metadata(Path::new("/proc/sys/net/ipv6").join(param_type))?.is_dir() {
            let target = Path::new("/proc/sys/net/ipv6")
                .join(param_type)
                .join(interface)
                .join(param);
            if target.exists() {
                fs::write(target, val.as_bytes())?;
            } else {
                debug!("Unsupported IPv6 {param_type} param {param}");
            }
        } else {
            debug!("IPv6 is not available on this system, cannot set param {param} to {val}");
        }

        Ok(())
    }

    pub fn configure_for_carplay(interface: &str, ip: &str) -> GadgetResult<()> {
        Self::disable_ipv6_ra(interface)?;
        Self::disable_ipv6_dad(interface)?;
        Self::fix_ipv6_neigh_timeouts(interface)?;
        Self::set_interface_ip(interface, ip)
    }

    pub fn disable_ipv6_ra(interface: &str) -> GadgetResult<()> {
        Self::set_ipv6_param(interface, "conf", "accept_ra", "0")?;
        Self::set_ipv6_param(interface, "conf", "autoconf", "0")?;
        Self::set_ipv6_param(interface, "conf", "router_solicitations", "0")
    }

    pub fn disable_ipv6_dad(interface: &str) -> GadgetResult<()> {
        Self::set_ipv6_param(interface, "conf", "addr_gen_mode", "1")?;
        Self::set_ipv6_param(interface, "conf", "accept_dad", "0")?;
        Self::set_ipv6_param(interface, "conf", "dad_transmits", "0")?;
        Self::set_ipv6_param(interface, "conf", "enhanced_dad", "0")?;
        Self::set_ipv6_param(interface, "conf", "optimistic_dad", "1")
    }

    pub fn fix_ipv6_neigh_timeouts(interface: &str) -> GadgetResult<()> {
        // Some weird glitch causes these to be randomly lost over cdc-ncm and without these aggressive retries, connecting to carplay-ctrl over IPv6 link-local
        // can either work instantly or continue glitching with "Timeout" for up to ~20 seconds
        Self::set_ipv6_param(interface, "neigh", "delay_first_probe_time", "0")?;
        Self::set_ipv6_param(interface, "neigh", "retrans_time_ms", "100")?;
        Self::set_ipv6_param(interface, "neigh", "mcast_solicit", "100")
    }

    /// The retry method retries callback several times in short intervals for these reasons:
    /// - the NCM kernel gadget will create a network interface asynchronously after gadget bind - soon after, but not synchronously
    /// - for operations using the interface name, it may become invalid shortly after creation, if udev decides to rename it
    ///
    /// In the second case, the name retrieval can be put inside of the callback to be retried.
    pub fn retry<T, E, F: Fn() -> Result<T, E>>(callback: F) -> Result<T, E> {
        retry(Self::CDC_NCM_RETRY_INTERVAL, Self::CDC_NCM_RETRIES, callback)
    }

    pub fn find_mac_address(ifname: &str) -> GadgetResult<MacAddr6> {
        use libc::{AF_INET, SIOCGIFHWADDR, SOCK_DGRAM, close, ifreq, ioctl, socket};

        // Use ioctl SIOCGIFHWADDR
        let fd = unsafe { socket(AF_INET, SOCK_DGRAM, 0) };
        if fd < 0 {
            return Err(io::Error::last_os_error())?;
        }
        let mut ifr: ifreq = unsafe { zeroed() };
        for (i, b) in ifname.as_bytes().iter().enumerate() {
            ifr.ifr_name[i] = *b as _;
        }
        let r = unsafe { ioctl(fd, SIOCGIFHWADDR as _, &mut ifr as *mut _) };
        let _ = unsafe { close(fd) };
        if r < 0 {
            return Err(io::Error::last_os_error())?;
        }

        let sa = unsafe { &ifr.ifr_ifru.ifru_hwaddr };
        let mac = MacAddr6::new(
            sa.sa_data[0] as u8,
            sa.sa_data[1] as u8,
            sa.sa_data[2] as u8,
            sa.sa_data[3] as u8,
            sa.sa_data[4] as u8,
            sa.sa_data[5] as u8,
        );
        Ok(mac)
    }

    /// Checks for `RUNNING` flag on the interface - allows filtering out interface on which `hostapd`/`wpa_supplicant` hasn't fully started yet.
    pub fn is_iface_running(name: &str) -> io::Result<bool> {
        let fd = unsafe { libc::socket(libc::AF_INET, libc::SOCK_DGRAM, 0) };
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }
        let _guard = unsafe { OwnedFd::from_raw_fd(fd) };

        let mut ifr: libc::ifreq = unsafe { mem::zeroed() };
        for (dst, src) in ifr.ifr_name.iter_mut().zip(name.as_bytes()) {
            *dst = *src as _;
        }

        let ret = unsafe { libc::ioctl(fd, libc::SIOCGIFFLAGS as _, &mut ifr as *mut _) };
        if ret < 0 {
            return Err(io::Error::last_os_error());
        }

        let flags = unsafe { ifr.ifr_ifru.ifru_flags } as libc::c_int;
        Ok(flags & libc::IFF_RUNNING != 0)
    }

    /// Check if link-local IPv6 address was marked as stable and Linux started allowing multicast traffic.
    pub fn is_mdns_v6_stable(iface: &str) -> io::Result<()> {
        fn if_nametoindex(name: &str) -> io::Result<u32> {
            let c = CString::new(name).unwrap();
            let idx = unsafe { libc::if_nametoindex(c.as_ptr()) };
            if idx == 0 { Err(io::Error::last_os_error()) } else { Ok(idx) }
        }

        unsafe fn make_mdns_addr(ifindex: u32) -> libc::sockaddr_in6 {
            let mut addr: libc::sockaddr_in6 = unsafe { mem::zeroed() };

            addr.sin6_family = libc::AF_INET6 as _;
            addr.sin6_port = u16::to_be(5354);
            addr.sin6_scope_id = ifindex;

            // ff02::fb
            let ip = Ipv6Addr::new(0xff02, 0, 0, 0, 0, 0, 0, 0x00fb);
            addr.sin6_addr = libc::in6_addr { s6_addr: ip.octets() };

            addr
        }

        unsafe {
            let ifindex = if_nametoindex(iface)?;

            let fd = libc::socket(libc::AF_INET6, libc::SOCK_DGRAM, 0);
            if fd < 0 {
                return Err(io::Error::last_os_error());
            }
            let _guard = OwnedFd::from_raw_fd(fd);

            let ret = libc::setsockopt(
                fd,
                libc::IPPROTO_IPV6,
                libc::IPV6_MULTICAST_IF,
                &ifindex as *const _ as *const libc::c_void,
                mem::size_of::<u32>() as _,
            );
            if ret < 0 {
                let e = io::Error::last_os_error();
                return Err(e);
            }

            let addr = make_mdns_addr(ifindex);
            let buf = [b'x'];

            let ret = libc::sendto(
                fd,
                buf.as_ptr() as *const _,
                buf.len(),
                0,
                &addr as *const _ as *const libc::sockaddr,
                mem::size_of::<libc::sockaddr_in6>() as _,
            );

            if ret < 0 { Err(io::Error::last_os_error()) } else { Ok(()) }
        }
    }
}
