# ARETE.md — Building applications on CNS/CP with the Arete SDK

> **What this file is.** A single-file briefing for anyone — human or AI agent — building applications on the CNS/CP architecture using the Arete SDK. Drop it into your project root, or point your coding agent at the canonical copy:
>
> `https://raw.githubusercontent.com/project-arete/sdk/main/ARETE.md`
>
> Read it fully before writing code. The architecture has a few load-bearing distinctions that, if violated, produce plausible-looking code that is wrong.

---

## 1. Mental model (read this even if you skip everything else)

**CNS/CP is a Layer 0 governance substrate.** It is not a communication protocol, not an agent framework, not an orchestration engine. It governs *whether* two nodes should connect, under what contract, and in what context — before any data flows.

You do not write "connect A to B." You write: A declares it can **provide** `cp:x`; B declares it **consumes** `cp:x`; the orchestrator (Arete) **brokers** the match and binds them if policy and context permit. Your application logic lives in *what to declare* and *what to do once bound* — never in *how to reach an endpoint*.

Core vocabulary — use these words precisely:

| Term | Meaning |
|---|---|
| **CNS** | Connectivity Naming System — resolution/naming layer ("DNS for governed connectivity") |
| **CP** | Connection Profile — a machine-evaluable contract for one interaction pattern |
| **Orchestration** | The "What" — workflow intent, system-level policy (Arete's job) |
| **Brokerage** | The "Who" — one matching action: consumer ↔ qualified provider via CP evaluation |
| **Provider / Consumer** | The exactly-two asymmetric roles every CP defines (wire terms: `server` / `client`) |
| **Mode 1 (in-band)** | Data flows through the CP network — mediated, auditable |
| **Mode 2 (out-of-band)** | CNS/CP governs the binding, then steps aside; data flows peer-to-peer |

The mode lives **in the CP definition**, not in a config flag. Read it from the CP and handle accordingly.

**Everything is deny-by-default.** Identity, roles, and a matching CP are required before anything binds. Never design an app that opens access and filters later.

## 2. Non-negotiables

1. **Never connect by address.** Connect by declared CP + role + context. A hardcoded endpoint in app logic is the anti-pattern this architecture exists to replace.
2. **Every capability is a CP** with a canonical name (`cp:usecase.name`), exactly two asymmetric roles, a schema, constraints, context, and a deliberate mode choice. Design the CP before writing handlers.
3. **Resolve every CP from the registry before use** (Section 3). If it isn't registered, stop — do not invent the profile.
4. **Conflicting values across multiple connections are app-level semantics, by design.** The mechanism delivers every connection's view faithfully; it never imposes a conflict policy. If your consumer binds to N providers, *your app* decides how to aggregate (average, min, max, last-writer…). Do not "fix" this in SDK/orchestrator code — surface disagreement honestly and resolve it in the application.
5. **Context is first-class.** A binding exists *in* a context; carry it, don't let authority drift across trust domains.
6. **Assume every action is audited**, especially in Mode 1.

## 3. The CP registry — `cp.padi.io`

Before declaring a provider or consumer for any CP, fetch its definition:

```
GET https://cp.padi.io/profiles/<cp-name>      (Accept: application/json)
```

Fetch the **raw JSON** — property flags are encoded by *key presence*, and summarized/rendered views lose them:

- `server` present → the **provider** writes this property; absent → the consumer writes it. Properties are named for **purpose**, never with direction prefixes — the `server` flag is the authority on who writes.
- `propagate` present → capability-level writes are **broadcast** into all active connections. Absent → not broadcast, but still usable via the **addressed channel**: any declared property can be written directly into one specific connection (`…/connections/<id>/properties/<prop>`), and the orchestrator mirrors it 1:1 to that peer only. Canonical case: a response property addressed back to the requester.
- `required` present → the property is required.

Registry governance: published CPs are **immutable** — design deliberately before publishing. The `versions` array takes **additive** changes only; a functional change means a **new CP name**. Use the `padi.test.*` namespace for development CPs. Unregistered ("local") profile names will not bind on a live realm — the control plane can't produce matchable version keys for a profile it can't resolve.

## 4. SDK quickstart

Repo: **https://github.com/project-arete/sdk** — bindings for Node (`nodejs/`), Python (`python/`), Rust (`rust/`). The repo is authoritative for exact API names; this file is authoritative for architecture and conventions.

The identity chain is the same in every binding, and IDs must stay **stable across restarts**:

```
Client → system() → node(id, name, upstream) → context(id, name)
       → provider(profile) | consumer(profile)     // .get / .put on properties
```

**Node** (`npm install arete-sdk`, ESM only):

```javascript
import { Client } from 'arete-sdk';
const client = new Client({ protocol: 'wss:', host: 'dashboard.test.cns.dev', port: 443 });
await client.waitForOpen(5000);
const system = await client.system();          // cache this — see gotcha 5.1
const node = system.node(nodeId, 'My App', false);
const ctx = node.context(ctxId, 'My Context');
const cap = ctx.provider('padi.light');        // or ctx.consumer(...)
```

Client emits `open` / `update` / `close` / `error`. `client.get(key, def)` reads the local key cache; `client.put(key, value)` is a raw key write (used for addressed per-connection writes).

**Python** (install from git: `pip install "arete-sdk @ git+https://github.com/project-arete/sdk.git#subdirectory=python"`): synchronous API mirroring the above; the client runs a daemon receiver thread.

**Auth:** credentials go in the WebSocket URL userinfo (`wss://user:pass@host:443`, URL-encoded). Test realms may accept unauthenticated connections.

**Public test realm:** `wss://dashboard.test.cns.dev:443` (no auth). Brokerage can take tens of seconds — use generous timeouts in test rigs.

## 5. Known gotchas (as of SDK v0.1.6, verified live July 2026)

These are field-verified. Check whether newer SDK releases have fixed them before working around.

1. **`client.system()` is not idempotent.** Every call re-registers the system under `os.hostname()`. Cache the System instance; if you rename the system, re-issue the rename after any re-registration.
2. **Capability re-declaration wipes values.** Re-issuing a `providers`/`consumers` declaration for an *existing* capability resets all its property values to empty strings — and the empties propagate into every connection. On startup/reconnect, check whether `…/<role>/<profile>/version` already exists in the key cache; if so, **skip the declaration** and operate on the existing key paths. (Values persist on the realm across disconnects — "values gone after restart" is almost always this bug, not realm data loss.)
3. **Don't use `.watch()`** (Node v0.1.6) — it has a null-match crash. Derive state from `client.keys` inside an `update` event handler instead.
4. **No keepalive, fragile reconnect.** The SDK sends no WebSocket pings and won't retry an unexpected clean close. Long-lived apps should add a ping (~30s) and reconnect logic — and on reconnect, re-attach without re-declaring capabilities (see gotcha 2).
5. **Python: don't trust `wait_for_open()`** — it can block for the full timeout on realms that never send the message it gates on. Poll `client.stats()['connection'] == 'online'` instead. (Node's `waitForOpen` works — it polls.)
6. **System ID off-Raspberry-Pi.** The SDK derives the system ID from Pi hardware; on other machines provide a stable substitute (e.g. a UUID from an env seed) before constructing the client.
7. **Registration commands must be awaited serially.** Bursting `systems`/`nodes`/`contexts`/`providers`/`consumers` commands without awaiting each response can silently drop declarations. The SDK's own command path awaits correctly — but any hand-rolled wire client must too.
8. **Electron:** run the SDK only in the main process; expose to renderers via a preload bridge. And never `npm install` an Electron project inside a cloud-synced folder (Drive/iCloud/Dropbox) — native builds break.

