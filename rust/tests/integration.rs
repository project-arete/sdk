mod testsupport;

use crate::testsupport::get_connection;
use std::time::Duration;

const DEFAULT_TIMEOUT_MILLIS: u64 = 500;

#[test]
fn can_connect() {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let _conn = get_connection!();
}

#[test]
fn can_get_a_key() {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let (client, _res) = get_connection!();
    client.wait_for_open(Duration::from_millis(DEFAULT_TIMEOUT_MILLIS))
        .unwrap();
    let _value = client.get("abc", None);
}

#[test]
fn can_get_stats() {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let (client, _res) = get_connection!();
    client.wait_for_open(Duration::from_millis(DEFAULT_TIMEOUT_MILLIS))
        .unwrap();
    let _stats = client.stats().unwrap();
}

#[test]
fn can_get_version() {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let (client, _res) = get_connection!();
    client.wait_for_open(Duration::from_millis(DEFAULT_TIMEOUT_MILLIS))
        .unwrap();
    let version = client.version().unwrap();
    assert!(!version.is_empty());
}

#[test]
fn can_get_all_keys() {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let (client, _res) = get_connection!();
    client.wait_for_open(Duration::from_millis(DEFAULT_TIMEOUT_MILLIS))
        .unwrap();
    let _keys = client.keys();
}
