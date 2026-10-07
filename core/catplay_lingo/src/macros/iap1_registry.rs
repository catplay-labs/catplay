/// Defines a lingo's message enum and static registry from its command IDs.
#[macro_export]
macro_rules! iap1_registry {
    ($lingo:literal; $($message:ident => $command:literal),* $(,)?) => {
        #[derive(Clone, PartialEq, Eq)]
        pub enum LingoMessage {
            $($message($message),)*
        }

        #[allow(deprecated)]
        impl core::fmt::Debug for LingoMessage {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                match self {
                    $(Self::$message(value) => core::fmt::Debug::fmt(value, f),)*
                }
            }
        }

        impl LingoMessage {
            pub fn cast<T: 'static>(&self) -> Option<&T> {
                match self {
                    $(Self::$message(value) => (value as &dyn core::any::Any).downcast_ref::<T>(),)*
                }
            }
        }

        pub struct LingoRegistry;

        impl $crate::Registry for LingoRegistry {
            const LINGO_ID: u8 = $lingo;
            type Message = LingoMessage;

            fn decode(
                command: u16,
                ctx: &$crate::DecodeContext,
                payload: &[u8],
            ) -> Result<LingoMessage, $crate::RegistryDecodeError> {
                match command {
                    $($command => <$message as $crate::Iap1Message>::decode(ctx, payload)
                        .map(LingoMessage::$message).map_err(Into::into),)*
                    _ => Err($crate::RegistryDecodeError::UnknownCommand {
                        lingo: Self::LINGO_ID,
                        command,
                    }),
                }
            }

            fn encode(
                message: &LingoMessage,
                ctx: &$crate::EncodeContext,
                writer: &mut $crate::Writer<'_>,
            ) -> Result<(), $crate::EncodeError> {
                match message {
                    $(LingoMessage::$message(value) =>
                        <$message as $crate::Iap1Message>::encode(value, ctx, writer),)*
                }
            }

            fn command_id(message: &LingoMessage) -> u16 {
                match message {
                    $(LingoMessage::$message(..) => $command,)*
                }
            }

            fn metadata(command: u16) -> Option<$crate::Iap1MessageMetadata> {
                match command {
                    $($command => Some(<$message as $crate::Iap1Message>::META),)*
                    _ => None,
                }
            }
        }

        impl $crate::RegisteredMessage for LingoMessage {
            type Registry = LingoRegistry;
        }
    };
}
