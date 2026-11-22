//! Exercise CatPlay's USB library and interface descriptors on the stock kernel.
use bytes::BytesMut;
use macaddr::MacAddr6;
use std::{
    io,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};
use usb_gadget::{
    Class, Config, Gadget, Id, OsDescriptor, Strings, default_udc,
    function::{
        custom::{Custom, Endpoint, EndpointDirection, EndpointReceiver, EndpointSender, Event, Interface, OsExtCompat},
        net::{Net, NetClass},
    },
};

#[cfg(all(target_arch = "arm", target_env = "musl"))]
#[path = "../../stock-clock.rs"]
mod stock_clock;

struct DiagnosticLog;
impl log::Log for DiagnosticLog {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        metadata.level() <= log::Level::Debug
    }
    fn log(&self, record: &log::Record<'_>) {
        if self.enabled(record.metadata()) {
            eprintln!("{}: {}", record.target(), record.args());
        }
    }
    fn flush(&self) {}
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    log::set_logger(&DiagnosticLog).map_err(|_| "Logger already installed")?;
    log::set_max_level(log::LevelFilter::Debug);
    let (rx, rx_direction) = EndpointDirection::host_to_device();
    let (tx, tx_direction) = EndpointDirection::device_to_host();
    let (mut custom, custom_handle) = Custom::builder()
        .with_interface(
            Interface::new(Class::vendor_specific(0xf0, 0), "iAP Interface")
                .with_endpoint(Endpoint::bulk(rx_direction))
                .with_endpoint(Endpoint::bulk(tx_direction))
                .with_os_ext_compat(OsExtCompat::new(*b"iAP2\0\0\0\0", [0; 8])),
        )
        .build();
    let mut builder = Net::builder(NetClass::Ncm);
    builder.dev_addr = Some(MacAddr6::new(2, 0x43, 0x50, 0, 0, 2));
    builder.host_addr = Some(MacAddr6::new(2, 0x43, 0x50, 0, 0, 1));
    builder.qmult = Some(10);
    let (net, net_handle) = builder.build();
    let registered = Gadget::new(
        Class::interface_specific(),
        Id::new(0x0525, 0xa4a1),
        Strings::new("CatPlay", "Anyka Rust USB Probe", "catplay-anyka-probe"),
    )
    .with_config(
        Config::new("Anyka transport diagnostic")
            .with_function(custom_handle)
            .with_function(net_handle),
    )
    .with_os_descriptor(OsDescriptor::microsoft())
    .register()?;
    println!("Rust gadget registered: {}", registered.path().display());
    if std::env::args().any(|arg| arg == "--register-only") {
        registered.remove()?;
        println!("Rust gadget registration and removal passed");
        return Ok(());
    }
    registered.bind(Some(&default_udc()?))?;
    println!("Rust gadget bound; network interface: {}", net.ifname()?.to_string_lossy());
    let enabled = Arc::new(AtomicBool::new(false));
    let event_enabled = enabled.clone();
    thread::spawn(move || -> io::Result<()> {
        loop {
            if let Some(event) = custom.event_timeout(Duration::from_secs(1))? {
                match event {
                    Event::Enable => {
                        event_enabled.store(true, Ordering::Relaxed);
                        println!("Rust FunctionFS enabled");
                    }
                    Event::Disable | Event::Unbind => {
                        event_enabled.store(false, Ordering::Relaxed);
                    }
                    Event::SetupHostToDevice(request) => {
                        request.recv_all()?;
                    }
                    Event::SetupDeviceToHost(request) => {
                        request.send(&[])?;
                    }
                    _ => {}
                }
            }
        }
    });
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    runtime.block_on(echo_bulk(rx, tx, enabled))?;
    Ok(())
}

async fn echo_bulk(mut rx: EndpointReceiver, mut tx: EndpointSender, enabled: Arc<AtomicBool>) -> io::Result<()> {
    loop {
        if !enabled.load(Ordering::Relaxed) {
            tokio::time::sleep(Duration::from_millis(100)).await;
            continue;
        }
        rx.try_recv(BytesMut::with_capacity(512))?;
        let data = rx
            .fetch_async()
            .await?
            .ok_or_else(|| io::Error::other("Missing USB receive completion"))?;
        if !data.is_empty() {
            let length = data.len();
            tx.send_async(data.freeze()).await?;
            tx.flush_async().await?;
            println!("Rust bulk echo: {length} bytes");
        }
    }
}
