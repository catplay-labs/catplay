use super::shared::ClientsDuplexTest;
use crate::{LinkEvent, LinkStatus, PacketOrDetect};

#[cfg(feature = "lingo")]
#[test]
fn legacy_input_is_forwarded_without_changing_link_state() {
    let mut test = ClientsDuplexTest::new();
    test.client.downgrade().unwrap();
    test.client_queue.lock().unwrap().clear();
    let packet = PacketOrDetect::Legacy(vec![0, 0x38, 0, 1]);
    test.client.read(packet);
    assert_eq!(test.client.status(), &LinkStatus::Downgrade);
    assert!(matches!(
        test.client_queue.lock().unwrap().pop_back(),
        Some(LinkEvent::ReadLegacy(body)) if body == [0, 0x38, 0, 1]
    ));
}

#[cfg(feature = "lingo_parser")]
#[test]
fn legacy_packet_display_decodes_known_command() {
    let packet = PacketOrDetect::Legacy(vec![0, 0x38, 0, 1]);
    assert!(format!("{packet}").contains("StartIDPS"));
}

#[cfg(feature = "lingo")]
#[test]
fn legacy_transmission_respects_frame_budget() {
    let mut test = ClientsDuplexTest::new();
    test.client.downgrade().unwrap();
    test.client_queue.lock().unwrap().clear();
    test.client.lingo.enqueue_tx(vec![0, 0x38, 0, 1]);
    test.client.lingo.enqueue_tx(vec![0, 0x38, 0, 2]);

    assert!(test.client.reconcile(1));
    assert!(matches!(
        test.client_queue.lock().unwrap().pop_front(),
        Some(LinkEvent::Write(PacketOrDetect::Legacy(body))) if body == [0, 0x38, 0, 1]
    ));
    assert!(test.client_queue.lock().unwrap().is_empty());

    assert!(!test.client.reconcile(2));
    assert!(matches!(
        test.client_queue.lock().unwrap().pop_front(),
        Some(LinkEvent::Write(PacketOrDetect::Legacy(body))) if body == [0, 0x38, 0, 2]
    ));
    assert!(test.client_queue.lock().unwrap().is_empty());
}
