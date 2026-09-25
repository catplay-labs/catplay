use catplay_hap::{
    HomekitStorageFile,
    pair_setup::{CARPLAY_MAGIC_PIN, PairSetup},
};

use criterion::{Criterion, criterion_group, criterion_main};

fn pair_setup(c: &mut Criterion) {
    c.bench_function("pairsetup", |b| {
        let storage_controller = &*HomekitStorageFile::memory();
        let storage_accessory = &*HomekitStorageFile::memory();
        b.iter(|| {
            let (controller, payload_m1) = PairSetup::client(CARPLAY_MAGIC_PIN);
            let accessory = PairSetup::server(CARPLAY_MAGIC_PIN);

            // --- M1: Controller -> Accessory ---
            let (accessory, payload_m2) = accessory
                .handle(storage_accessory, &payload_m1)
                .expect("accessory M1 failed");
            // --- M2: Accessory -> Controller ---
            let (controller, payload_m3) = controller
                .handle(storage_controller, &payload_m2)
                .expect("controller M2 failed");
            // --- M3: Controller -> Accessory ---
            let (accessory, payload_m4) = accessory
                .handle(storage_accessory, &payload_m3)
                .expect("accessory M3 failed");
            // --- M4: Accessory -> Controller ---
            let (controller, payload_m5) = controller
                .handle(storage_controller, &payload_m4)
                .expect("controller M4 failed");

            // --- M5: Controller -> Accessory ---
            let (_accessory, payload_m6) = accessory
                .handle(storage_accessory, &payload_m5)
                .expect("accessory M5 failed");
            // --- M6: Accessory -> Controller ---
            let (_controller, _) = controller
                .handle(storage_controller, &payload_m6)
                .expect("controller M6 failed");
        });
    });
}

criterion_group!(benches, pair_setup);
criterion_main!(benches);
