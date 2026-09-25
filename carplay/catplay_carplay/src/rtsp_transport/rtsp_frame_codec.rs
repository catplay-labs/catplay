use bytes::{BufMut, BytesMut};
use catplay_tokio::BytesMutUtil;
use catplay_tokio::{Decoder, Encoder};
use catplay_tracing::hexdump::pretty_hexdump;
use log::debug;
use log::trace;
use std::mem;
use std::str::FromStr;

use crate::{rtsp_frame::*, rtsp_transport::RtspFrame};

pub struct RtspFrameCodec {
    state: CodecState,
    limits: Limits,
}

#[derive(Debug, Clone)]
enum CodecState {
    Headers {
        headers_scanned_until: usize,
    },
    RequestBody {
        method: RtspMethod,
        url: RtspString,
        proto: RtspString,
        headers: Vec<(HttpHeader, RtspString)>,
        content_length: usize,
        cseq: Option<u32>,
    },
    ResponseBody {
        proto: RtspString,
        status: HttpStatus,
        headers: Vec<(HttpHeader, RtspString)>,
        content_length: usize,
        cseq: Option<u32>,
    },
}

impl Default for CodecState {
    fn default() -> Self {
        Self::Headers { headers_scanned_until: 0 }
    }
}

impl Default for RtspFrameCodec {
    fn default() -> Self {
        Self {
            state: CodecState::Headers { headers_scanned_until: 0 },
            limits: Limits::default(),
        }
    }
}

struct Limits {
    headers_size: usize,
    request_line_size: usize,
    headers_count: usize,
    header_name_size: usize,
    header_value_size: usize,

    payload_size: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            headers_size: 4096,
            request_line_size: 1024,
            headers_count: 64,
            header_name_size: 128,
            header_value_size: 256,
            // 2MB to account for transfers of big OEM icons in /info
            payload_size: 2 * 1024 * 1024,
        }
    }
}

impl RtspFrameCodec {
    fn parse_request_line_parts(mut request_line: RtspString) -> Option<(RtspString, RtspString, RtspString)> {
        #[inline]
        fn parse_token(request_line: &mut RtspString) -> Option<RtspString> {
            let non_ws_prefix = request_line
                .as_str()
                .as_bytes()
                .iter()
                .position(|b| !b.is_ascii_whitespace())
                .unwrap_or(request_line.len());
            if non_ws_prefix > 0 {
                let _ = request_line.split_to(non_ws_prefix);
            }
            if request_line.is_empty() {
                return None;
            }

            let token_len = request_line
                .as_str()
                .as_bytes()
                .iter()
                .position(|b| b.is_ascii_whitespace())
                .unwrap_or(request_line.len());
            Some(request_line.split_to(token_len))
        }

        let p0 = parse_token(&mut request_line)?;
        let p1 = parse_token(&mut request_line)?;
        let p2 = parse_token(&mut request_line)?;
        Some((p0, p1, p2))
    }

    fn find_http_request_end(src: &[u8], search_start: usize, search_len: usize) -> Option<usize> {
        let end = search_len.min(src.len());
        if end <= search_start {
            return None;
        }

        let res = src[search_start..end]
            .windows(4)
            .position(|w| w == b"\r\n\r\n");
        res.map(|x| x + 4 + search_start)
    }

