# Project Arete SDK for Python

## Examples

- [#1: The Switch and The Light](examples/01_the_switch_and_the_light/)

## Installing

```shell
$ python3 -m pip install arete-sdk
```

## Using

```python
import ssl
from arete_sdk import Client

ssl_context = ssl.SSLContext(protocol=ssl.PROTOCOL_TLS_CLIENT)
ssl_context.check_hostname = False
ssl_context.verify_mode = ssl.CERT_NONE
client = Client.connect('wss://dashboard.test.cns.dev:443', ssl=ssl_context)
client.wait_for_open()
...
```

See the [examples](#examples) for further usage details.

## Developing

See the [Developer's Guide](DEVELOPING.md) for build and test instructions.
