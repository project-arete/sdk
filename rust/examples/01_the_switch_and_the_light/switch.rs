#![allow(unused)]

const DEFAULT_TIMEOUT_MILLIS: u64 = 500;
const GPIO04: u32 = 4;

const CONTEXT_ID: &str = "uRLoYsXEY7nsbs9fRdjM8A";
const CONTEXT_NAME: &str = "Building 23, Office 41-B";

const NODE_ID: &str = "ozr9fZbU8i7hMdjEjuTS2o";
const NODE_NAME: &str = "arete-sdk-01-switch";

const PADI_LIGHT_PROFILE: &str = "padi.light";

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

    // Register as a provider of state for the "padi.light" profile
    client.add_provider(NODE_ID, CONTEXT_ID, PADI_LIGHT_PROFILE);
    eprintln!("Registered as provider of state for {PADI_LIGHT_PROFILE} profile for context {CONTEXT_ID}");

    // Read initial switch state, and sync it with Arete
    let line_request_flags = LineRequestFlags::INPUT | LineRequestFlags::ACTIVE_LOW;
    let state = pin
        .request(line_request_flags.clone(), 0, NODE_NAME)
        .unwrap()
        .get_value()
        .unwrap()
        > 0;
    client.put_property(
        NODE_ID,
        CONTEXT_ID,
        PADI_LIGHT_PROFILE,
        "sOut",
        if state { "1" } else { "0" },
    );
    eprintln!("Switch is initially {}", if state { "ON" } else { "OFF" });

    // Startup complete
    eprintln!("Switch service started");

    // Detect future changes in switch state, and sync it with Arete
    let mut pin_events = pin
        .events(line_request_flags, EventRequestFlags::BOTH_EDGES, NODE_NAME)
        .unwrap();
    loop {
        let event = pin_events.get_event().unwrap();
        let state = event.event_type() == EventType::FallingEdge;
        client.put_property(
            NODE_ID,
            CONTEXT_ID,
            PADI_LIGHT_PROFILE,
            "sOut",
            if state { "1" } else { "0" },
        );
        eprintln!("Switch is now {}", if state { "ON" } else { "OFF" });
    }
}
