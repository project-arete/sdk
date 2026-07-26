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
4. **Conflicting values across multiple connections are app-level semantics, by design.** The mechanism delivers every connection's view faithfully; it never imposes a conflict policy. When one of your capabilities is bound to N peers — and either role can be, see §6 — *your app* decides how to combine what it receives (average, min, max, logical OR, sum…). Do not "fix" this in SDK/orchestrator code — surface disagreement honestly and resolve it in the application.
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

> **Read this before your first line of code.** The `Client` constructor derives a system ID from Raspberry Pi hardware and throws `Unable to detect System ID on this platform` everywhere else. There is no constructor option for it (§7 gotcha 6). **On Python you can work around it in two lines and it works today. On Node you cannot** — the ID is a private field set in the constructor, so off-Pi Node requires patching the installed SDK at install time. If you are starting out and not on a Pi, **start with Python.**

The identity chain is the same in every binding, and IDs must stay **stable across restarts**:

```
Client → system() → node(id, name, upstream) → context(id, name)
       → provider(profile) | consumer(profile)     // .get / .put on properties
```

**Node** (`npm install arete-sdk`, ESM only) — off a Raspberry Pi this needs the SDK patched first, see above:

```javascript
import { Client } from 'arete-sdk';

// Always pass protocol/host/port explicitly: the constructor otherwise falls
// back to browser `location` globals, which do not exist under Node.
const client = new Client({ protocol: 'wss:', host: 'test.aretehosting.com', port: 443 });
await client.waitForOpen(5000);

// Every step of the chain returns a Promise — await all four.
const system = await client.system();                    // cache this — see §7 gotcha 1
const node   = await system.node(nodeId, 'My App', false);
const ctx    = await node.context(ctxId, 'My Context');
const cap    = await ctx.provider('padi.light');         // or ctx.consumer(...)
```

Client emits `open` / `update` / `close` / `error`. `client.get(key, def)` reads the local key cache; `client.put(key, value)` is a raw key write (used for addressed per-connection writes).

**Python** (`pip install "arete-sdk @ git+https://github.com/project-arete/sdk.git#subdirectory=python"`, plus `websockets>=13,<16`; needs Python 3.11+). The identity chain is the same, but **construction is not** — the client is created by a `connect` classmethod, not by calling `Client(...)`. The API is synchronous and runs a daemon receiver thread.

```python
import time
import arete_sdk.client as ac

# Off a Raspberry Pi, substitute a stable system ID before creating a client.
# Persist this value — it must not change between restarts.
ac.get_system_id = lambda: 'a7c1e2d4-5b6f-4a80-9c31-2e8f0d5b7a44'

client = ac.Client.connect('wss://test.aretehosting.com:443')
while (client.stats() or {}).get('connection') != 'online':   # see §7 gotcha 5
    time.sleep(0.25)

system = client.system()
node = system.node(node_id, 'My App', False)
ctx = node.context(ctx_id, 'My Context')
cap = ctx.consumer('padi.light')                              # or ctx.provider(...)
```

**Browser and PWA — the SDK does not run in a browser.** It imports `fs`, `os`, and `ws`, and derives identity from Pi hardware. Do not bundle it for the web. The working pattern is a small browser client speaking the same wire protocol — a `WebSocket`, the same key cache, and `command()` / `put()`. Project Arete's own PWAs are built this way (`browser-arete.js` in [arete-monitor-pwa](https://github.com/project-arete/arete-monitor-pwa), the widget PWA bridge, and [Firefly Island](https://github.com/project-arete/firefly-island)); copy one rather than inventing another.

Browser identity is where this fails, and it fails *silently*:

- Generate the system ID **once per app installation** — `crypto.randomUUID()` — and persist it before connecting. Node and context IDs likewise (22-character base62).
- Persist it under a **storage key unique to that app** (`arete-monitor-identity`, `arete-widget-system`, …). Two apps sharing one fallback register as **the same system on the realm** — which presents as a brokerage fault and is not one.
- Never derive the system ID from the user-facing system name, and never ship a shared hard-coded fallback.
- Issue `systems`, `nodes`, `contexts` **one at a time, awaiting each** (§7 gotcha 7).

