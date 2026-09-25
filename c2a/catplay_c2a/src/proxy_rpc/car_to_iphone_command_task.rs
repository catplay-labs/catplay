use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::Instant,
};

use catplay_carplay::{
    carplay_rx::AirPlayReceiverHandleRef,
    events::CommandPending,
    msg::{Command, CommandType},
    rtsp_frame::{RtspError, RtspFuture, RtspResponse},
};
use log::{error, info};

pub(super) struct CarToIphoneCommandTask {
    phone: AirPlayReceiverHandleRef,
    cmd: Option<CommandPending>,
    command: Option<Command>,
    cmd_type: CommandType,
    start: Instant,
    response: Option<RtspFuture>,
}

pub(super) struct CarToIphoneCommandTaskOutput {
    pub cmd: CommandPending,
    pub command: Command,
    pub response: Option<RtspResponse>,
}

impl CarToIphoneCommandTask {
    pub fn new(phone: AirPlayReceiverHandleRef, cmd: CommandPending, command: Command) -> Self {
        let cmd_type = command.get_type();
        let response = match phone.send_command(command.clone()) {
            Ok(response) => Some(response),
            Err(err) => {
                error!("Failed to reserialize/schedule car command {:?}: {err}", command);
                phone.close(RtspError::UnexpectedState("Sanity violation: failed to schedule command".into()));
                None
            }
        };

        Self {
            phone,
            cmd: Some(cmd),
            cmd_type,
            command: Some(command),
            start: Instant::now(),
            response,
        }
    }

    fn take_output(&mut self, response: Option<RtspResponse>) -> CarToIphoneCommandTaskOutput {
        CarToIphoneCommandTaskOutput {
            cmd: self.cmd.take().expect("command pending was already taken"),
            command: self.command.take().expect("command was already taken"),
            response,
        }
    }
}

impl Future for CarToIphoneCommandTask {
    type Output = CarToIphoneCommandTaskOutput;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // SAFETY: this is a manual pin projection. `response` may contain
        // `RtspFuture`, which is !Unpin because it stores an unboxed sleep.
        // Once `response` is initialized, this future never moves or replaces it.
        let this = unsafe { self.get_unchecked_mut() };

        let Some(response) = this.response.as_mut() else {
            return Poll::Ready(this.take_output(None));
        };

        // SAFETY: `response` is pinned in place by the enclosing
        // `CarToIphoneCommandTask` and is not moved after insertion above.
        match unsafe { Pin::new_unchecked(response) }.poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Ok(response)) => {
                info!(
                    "iPhone responded to command {:?} in {:?} with status {}",
                    this.cmd_type,
                    Instant::now() - this.start,
                    response.status
                );
                let response_for_car = response.clone();
                this.cmd
                    .as_mut()
                    .expect("command pending was already taken")
                    .respond(Ok(response_for_car));
                Poll::Ready(this.take_output(Some(response)))
            }
            Poll::Ready(Err(err)) => {
                error!("iPhone failed to respond to command {:?}: {err}", this.command);
                this.phone
                    .close(RtspError::UnexpectedState("Sanity violation: command timeout".into()));
                Poll::Ready(this.take_output(None))
            }
        }
    }
}