    pub fn decode_with_limit(&mut self, src: &mut BytesMut, visible_len: usize) -> RtspResult<Option<RtspFrame>> {
        use CodecState::*;

        let visible_len = visible_len.min(src.len());
        trace!("Next RTSP decode at len {} (visible {})...", src.len(), visible_len);

        let state = mem::take(&mut self.state);
        match state {
            Headers { headers_scanned_until } => {
                BytesMutUtil::ensure_writable(src, self.limits.headers_size);
                if visible_len == 0 {
                    self.state = Headers { headers_scanned_until };
                    return Ok(None);
                }

                let (search_len, overflow_maybe) = (visible_len.min(self.limits.headers_size), visible_len > self.limits.headers_size);
                let pos = Self::find_http_request_end(src, headers_scanned_until, search_len);
                trace!("Reading headers(pos,overflow_maybe): {pos:?} {overflow_maybe}");

                match (pos, overflow_maybe) {
                    (None, true) => {
                        #[cfg(debug_assertions)]
                        trace!("Buffer before headers overflow: {}", pretty_hexdump(src));
                        return Err(RtspError::HeadersTooBig);
                    }
                    (None, false) => {
                        #[cfg(debug_assertions)]
                        trace!("Buffer before headers decode: {}", pretty_hexdump(&src[..visible_len]));
                        self.state = Headers {
                            headers_scanned_until: visible_len.saturating_sub(4),
                        };
                        return Ok(None);
                    }
                    _ => {}
                }
                let pos = pos.unwrap();

                let mut header_block = RtspString::from_bytes(src.split_to(pos))?;
                let request_line_end = header_block
                    .as_str()
                    .find("\r\n")
                    .ok_or(RtspError::ProtocolViolation("no request line"))?;
                if request_line_end > self.limits.request_line_size {
                    return Err(RtspError::HeadersTooBig);
                }

                let request_line_buf = header_block.split_to(request_line_end);
                let Some((p0, p1, p2)) = Self::parse_request_line_parts(request_line_buf) else {
                    return Err(RtspError::ProtocolViolation("request line too short"));
                };
                let crlf = header_block.split_to(2);
                if crlf != "\r\n" {
                    return Err(RtspError::ProtocolViolation("missing CRLF after request line"));
                }

                // request -> $method $url $proto; response -> $proto $status_code $reason_phrase
                let is_response = p0.as_str().contains('/'); // HTTP/1.1

                let mut headers = Vec::<(HttpHeader, RtspString)>::new();
                while !header_block.is_empty() {
                    let line_end = header_block
                        .as_str()
                        .find("\r\n")
                        .ok_or(RtspError::ProtocolViolation("malformed header line"))?;

                    if line_end == 0 {
                        let crlf = header_block.split_to(2); // trailing CRLF
                        if crlf != "\r\n" {
                            return Err(RtspError::ProtocolViolation("missing trailing CRLF"));
                        }
                        break;
                    }

                    let line_str = &header_block.as_str()[..line_end];
                    let header_sep = line_str
                        .find(": ")
                        .ok_or(RtspError::ProtocolViolation("header missing ': ' separator"))?;
                    let key =
                        HttpHeader::from_str(&line_str[..header_sep]).map_err(|_| RtspError::ProtocolViolation("invalid header name"))?;

                    let mut line = header_block.split_to(line_end);
                    let crlf = header_block.split_to(2); // CRLF
                    if crlf != "\r\n" {
                        return Err(RtspError::ProtocolViolation("missing CRLF after header line"));
                    }
                    let _ = line.split_to(header_sep);
                    let sep = line.split_to(2); // ": "
                    if sep != ": " {
                        return Err(RtspError::ProtocolViolation("header missing ': ' separator"));
                    }

                    let value = line;
                    headers.push((key, value));
                }

                if headers.len() > self.limits.headers_count {
                    return Err(RtspError::HeadersTooBig);
                }

                for (key, value) in headers.iter() {
                    if key.as_str().len() > self.limits.header_name_size || value.as_str().len() > self.limits.header_value_size {
                        return Err(RtspError::HeadersTooBig);
                    }
                }

                let content_length =
                    RtspRequest::extract_content_length(&headers).ok_or(RtspError::ProtocolViolation("unparsable content length"))?;

                if content_length > self.limits.payload_size {
                    return Err(RtspError::PayloadTooBig(content_length, self.limits.payload_size));
                }

                BytesMutUtil::ensure_writable(src, content_length);

                let cseq = RtspRequest::extract_cseq(&headers);

                self.state = match is_response {
                    true => {
                        let proto = p0;
                        let code = p1
                            .as_str()
                            .parse::<u16>()
                            .map_err(|_| RtspError::ProtocolViolation("invalid http status code"))?;
                        let status = HttpStatus::from_code(code);
                        ResponseBody {
                            proto,
                            status,
                            headers,
                            content_length,
                            cseq,
                        }
                    }
                    false => {
                        let method = RtspMethod::from_str(p0.as_str())
                            .ok()
                            .unwrap_or(RtspMethod::Unknown);
                        let url = p1;
                        let proto = p2;
                        RequestBody {
                            method,
                            url,
                            proto,
                            headers,
                            content_length,
                            cseq,
                        }
                    }
                };

                #[cfg(debug_assertions)]
                trace!("Next state {:?}", self.state);
                self.decode_with_limit(src, visible_len.saturating_sub(pos))
            }
            RequestBody {
                method,
                url,
                proto,
                headers,
                content_length,
                cseq,
            } => {
                if visible_len < content_length {
                    debug!(
                        "Waiting for {} more bytes (have {}, need {})",
                        content_length - visible_len,
                        visible_len,
                        content_length
                    );
                    self.state = RequestBody {
                        method,
                        url,
                        proto,
                        headers,
                        content_length,
                        cseq,
                    };
                    return Ok(None);
                }

                let body = src.split_to(content_length);
                let frame = RtspFrame::Request(RtspRequest {
                    method,
                    url,
                    proto,
                    headers,
                    payload: body,
                    cseq,
                });

                self.state = Headers { headers_scanned_until: 0 };
                Ok(Some(frame))
            }
            ResponseBody {
                proto,
                status,
                headers,
                content_length,
                cseq,
            } => {
                if visible_len < content_length {
                    debug!(
                        "Waiting for {} more bytes (have {}, need {})",
                        content_length - visible_len,
                        visible_len,
                        content_length
                    );
                    self.state = ResponseBody {
                        proto,
                        status,
                        headers,
                        content_length,
                        cseq,
                    };
                    return Ok(None);
                }

                let body = src.split_to(content_length);
                let frame = RtspFrame::Response(RtspResponse {
                    proto,
                    status,
                    headers,
                    cseq,
                    payload: body,
                });
                self.state = Headers { headers_scanned_until: 0 };
                Ok(Some(frame))
            }
        }
    }

    pub fn decode(&mut self, src: &mut BytesMut) -> RtspResult<Option<RtspFrame>> {
        self.decode_with_limit(src, src.len())
    }

    pub fn encode(&mut self, item: RtspFrame, dst: &mut BytesMut) -> RtspResult<()> {
        match item {
            RtspFrame::Request(mut request) => {
                request.serialize_without_payload(dst);
                dst.put(request.payload);
                Ok(())
            }
            RtspFrame::Response(mut response) => {
                response.serialize_without_payload(dst);
                dst.put(response.payload);
                Ok(())
            }
        }
    }
}

impl Decoder for RtspFrameCodec {
    type Item = RtspFrame;
    type Error = RtspError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        self.decode(src)
    }
}

impl Encoder<RtspFrame> for RtspFrameCodec {
    type Error = RtspError;

    fn encode(&mut self, item: RtspFrame, dst: &mut BytesMut) -> Result<(), Self::Error> {
        self.encode(item, dst)
    }
}
