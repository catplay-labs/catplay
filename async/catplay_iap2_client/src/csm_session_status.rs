use catplay_iap2_link::LinkStatus;

use crate::CsmSessionError;

#[derive(Debug, Clone, PartialEq, Default)]
pub enum CsmSessionStatus {
    #[default]
    Detecting,
    Negotiating,
    Writable,
    Unwritable,
    Recovery,
    Downgrade,

    Error(CsmSessionError),
}

impl CsmSessionStatus {
    pub fn is_error(&self) -> bool {
        matches!(self, CsmSessionStatus::Error(_))
    }

    pub fn is_final(&self) -> bool {
        self.is_error()
    }
}

impl From<LinkStatus> for CsmSessionStatus {
    fn from(value: LinkStatus) -> Self {
        match value {
            LinkStatus::Detecting => CsmSessionStatus::Detecting,
            LinkStatus::Negotiating => CsmSessionStatus::Negotiating,
            LinkStatus::Writable => CsmSessionStatus::Writable,
            LinkStatus::Unwritable => CsmSessionStatus::Unwritable,
            LinkStatus::Recovery => CsmSessionStatus::Recovery,
            LinkStatus::Downgrade => CsmSessionStatus::Downgrade,
            LinkStatus::Error(err) => CsmSessionStatus::Error(err.into()),
        }
    }
}
