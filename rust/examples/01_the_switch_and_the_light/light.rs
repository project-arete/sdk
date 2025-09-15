#![allow(unused)]

const DEFAULT_TIMEOUT_MILLIS: u64 = 500;
const GPIO23: u32 = 23;
const DESIRED_STATE_KEY: &str = "cns/network/nodes/sri4FZUq2V7S4ik2PrG4pj/contexts/kMqdHs8ZcskdkCvf1VpfSZ/provider/padi.button/connections/geizaJngWyA1AL3Nhn5dzD/properties/sState";

const NODE_ID: &str = "onqXVczGoymQkFc3UN6qcM";
const NODE_NAME: &str = "arete-sdk-01-light";

const CONTEXT_ID: &str = "uRLoYsXEY7nsbs9fRdjM8A";
const CONTEXT_NAME: &str = "Building 23, Office 41-B";

const PADI_LIGHT_PROFILE: &str = "padi.light";

#[cfg(not(target_os = "linux"))]
pub fn main() {
    panic!("This example is currently only available on Linux")
}

#[cfg(target_os = "linux")]
pub fn main() {
    use gpio_cdev::{Chip, LineRequestFlags};
    use serde_json::Value;
    use std::time::Duration;

    // Configure pin
    let mut chip = Chip::new("/dev/gpiochip0").unwrap();
    let pin = chip.get_line(GPIO23).unwrap();
    let pin_handle = pin.request(LineRequestFlags::OUTPUT, 0, NODE_NAME).unwrap();

    // Connect to Arete control plane
    let _ = rustls::crypto::ring::default_provider().install_default();
    let (mut client, _res) = arete_sdk::connect("wss://dashboard.test.cns.dev:443").unwrap();
    client
        .wait_for_open(Duration::from_millis(DEFAULT_TIMEOUT_MILLIS))
        .unwrap();
    eprintln!("Connected to Arete control plane");

    // Register this node and its context with the control plane
    let system = client.system().unwrap();
    let node = system.node(NODE_ID, NODE_NAME, false, None).unwrap();
    eprintln!("Registered as node {NODE_ID}");
    let _context = node.context(CONTEXT_ID, CONTEXT_NAME).unwrap();
    eprintln!("Registered context {CONTEXT_ID} for node {NODE_ID}");

    // Register as a consumer of state for the "padi.light" profile
    client.add_consumer(NODE_ID, CONTEXT_ID, PADI_LIGHT_PROFILE);
    eprintln!("Registered as consumer of state for {PADI_LIGHT_PROFILE} profile for context {CONTEXT_ID}");

    // Realize initial desired state
    if let Some(value) = client.get(DESIRED_STATE_KEY, Some("0".into())).unwrap() {
        let desired_state = match value {
            Value::String(value) => value == "1",
            _ => false,
        };
        pin_handle.set_value(if desired_state { 1 } else { 0 }).unwrap();
        eprintln!("Light is initially {}", if desired_state { "ON" } else { "OFF" });
    }

    // Detect future changes to desired state, and try to actualize it
    let updates_rx = client.on_update().unwrap();
    std::thread::spawn(move || {
        loop {
            let event = updates_rx.recv().unwrap();
            if let Some(value) = event.keys.get(DESIRED_STATE_KEY) {
                let desired_state = match value {
                    Value::String(value) => value == "1",
                    _ => false,
                };
                pin_handle.set_value(if desired_state { 1 } else { 0 }).unwrap();
                eprintln!("Light is now {}", if desired_state { "ON" } else { "OFF" });
            }
        }
    });

    // Startup complete
    eprintln!("Light service started");
    loop {
        std::thread::sleep(Duration::from_secs(1));
    }
}
