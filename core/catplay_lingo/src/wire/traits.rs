use super::{
    CommandKey, DecodeContext, DecodeError, EncodeContext, EncodeError, Iap1MessageMetadata, Iap1Projector, Iap1StepContext, Reader,
    Writer, decode_projected_payload, encode_projected_message,
};
use catplay_reflect::{FieldProject, StringPool};

pub trait Iap1Decode: Sized {
    fn decode(reader: &mut Reader<'_>) -> Result<Self, DecodeError>;
}

pub trait Iap1Encode {
    fn encode(&self, writer: &mut Writer<'_>) -> Result<(), EncodeError>;
}

pub trait Iap1StructName {
    type StringPool: StringPool;
    const STRUCT_NAME_HANDLE: Self::StringPool;
    const STRUCT_NAME: &'static str;

    #[doc(hidden)]
    fn fmt_projected_debug(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    where
        Self: FieldProject<Iap1Projector<Self::StringPool>, Iap1StepContext<Self::StringPool>> + Sized,
    {
        super::debug_projected_fields(self, Self::STRUCT_NAME, formatter)
    }
}

pub trait Iap1Message:
    Sized + Default + PartialEq + FieldProject<Iap1Projector<Self::StringPool>, Iap1StepContext<Self::StringPool>> + Iap1StructName
{
    const META: Iap1MessageMetadata;
    const KEY: CommandKey;
    const FIELD_BITS: &'static [u32];
    const LENGTH_SELECTED: bool;

    fn decode(ctx: &DecodeContext, payload: &[u8]) -> Result<Self, DecodeError> {
        let mut value = Self::default();
        decode_projected_payload(
            &mut value,
            <Self as FieldProject<Iap1Projector<Self::StringPool>, Iap1StepContext<Self::StringPool>>>::fields(),
            Self::STRUCT_NAME,
            ctx,
            payload,
        )?;
        Ok(value)
    }

    fn encode(&self, ctx: &EncodeContext, writer: &mut Writer<'_>) -> Result<(), EncodeError> {
        encode_projected_message(
            self,
            <Self as FieldProject<Iap1Projector<Self::StringPool>, Iap1StepContext<Self::StringPool>>>::fields(),
            Self::STRUCT_NAME,
            Self::FIELD_BITS,
            Self::LENGTH_SELECTED,
            ctx,
            writer,
        )
    }
}

pub trait Iap1DecodeCounted<'a>: Sized {
    fn decode_counted(reader: &mut Reader<'a>, count: usize) -> Result<Self, DecodeError>;
}
pub trait Iap1EncodeCounted {
    fn encode_counted(&self, writer: &mut Writer<'_>, count: usize, field: &'static str) -> Result<(), EncodeError>;
}
pub trait Iap1DecodeRemaining: Sized {
    fn decode_remaining(reader: &mut Reader<'_>, field: &'static str) -> Result<Self, DecodeError>;
}
