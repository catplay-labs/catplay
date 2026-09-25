use std::{
    collections::{HashMap, HashSet},
    io,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV6},
    os::fd::{AsRawFd, FromRawFd, OwnedFd},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use log::{debug, info, warn};
use tokio::{
    io::unix::AsyncFd,
    net::UdpSocket,
    sync::broadcast,
    sync::mpsc,
    time::{interval, sleep},
};
use tokio_util::task::AbortOnDropHandle;

use crate::{
    BonjourMeta, BonjourType,
    raw::{CLASS_IN, MdnsAnnouncer, MdnsCoder, TYPE_A, TYPE_AAAA, TYPE_PTR},
};

#[derive(Debug, Clone)]
pub enum RawMdnsEvent {
    Resolved(BonjourMeta),
    Removed { service_type: String, fullname: String },
}

#[derive(Clone)]
pub struct RawMdnsBackend {
    inner: Arc<RawMdnsBackendInner>,
}

struct RawMdnsBackendInner {
    state: Arc<RawMdnsState>,
    _tasks: Vec<AbortOnDropHandle<()>>,
    _btype: BonjourType,
}

struct RawMdnsState {
    iface: String,
    announcer: MdnsAnnouncer,
    iface_ips: Mutex<IfaceIps>,
    registered: Mutex<HashMap<String, RegisteredService>>,
    observed: Mutex<HashMap<String, ObservedService>>,
    event_tx: broadcast::Sender<RawMdnsEvent>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct IfaceIps {
    v4: Vec<Ipv4Addr>,
    v6: Option<Ipv6Addr>,
}

#[derive(Clone)]
struct RegisteredService {
    meta: BonjourMeta,
    packet: Arc<Vec<u8>>,
    names: HashSet<String>,
    canceled: Arc<AtomicBool>,
    auto_ips: bool,
}

#[derive(Clone)]
struct ObservedService {
    meta: BonjourMeta,
    expires_at: Instant,
}

impl RawMdnsBackend {
    pub fn new(iface: &str, btype: BonjourType) -> io::Result<Self> {
        let mut has_v4 = false;
        let mut has_v6 = false;

        let announcer = MdnsAnnouncer::new(iface)?;
        let init_v6 = announcer.ll_addr;
        let mut v4 = announcer.iface_v4_addrs().to_vec();
        v4.sort_unstable();
        v4.dedup();
        let (event_tx, _) = broadcast::channel(128);
        let state = Arc::new(RawMdnsState {
            iface: iface.to_string(),
            announcer,
            iface_ips: Mutex::new(IfaceIps { v4, v6: init_v6 }),
            registered: Mutex::new(HashMap::new()),
            observed: Mutex::new(HashMap::new()),
            event_tx,
        });

        let mut tasks = Vec::new();
        let (signal_tx, signal_rx) = mpsc::channel::<()>(64);

        if let Some(sock_v4) = state.announcer.clone_v4_socket()?
            && (btype == BonjourType::Ipv4 || btype == BonjourType::Ipv6AndIpv4)
        {
            let state2 = state.clone();
            let tokio_sock = UdpSocket::from_std(sock_v4)?;
            tasks.push(AbortOnDropHandle::new(tokio::spawn(async move {
                Self::reader_task(state2, tokio_sock).await;
            })));

            has_v4 = true;
        }

        if let Some(sock_v6) = state.announcer.clone_v6_socket()?
            && (btype == BonjourType::Ipv6 || btype == BonjourType::Ipv6AndIpv4)
        {
            let state2 = state.clone();
            let tokio_sock = UdpSocket::from_std(sock_v6)?;
            tasks.push(AbortOnDropHandle::new(tokio::spawn(async move {
                Self::reader_task(state2, tokio_sock).await;
            })));

            has_v6 = true;
        }

        {
            let state2 = state.clone();
            tasks.push(AbortOnDropHandle::new(tokio::spawn(async move {
                Self::expiry_task(state2).await;
            })));
        }
        {
            let state2 = state.clone();
            let signal_tx2 = signal_tx.clone();
            tasks.push(AbortOnDropHandle::new(tokio::spawn(async move {
                Self::netlink_signal_task(state2, signal_tx2).await;
            })));
        }
        {
            let state2 = state.clone();
            tasks.push(AbortOnDropHandle::new(tokio::spawn(async move {
                Self::reconcile_task(state2, signal_rx).await;
            })));
        }

        info!("Started Bonjour backend on '{iface}', ipv6: {has_v6}, ipv4: {has_v4}");

        Ok(Self {
            inner: Arc::new(RawMdnsBackendInner {
                state,
                _tasks: tasks,
                _btype: btype,
            }),
        })
    }

