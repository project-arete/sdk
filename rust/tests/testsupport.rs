#[allow(unused)]
macro_rules! get_connection {
    () => {{
        const TEST_ARETE_URL: &str = "TEST_ARETE_URL";
        let arete_url = match std::env::var(TEST_ARETE_URL) {
            Ok(url) => url,
            _ => {
                eprintln!("Skipping test because no {TEST_ARETE_URL} is set");
                return;
            }
        };
        arete_sdk::connect(&arete_url).unwrap()
    }};
}

#[allow(unused)]
pub(crate) use get_connection;
