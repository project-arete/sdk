#![allow(unused)]

const APPNAME: &str = "arete-sdk-01-switch";
const DEFAULT_TIMEOUT_MILLIS: u64 = 500;
const GPIO04: u32 = 4;
const NODE_ID: &str = "ozr9fZbU8i7hMdjEjuTS2o";
const CONTEXT_ID: &str = "aRDhTq6wLUWd8X3xjRMkp4";
const CONTEXT_NAME: &str = "Building 4, 3rd floor, Office 23-B";
const DESIRED_STATE_KEY: &str = "cns/network/nodes/sri4FZUq2V7S4ik2PrG4pj/contexts/kMqdHs8ZcskdkCvf1VpfSZ/provider/padi.button/connections/geizaJngWyA1AL3Nhn5dzD/properties/sState";

#[cfg(not(target_os = "linux"))]
pub fn main() {
    panic!("This example is currently only available on Linux")
}

#[cfg(target_os = "linux")]
pub fn main() {
    use gpio_cdev::{Chip, EventRequestFlags, EventType, LineRequestFlags};
    use std::time::Duration;

    // Configure pin
    let mut chip = Chip::new("/dev/gpiochip0").unwrap();
    let pin = chip.get_line(GPIO04).unwrap();

    // Connect to Arete control plane
    let _ = rustls::crypto::ring::default_provider().install_default();
    let (mut conn, _res) = arete_sdk::connect("wss://dashboard.test.cns.dev:443").unwrap();
    conn.wait_for_open(Duration::from_millis(DEFAULT_TIMEOUT_MILLIS))
        .unwrap();
    eprintln!("Connected to Arete control plane");

    // Register this node with the control plane
    conn.add_node(NODE_ID, APPNAME, false, None).unwrap();
    eprintln!("Registered as node {NODE_ID} with Arete control plane");

    // Register a context for this node
    conn.add_context(NODE_ID, CONTEXT_ID, CONTEXT_NAME, None).unwrap();
    eprintln!("Registered context {CONTEXT_ID} for node {NODE_ID} with Arete control plane");

    // Read initial switch state, and sync it with Arete
    let line_request_flags = LineRequestFlags::INPUT | LineRequestFlags::ACTIVE_LOW;
    let state = pin
        .request(line_request_flags.clone(), 0, APPNAME)
        .unwrap()
        .get_value()
        .unwrap()
        > 0;
    conn.put(DESIRED_STATE_KEY, if state { "1" } else { "0" });
    eprintln!("Switch is initially {}", if state { "ON" } else { "OFF" });

    // Startup complete
    eprintln!("Switch service started");

    // Detect future changes in switch state, and sync it with Arete
    let mut pin_events = pin
        .events(line_request_flags, EventRequestFlags::BOTH_EDGES, APPNAME)
        .unwrap();
    loop {
        let event = pin_events.get_event().unwrap();
        let state = event.event_type() == EventType::FallingEdge;
        conn.put(DESIRED_STATE_KEY, if state { "1" } else { "0" });
        eprintln!("Switch is now {}", if state { "ON" } else { "OFF" });
    }
}