    pub fn subscribe(&self) -> broadcast::Receiver<RawMdnsEvent> {
        self.inner.state.event_tx.subscribe()
    }

    pub fn snapshot_service(&self, service_type: &str) -> Vec<BonjourMeta> {
        self.inner
            .state
            .observed
            .lock()
            .unwrap()
            .values()
            .filter(|v| v.meta.service_type == service_type)
            .map(|v| v.meta.clone())
            .collect()
    }

    pub fn query_service(&self, service_type: &str) {
        let packet = build_ptr_query_packet(service_type);
        let _ = self.inner.state.announcer.send_mdns(&packet);
    }

    pub fn register(&self, meta: &BonjourMeta) -> io::Result<Arc<AtomicBool>> {
        let mut meta = meta.clone();
        let auto_ips = meta.ips.is_empty();
        if auto_ips {
            let snap = self.inner.state.iface_ips.lock().unwrap().clone();
            for v4 in snap.v4 {
                meta.ips.push(IpAddr::V4(v4));
            }
            if let Some(v6) = snap.v6 {
                meta.ips.push(IpAddr::V6(v6));
            }
        }
        meta.addrs = meta
            .ips
            .iter()
            .map(|ip| to_scoped_addr(*ip, meta.port, self.inner.state.announcer.ifindex()))
            .collect();

        let packet = MdnsCoder::build_announcement_packet(
            &meta.service_type,
            &meta.instance_name,
            &meta.hostname,
            meta.port,
            &meta.txt,
            &meta.ips,
            4500,
        );

        let mut names = HashSet::new();
        names.insert(meta.service_type.to_ascii_lowercase());
        names.insert(meta.fullname.to_ascii_lowercase());
        names.insert(meta.hostname.to_ascii_lowercase());
        names.insert("_services._dns-sd._udp.local.".to_string());

        let canceled = Arc::new(AtomicBool::new(false));
        let reg = RegisteredService {
            meta: meta.clone(),
            packet: Arc::new(packet),
            names,
            canceled: canceled.clone(),
            auto_ips,
        };

        let replaced = self
            .inner
            .state
            .registered
            .lock()
            .unwrap()
            .insert(meta.fullname.clone(), reg.clone());
        if let Some(old) = replaced {
            old.canceled.store(true, Ordering::Relaxed);
        }

        let _ = self
            .inner
            .state
            .event_tx
            .send(RawMdnsEvent::Resolved(meta.clone()));
        Self::reannounce_fullname(&self.inner.state, &meta.fullname);

        let state = self.inner.state.clone();
        let fullname = meta.fullname.clone();
        let canceled_burst = canceled.clone();
        tokio::spawn(async move {
            const BURST_DELAYS: [Duration; 4] = [
                Duration::from_millis(120),
                Duration::from_millis(250),
                Duration::from_millis(500),
                Duration::from_millis(1000),
            ];
            for delay in BURST_DELAYS {
                sleep(delay).await;
                if canceled_burst.load(Ordering::Relaxed) {
                    return;
                }

                let still_registered = {
                    let guard = state.registered.lock().unwrap();
                    guard.contains_key(&meta.fullname)
                };
                if !still_registered {
                    return;
                }

                Self::reannounce_fullname(&state, &fullname);
            }
        });

        Ok(canceled)
    }

    pub fn unregister(&self, meta: &BonjourMeta, token: &Arc<AtomicBool>) {
        let mut removed = false;
        {
            let mut guard = self.inner.state.registered.lock().unwrap();
            if let Some(reg) = guard.get(&meta.fullname)
                && Arc::ptr_eq(&reg.canceled, token)
            {
                let reg = guard.remove(&meta.fullname).unwrap();
                reg.canceled.store(true, Ordering::Relaxed);
                removed = true;
            }
        }

        if removed {
            let _ = self.inner.state.event_tx.send(RawMdnsEvent::Removed {
                service_type: meta.service_type.clone(),
                fullname: meta.fullname.clone(),
            });
        } else {
            warn!("Ignoring stale unregister for {}", meta.fullname);
        }
    }

