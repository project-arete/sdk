mod testsupport;

use crate::testsupport::{get_connection};

#[test]
fn can_connect() {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let _conn = get_connection!();
}

#[test]
fn can_get_a_key() {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let (conn, _res) = get_connection!();
    let _value = conn.get("abc", None);
}

#[test]
fn can_get_all_keys() {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let (conn, _res) = get_connection!();
    let _keys = conn.keys();
}
