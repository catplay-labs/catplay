use std::collections::HashMap;
use std::net::IpAddr;

pub struct CarPlayCtrlHelper;

pub const CLASS_IN: u16 = 0x0001;
pub const CACHE_FLUSH: u16 = 0x8000;

pub const TYPE_A: u16 = 1;
pub const TYPE_AAAA: u16 = 28;
pub const TYPE_PTR: u16 = 12;
pub const TYPE_TXT: u16 = 16;
pub const TYPE_SRV: u16 = 33;
pub const TYPE_NSEC: u16 = 47;

#[derive(Default)]
struct NameCompressor {
    map: HashMap<String, u16>,
}

impl NameCompressor {
    fn remember_suffix(&mut self, suffix: &str, offset: u16) {
        self.map.entry(suffix.to_string()).or_insert(offset);
    }

    fn lookup(&self, suffix: &str) -> Option<u16> {
        self.map.get(suffix).copied()
    }
}

// ================= DnsBuilder =================

pub struct MdnsBuilder {
    msg: Vec<u8>,
    comp: NameCompressor,
    arcount: usize,
    ancount: usize,
}

impl MdnsBuilder {
    const HEADER_SIZE: u16 = 12;

    pub fn new(cap: usize) -> Self {
        Self {
            msg: Vec::with_capacity(cap),
            comp: NameCompressor::default(),
            arcount: 0,
            ancount: 0,
        }
    }

    pub fn build_header(&self) -> [u8; Self::HEADER_SIZE as _] {
        let qdcount: u16 = 0; // unsolicited mDNS announce
        let ancount: u16 = self.ancount as u16;
        let nscount: u16 = 0; // mDNS = always 0
        let arcount: u16 = self.arcount as u16;

        let mut hdr = [0u8; 12];

        // ID = 0 (mDNS)
        hdr[0..2].copy_from_slice(&0u16.to_be_bytes());
        // Flags: response + authoritative answer (0x8400)
        hdr[2..4].copy_from_slice(&0x8400u16.to_be_bytes());
        // QDCOUNT
        hdr[4..6].copy_from_slice(&qdcount.to_be_bytes());
        // ANCOUNT
        hdr[6..8].copy_from_slice(&ancount.to_be_bytes());
        // NSCOUNT
        hdr[8..10].copy_from_slice(&nscount.to_be_bytes());
        // ARCOUNT
        hdr[10..12].copy_from_slice(&arcount.to_be_bytes());
        hdr
    }

    pub fn build_data(self) -> Vec<u8> {
        self.msg
    }

    fn write_u16(&mut self, v: u16) {
        self.msg.extend_from_slice(&v.to_be_bytes());
    }

    fn write_u32(&mut self, v: u32) {
        self.msg.extend_from_slice(&v.to_be_bytes());
    }

    pub fn write_name(&mut self, fqdn: &str) {
        debug_assert!(fqdn.ends_with('.'));

        let labels: Vec<&str> = fqdn.trim_end_matches('.').split('.').collect();

        let mut best: Option<(usize, u16)> = None;
        for i in 0..labels.len() {
            let suffix = labels[i..].join(".") + ".";
            if let Some(off) = self.comp.lookup(&suffix) {
                best = Some((i, off));
                break;
            }
        }

        let prefix_len = best.map(|(i, _)| i).unwrap_or(labels.len());

        for j in 0..prefix_len {
            let offset = self.msg.len() as u16;
            let suffix = labels[j..].join(".") + ".";
            self.comp
                .remember_suffix(&suffix, offset + Self::HEADER_SIZE);

            let lb = labels[j].as_bytes();
            self.msg.push(lb.len() as u8);
            self.msg.extend_from_slice(lb);
        }

        if let Some((_, off)) = best {
            self.write_u16(0xC000 | (off & 0x3FFF));
        } else {
            self.msg.push(0);
        }
    }

    pub fn rr_aaaa(&mut self, name: &str, ttl: u32, addr: [u8; 16], flush: bool) {
        self.write_name(name);
        self.write_u16(TYPE_AAAA);
        self.write_u16(CLASS_IN | if flush { CACHE_FLUSH } else { 0 });
        self.write_u32(ttl);
        self.write_u16(16);
        self.msg.extend_from_slice(&addr);
        self.ancount += 1;
    }

    pub fn rr_a(&mut self, name: &str, ttl: u32, addr: [u8; 4], flush: bool) {
        self.write_name(name);
        self.write_u16(TYPE_A);
        self.write_u16(CLASS_IN | if flush { CACHE_FLUSH } else { 0 });
        self.write_u32(ttl);
        self.write_u16(4);
        self.msg.extend_from_slice(&addr);
        self.ancount += 1;
    }