    pub fn reannounce(&self, fullname: &str, token: &Arc<AtomicBool>) {
        let packet = {
            let guard = self.inner.state.registered.lock().unwrap();
            guard.get(fullname).and_then(|r| {
                if Arc::ptr_eq(&r.canceled, token) {
                    Some(r.packet.clone())
                } else {
                    None
                }
            })
        };
        if let Some(packet) = packet {
            let _ = self.inner.state.announcer.send_mdns(packet.as_slice());
        } else {
            warn!("Ignoring stale reannounce for {}", fullname);
        }
    }

    async fn reader_task(state: Arc<RawMdnsState>, sock: UdpSocket) {
        let mut buf = [0u8; 2048];
        loop {
            match sock.recv_from(&mut buf).await {
                Ok((n, src)) => {
                    if n == 0 {
                        continue;
                    }
                    Self::on_packet(state.clone(), &buf[..n], src);
                }
                Err(err) => {
                    warn!("mDNS raw receive failed: {err}");
                    sleep(Duration::from_millis(200)).await;
                }
            }
        }
    }

    async fn expiry_task(state: Arc<RawMdnsState>) {
        let mut tick = interval(Duration::from_secs(1));
        loop {
            tick.tick().await;
            let now = Instant::now();

            let mut removed = Vec::new();
            {
                let mut guard = state.observed.lock().unwrap();
                let keys: Vec<_> = guard
                    .iter()
                    .filter_map(|(k, v)| if v.expires_at <= now { Some(k.clone()) } else { None })
                    .collect();
                for key in keys {
                    if let Some(value) = guard.remove(&key) {
                        removed.push((value.meta.service_type, key));
                    }
                }
            }

            for (service_type, fullname) in removed {
                let _ = state
                    .event_tx
                    .send(RawMdnsEvent::Removed { service_type, fullname });
            }
        }
    }

    async fn netlink_signal_task(state: Arc<RawMdnsState>, signal_tx: mpsc::Sender<()>) {
        match Self::create_netlink_listener() {
            Ok(listener) => {
                let mut buf = [0u8; 8192];
                loop {
                    let mut ready = match listener.readable().await {
                        Ok(v) => v,
                        Err(err) => {
                            warn!("netlink readable wait failed: {err}");
                            sleep(Duration::from_millis(300)).await;
                            continue;
                        }
                    };

                    let fd = listener.get_ref().as_raw_fd();
                    let n = unsafe { libc::recv(fd, buf.as_mut_ptr() as *mut _, buf.len(), libc::MSG_DONTWAIT) };

                    if n > 0 {
                        let _ = signal_tx.try_send(());
                    } else if n < 0 {
                        let err = io::Error::last_os_error();
                        if err.kind() != io::ErrorKind::WouldBlock {
                            warn!("netlink recv failed: {err}");
                        }
                    }

                    ready.clear_ready();
                }
            }
            Err(err) => {
                warn!("failed to start netlink listener for mDNS reconcile: {err}");
                let _ = state; // keep signature consistent
            }
        }
    }

    async fn reconcile_task(state: Arc<RawMdnsState>, mut signal_rx: mpsc::Receiver<()>) {
        let mut periodic = interval(Duration::from_secs(10));
        loop {
            tokio::select! {
                _ = periodic.tick() => {}
                msg = signal_rx.recv() => {
                    if msg.is_none() {
                        return;
                    }
                }
            }

            if let Err(err) = Self::reconcile_ips_once(state.clone()) {
                warn!("mDNS raw reconcile failed: {err}");
            }
        }
    }