Browsers cannot set WebSocket headers, so when bearer tokens arrive they will not reach a browser client by the route Node uses.

**Auth (SDK 0.1.6 — unsettled; expect this to change).** The `Client` constructor accepts only `protocol`, `host`, and `port`. Its `username`/`password` options exist in the source but are commented out, and the connection URI is assembled from those three values alone. The only way to pass credentials today is to fold them into the `host` string as URL userinfo — `host: 'user:pass@realm.example.com'`, URL-encoded — which is a workaround rather than a supported interface, and one that puts secrets somewhere they leak into logs and error reports. Realm-issued bearer tokens are being added; until they land, treat authentication as an open question and keep credentials out of anything you commit. Open realms, including the sandbox below, accept unauthenticated connections — and a realm that requires no token is public by definition (§6).

**A realm to point at.** `wss://test.aretehosting.com:443` is a shared sandbox that accepts anonymous connections — fine for first experiments, but it is shared, so treat anything you put there as public and disposable. For anything beyond a first try, create your own realm at [aretehosting.com](https://aretehosting.com): it takes a couple of minutes, it is free at the time of writing, and nobody else's experiments are in it.

On a healthy realm brokerage is fast — a provider and consumer declared in the same context bind in about a second, and a broadcast write reaches the peer in well under one. If you are waiting tens of seconds, suspect the realm rather than your code.

## 5. Bootstrapping an application into a realm

Declaring a capability is not the first thing an application does — it is the third. Every app crosses the same gates in the same order, each a precondition for the next. User-facing apps should surface them in this order rather than bury them in a config file: a user who cannot get past gate 2 otherwise has no way to understand why nothing is happening.

**Gate 1 — realm and identity.** Collect the realm address and credentials, then register a system and a node. You supply the IDs, and they **must be stable across restarts** — persist them; never generate them at startup. The SDK names the system after `os.hostname()` unless you rename it (§7 gotcha 1), so let the user set the name it will carry on the realm.

Six identifiers, and they are not interchangeable:

| Identifier | What it scopes | Persist | User-editable |
|---|---|---|---|
| System ID | one installation of one app | Yes | No |
| System name | its realm-facing label | Yes | Yes |
| Node ID | an application instance within that system | Yes | No |
| Node name | that instance's label | Yes | Yes |
| Context ID | the binding scope, shared across systems | Yes | No |
| Context name | a local human label, per system | Yes | Yes |

Two apps on one device are normally **two systems**, not two nodes under one — a system is an installation boundary. Project Arete's own PWAs each hold their own system identity under their own storage key, for exactly that reason.

**Gate 2 — context.** A capability is declared *in* a context. There is no default and no global scope, so the app cannot proceed without one. Existing contexts are discoverable from the key cache — `cns/<sys>/nodes/<node>/contexts/<ctxId>/name` — and so are the capabilities declared in them: `…/contexts/<ctxId>/<provider|consumer>/<profile>/version`. Together those answer the only question that matters: *is there a context that already holds the complementary role for a CP I implement?*

> **Enumerate what you can see, and don't assume that is everything.** Visibility is granted by the realm, not by the mechanism (§6). A realm requiring no token is public and exposes its whole namespace to any client; with tokens the default becomes privacy within the realm, and an individual context may carry its own token.

Four cases follow, and a UI should handle all four:

- **A complementary context exists** — offer *join*, and make it the default. This is the case that actually produces connections.
- **Contexts exist, but the user wants a separate one** — offer *create* alongside them.
- **Nothing complementary exists** — *create* is the only option; prompt for a name.
- **A protected context** — one carrying its own token is **granted, not discovered**. It will not arrive through the picker, so accept its token as a way in rather than assuming every joinable context is visible.

**Discovery is not a one-shot scan.** Other applications register contexts and capabilities continuously, so a context can appear moments after you looked — a picker that scans once and stops will miss the partner that was still starting up. Keep the list live for as long as the user is choosing from it.

**Take the context's display name from `…/contexts/<ctxId>/name` and nowhere else.** CP properties such as `sLabel` or `cLabel` are capability data whose meaning is specific to that profile; they are not context metadata, and a picker that borrows them will mislabel rows the moment a different CP appears.

**The context ID is what matches; the name is only a label.** Binding happens within a context ID, and different systems routinely name the same context differently — realm-wide views group by ID and display the most common name variant. Hence the trap: two users who both choose *create* and both type "Kitchen" get two different IDs and will never bind, while the UI shows what looks like a match. Defaulting to *join* is what prevents this, which makes it load-bearing rather than a convenience.

**Gate 3 — declaration.** Declare provider or consumer for each CP the app implements, in the chosen context. Resolve every profile from the registry first (§3), and on restart do not blindly re-declare (§7 gotcha 2).

**Creating a context is not declaring a capability.** Registering a context makes it exist; nothing can match it until you also declare your role there. A UI that says "created" at gate 2 invites the user to expect an immediate connection and then look for the fault. Show it as *created, not yet published*, and do not count it as compatible — yours or anyone else's — until the declaration has landed.

**Gate 4 — bind.** Declared is not connected. A capability is **bound** once it has at least one connection (`…/<role>/<profile>/connections/<connId>/…`) and **unbound — awaiting broker** while it has none. Unbound is a normal state, not an error — a capability with no counterpart yet simply waits. On a healthy realm binding takes about a second, but it is never instantaneous, so show the state explicitly with a short grace period before drawing attention to it. Otherwise users assume the setup they just completed has failed.

## 6. Working with multiple connections

A capability is not a socket. One declaration can be bound to many peers at once, each connection carrying its own independent view of the properties, and that is the normal case rather than an edge case. This is the part of CNS/CP with no analogue in the protocols you already know, so it is where inherited instincts do the most damage: pub/sub habits produce code that collapses N peers into a single value and quietly discards the thing that mattered.

**Two flags, four behaviours.** `server` decides *who writes* a property; `propagate` decides *whether that write broadcasts*. They are independent, and all four combinations occur in published CPs:

| | `propagate` present — **broadcast** to every connection | `propagate` absent — **addressed** to one connection |
|---|---|---|
| **`server` present** — provider writes | `padi.light` → `sOut` | `padi.game.beacon` → `granted` |
| **`server` absent** — consumer writes | `padi.light` → `cState` | `padi.game.beacon` → `feed` |

`propagate` has nothing to do with being a provider. Either role may own writable properties, and either role's writes may broadcast. Read both flags for every property; never infer one from the role. (`padi.test.propagate` exists to demonstrate this — it carries all four combinations in a single profile.)

**Reading: every side can face many.** A consumer bound to three providers receives three values for a provider-written property. A `padi.light` switch bound to three lights is a *provider* receiving three `cState` values. Same problem, either role. Per-connection values live at `…/<role>/<profile>/connections/<connId>/properties/<prop>`, and a connection carries both sides' properties mirrored onto both endpoints.

The mechanism will never resolve disagreement for you (§2 rule 4) — it delivers each connection's view faithfully and stops there. Choices taken by working applications: **average, minimum, or maximum** where values are numeric and a summary is meaningful; a **logical OR** where any peer asserting is sufficient, so a light stays lit while anyone is still holding it and tug-of-war is legal by design; a **sum** where contributions accumulate. Choose deliberately and write the choice down, because it *is* your application's semantics.

**Writing: broadcast or address.** Writing to the capability property broadcasts to every connection — for a property flagged `propagate`. Writing to `…/connections/<connId>/properties/<prop>` reaches exactly one peer, which is how a response is returned to whoever asked. Both are available to whichever role owns the property.

**Show the connections; do not collapse them.** The substrate always knows which peer contributed what, so a UI that renders only the aggregate is discarding information it was handed. The working pattern is one visual token per connection — a pill, a dot, a bead in the peer's colour — carrying that connection's live values, alongside a separate token for the broadcast or aggregate view. When a user asks "but who is doing this?", the answer is already in the data.

**Visibility is realm-governed.** Addressed means *delivered to one peer*. It does not mean *hidden from others*: an addressed write lands in a key like any other, and who may read that key is decided by the realm. A realm requiring no token is public. With tokens, the default is privacy within the realm, and a context may carry its own token so it can be protected individually. Treat addressing as delivery, never as confidentiality — let the realm's policy, not a property flag, tell you who can see a value.

## 7. Known gotchas (as of SDK v0.1.6, verified live July 2026)

These are field-verified. Check whether newer SDK releases have fixed them before working around.

1. **`client.system()` overwrites your system name.** Every call re-registers the system under the local hostname (`os.hostname()`, `socket.gethostname()`) — verified live: a system you renamed reverts as soon as `system()` is called again. Cache the System instance instead of re-fetching it, and re-issue your chosen name after anything that re-registers.
2. **Capability re-declaration wipes values.** Re-issuing a `providers`/`consumers` declaration for an *existing* capability resets all its property values to empty strings — and the empties propagate into every connection. On startup/reconnect, check whether `…/<role>/<profile>/version` already exists in the key cache; if so, **skip the declaration** and operate on the existing key paths. (Values persist on the realm across disconnects — "values gone after restart" is almost always this bug, not realm data loss.)
3. **Don't use `.watch()`** (Node v0.1.6) — it has a null-match crash. Derive state from `client.keys` inside an `update` event handler instead.
4. **No keepalive, fragile reconnect.** The SDK sends no WebSocket pings and won't retry an unexpected clean close. Long-lived apps should add a ping (~30s) and reconnect logic — and on reconnect, re-attach without re-declaring capabilities (see gotcha 2).
5. **Python: don't trust `wait_for_open()`** — it can block for the full timeout on realms that never send the message it gates on. Poll `client.stats()['connection'] == 'online'` instead. (Node's `waitForOpen` works — it polls.)
6. **System ID off-Raspberry-Pi — no supported fix.** The `Client` constructor calls `get_system_id()` immediately; it reads the Pi device-tree files and otherwise throws `Unable to detect System ID on this platform`. There is no `systemId` constructor option, so this blocks ordinary Linux, macOS, Windows, containers, CI, and Electron development hosts. **Python:** rebind `arete_sdk.client.get_system_id` before creating a client — two lines, shown in §4. **Node:** the ID is a private field, so the only route is patching the installed SDK at install time (a `postinstall` script substituting a stable UUID, e.g. from an environment seed via `uuidv5`), re-applied on every `npm install`. Either way the ID must be **persisted**, not regenerated. A public identity-injection option is the right fix; treat this as an open SDK gap, not something your application configures away.
7. **Registration commands must be awaited serially.** Bursting `systems`/`nodes`/`contexts`/`providers`/`consumers` commands without awaiting each response can silently drop declarations. The SDK's own command path awaits correctly — but any hand-rolled wire client must too.
8. **Electron:** run the SDK only in the main process; expose to renderers via a preload bridge. And never `npm install` an Electron project inside a cloud-synced folder (Drive/iCloud/Dropbox) — native builds break.
9. **Registration calls are upserts — they overwrite names.** `system.node(id, name)` and `node.context(id, name)` are writes, not read-only attaches: each one sets the name you pass. Verified live — re-registering a context under a different name replaces the one the user chose. There is no attach-without-mutating call, so on reconnect either pass exactly the name you want kept, or don't re-register at all and work from the existing key paths.

## 8. Designing a new capability — checklist

1. What is the CP? Name it `usecase.name`, no direction prefixes on properties.
2. Define the two roles and which side writes each property (`server` flag).
3. Decide broadcast vs addressed per property (`propagate` flag) and Mode 1 vs Mode 2 — deliberately.
4. Register in `padi.test.*` first; iterate there (published CPs are immutable).
5. Plan multi-connection semantics **in the app** (aggregation, conflict display) — expect a consumer to face N providers.
6. Then, and only then, write the handlers.

## 9. References

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