    pub fn rr_ptr(&mut self, name: &str, ttl: u32, target: &str, flush: bool) {
        self.write_name(name);
        self.write_u16(TYPE_PTR);
        self.write_u16(CLASS_IN | if flush { CACHE_FLUSH } else { 0 });
        self.write_u32(ttl);

        let rdlen_pos = self.msg.len();
        self.write_u16(0);
        let start = self.msg.len();

        self.write_name(target);

        let rdlen = (self.msg.len() - start) as u16;
        self.msg[rdlen_pos..rdlen_pos + 2].copy_from_slice(&rdlen.to_be_bytes());
        self.ancount += 1;
    }

    pub fn rr_srv(&mut self, name: &str, ttl: u32, port: u16, target: &str, flush: bool) {
        self.write_name(name);
        self.write_u16(TYPE_SRV);
        self.write_u16(CLASS_IN | if flush { CACHE_FLUSH } else { 0 });
        self.write_u32(ttl);

        let rdlen_pos = self.msg.len();
        self.write_u16(0);
        let start = self.msg.len();

        self.write_u16(0);
        self.write_u16(0);
        self.write_u16(port);
        self.write_name(target);

        let rdlen = (self.msg.len() - start) as u16;
        self.msg[rdlen_pos..rdlen_pos + 2].copy_from_slice(&rdlen.to_be_bytes());
        self.ancount += 1;
    }

    pub fn rr_txt(&mut self, name: &str, ttl: u32, entries: &[&[u8]], flush: bool) {
        self.write_name(name);
        self.write_u16(TYPE_TXT);
        self.write_u16(CLASS_IN | if flush { CACHE_FLUSH } else { 0 });
        self.write_u32(ttl);

        let rdlen_pos = self.msg.len();
        self.write_u16(0);
        let start = self.msg.len();

        for e in entries {
            self.msg.push(e.len() as u8);
            self.msg.extend_from_slice(e);
        }

        let rdlen = (self.msg.len() - start) as u16;
        self.msg[rdlen_pos..rdlen_pos + 2].copy_from_slice(&rdlen.to_be_bytes());
        self.ancount += 1;
    }

    pub fn rr_nsec(&mut self, name: &str, ttl: u32, next: &str, types: &[u16], flush: bool) {
        self.write_name(name);
        self.write_u16(TYPE_NSEC);
        self.write_u16(CLASS_IN | if flush { CACHE_FLUSH } else { 0 });
        self.write_u32(ttl);

        let rdlen_pos = self.msg.len();
        self.write_u16(0);
        let start = self.msg.len();

        self.write_name(next);

        let mut bitmap = [0u8; 32];
        for &t in types {
            bitmap[t as usize / 8] |= 1 << (7 - (t % 8));
        }

        let mut len = bitmap.len();
        while len > 0 && bitmap[len - 1] == 0 {
            len -= 1;
        }

        self.msg.push(0);
        self.msg.push(len as u8);
        self.msg.extend_from_slice(&bitmap[..len]);

        let rdlen = (self.msg.len() - start) as u16;
        self.msg[rdlen_pos..rdlen_pos + 2].copy_from_slice(&rdlen.to_be_bytes());
        self.arcount += 1;
    }
}

pub struct MdnsCoder;

impl MdnsCoder {
    pub fn build_announcement_packet(
        service_type: &str,
        instance_name: &str,
        hostname: &str,
        port: u16,
        txt: &HashMap<String, String>,
        ips: &[IpAddr],
        ttl: u32,
    ) -> Vec<u8> {
        let fullname = format!("{instance_name}.{service_type}");
        let services_fqdn = "_services._dns-sd._udp.local.";

        let mut b = MdnsBuilder::new(1500);
        b.rr_ptr(service_type, ttl, &fullname, false);
        b.rr_ptr(services_fqdn, ttl, service_type, false);
        b.rr_srv(&fullname, ttl, port, hostname, true);

        let mut txt_entries: Vec<Vec<u8>> = txt
            .iter()
            .map(|(k, v)| format!("{k}={v}").into_bytes())
            .collect();
        txt_entries.sort_unstable();
        let txt_refs = txt_entries.iter().map(|e| e.as_slice()).collect::<Vec<_>>();
        b.rr_txt(&fullname, ttl, &txt_refs, true);

        for ip in ips {
            match ip {
                IpAddr::V4(v4) => b.rr_a(hostname, ttl, v4.octets(), true),
                IpAddr::V6(v6) => b.rr_aaaa(hostname, ttl, v6.octets(), true),
            }
        }

        let mut out = Vec::new();
        out.extend_from_slice(&b.build_header());
        out.extend_from_slice(&b.build_data());
        out
    }
}
