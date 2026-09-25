use std::{error::Error, process::Command};

use catplay_csm::{decoder::CsmPacketUtil, msg::StartIdentification};
use catplay_iap2_client::CsmSessionCallbacks;
use catplay_iap2_usb_gadget::{AccessoryGadget, gadget::GadgetHelper};
use catplay_iap2_usb_host::{CarPlayPhoneGadget, CarPlayPhoneGadgetStatus};
use catplay_tracing::logger::setup_test_logger;
use catplay_util::{EventReconciler, EventSleeper};
use log::info;
use tokio::{select, sync::mpsc};

#[ctor::ctor]
pub fn init_logger() {
    setup_test_logger(true);
}

// struct
// #[tokio::test]
// async fn test_gadget_handshake() -> Result<(), Box<dyn Error>> {
//     let _ = Command::new("/sbin/modprobe").arg("-q").arg("dummy_hcd").arg("num=2").status();

//     let _ = GadgetHelper::cleanup_once();

//     let mut gadget = CarPlayPhoneGadget::new_with_csm(Some("dummy_udc.0"), "default", false, false, CsmSessionCallbacks::noop)?;
//     let mut gadget_hu = AccessoryGadget::carplay(Some("dummy_udc.1"), false, CsmSessionCallbacks::noop)?;

//     loop {
//         if matches!(gadget_hu.state(), Ok(AccessoryStatus::CarPlaySession { .. }))
//             && matches!(gadget.state(), Ok(CarPlayPhoneGadgetStatus::CarPlaySession { .. }))
//         {
//             info!("Test finished OK");
//             return Ok(());
//         }

//         gadget.reconcile().await;
//         gadget_hu.reconcile().await;

//         select! {
//             _ = gadget.sleep() => {}
//             _ = gadget_hu.sleep() => {}
//         }
//         gadget.sleep().await;
//     }
// }

#[tokio::test]
async fn test_iap2_exchange() -> Result<(), Box<dyn Error>> {
    let _ = Command::new("/sbin/modprobe")
        .arg("-q")
        .arg("dummy_hcd")
        .arg("num=2")
        .status();

    let _ = GadgetHelper::cleanup_once();
    let cb = || CsmSessionCallbacks::new(async move |a| a.send(&StartIdentification {}), async move |_a, _b| Ok(()));

    let mut os = mpsc::channel(1);
    let os0 = os.0;

    let cb0 = move || {
        CsmSessionCallbacks::new(|_a| async move { Ok(()) }, {
            let os = os0.clone();
            move |packet, _handle| {
                let os = os.clone();
                async move {
                    os.send(packet).await.unwrap();
                    Ok(())
                }
            }
        })
    };

    let mut gadget = CarPlayPhoneGadget::new_with_csm(Some("dummy_udc.0"), "default", false, cb)?;
    let mut gadget_hu = AccessoryGadget::carplay(Some("dummy_udc.1"), false, cb0)?;

    loop {
        let _ = gadget.reconcile().await;
        let _ = gadget_hu.reconcile().await;

        select! {
            // _ = gadget.sleep() => {}
            _ = gadget_hu.sleep() => {}
            Some(v) = os.1.recv() => {
                assert!(StartIdentification::cast(&v).is_some());
                info!("CSM exchange finished: {v:?}");
                return Ok(());
            }
        }
        gadget.sleep().await;
    }
}
