use std::fmt;

use bytes::BytesMut;
use catplay_tracing::hexdump::pretty_hexdump_limited;

use super::RtspMethod;
use crate::rtsp_frame::{HttpHeader, RtspResult, RtspString};
use catplay_plist::PlistSerializable;

#[derive(Debug, Clone)]
pub struct RtspRequest {
    pub method: RtspMethod,
    pub url: RtspString,
    pub proto: RtspString,
    pub headers: Vec<(HttpHeader, RtspString)>,
    pub payload: BytesMut,
    pub cseq: Option<u32>,
}

impl RtspRequest {
    pub const RTSP_DEFAULT_PROTO: RtspString = RtspString::from_static("RTSP/1.0");

    pub fn new(method: RtspMethod, url: impl Into<RtspString>) -> Self {
        RtspRequest {
            method,
            url: url.into(),
            proto: Self::RTSP_DEFAULT_PROTO,
            headers: Vec::new(),
            payload: BytesMut::new(),
            cseq: None,
        }
    }

    pub fn with_payload(method: RtspMethod, url: impl Into<RtspString>, payload: impl Into<BytesMut>) -> Self {
        RtspRequest {
            method,
            url: url.into(),
            proto: Self::RTSP_DEFAULT_PROTO,
            headers: Vec::new(),
            payload: payload.into(),
            cseq: None,
        }
    }

    pub fn with_plist<T: PlistSerializable>(method: RtspMethod, url: impl Into<RtspString>, payload: T) -> RtspResult<Self> {
        let mut req = RtspRequest {
            method,
            url: url.into(),
            proto: Self::RTSP_DEFAULT_PROTO,
            headers: Vec::new(),
            payload: BytesMut::new(),
            cseq: None,
        };

        req.set_plist(payload)?;
        Ok(req)
    }

    pub fn post(url: impl Into<RtspString>, payload: impl Into<BytesMut>) -> Self {
        Self::with_payload(RtspMethod::Post, url, payload)
    }

    pub fn get(url: impl Into<RtspString>) -> Self {
        Self::new(RtspMethod::Get, url)
    }

    pub fn extract_content_length(headers: &[(HttpHeader, RtspString)]) -> Option<usize> {
        let cseq = Self::extract_header(headers, &HttpHeader::ContentLength);
        match cseq {
            None => Some(0),                          // No payload
            Some(cseq) => cseq.parse::<usize>().ok(), // Return None if unparsable
        }
    }

    pub fn extract_cseq(headers: &[(HttpHeader, RtspString)]) -> Option<u32> {
        let cseq = Self::extract_header(headers, &HttpHeader::CSeq)?;
        cseq.parse::<u32>().ok()
    }

    pub fn extract_header<'a>(headers: &'a [(HttpHeader, RtspString)], header: &HttpHeader) -> Option<&'a str> {
        headers
            .iter()
            .find(|(k, _)| k == header)
            .map(|(_, v)| v.as_str())
    }

    pub fn get_header(&self, header: &HttpHeader) -> Option<&str> {
        Self::extract_header(&self.headers, header)
    }

    pub fn set_header(&mut self, header: HttpHeader, val: impl Into<RtspString>) {
        self.del_header(&header);
        self.headers.push((header, val.into()));
    }

    pub fn del_header(&mut self, header: &HttpHeader) {
        self.headers
            .retain_mut(|(_header, _value)| _header != header);
    }

    fn fix_dynamic_headers(&mut self) {
        self.del_header(&HttpHeader::ContentLength);
        self.del_header(&HttpHeader::CSeq);
    }

    pub fn serialize_header(key: impl AsRef<str>, value: impl AsRef<str>, out: &mut BytesMut) {
        for c in [key.as_ref().as_bytes(), b": ", value.as_ref().as_bytes(), b"\r\n"] {
            out.extend_from_slice(c);
        }
    }

    pub fn serialize_without_payload(&mut self, out: &mut BytesMut) {
        for c in [
            self.method.as_ref().as_bytes(),
            b" ",
            self.url.as_ref().as_bytes(),
            b" ",
            self.proto.as_ref().as_bytes(),
            b"\r\n",
        ] {
            out.extend_from_slice(c);
        }

        self.fix_dynamic_headers();

        for (k, v) in self.headers.iter() {
            Self::serialize_header(k, v, out);
        }

        if !self.payload.is_empty() {
            Self::serialize_header(HttpHeader::ContentLength, self.payload.len().to_string(), out);
        }
        if let Some(cseq) = self.cseq {
            Self::serialize_header(HttpHeader::CSeq, cseq.to_string(), out);
        }
        out.extend_from_slice(b"\r\n");
    }
}

impl fmt::Display for RtspRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const MAX_TRACE: usize = 256;

        writeln!(f, "{} {} {}", self.method, self.url, self.proto)?;

        let mut copy = self.clone();
        copy.fix_dynamic_headers();

        for (k, v) in copy.headers.iter() {
            writeln!(f, "{}: {}", k, v)?;
        }

        if !self.payload.is_empty() {
            writeln!(f, "Content-Length: {}", self.payload.len())?;
        }

        if let Some(cseq) = self.cseq {
            writeln!(f, "CSeq: {}", cseq)?;
        }

        if !self.payload.is_empty() {
            writeln!(f, "\n--- Payload ({} bytes) ---", self.payload.len())?;
            write!(f, "{}", pretty_hexdump_limited(&self.payload, MAX_TRACE))?;
        }

        if let Ok(payload) = catplay_plist::from_bytes::<catplay_plist::Value>(&self.payload) {
            use catplay_plist::pretty_plist_value;

            writeln!(f, "\n--- bplist payload ---")?;
            write!(f, "{:?}", pretty_plist_value(&payload))?;
        }
        Ok(())
    }
}
