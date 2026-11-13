use serde::Deserialize;

#[derive(Debug, Deserialize, Default, Clone)]
pub struct AppConfig {
    pub debug: bool,
    pub persist_dir: Option<String>,

    pub mfi: MfiConfig,
    pub bluetooth: BluetoothConfig,

    pub wifi: WifiConfig,
    pub wifi_network: WifiNetwork,

    pub gadget: GadgetConfig,
    #[serde(default)]
    pub hardware: HardwareConfig,
}

/// Explicit opt-in: pin assignments differ between dongle revisions.
#[derive(Debug, Deserialize, Default, Clone)]
#[serde(default)]
pub struct HardwareConfig {
    pub enabled: bool,
    pub button_gpiochip: Option<String>,
    pub button_chip_label: Option<String>,
    pub button_line: Option<u32>,
    pub button_active_low: bool,
    pub button_bias_pull_up: bool,
    pub led_path: Option<String>,
    pub led_spi_device: Option<String>,
}

// MFI start

#[derive(Debug, Deserialize, Default, Clone)]
pub struct MfiConfig {
    pub server: Option<MfiConfigServer>,
    pub client: Option<MfiConfigClient>,
    pub i2c: Option<MfiConfigI2C>,
    pub selftest: bool,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct MfiConfigServer {
    pub enabled: bool,
    pub bind: String,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct MfiConfigClient {
    pub remote: String,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct MfiConfigI2C {
    pub bus_offset: u32,
    pub dev_addr: u8,
    pub timeout_ms: u64,
}

// Bluetooth start

#[derive(Debug, Deserialize, Default, Clone)]
pub struct BluetoothConfig {
    pub enabled: bool,
    pub device: String,
    pub name: String,
}

// Wifi start

#[derive(Debug, Deserialize, Default, Clone)]
pub struct WifiConfig {
    pub enabled: bool,
    pub device: String,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct WifiNetwork {
    pub ssid: String,
    pub password: String,
    pub wpa: bool,
    pub channel: Option<u8>,
}

// Gadget start

#[derive(Debug, Deserialize, Default, Clone)]
pub struct GadgetConfig {
    pub enabled: bool,
    pub pinned: bool,
    pub udc_car: Option<String>,
    pub udc_extra: Option<String>,
}
