#![allow(unused)]

const APPNAME: &str = "arete-sdk-01-light";
const DEFAULT_TIMEOUT_MILLIS: u64 = 500;
const GPIO23: u32 = 23;
const NODE_ID: &str = "onqXVczGoymQkFc3UN6qcM";
const DESIRED_STATE_KEY: &str = "cns/network/nodes/sri4FZUq2V7S4ik2PrG4pj/contexts/kMqdHs8ZcskdkCvf1VpfSZ/provider/padi.button/connections/geizaJngWyA1AL3Nhn5dzD/properties/sState";

#[cfg(not(target_os = "linux"))]
pub fn main() {
    panic!("This example is currently only available on Linux")
}

#[cfg(target_os = "linux")]
pub fn main() {
    use gpio_cdev::{Chip, LineRequestFlags};
    use std::time::Duration;

    // Configure pin
    let mut chip = Chip::new("/dev/gpiochip0").unwrap();
    let pin = chip.get_line(GPIO23).unwrap();
    let _pin_handle = pin.request(LineRequestFlags::OUTPUT, 0, APPNAME);

    // Connect to Arete control plane
    let _ = rustls::crypto::ring::default_provider().install_default();
    let (mut conn, _res) = arete_sdk::connect("wss://dashboard.test.cns.dev:443").unwrap();
    conn.wait_for_open(Duration::from_millis(DEFAULT_TIMEOUT_MILLIS))
        .unwrap();
    eprintln!("Connected to Arete control plane");

    // Register this node with the control plane
    conn.add_system().unwrap();
    conn.add_node(NODE_ID, APPNAME, false, None).unwrap();
    eprintln!("Registered as node {NODE_ID} on Arete control plane");

    // Detect future changes in desired state, and try to actualize it
    // TODO(https://github.com/project-arete/sdk/issues/56)

    // Startup complete
    eprintln!("Switch service started");
    loop {
        std::thread::sleep(Duration::from_secs(1));
    }
}
