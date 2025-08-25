mod testsupport;

use crate::testsupport::{get_connection};

#[test]
fn can_connect() {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let _connection = get_connection!();
}
