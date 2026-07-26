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

The identity chain is the same in every binding, and IDs must stay **stable across restarts**:

```
Client → system() → node(id, name, upstream) → context(id, name)
       → provider(profile) | consumer(profile)     // .get / .put on properties
```

**Node** (`npm install arete-sdk`, ESM only):

```javascript
import { Client } from 'arete-sdk';

// Always pass protocol/host/port explicitly: the constructor otherwise falls
// back to browser `location` globals, which do not exist under Node.
const client = new Client({ protocol: 'wss:', host: 'dashboard.test.cns.dev', port: 443 });
await client.waitForOpen(5000);

// Every step of the chain returns a Promise — await all four.
const system = await client.system();                    // cache this — see §7 gotcha 1
const node   = await system.node(nodeId, 'My App', false);
const ctx    = await node.context(ctxId, 'My Context');
const cap    = await ctx.provider('padi.light');         // or ctx.consumer(...)
```

Client emits `open` / `update` / `close` / `error`. `client.get(key, def)` reads the local key cache; `client.put(key, value)` is a raw key write (used for addressed per-connection writes).

**Python** (install from git: `pip install "arete-sdk @ git+https://github.com/project-arete/sdk.git#subdirectory=python"`): synchronous API mirroring the above; the client runs a daemon receiver thread.

**Auth (SDK 0.1.6 — unsettled; expect this to change).** The `Client` constructor accepts only `protocol`, `host`, and `port`. Its `username`/`password` options exist in the source but are commented out, and the connection URI is assembled from those three values alone. The only way to pass credentials today is to fold them into the `host` string as URL userinfo — `host: 'user:pass@realm.example.com'`, URL-encoded — which is a workaround rather than a supported interface, and one that puts secrets somewhere they leak into logs and error reports. Realm-issued bearer tokens are being added; until they land, treat authentication as an open question and keep credentials out of anything you commit. Test realms, including `dashboard.test.cns.dev`, accept unauthenticated connections.

**Public test realm:** `wss://dashboard.test.cns.dev:443` (no auth). Brokerage can take tens of seconds — use generous timeouts in test rigs.

## 5. Bootstrapping an application into a realm

Declaring a capability is not the first thing an application does — it is the third. Every app crosses the same gates in the same order, each a precondition for the next. User-facing apps should surface them in this order rather than bury them in a config file: a user who cannot get past gate 2 otherwise has no way to understand why nothing is happening.

**Gate 1 — realm and identity.** Collect the realm address and credentials, then register a system and a node. You supply the IDs, and they **must be stable across restarts** — persist them; never generate them at startup. The SDK names the system after `os.hostname()` unless you rename it (§7 gotcha 1), so let the user set the name it will carry on the realm.

**Gate 2 — context.** A capability is declared *in* a context. There is no default and no global scope, so the app cannot proceed without one. Existing contexts are discoverable from the key cache — `cns/<sys>/nodes/<node>/contexts/<ctxId>/name` — and so are the capabilities declared in them: `…/contexts/<ctxId>/<provider|consumer>/<profile>/version`. Together those answer the only question that matters: *is there a context that already holds the complementary role for a CP I implement?*

> **Enumerate what you can see, and don't assume that is everything.** Visibility is granted by the realm, not by the mechanism (§6). A realm requiring no token is public and exposes its whole namespace to any client; with tokens the default becomes privacy within the realm, and an individual context may carry its own token.

Four cases follow, and a UI should handle all four:

- **A complementary context exists** — offer *join*, and make it the default. This is the case that actually produces connections.
- **Contexts exist, but the user wants a separate one** — offer *create* alongside them.
- **Nothing complementary exists** — *create* is the only option; prompt for a name.
- **A protected context** — one carrying its own token is **granted, not discovered**. It will not arrive through the picker, so accept its token as a way in rather than assuming every joinable context is visible.

**The context ID is what matches; the name is only a label.** Binding happens within a context ID, and different systems routinely name the same context differently — realm-wide views group by ID and display the most common name variant. Hence the trap: two users who both choose *create* and both type "Kitchen" get two different IDs and will never bind, while the UI shows what looks like a match. Defaulting to *join* is what prevents this, which makes it load-bearing rather than a convenience.

**Gate 3 — declaration.** Declare provider or consumer for each CP the app implements, in the chosen context. Resolve every profile from the registry first (§3), and on restart do not blindly re-declare (§7 gotcha 2).

**Gate 4 — bind.** Declared is not connected. A capability is **bound** once it has at least one connection (`…/<role>/<profile>/connections/<connId>/…`) and **unbound — awaiting broker** while it has none. Unbound is a normal state, not an error: brokerage can take tens of seconds. Show it explicitly, with a short grace period before drawing attention to it, or users will assume the setup they just completed has failed.

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

1. **`client.system()` is not idempotent.** Every call re-registers the system under `os.hostname()`. Cache the System instance; if you rename the system, re-issue the rename after any re-registration.
2. **Capability re-declaration wipes values.** Re-issuing a `providers`/`consumers` declaration for an *existing* capability resets all its property values to empty strings — and the empties propagate into every connection. On startup/reconnect, check whether `…/<role>/<profile>/version` already exists in the key cache; if so, **skip the declaration** and operate on the existing key paths. (Values persist on the realm across disconnects — "values gone after restart" is almost always this bug, not realm data loss.)
3. **Don't use `.watch()`** (Node v0.1.6) — it has a null-match crash. Derive state from `client.keys` inside an `update` event handler instead.
4. **No keepalive, fragile reconnect.** The SDK sends no WebSocket pings and won't retry an unexpected clean close. Long-lived apps should add a ping (~30s) and reconnect logic — and on reconnect, re-attach without re-declaring capabilities (see gotcha 2).
5. **Python: don't trust `wait_for_open()`** — it can block for the full timeout on realms that never send the message it gates on. Poll `client.stats()['connection'] == 'online'` instead. (Node's `waitForOpen` works — it polls.)
6. **System ID off-Raspberry-Pi — no supported fix.** The `Client` constructor calls `get_system_id()` immediately. That function reads the Pi device-tree files and otherwise throws `Unable to detect System ID on this platform`, and there is no `systemId` constructor option — so this cannot be solved from application code. It blocks ordinary Linux, macOS, Windows, containers, CI, and Electron development hosts. The working practice today is to patch the SDK at install time (a `postinstall` script substituting a stable UUID, e.g. derived from an environment seed via `uuidv5`) and to re-apply that patch on every `npm install`. A public identity-injection option is the right fix; treat this as an open SDK gap rather than something your application configures away.
7. **Registration commands must be awaited serially.** Bursting `systems`/`nodes`/`contexts`/`providers`/`consumers` commands without awaiting each response can silently drop declarations. The SDK's own command path awaits correctly — but any hand-rolled wire client must too.
8. **Electron:** run the SDK only in the main process; expose to renderers via a preload bridge. And never `npm install` an Electron project inside a cloud-synced folder (Drive/iCloud/Dropbox) — native builds break.

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