    fn reconcile_ips_once(state: Arc<RawMdnsState>) -> io::Result<()> {
        let (mut v4, v6) = MdnsAnnouncer::snapshot_iface_addrs(&state.iface)?;
        v4.sort_unstable();
        v4.dedup();
        let new_ips = IfaceIps { v4, v6 };

        let changed = {
            let mut guard = state.iface_ips.lock().unwrap();
            if *guard == new_ips {
                false
            } else {
                *guard = new_ips.clone();
                true
            }
        };
        if !changed {
            return Ok(());
        }

        debug!("mDNS iface IP set changed on {}, refreshing auto-IP services", state.iface);
        let mut to_reannounce = Vec::new();
        {
            let mut regs = state.registered.lock().unwrap();
            for reg in regs.values_mut() {
                if !reg.auto_ips {
                    continue;
                }

                let mut new_meta = reg.meta.clone();
                new_meta.ips.clear();
                for ip in &new_ips.v4 {
                    new_meta.ips.push(IpAddr::V4(*ip));
                }
                if let Some(v6) = new_ips.v6 {
                    new_meta.ips.push(IpAddr::V6(v6));
                }
                new_meta.addrs = new_meta
                    .ips
                    .iter()
                    .map(|ip| to_scoped_addr(*ip, new_meta.port, state.announcer.ifindex()))
                    .collect();

                let packet = MdnsCoder::build_announcement_packet(
                    &new_meta.service_type,
                    &new_meta.instance_name,
                    &new_meta.hostname,
                    new_meta.port,
                    &new_meta.txt,
                    &new_meta.ips,
                    4500,
                );
                reg.meta = new_meta.clone();
                reg.packet = Arc::new(packet);
                to_reannounce.push(new_meta);
            }
        }

        for meta in to_reannounce {
            let _ = state.event_tx.send(RawMdnsEvent::Resolved(meta.clone()));
            Self::reannounce_fullname(&state, &meta.fullname);
        }

        Ok(())
    }

    fn reannounce_fullname(state: &RawMdnsState, fullname: &str) {
        let packet = {
            let guard = state.registered.lock().unwrap();
            guard.get(fullname).map(|r| r.packet.clone())
        };
        if let Some(packet) = packet {
            let _ = state.announcer.send_mdns(packet.as_slice());
        }
    }

    fn create_netlink_listener() -> io::Result<AsyncFd<OwnedFd>> {
        let fd = unsafe { libc::socket(libc::AF_NETLINK, libc::SOCK_RAW | libc::SOCK_NONBLOCK, libc::NETLINK_ROUTE) };
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }
        let fd = unsafe { OwnedFd::from_raw_fd(fd) };

        let groups = (libc::RTMGRP_IPV4_IFADDR | libc::RTMGRP_IPV6_IFADDR | libc::RTMGRP_LINK) as u32;
        let mut addr: libc::sockaddr_nl = unsafe { std::mem::zeroed() };
        addr.nl_family = libc::AF_NETLINK as u16;
        addr.nl_pid = 0;
        addr.nl_groups = groups;

        let ret = unsafe {
            libc::bind(
                fd.as_raw_fd(),
                &addr as *const _ as *const libc::sockaddr,
                std::mem::size_of::<libc::sockaddr_nl>() as _,
            )
        };
        if ret < 0 {
            return Err(io::Error::last_os_error());
        }

        AsyncFd::new(fd)
    }

    fn on_packet(state: Arc<RawMdnsState>, packet: &[u8], _src: SocketAddr) {
        let Some(parsed) = ParsedDnsMessage::parse(packet) else {
            return;
        };

        if !parsed.is_response {
            Self::answer_queries(state, &parsed.questions);
            return;
        }

        Self::handle_responses(state, &parsed.records);
    }

    fn answer_queries(state: Arc<RawMdnsState>, questions: &[DnsQuestion]) {
        if questions.is_empty() {
            return;
        }

        let regs = state
            .registered
            .lock()
            .unwrap()
            .values()
            .cloned()
            .collect::<Vec<_>>();
        for reg in regs {
            let should_reply = questions.iter().any(|q| {
                let name = q.name.to_ascii_lowercase();
                reg.names.contains(&name)
            });

            if should_reply {
                debug!("mDNS raw reply for {}", reg.meta.fullname);
                let _ = state.announcer.send_mdns(reg.packet.as_slice());
            }
        }
    }