## 6. Designing a new capability — checklist

1. What is the CP? Name it `usecase.name`, no direction prefixes on properties.
2. Define the two roles and which side writes each property (`server` flag).
3. Decide broadcast vs addressed per property (`propagate` flag) and Mode 1 vs Mode 2 — deliberately.
4. Register in `padi.test.*` first; iterate there (published CPs are immutable).
5. Plan multi-connection semantics **in the app** (aggregation, conflict display) — expect a consumer to face N providers.
6. Then, and only then, write the handlers.

## 7. References

| Resource | URL |
|---|---|
| Arete SDK (Node, Python, Rust) | https://github.com/project-arete/sdk |
| Project Arete GitHub org | https://github.com/project-arete |
| Project Arete website | https://projectarete.io |
| CNS/CP website | https://cnscp.io |
| CNS/CP specification (incl. licensing) | https://github.com/CNSCP/specification |
| CP registry | https://cp.padi.io |
| Widget library (example widgets) | https://github.com/project-arete/widget-library |
| USPTO Patent No. 12,519,860 | https://patents.google.com/patent/US12519860B2/en |

---

*This file is maintained in the [project-arete/sdk](https://github.com/project-arete/sdk) repository. The repo is authoritative for API surface; the [CNS/CP specification](https://github.com/CNSCP/specification) is authoritative for the architecture.*
