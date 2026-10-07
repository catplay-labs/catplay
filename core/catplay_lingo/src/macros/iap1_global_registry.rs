// This list controls the modules and every arm of the global registry.
// Each entry names the public module because `macro_rules!` cannot join identifiers.
macro_rules! iap1_global_registry {
    (strings = $strings:path; $($variant:ident($id:literal, $module:ident)),+ $(,)?) => {
        $(pub mod $module;)+

        /// A message from any enabled iAP1 lingo.
        #[derive(Clone, PartialEq, Eq)]
        pub enum LingoMessage {
            $($variant($module::LingoMessage),)+
        }

        impl core::fmt::Debug for LingoMessage {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                match self {
                    $(Self::$variant(message) => {
                        f.write_str(const { <$strings>::require_str(stringify!($variant)) })?;
                        f.write_str(".")?;
                        core::fmt::Debug::fmt(message, f)
                    },)+
                }
            }
        }

        impl LingoMessage {
            pub fn cast<T: 'static>(&self) -> Option<&T> {
                match self {
                    $(Self::$variant(message) => message.cast::<T>(),)+
                }
            }
        }

        pub struct LingoRegistry;

        impl crate::ProtocolRegistry for LingoRegistry {
            type Message = LingoMessage;

            fn decode(
                lingo: u8,
                command: u16,
                ctx: &crate::DecodeContext,
                payload: &[u8],
            ) -> Result<Self::Message, crate::RegistryDecodeError> {
                match lingo {
                    $($id => <$module::LingoRegistry as crate::Registry>::decode(command, ctx, payload)
                        .map(LingoMessage::$variant),)+
                    _ => Err(crate::RegistryDecodeError::UnknownLingo(lingo)),
                }
            }

            fn encode(
                message: &Self::Message,
                ctx: &crate::EncodeContext,
                writer: &mut $crate::Writer<'_>,
            ) -> Result<(), crate::EncodeError> {
                match message {
                    $(LingoMessage::$variant(value) =>
                        <$module::LingoRegistry as crate::Registry>::encode(value, ctx, writer),)+
                }
            }

            fn lingo_id(message: &Self::Message) -> u8 {
                match message {
                    $(LingoMessage::$variant(..) => $id,)+
                }
            }

            fn command_id(message: &Self::Message) -> u16 {
                match message {
                    $(LingoMessage::$variant(value) =>
                        <$module::LingoRegistry as crate::Registry>::command_id(value),)+
                }
            }

            fn metadata(
                lingo: u8,
                command: u16,
            ) -> Result<Option<crate::Iap1MessageMetadata>, crate::RegistryDecodeError> {
                match lingo {
                    $($id => Ok(<$module::LingoRegistry as crate::Registry>::metadata(command)),)+
                    _ => Err(crate::RegistryDecodeError::UnknownLingo(lingo)),
                }
            }
        }

        $(impl From<$module::LingoMessage> for LingoMessage {
            fn from(message: $module::LingoMessage) -> Self {
                Self::$variant(message)
            }
        })+
    };
}

pub(crate) use iap1_global_registry;