    fn handle_responses(state: Arc<RawMdnsState>, records: &[DnsRecord]) {
        let mut ptrs: HashMap<String, (String, u32)> = HashMap::new();
        let mut srv: HashMap<String, (String, u16, u32)> = HashMap::new();
        let mut txt: HashMap<String, (HashMap<String, String>, u32)> = HashMap::new();
        let mut host_ips: HashMap<String, (Vec<IpAddr>, u32)> = HashMap::new();

        for rr in records {
            match &rr.data {
                DnsRData::Ptr(target) => {
                    if let Some((_, service_type)) = split_instance_and_service_type(target) {
                        if rr.ttl == 0 {
                            debug!("mDNS goodbye (PTR TTL=0): removing service_type={service_type}, fullname={target}");
                            let _ = state.event_tx.send(RawMdnsEvent::Removed {
                                service_type,
                                fullname: target.clone(),
                            });
                            state.observed.lock().unwrap().remove(target);
                        } else {
                            ptrs.insert(target.clone(), (rr.name.clone(), rr.ttl));
                        }
                    }
                }
                DnsRData::Srv { target, port } => {
                    srv.insert(rr.name.clone(), (target.clone(), *port, rr.ttl));
                }
                DnsRData::Txt(map) => {
                    txt.insert(rr.name.clone(), (map.clone(), rr.ttl));
                }
                DnsRData::A(ip) => {
                    let entry = host_ips
                        .entry(dns_name_key(&rr.name))
                        .or_insert_with(|| (Vec::new(), rr.ttl));
                    entry.0.push(IpAddr::V4(*ip));
                    entry.1 = entry.1.min(rr.ttl);
                }
                DnsRData::Aaaa(ip) => {
                    let entry = host_ips
                        .entry(dns_name_key(&rr.name))
                        .or_insert_with(|| (Vec::new(), rr.ttl));
                    entry.0.push(IpAddr::V6(*ip));
                    entry.1 = entry.1.min(rr.ttl);
                }
                DnsRData::Unknown => {}
            }
        }

        let mut touched = HashSet::new();
        touched.extend(ptrs.keys().cloned());
        touched.extend(srv.keys().cloned());
        touched.extend(txt.keys().cloned());
        if !host_ips.is_empty() {
            let observed = state.observed.lock().unwrap();
            touched.extend(observed.iter().filter_map(|(fullname, observed)| {
                host_ips
                    .contains_key(&dns_name_key(&observed.meta.hostname))
                    .then(|| fullname.clone())
            }));
        }

        let now = Instant::now();
        for fullname in touched {
            let Some((instance_name, service_type)) = split_instance_and_service_type(&fullname) else {
                continue;
            };

            let mut old = state.observed.lock().unwrap().get(&fullname).cloned();

            let hostname = srv
                .get(&fullname)
                .map(|v| v.0.clone())
                .or_else(|| old.as_ref().map(|o| o.meta.hostname.clone()))
                .unwrap_or_default();
            let port = srv
                .get(&fullname)
                .map(|v| v.1)
                .or_else(|| old.as_ref().map(|o| o.meta.port))
                .unwrap_or(0);

            let txt_map = txt
                .get(&fullname)
                .map(|v| v.0.clone())
                .or_else(|| old.as_ref().map(|o| o.meta.txt.clone()))
                .unwrap_or_default();

            let ips = host_ips
                .get(&dns_name_key(&hostname))
                .map(|v| v.0.clone())
                .or_else(|| old.as_ref().map(|o| o.meta.ips.clone()))
                .unwrap_or_default();

            if port == 0 || hostname.is_empty() {
                continue;
            }

            let ttl = [
                ptrs.get(&fullname).map(|v| v.1),
                srv.get(&fullname).map(|v| v.2),
                txt.get(&fullname).map(|v| v.1),
                host_ips.get(&dns_name_key(&hostname)).map(|v| v.1),
            ]
            .into_iter()
            .flatten()
            .min()
            .unwrap_or(120);

            let meta = BonjourMeta {
                iface: state.iface.clone(),
                hostname: hostname.clone(),
                port,
                ips: ips.clone(),
                addrs: ips
                    .iter()
                    .map(|ip| to_scoped_addr(*ip, port, state.announcer.ifindex()))
                    .collect(),
                txt: txt_map,
                service_type: service_type.clone(),
                instance_name,
                fullname: fullname.clone(),
            };

            let changed = old.as_mut().map(|o| o.meta != meta).unwrap_or(true);

            state.observed.lock().unwrap().insert(
                fullname.clone(),
                ObservedService {
                    meta: meta.clone(),
                    expires_at: now + Duration::from_secs(ttl.max(1) as u64),
                },
            );

            if changed {
                if meta.addrs.is_empty() {
                    debug!(
                        "mDNS resolved {} without host addresses for host {}; available host address records: {:?}; querying host",
                        meta.fullname,
                        meta.hostname,
                        host_ips.keys().collect::<Vec<_>>()
                    );
                    let packet = build_addr_query_packet(&meta.hostname);
                    let _ = state.announcer.send_mdns(&packet);
                } else {
                    let _ = state.event_tx.send(RawMdnsEvent::Resolved(meta));
                }
            }
        }
    }
}

