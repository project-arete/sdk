# Project Arete SDK for NodeJS

## Installing

Add to your NodeJS project using the latest release published to NPM with:

```shell
$ npm install --save arete-sdk
```

Or install the local Git workspace with:

```shell
$ npm install
```

## Using

```javascript
import { Client } from 'arete-sdk';
let client = new Client({
  protocol: 'wss:',
  host: 'dashboard.test.cns.dev',
  port: 443,
});
await client.waitForOpen(5000);
...
```

See the [examples](#examples) for further usage details.

## Examples

- [#1: The Switch and The Light](examples/01_the_switch_and_the_light/)

## Developing

See the [Developer's Guide](DEVELOPING.md) for build and test instructions.
