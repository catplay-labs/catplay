use std::fmt;

use bytes::BytesMut;
use catplay_tracing::hexdump::pretty_hexdump_limited;

use crate::rtsp_frame::{RtspError, RtspResult, RtspString};
use catplay_plist::PlistSerializable;

use super::{HttpHeader, HttpStatus, RtspRequest};

#[derive(Debug, Clone)]
pub struct RtspResponse {
    pub proto: RtspString,
    pub status: HttpStatus,
    pub headers: Vec<(HttpHeader, RtspString)>,
    pub payload: BytesMut,
    pub cseq: Option<u32>,
}

impl RtspResponse {
    pub fn new(cseq: Option<u32>, status: HttpStatus) -> Self {
        Self {
            proto: RtspRequest::RTSP_DEFAULT_PROTO,
            status,
            headers: Vec::new(),
            payload: BytesMut::new(),
            cseq,
        }
    }

    pub fn with_payload(cseq: Option<u32>, status: HttpStatus, payload: Vec<u8>) -> Self {
        Self {
            proto: RtspRequest::RTSP_DEFAULT_PROTO,
            status,
            headers: Vec::new(),
            payload: BytesMut::from(&payload[..]),
            cseq,
        }
    }

    pub fn set_header(&mut self, header: HttpHeader, val: impl Into<RtspString>) {
        self.del_header(&header);
        self.headers.push((header, val.into()));
    }

    pub fn del_header(&mut self, header: &HttpHeader) {
        self.headers
            .retain_mut(|(_header, _value)| _header != header);
    }

    pub fn get_header(&self, header: &HttpHeader) -> Option<&str> {
        RtspRequest::extract_header(&self.headers, header)
    }

    fn fix_dynamic_headers(&mut self) {
        self.del_header(&HttpHeader::ContentLength);
        self.del_header(&HttpHeader::CSeq);
    }

    pub fn serialize_without_payload(&mut self, out: &mut BytesMut) {
        for c in [
            self.proto.as_ref().as_bytes(),
            b" ",
            &self.status.as_code_str(),
            b" ",
            self.status.reason_phrase().as_bytes(),
            b"\r\n",
        ] {
            out.extend_from_slice(c);
        }

        self.fix_dynamic_headers();

        for (k, v) in self.headers.iter() {
            RtspRequest::serialize_header(k, v, out);
        }

        if !self.payload.is_empty() {
            RtspRequest::serialize_header(HttpHeader::ContentLength, self.payload.len().to_string(), out);
        }
        if let Some(cseq) = self.cseq {
            RtspRequest::serialize_header(HttpHeader::CSeq, cseq.to_string(), out);
        }

        out.extend_from_slice(b"\r\n");
    }
}

impl RtspResponse {
    pub fn ok(self) -> RtspResult<Self> {
        match self.status {
            HttpStatus::Ok => Ok(self),
            _ => Err(RtspError::Code(self.status)),
        }
    }

    pub fn ok_payload<S: PlistSerializable>(self) -> RtspResult<S> {
        self.ok()?.get_plist()
    }
}

impl fmt::Display for RtspResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const MAX_TRACE: usize = 256;

        writeln!(f, "{} {} {}", self.proto, self.status.as_code(), self.status.reason_phrase())?;

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