fn build_ptr_query_packet(service_type: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(128);
    // DNS header: id=0, flags=0(query), qd=1, an=0, ns=0, ar=0
    out.extend_from_slice(&0u16.to_be_bytes());
    out.extend_from_slice(&0u16.to_be_bytes());
    out.extend_from_slice(&1u16.to_be_bytes());
    out.extend_from_slice(&0u16.to_be_bytes());
    out.extend_from_slice(&0u16.to_be_bytes());
    out.extend_from_slice(&0u16.to_be_bytes());

    for label in service_type.trim_end_matches('.').split('.') {
        let bytes = label.as_bytes();
        let take = bytes.len().min(u8::MAX as usize);
        out.push(take as u8);
        out.extend_from_slice(&bytes[..take]);
    }
    out.push(0);
    out.extend_from_slice(&TYPE_PTR.to_be_bytes());
    out.extend_from_slice(&CLASS_IN.to_be_bytes());
    out
}

fn build_addr_query_packet(hostname: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(160);
    // DNS header: id=0, flags=0(query), qd=2, an=0, ns=0, ar=0
    out.extend_from_slice(&0u16.to_be_bytes());
    out.extend_from_slice(&0u16.to_be_bytes());
    out.extend_from_slice(&2u16.to_be_bytes());
    out.extend_from_slice(&0u16.to_be_bytes());
    out.extend_from_slice(&0u16.to_be_bytes());
    out.extend_from_slice(&0u16.to_be_bytes());

    write_question(&mut out, hostname, TYPE_A);
    write_question(&mut out, hostname, TYPE_AAAA);

    out
}

fn write_question(out: &mut Vec<u8>, name: &str, qtype: u16) {
    for label in name.trim_end_matches('.').split('.') {
        let bytes = label.as_bytes();
        let take = bytes.len().min(u8::MAX as usize);
        out.push(take as u8);
        out.extend_from_slice(&bytes[..take]);
    }
    out.push(0);
    out.extend_from_slice(&qtype.to_be_bytes());
    out.extend_from_slice(&CLASS_IN.to_be_bytes());
}

fn dns_name_key(name: &str) -> String {
    name.to_ascii_lowercase()
}

fn to_scoped_addr(ip: IpAddr, port: u16, ifindex: u32) -> SocketAddr {
    match ip {
        IpAddr::V4(v4) => SocketAddr::new(IpAddr::V4(v4), port),
        IpAddr::V6(v6) => {
            if v6.is_unicast_link_local() {
                SocketAddr::V6(SocketAddrV6::new(v6, port, 0, ifindex))
            } else {
                SocketAddr::V6(SocketAddrV6::new(v6, port, 0, 0))
            }
        }
    }
}

#[derive(Debug)]
struct ParsedDnsMessage {
    is_response: bool,
    questions: Vec<DnsQuestion>,
    records: Vec<DnsRecord>,
}

#[derive(Debug)]
struct DnsQuestion {
    name: String,
}

#[derive(Debug)]
struct DnsRecord {
    name: String,
    ttl: u32,
    data: DnsRData,
}

#[derive(Debug)]
enum DnsRData {
    Ptr(String),
    Srv { target: String, port: u16 },
    Txt(HashMap<String, String>),
    A(Ipv4Addr),
    Aaaa(Ipv6Addr),
    Unknown,
}

