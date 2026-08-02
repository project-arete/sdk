# ARETE-SDK.md — Arete SDK mechanics and known defects

> **Read [ARETE.md](https://raw.githubusercontent.com/project-arete/sdk/main/ARETE.md) first.** That file is the architecture and profile design; this one is how to drive the SDK and what currently breaks. Building from this file alone produces working code on top of a Connection Profile that may never have been a contract.
>
> `https://raw.githubusercontent.com/project-arete/sdk/main/ARETE-SDK.md`

**Applies to:** `arete-sdk` 0.1.6 (npm, crates.io); Python installed from git — not published to PyPI.
**Verified:** 2026-08-02, against `test.aretehosting.com`.

This file is version-scoped and is expected to **shrink**. Most of §3 describes defects rather than design; when the SDK is fixed those entries should be deleted rather than reworded. If you installed a pinned SDK version, read this file at the matching tag rather than at `main`.

---

## 1. Getting connected

Repo: **https://github.com/project-arete/sdk** — bindings for Node (`nodejs/`), Python (`python/`), Rust (`rust/`). The repo is authoritative for exact API names.

> **Read this before your first line of code.** The `Client` constructor derives a system ID from Raspberry Pi hardware and throws `Unable to detect System ID on this platform` everywhere else. There is no constructor option for it (§3 gotcha 6). **On Python you can work around it in two lines and it works today. On Node you cannot** — the ID is a private field set in the constructor, so off-Pi Node requires patching the installed SDK at install time. If you are starting out and not on a Pi, **start with Python.**

The identity chain is the same in every binding, and IDs must stay **stable across restarts** (ARETE.md §5):

```
Client → system() → node(id, name, upstream) → context(id, name)
       → provider(profile) | consumer(profile)     // .get / .put on properties
```

### Node

`npm install arete-sdk`, ESM only. Off a Raspberry Pi this needs the SDK patched first, see above.

```javascript
import { Client } from 'arete-sdk';

// Always pass protocol/host/port explicitly: the constructor otherwise falls
// back to browser `location` globals, which do not exist under Node.
const client = new Client({ protocol: 'wss:', host: 'test.aretehosting.com', port: 443 });
await client.waitForOpen(5000);

// Every step of the chain returns a Promise — await all four.
const system = await client.system();                    // cache this — see §3 gotcha 1
const node   = await system.node(nodeId, 'My App', false);
const ctx    = await node.context(ctxId, 'My Context');
const cap    = await ctx.provider('padi.light');         // or ctx.consumer(...)
```

Client emits `open` / `update` / `close` / `error`. `client.get(key, def)` reads the local key cache; `client.put(key, value)` is a raw key write, used for addressed per-connection writes.

### Python

`pip install "arete-sdk @ git+https://github.com/project-arete/sdk.git#subdirectory=python"`, plus `websockets>=13,<16`; needs Python 3.11+. **`pip install arete-sdk` does not work — the package is not on PyPI.**

The identity chain is the same, but **construction is not** — the client is created by a `connect` classmethod, not by calling `Client(...)`. The API is synchronous and runs a daemon receiver thread.

```python
import time
import arete_sdk.client as ac

# Off a Raspberry Pi, substitute a stable system ID before creating a client.
# Persist this value — it must not change between restarts.
ac.get_system_id = lambda: 'a7c1e2d4-5b6f-4a80-9c31-2e8f0d5b7a44'

client = ac.Client.connect('wss://test.aretehosting.com:443')
while (client.stats() or {}).get('connection') != 'online':   # see §3 gotcha 5
    time.sleep(0.25)

system = client.system()
node = system.node(node_id, 'My App', False)
ctx = node.context(ctx_id, 'My Context')
cap = ctx.consumer('padi.light')                              # or ctx.provider(...)
```

### Browser and PWA — the SDK does not run in a browser

It imports `fs`, `os`, and `ws`, and derives identity from Pi hardware. Do not bundle it for the web. The working pattern is a small browser client speaking the same wire protocol — a `WebSocket`, the same key cache, and `command()` / `put()`. Project Arete's own PWAs are built this way (`browser-arete.js` in [arete-monitor-pwa](https://github.com/project-arete/arete-monitor-pwa), the widget PWA bridge, and [Firefly Island](https://github.com/project-arete/firefly-island)); copy one rather than inventing another.

Browser identity is where this fails, and it fails *silently*:

- Generate the system ID **once per app installation** — `crypto.randomUUID()` — and persist it before connecting. Node and context IDs likewise (22-character base62).
- Persist it under a **storage key unique to that app** (`arete-monitor-identity`, `arete-widget-system`, …). Two apps sharing one fallback register as **the same system on the realm** — which presents as a brokerage fault and is not one.
- Never derive the system ID from the user-facing system name, and never ship a shared hard-coded fallback.
- Issue `systems`, `nodes`, `contexts` **one at a time, awaiting each** (§3 gotcha 7).

Browsers cannot set WebSocket headers, so when bearer tokens arrive they will not reach a browser client by the route Node uses.

## 2. Realms and auth

**Auth (SDK 0.1.6 — unsettled; expect this to change).** The `Client` constructor accepts only `protocol`, `host`, and `port`. Its `username`/`password` options exist in the source but are commented out, and the connection URI is assembled from those three values alone. The only way to pass credentials today is to fold them into the `host` string as URL userinfo — `host: 'user:pass@realm.example.com'`, URL-encoded — which is a workaround rather than a supported interface, and one that puts secrets somewhere they leak into logs and error reports. Realm-issued bearer tokens are being added; until they land, treat authentication as an open question and keep credentials out of anything you commit. Open realms accept unauthenticated connections — and a realm that requires no token is public by definition (ARETE.md §6).

**A realm to point at.** `wss://test.aretehosting.com:443` is a shared sandbox that accepts anonymous connections — fine for first experiments, but it is shared, so treat anything you put there as public and disposable. For anything beyond a first try, create your own realm at [aretehosting.com](https://aretehosting.com): it takes a couple of minutes, it is free at the time of writing, and nobody else's experiments are in it.

On a healthy realm brokerage is fast — a provider and consumer declared in the same context bind in about a second, and a broadcast write reaches the peer in well under one. If you are waiting tens of seconds, suspect the realm rather than your code.

## 3. Known defects and workarounds

Field-verified against 0.1.6. Check whether a newer release has fixed them before working around.

1. **`client.system()` overwrites your system name.** Every call re-registers the system under the local hostname (`os.hostname()`, `socket.gethostname()`) — verified live: a system you renamed reverts as soon as `system()` is called again. Cache the System instance instead of re-fetching it, and re-issue your chosen name after anything that re-registers.
2. **Capability re-declaration wipes values.** Re-issuing a `providers`/`consumers` declaration for an *existing* capability resets all its property values to empty strings — and the empties propagate into every connection. On startup or reconnect, check whether `…/<role>/<profile>/version` already exists in the key cache; if so, **skip the declaration** and operate on the existing key paths. (Values persist on the realm across disconnects — "values gone after restart" is almost always this bug, not realm data loss.) Note the interaction with ARETE.md §3.3: an empty string from this defect is indistinguishable at the consumer from a legitimately absent value.
3. **Don't use `.watch()`** (Node 0.1.6) — it has a null-match crash. Derive state from `client.keys` inside an `update` event handler instead.
4. **No keepalive, fragile reconnect.** The SDK sends no WebSocket pings and won't retry an unexpected clean close. Long-lived apps should add a ping (~30s) and reconnect logic — and on reconnect, re-attach without re-declaring capabilities (see gotcha 2).
5. **Python: don't trust `wait_for_open()`** — it can block for the full timeout on realms that never send the message it gates on. Poll `client.stats()['connection'] == 'online'` instead. (Node's `waitForOpen` works — it polls.)
6. **System ID off-Raspberry-Pi — no supported fix.** The `Client` constructor calls `get_system_id()` immediately; it reads the Pi device-tree files and otherwise throws `Unable to detect System ID on this platform`. There is no `systemId` constructor option, so this blocks ordinary Linux, macOS, Windows, containers, CI, and Electron development hosts. **Python:** rebind `arete_sdk.client.get_system_id` before creating a client — two lines, shown in §1. **Node:** the ID is a private field, so the only route is patching the installed SDK at install time (a `postinstall` script substituting a stable UUID, e.g. from an environment seed via `uuidv5`), re-applied on every `npm install`. Either way the ID must be **persisted**, not regenerated. A public identity-injection option is the right fix; treat this as an open SDK gap, not something your application configures away.
7. **Registration commands must be awaited serially.** Bursting `systems`/`nodes`/`contexts`/`providers`/`consumers` commands without awaiting each response can silently drop declarations. The SDK's own command path awaits correctly — but any hand-rolled wire client must too.
8. **Electron:** run the SDK only in the main process; expose to renderers via a preload bridge. And never `npm install` an Electron project inside a cloud-synced folder (Drive/iCloud/Dropbox) — native builds break.
9. **Registration calls are upserts — they overwrite names.** `system.node(id, name)` and `node.context(id, name)` are writes, not read-only attaches: each one sets the name you pass. Verified live — re-registering a context under a different name replaces the one the user chose. There is no attach-without-mutating call, so on reconnect either pass exactly the name you want kept, or don't re-register at all and work from the existing key paths.
10. **Python: a non-empty command response reads as a timeout.** The receiver records a response only when it equals the empty string, so any other value leaves the request unsatisfied until `wait_for_response` gives up after five seconds and raises `Timed out waiting for response`. The error branch is unreachable for the same reason — a realm-reported error can only ever surface as a timeout. Observed intermittently during registration; retry rather than assuming the command failed, and confirm against the key namespace.
11. **Applications cannot select a CP version.** No binding exposes a version parameter, and the control plane assigns version `1` — verified live against a profile with a published v2. Since profile, version and context must all match for a binding to form, a second version cannot currently be declared or bound. See ARETE.md §4.

## 4. References

| Resource | URL |
|---|---|
| **ARETE.md — architecture and CP design** | https://raw.githubusercontent.com/project-arete/sdk/main/ARETE.md |
| Arete SDK | https://github.com/project-arete/sdk |
| CP registry | https://cp.padi.io |
| Arete Hosting | https://aretehosting.com |
| Project Arete website | https://projectarete.io |

---

*Maintained in the [project-arete/sdk](https://github.com/project-arete/sdk) repository alongside [ARETE.md](ARETE.md).*
