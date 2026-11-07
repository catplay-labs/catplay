use crate::{
    audio::{AudioStreamBasicDescription, RingConsumer, Zeroable},
    rtsp_frame::RtspResult,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioPlayerEvent {
    /// The first UDP datagram arrived, creating an anchor for the audio timeline.
    /// 
    /// Emitted once per session, independently of the latency-delayed Start.
    Anchor,
    /// The initial latency has elapsed; the consumer may start reading.
    Start,
    /// Suspend consumption while retaining buffered audio.
    Pause,
    /// The session has ended. Release the ring and playback resources.
    Stop,
    /// The producer committed new positions on the audio timeline.
    AddedDataToRing,
}

/// Receives a PCM ring and playback lifecycle notifications.
///
/// The user owns the consumer and handles missing samples explicitly through
/// the ring API. Events never consume or copy samples on the user's behalf.
pub trait AudioPlayer: Send + 'static {
    type Sample: Zeroable;

    fn init(&mut self, ring: RingConsumer<Self::Sample>, format: AudioStreamBasicDescription) -> RtspResult<()>;

    fn on_event(&mut self, event: AudioPlayerEvent);
}

pub type AudioPlayerBox<S> = Box<dyn AudioPlayer<Sample = S>>;
