use super::*;
use catplay_iap2_client::CsmSessionCallbacks;
use std::{
    io,
    pin::Pin,
    sync::atomic::{AtomicBool, Ordering},
    task::{Context, Poll},
};
use tokio::io::ReadBuf;

struct Transport {
    errno: Option<i32>,
    dropped: Arc<AtomicBool>,
}

impl Drop for Transport {
    fn drop(&mut self) {
        self.dropped.store(true, Ordering::SeqCst);
    }
}

impl AsyncRead for Transport {
    fn poll_read(self: Pin<&mut Self>, _: &mut Context<'_>, _: &mut ReadBuf<'_>) -> Poll<io::Result<()>> {
        match self.errno {
            Some(errno) => Poll::Ready(Err(io::Error::from_raw_os_error(errno))),
            None => Poll::Pending,
        }
    }
}

impl AsyncWrite for Transport {
    fn poll_write(self: Pin<&mut Self>, _: &mut Context<'_>, data: &[u8]) -> Poll<io::Result<usize>> {
        Poll::Ready(Ok(data.len()))
    }

    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

fn pipe(errno: Option<i32>, dropped: Arc<AtomicBool>) -> AsyncClientStream<Transport> {
    let (client, drain) = AsyncClient::new(true, CsmRemote::usb_gadget(), Box::new(CsmSessionCallbacks::noop()));
    AsyncClientStream::new(client, drain, Transport { errno, dropped })
}

#[tokio::test]
async fn reset_drops_old_transport_before_reopening_in_the_same_pass() {
    let dropped = Arc::new(AtomicBool::new(false));
    let mut old = pipe(Some(libc::ECONNRESET), dropped.clone());
    old.sleep().await;
    let mut current = Some(old);

    reconcile_iap2_pipe(&mut current, || {
        assert!(dropped.load(Ordering::SeqCst));
        Ok(pipe(None, Arc::default()))
    })
    .await
    .unwrap();
    assert!(current.is_some());

    reconcile_iap2_pipe(&mut current, || panic!("active client must be retained"))
        .await
        .unwrap();
}

#[tokio::test]
async fn missing_link_or_node_and_reset_during_open_retry_on_next_pass() {
    let mut current: Option<AsyncClientStream<Transport>> = None;
    for err in [
        GadgetError::FailedGadgetStatusCheck(io::Error::from_raw_os_error(libc::ENOENT).into()),
        GadgetError::from(io::Error::from_raw_os_error(libc::ENOENT)),
        GadgetError::from(io::Error::from_raw_os_error(libc::ECONNRESET)),
    ] {
        reconcile_iap2_pipe(&mut current, || Err(err))
            .await
            .unwrap();
        assert!(current.is_none());
    }
    reconcile_iap2_pipe(&mut current, || Ok(pipe(None, Arc::default())))
        .await
        .unwrap();
    assert!(current.is_some());
}

#[tokio::test]
async fn other_transport_errors_propagate_without_reopening() {
    // ENOENT is retryable only while resolving/opening the pipe.
    for errno in [libc::ESHUTDOWN, libc::EIO, libc::ENOENT] {
        let mut old = pipe(Some(errno), Arc::default());
        old.sleep().await;
        let err = reconcile_iap2_pipe(&mut Some(old), || panic!("must enter gadget reset"))
            .await
            .unwrap_err();
        assert!(
            matches!(err, CarPlayPhoneGadgetError::AccessoryDisconnectedIAp2(CsmSessionError::Io(e))
            if e.raw_os_error() == Some(errno))
        );
    }
}

#[tokio::test]
async fn other_open_errors_are_fatal_including_invalid_links() {
    for err in [
        GadgetError::from(io::Error::from_raw_os_error(libc::EACCES)),
        GadgetError::from(io::Error::from_raw_os_error(libc::ENODEV)),
        GadgetError::FailedGadgetStatusCheck(io::Error::new(io::ErrorKind::InvalidData, "invalid iAP2 sysfs link").into()),
    ] {
        let mut current: Option<AsyncClientStream<Transport>> = None;
        assert!(matches!(
            reconcile_iap2_pipe(&mut current, || Err(err)).await,
            Err(CarPlayPhoneGadgetError::FailedIap2Pipe(_))
        ));
    }
}

#[tokio::test]
async fn absent_pipe_waits_for_notification_or_negotiation_timeout() {
    let mut gadget = CarPlayPhoneGadget {
        iphone_instance: format!("missing-test-instance-{}", Uuid::new_v4()),
        pinned: false,
        udc: None,
        gadget: None,
        accessory: Some(AccessoryData {
            vid: String::new(),
            pid: String::new(),
            manufacturer: String::new(),
            product: String::new(),
            ncm: None,
        }),
        accessory_iap2: None,
        csm: Arc::new(|| Box::new(CsmSessionCallbacks::noop())),
        burst_wakeups: false,
        iap2_deadline: None,
    };
    let waiting = Ok(CarPlayPhoneGadgetStatus::WaitingForIAp2Session);
    let update = Instant::now();
    assert_eq!(gadget.render(waiting.clone(), update).await, waiting);
    assert_eq!(gadget.iap2_deadline, Some(update + CarPlayPhoneGadget::TIMEOUT_IAP2_NEGOTIATE));
    assert!(!gadget.burst_wakeups);

    // No gadget, pipe, or periodic polling: the negotiation deadline wakes us.
    gadget.iap2_deadline = Some(Instant::now());
    tokio::time::timeout(Duration::from_secs(1), gadget.sleep())
        .await
        .unwrap();
    let err = gadget
        .render(waiting, Instant::now() - CarPlayPhoneGadget::TIMEOUT_IAP2_NEGOTIATE)
        .await;
    assert_eq!(err, Err(CarPlayPhoneGadgetError::AccessoryFailedToInitIAp2));
    assert!(gadget.on_update(err).await.is_err());
    assert!(gadget.accessory.is_none());
    assert!(gadget.burst_wakeups);
    assert!(gadget.iap2_deadline.is_none());
}
