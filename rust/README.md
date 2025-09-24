# Project Arete SDK for Rust

## Examples

- [#1: The Switch and The Light](examples/01_the_switch_and_the_light/)

## Using

```rust
let (mut client, _res) = arete_sdk::connect("wss://dashboard.test.cns.dev:443").unwrap();
client.wait_for_open(Duration::from_millis(DEFAULT_TIMEOUT_MILLIS)).unwrap();
...
```

See the [examples](#examples) for further usage details.

## Development Environment

Install Rust, then:

```shell
$ cargo add arete-sdk
```

## Testing

```shell
$ cargo test
```

## Integration Testing

```shell
$ TEST_ARETE_URL=wss://dashboard.test.cns.dev cargo test
```