impl ParsedDnsMessage {
    fn parse(buf: &[u8]) -> Option<Self> {
        if buf.len() < 12 {
            return None;
        }

        let flags = u16::from_be_bytes([buf[2], buf[3]]);
        let is_response = (flags & 0x8000) != 0;
        let qd = u16::from_be_bytes([buf[4], buf[5]]) as usize;
        let an = u16::from_be_bytes([buf[6], buf[7]]) as usize;
        let ns = u16::from_be_bytes([buf[8], buf[9]]) as usize;
        let ar = u16::from_be_bytes([buf[10], buf[11]]) as usize;

        let mut pos = 12usize;
        let mut questions = Vec::new();
        for _ in 0..qd {
            let (name, p) = parse_name(buf, pos)?;
            pos = p;
            if pos + 4 > buf.len() {
                return None;
            }
            pos += 4;
            questions.push(DnsQuestion { name });
        }

        let mut records = Vec::new();
        for _ in 0..(an + ns + ar) {
            let (name, p) = parse_name(buf, pos)?;
            pos = p;
            if pos + 10 > buf.len() {
                return None;
            }
            let rr_type = u16::from_be_bytes([buf[pos], buf[pos + 1]]);
            let _rr_class = u16::from_be_bytes([buf[pos + 2], buf[pos + 3]]);
            let ttl = u32::from_be_bytes([buf[pos + 4], buf[pos + 5], buf[pos + 6], buf[pos + 7]]);
            let rdlen = u16::from_be_bytes([buf[pos + 8], buf[pos + 9]]) as usize;
            pos += 10;
            if pos + rdlen > buf.len() {
                return None;
            }
            let rstart = pos;
            let rend = pos + rdlen;
            pos = rend;

            let data = match rr_type {
                12 => {
                    let (target, _) = parse_name(buf, rstart)?;
                    DnsRData::Ptr(target)
                }
                33 => {
                    if rdlen < 6 {
                        DnsRData::Unknown
                    } else {
                        let port = u16::from_be_bytes([buf[rstart + 4], buf[rstart + 5]]);
                        let (target, _) = parse_name(buf, rstart + 6)?;
                        DnsRData::Srv { target, port }
                    }
                }
                16 => {
                    let mut map = HashMap::new();
                    let mut i = rstart;
                    while i < rend {
                        let len = buf[i] as usize;
                        i += 1;
                        if i + len > rend {
                            break;
                        }
                        let txt = std::str::from_utf8(&buf[i..i + len])
                            .ok()
                            .unwrap_or_default();
                        if let Some(eq) = txt.find('=') {
                            map.insert(txt[..eq].to_string(), txt[eq + 1..].to_string());
                        }
                        i += len;
                    }
                    DnsRData::Txt(map)
                }
                1 => {
                    if rdlen == 4 {
                        DnsRData::A(Ipv4Addr::new(buf[rstart], buf[rstart + 1], buf[rstart + 2], buf[rstart + 3]))
                    } else {
                        DnsRData::Unknown
                    }
                }
                28 => {
                    if rdlen == 16 {
                        let mut oct = [0u8; 16];
                        oct.copy_from_slice(&buf[rstart..rstart + 16]);
                        DnsRData::Aaaa(Ipv6Addr::from(oct))
                    } else {
                        DnsRData::Unknown
                    }
                }
                _ => DnsRData::Unknown,
            };

            records.push(DnsRecord { name, ttl, data });
        }

        Some(Self {
            is_response,
            questions,
            records,
        })
    }
}

fn parse_name(buf: &[u8], start: usize) -> Option<(String, usize)> {
    let mut name = String::new();
    let mut pos = start;
    let mut jumped = false;
    let mut jump_end = start;
    let mut steps = 0usize;

    loop {
        if pos >= buf.len() {
            return None;
        }
        let len = buf[pos];
        if len & 0xC0 == 0xC0 {
            if pos + 1 >= buf.len() {
                return None;
            }
            let ptr = (((len as u16 & 0x3F) << 8) | buf[pos + 1] as u16) as usize;
            if !jumped {
                jump_end = pos + 2;
                jumped = true;
            }
            pos = ptr;
            steps += 1;
            if steps > 20 {
                return None;
            }
            continue;
        }

        pos += 1;
        if len == 0 {
            break;
        }
        let len = len as usize;
        if pos + len > buf.len() {
            return None;
        }
        if !name.is_empty() {
            name.push('.');
        }
        name.push_str(std::str::from_utf8(&buf[pos..pos + len]).ok()?);
        pos += len;
    }

    name.push('.');
    Some((name, if jumped { jump_end } else { pos }))
}

fn split_instance_and_service_type(fullname: &str) -> Option<(String, String)> {
    let labels: Vec<&str> = fullname.trim_end_matches('.').split('.').collect();
    if labels.len() < 4 {
        return None;
    }

    let idx = labels.iter().position(|l| l.starts_with('_'))?;
    if idx == 0 || idx + 2 >= labels.len() {
        return None;
    }

    let instance = labels[..idx].join(".");
    let service_type = format!("{}.", labels[idx..].join("."));
    Some((instance, service_type))
}
