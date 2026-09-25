use crate::rtsp_frame::{HttpHeader, RtspError, RtspRequest, RtspResponse, RtspResult};
use catplay_plist::PlistSerializable;

impl RtspResponse {
    pub fn get_plist<S: PlistSerializable>(&self) -> RtspResult<S> {
        S::pdecode(&self.payload).map_err(RtspError::UnparsablePlist)
    }

    pub fn set_plist<S: PlistSerializable>(&mut self, data: S) -> RtspResult<()> {
        self.payload.clear();
        match data
            .pencode_into(&mut self.payload)
            .map_err(RtspError::SerializationFailed)
        {
            Ok(_) => {
                self.set_header(HttpHeader::ContentType, "application/x-apple-binary-plist");

                Ok(())
            }
            Err(err) => Err(err),
        }
    }
}

impl RtspRequest {
    pub fn get_plist<S: PlistSerializable>(&self) -> RtspResult<S> {
        S::pdecode(&self.payload).map_err(RtspError::UnparsablePlist)
    }

    pub fn set_plist<S: PlistSerializable>(&mut self, data: S) -> RtspResult<()> {
        self.payload.clear();
        match data
            .pencode_into(&mut self.payload)
            .map_err(RtspError::SerializationFailed)
        {
            Ok(_) => {
                self.set_header(HttpHeader::ContentType, "application/x-apple-binary-plist");

                Ok(())
            }
            Err(err) => Err(err),
        }
    }
}
