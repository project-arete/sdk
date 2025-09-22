#![allow(unused)]

const DEFAULT_TIMEOUT_MILLIS: u64 = 500;
const GPIO23: u32 = 23;

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

    // Connect to Arete control plane
    let _ = rustls::crypto::ring::default_provider().install_default();
    let (mut client, _res) = arete_sdk::connect("wss://dashboard.test.cns.dev:443").unwrap();
    client
        .wait_for_open(Duration::from_millis(DEFAULT_TIMEOUT_MILLIS))
        .unwrap();

    // Register with the control plane
    let system = client.system().unwrap();
    let node = system.node(NODE_ID, NODE_NAME, false, None).unwrap();
    let context = node.context(CONTEXT_ID, CONTEXT_NAME).unwrap();

    // Read initial actual state of the light, and sync it with Arete
    let state = pin
        .request(LineRequestFlags::INPUT | LineRequestFlags::ACTIVE_LOW, 0, NODE_NAME)
        .unwrap()
        .get_value()
        .unwrap()
        > 0;
    let consumer = context.consumer(PADI_LIGHT_PROFILE).unwrap();
    consumer.put("cState", if state { "1" } else { "0" }).unwrap();
    eprintln!("Light is initially {}", if state { "ON" } else { "OFF" });

    // Detect future changes to desired state, and try to actualize it
    std::thread::spawn(move || {
        let updates_rx = consumer.watch().unwrap();
        loop {
            let event = updates_rx.recv().unwrap();
            if event.property == "sOut" {
                let desired_state = match event.value {
                    Value::String(value) => value == "1",
                    _ => false,
                };
                let pin_handle = pin.request(LineRequestFlags::OUTPUT, 0, NODE_NAME).unwrap();
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
