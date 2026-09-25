use catplay_bt::{CARPLAY_EIR_ACCESSORY, CARPLAY_HCI_CLASS_ACCESSORY_MGMT_BYTES};

fn main() {
    let hci = catplay_bt::HciSocket::new(1).unwrap(); // hci1
    hci.add_uuid(CARPLAY_EIR_ACCESSORY).unwrap();
    hci.set_class(CARPLAY_HCI_CLASS_ACCESSORY_MGMT_BYTES)
        .unwrap();
}
