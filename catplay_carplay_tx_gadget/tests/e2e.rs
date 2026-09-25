use std::{error::Error, process::Command};

use catplay_carplay::{
    carplay_rx::{AirPlayReceiverSink, sink::AirPlayServerShared},
    rtsp_frame::RtspError,
};
use catplay_carplay_rx_gadget::{CarPlayUsbGadget, CarPlayUsbGadgetState};
use catplay_carplay_tx_gadget::client::{CarPlayUsbClientGadget, CarPlayUsbClientGadgetStatus};
use catplay_hap::HomekitStorageFile;
use catplay_iap2_usb::gadget::GadgetHelper;
use catplay_tracing::logger::setup_test_logger;
use catplay_util::{AsyncShutdown, EventReconciler, EventSleeper};
use log::info;
use tokio::select;

// #[ctor::ctor]
// pub fn init_logger() {
//     setup_test_logger(true);
// }

struct MockSink {}
impl AirPlayReceiverSink for MockSink {}
impl EventSleeper for MockSink {}
impl EventReconciler for MockSink {
    type Error = RtspError;
}
impl AsyncShutdown for MockSink {}

#[tokio::test]
async fn test_gadget_handshake() -> Result<(), Box<dyn Error>> {
    setup_test_logger(true);

    let _ = Command::new("/sbin/modprobe")
        .arg("-q")
        .arg("dummy_hcd")
        .arg("num=2")
        .status();

    let _ = GadgetHelper::cleanup_once();

    let hk = HomekitStorageFile::memory();
    let hk_rx = HomekitStorageFile::memory();

    let mut gadget = CarPlayUsbClientGadget::new("default", "bonjour_id", Some("dummy_udc.0"), hk.clone(), false)?;

    let shared = AirPlayServerShared::new();
    let mut gadget_hu = CarPlayUsbGadget::new(hk_rx.clone(), None, shared, Some("dummy_udc.1"), false, "name", || MockSink {});

    loop {
        if matches!(gadget.state(), Ok(CarPlayUsbClientGadgetStatus::TransmitterReadyForPickup))
            && matches!(gadget_hu.state(), Ok(CarPlayUsbGadgetState::Receiving))
        {
            info!("Test finished OK");
            drop(gadget); // Reverse drop order to test "Cancelled inflight TransferGuard"
            return Ok(());
        }

        let _ = gadget.reconcile().await;
        let _ = gadget_hu.reconcile().await;

        select! {
            _ = gadget.sleep() => {}
            _ = gadget_hu.sleep() => {}
            // _ = gadget_hu.sleep() => {}
        }
    }

    Ok(())
}
