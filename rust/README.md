# Project Arete SDK for Rust

## Installing

Add to your Rust project with:

```shell
$ cargo add arete-sdk
```

## Using

```rust
let (mut client, _res) = arete_sdk::connect("wss://dashboard.test.cns.dev:443").unwrap();
client.wait_for_open(Duration::from_millis(DEFAULT_TIMEOUT_MILLIS)).unwrap();
...
```

See the [examples](#examples) for further usage details.

## Examples

- [#1: The Switch and The Light](examples/01_the_switch_and_the_light/)

## Developing

See the [Developer's Guide](DEVELOPING.md) for build and test instructions.
