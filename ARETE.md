# ARETE.md — Designing and building on CNS/CP

> **What this file is.** A single-file briefing for anyone — human or AI agent — designing Connection Profiles and building applications on the CNS/CP architecture. Drop it into your project root, or point your coding agent at the canonical copy:
>
> `https://raw.githubusercontent.com/project-arete/sdk/main/ARETE.md`
>
> **This file is architecture and profile design. SDK mechanics live in a companion file** — quickstart, language bindings, browser support and version-specific defects:
>
> `https://raw.githubusercontent.com/project-arete/sdk/main/ARETE-SDK.md`
>
> Read this one first, and read it fully. **Do not start with the companion.** The most common failure mode is a well-built application standing on a Connection Profile that was never a contract in the first place — and the architecture has a few load-bearing distinctions that, if violated, produce plausible-looking code that is wrong.

---

## 1. Mental model (read this even if you skip everything else)

**CNS/CP is a Layer 0 governance substrate.** It is not a communication protocol, not an agent framework, not an orchestration engine. It governs *whether* two nodes should connect, under what contract, and in what context — before any data flows.

You do not write "connect A to B." You write: A declares it can **provide** `cp:x`; B declares it **consumes** `cp:x`; the orchestrator (Arete) **brokers** the match and binds them if policy and context permit. Your application logic lives in *what to declare* and *what to do once bound* — never in *how to reach an endpoint*.

Core vocabulary — use these words precisely:

| Term | Meaning |
|---|---|
| **CNS** | Connectivity Naming System — resolution/naming layer ("DNS for governed connectivity") |
| **CP** | Connection Profile — a machine-evaluable contract for one interaction pattern |
| **Provider / Consumer** | The exactly-two asymmetric roles every CP defines. See §2 — this is the load-bearing concept |
| **Context** | The bounded scope a binding exists in. Establishes *which* thing is being discussed |
| **Realm** | The bounded governance scope within which policy is declared and authorization decided |
| **Orchestration** | The "What" — workflow intent, system-level policy (Arete's job) |
| **Brokerage** | The "Who" — one matching action: consumer ↔ qualified provider via CP evaluation |
| **Mode 1 (in-band)** | Data flows through the CP network — mediated, auditable |
| **Mode 2 (out-of-band)** | CNS/CP governs the binding, then steps aside; data flows peer-to-peer |

The mode lives **in the CP definition**, not in a config flag. Read it from the CP and handle accordingly.

### 1.1 How a binding forms

A binding is not a persistent configuration you install. It is the outcome of a sequence, and every step can refuse:

**Register** → **Authorize** → **Declare** → **Reconcile** → **Match** → **Bind**

Registration and authorization admit a node to a realm. Declaration states which CPs it can fulfil, in which role, in which context. Reconcile, match and bind are the substrate's work. Nothing in your application code reaches across this sequence — you declare, and you handle what arrives. §5 covers what your application must do at each gate.

### 1.2 The non-negotiables

1. **Never connect by address.** Connect by declared CP + role + context. A hardcoded endpoint in application logic is the anti-pattern this architecture exists to replace.
2. **Every capability is a CP** with a canonical name, exactly two asymmetric roles, stated authority, and a deliberate mode choice. Design the CP before writing handlers (§3).
3. **Resolve every CP from the registry before use** (§4). If it isn't registered, stop — do not invent the profile.
4. **Conflicting values across multiple connections are application semantics, by design.** The mechanism delivers every connection's view faithfully; it never imposes a conflict policy. When one of your capabilities is bound to N peers — and either role can be, see §6 — *your app* decides how to combine what it receives. Do not "fix" this in SDK or orchestrator code.
5. **Context is first-class.** A binding exists *in* a context; carry it, don't let authority drift across trust domains.
6. **Deny-by-default, and assume every action is audited.** Identity, roles, and a matching CP are required before anything binds. Never design an app that opens access and filters later.

---

## 2. The role pair — the load-bearing concept

Every CP defines **exactly two asymmetric roles: provider and consumer.** This is not a labelling convention. It is the thing that makes a CP a contract rather than a schema.

### 2.1 Why the pair is what matters

Ontologies and data models — Haystack, Brick, 223P, an API schema, a database — describe **what a thing is**. None of them state **who may do what to whom**. That gap is what a CP closes, and the role pair is how it closes it.

**Authority attaches to the role, not to the property, and not to the field.** A provider of an observation CP has authority to publish observations. That says nothing about whether it may accept commands. A consumer authorized to observe is not thereby authorized to control. Two roles with unbounded scope is just client/server; the pair only does useful work when each side's authority is bounded and stated.

**Each capability *is* its sourced property set.** Every property in a profile names the role that **sources** it, and the specification defines the two capabilities in exactly those terms: *a Provider Capability is the set of all properties whose source is "provider"; a Consumer Capability is the set of all properties whose source is "consumer".* So naming a property's source is not annotating it — it is deciding which of the two capabilities the property belongs to. The partition is what makes the pair concrete.

Keep the two ideas apart, because one word has been doing both jobs and it has misled readers:

| | What it governs | Where it is decided |
|---|---|---|
| **Authority** | Whether you may bind at all, and in what role | Realm and context policy (§3.5) |
| **Source** | Which of the two capabilities a property belongs to, and therefore which direction its value travels | The profile, per property (§4) |

Source is **not** a permission. It is not access control, and it grants nothing. It is the partition line through the property set.

Three consequences follow, and they drive everything in §3:

1. **The pair determines CP boundaries.** Where authority differs, you need a different CP.
2. **The pair determines authorization.** You grant or withhold a whole role in a context — you never hide fields.
3. **The pair must exist before the property list.** If you cannot name both parties and what each is entitled to do, you do not yet have a CP to list properties for.

### 2.2 A note on `server` and `client`

The concept above is called **`Source`** in the specification, with values `provider` and `consumer`. The registry encodes the same thing with a flag named `server` (§4). That name is historical — it dates from before it was settled that a property has exactly two possible origins, when the field named *who serves this up* and `server` was one candidate value among several.

**Do not read provider/consumer as client/server.** Client/server carries four decades of request/response connotation — a passive server answering an active client — which is the wrong shape. Provider and consumer are asymmetric in *what each sources*, not in who speaks first. Either side may write its own properties; either side may initiate; both are bound by the same contract.

**Watch for one specific trap.** In a registry record, `server` appears at *two* levels meaning *two different things*:

```json
{ "name": "padi.light",
  "server": "A Controller",                ← prose describing the provider ROLE
  "client": "A Light being controlled",
  "versions": [{ "properties": [
      { "name": "sOut", "server": null }   ← the SOURCE flag: provider-sourced
  ]}]}
```

Reading a profile top to bottom, you meet `server` as role prose before you meet it as a source flag, and it is easy to carry the first meaning into the second. That is the single most common misreading of a CP, and it is what produces the mistaken idea that properties carry authority.

### 2.3 Multiplicity is not arity

"Many observers" and "many controllers" are common and fully supported. They are many *connections*, each with exactly two roles. Multiplicity never adds a third role to a CP. §6 covers the mechanics.

If you find yourself needing a third party in a single interaction, you have either two CPs or an unmodelled role. See the party-as-property test in §3.2.

---

## 3. Designing a Connection Profile

Design the CP before you write a handler. This is the section most often skipped and most often the cause of rework — published CPs are immutable.

### 3.1 Decomposition: how many CPs, and where to cut

The question every domain hits: given a system with hundreds of possible data points, how many CPs should it have?

**The rule: split where authority differs, not where structure differs.**

Structure — air paths, floors, units, sub-assemblies, org charts — is a modelling fact. It belongs in context and in property names. Authority — who may observe this, who may command that — is a governance fact, and it is the only thing that justifies a CP boundary.

**Stopping condition:** keep splitting until every property in a CP falls under a single authority scope for a single role pair. Then stop.

Apply this together with the "too fine" test below, not before it. On its own the stopping condition pushes toward splitting; the too-fine test pushes back. Neither is correct alone.

### 3.2 Four tests

Apply these to any candidate CP.

**Too coarse.** If granting this CP to a party forces you to grant something you would want to withhold, split it. *Example: bundling fan control into a general operation profile means an application that needs fan authority receives full unit control.*

**Too fine.** If two CPs are always granted together, to the same role pair, under the same authority, merge them. Atomic per-point profiles maximise exact matching at the cost of declaration count and correlation burden; they are appropriate for compiler-backed registries and selective contracts, not as a default.

**Party-as-property.** If a party appears as a *property value* — `who`, `requestedBy`, `assignedTo`, `approvedBy`, `owner` — you have a role you have not modelled. Decide deliberately: either that party is a real role (and needs its own CP with its own authority), or the field is attribution for audit only. Both are legitimate; leaving it undecided is not.

**Naming level.** If you can rename the CP to a more general term without changing a single property, you named it too specifically. *Example: a profile named for rooftop units whose properties are all air-handler-general should be named for air handlers.* Run this test before publication — names are immutable, and an over-specific name fragments the registry permanently.

### 3.3 Absence must be unambiguous

A consumer that receives nothing for a property must be able to tell **why**. There are exactly four causes, and confusing them is a governance failure, not a cosmetic one:

| Cause | Meaning | Resolved by |
|---|---|---|
| **Not modelled** | The system has no such concept | The CP is not declared, or the property is not in the profile |
| **Not implemented** | Modelled in general, absent in this instance | Property not declared by this provider |
| **Not authorized** | Exists, but this consumer may not see it | The connection does not exist — no binding, nothing delivered |
| **Not currently valid** | Exists and is authorized, but stale, faulted or unavailable | Per-value `status` |

The first three are answered by **what is declared and bound**. Only the fourth needs a runtime field. This is why authorization is never field hiding: a hidden field is indistinguishable from a broken sensor, and the consumer cannot act correctly on either.

> **Design target, not current behaviour.** Row 2 has no mechanism today. Declaring a capability creates a key for **every** property in the profile, initialised to an empty string — verified live: a `padi.light` provider that writes only `sOut` still publishes `sLabel = ""`. A provider cannot decline to declare one property. So an empty string presently means *not implemented*, *legitimately empty*, and *wiped by the re-declaration defect* (SDK companion, gotcha 2) all at once. Design profiles as though row 2 works, state absence semantics explicitly, and treat the substrate gap as a known limitation.

**An absent property never means zero, false, or off.** State this explicitly in every profile that has optional properties. A null or release value must be represented distinctly so that releasing a command is never confused with commanding `false`.

### 3.4 Value envelopes

A bare scalar loses the information a consumer needs to use it safely. For any measured or sampled value, consider carrying the envelope with the value:

```json
{
  "value": 56.4,
  "kind": "Number",
  "unit": "°F",
  "status": "ok",
  "timestamp": "2026-08-01T08:51:00-04:00"
}
```

**Envelope granularity must match payload granularity.** One `status` field covering a profile that carries twenty values cannot tell a consumer which of the twenty is faulted. If a CP carries N independently-sourced values, it needs N statuses. This is the most common defect in first-draft profiles, and the principle holds regardless of how you represent the envelope.

Commands carry their own envelope: `requestId`, `priority`, `duration`, `who`. Responses echo `requestId` and are addressed back to the requester only.

> **The envelope shape is an open decision, and it has a cost.** Property values in the registry are strings, and there is no type system — an envelope is serialised JSON that every consumer must parse and nothing validates. More consequentially, fields *inside* an envelope are invisible to `server` and `propagate`: twenty statuses expressed as twenty properties each get their own source and delivery semantics, and the same twenty inside envelopes get none. You would be trading away the flag system that §4 uses to encode the decisions you made here. The `timestamp` is likewise writer-asserted — the namespace carries no timestamps of its own. Adopt the granularity rule unconditionally; decide the representation deliberately.

### 3.5 Authorization is realm and context level

Authorization is decided in the realm, and applies to a role in a context. A party either receives a binding or does not.

**Never filter fields to implement permission.** If a consumer should not see something, it should not be bound to a CP that carries it — which is what §3.1's decomposition rule exists to make possible. Field filtering reintroduces exactly the ambiguity §3.3 eliminates, and moves policy enforcement into the payload where it cannot be governed.

Addressed delivery controls *destination*, not confidentiality. It is not an access control — see §6.

### 3.6 Conflicting values are application semantics

Where a domain has its own arbitration — a control priority scheme, an approval chain — the CP's job is to make arbitration **observable**: carry `priority`, `who`, `requestId`, and publish the effective winning state. The CP does not perform the arbitration. §6 covers the mechanics and the aggregation choices available to you.

### 3.7 Authoring checklist

In this order. The order is the point.

1. **Name the two parties and the authority each holds.** If you cannot, stop — there is no CP here yet.
2. **Apply the four tests** (§3.2). Adjust boundaries until they pass.
3. **Name the CP** at the most general level whose property set is unchanged. Format `usecase.name`, no direction prefixes on property names.
4. **List properties, and name each one's source** (`server` flag) — which is to say, decide which of the two capabilities it belongs to (§2.1).
5. **Decide delivery per property** (`propagate` flag): broadcast to all connections, or addressed to one.
6. **Decide Mode 1 or Mode 2**, deliberately.
7. **Define absence and value semantics** (§3.3, §3.4): which properties are optional, what a missing value means, what envelope each value carries.
8. **Register in `padi.test.*`** and iterate there. Published CPs are immutable.
9. **Plan multi-connection semantics in the app** (§6) — expect either role to face N peers.
10. **Then** write the handlers.

---

## 4. The CP registry — `cp.padi.io`

Before declaring a provider or consumer for any CP, fetch its definition:

```
GET https://cp.padi.io/profiles/<cp-name>      (Accept: application/json)
```

Fetch the **raw JSON** — property flags are encoded by *key presence*, and summarized or rendered views lose them:

- `server` present → the property is **provider-sourced**; absent → **consumer-sourced**. This is the specification's `Source` field under its historical name (§2.2). Properties are named for **purpose**, never with direction prefixes — the flag, not the name, states the source.
  **Precisely:** the flag is present *with a null value*. The orchestrator tests `server === null`, so a `server` key carrying any other value would read as consumer-sourced and silently reverse the property's direction. Every profile in the registry emits `null` today; nothing else is safe to assume.
- `propagate` present → capability-level writes are **broadcast** into all active connections. Absent → not broadcast, but still usable via the **addressed channel**: any declared property can be written directly into one specific connection (`…/connections/<id>/properties/<prop>`), and the orchestrator mirrors it 1:1 to that peer only. Canonical case: a response property addressed back to the requester.
- `required` present → the property is required.

These flags are the **encoding** of decisions you made in §3 — they are not the decisions themselves. A profile whose `server` flags are correct but whose role pair was never stated is not a contract.

Registry governance: published CPs are **immutable** — design deliberately before publishing. Use the `padi.test.*` namespace for development CPs. Unregistered ("local") profile names will not bind on a live realm — the control plane can't produce matchable version keys for a profile it can't resolve.

**What counts as a functional change** (new CP name required): altering the role pair or either side's authority; changing which side writes an existing property; changing a property's meaning, unit or value domain; making an optional property required; removing a property. **Additive** (same CP, new version): adding an optional property; adding an enum member that does not change existing members' meaning; clarifying documentation.

> **The additive path is currently unreachable.** Verified live: no API accepts a version — neither the SDK nor a browser client — and the control plane assigns version `1`. A profile with a published second version was assigned v1 on a fresh declaration, so a v2 cannot presently be declared or bound. Treat the rules above as the intended governance model, and check before relying on a new version reaching anyone. The distinction between a new *name* and a new *version* also extends beyond what the specification currently states, and is pending reconciliation.

---

## 5. Bootstrapping an application into a realm

Declaring a capability is not the first thing an application does — it is the third. Every app crosses the same gates in the same order, each a precondition for the next. User-facing apps should surface them in this order rather than bury them in a config file: a user who cannot get past gate 2 otherwise has no way to understand why nothing is happening.

**Gate 1 — realm and identity.** Collect the realm address and credentials, then register a system and a node. You supply the IDs, and they **must be stable across restarts** — persist them; never generate them at startup. The SDK names the system after the local hostname unless you rename it (SDK companion, gotcha 1), so let the user set the name it will carry on the realm.

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

**Gate 3 — declaration.** Declare provider or consumer for each CP the app implements, in the chosen context. Resolve every profile from the registry first (§4), and on restart do not blindly re-declare (SDK companion, gotcha 2).

**Creating a context is not declaring a capability.** Registering a context makes it exist; nothing can match it until you also declare your role there. A UI that says "created" at gate 2 invites the user to expect an immediate connection and then look for the fault. Show it as *created, not yet published*, and do not count it as compatible — yours or anyone else's — until the declaration has landed.

**Gate 4 — bind.** Declared is not connected. A capability is **bound** once it has at least one connection (`…/<role>/<profile>/connections/<connId>/…`) and **unbound — awaiting broker** while it has none. Unbound is a normal state, not an error — a capability with no counterpart yet simply waits. On a healthy realm binding takes about a second, but it is never instantaneous, so show the state explicitly with a short grace period before drawing attention to it. Otherwise users assume the setup they just completed has failed.

---

## 6. Working with multiple connections

A capability is not a socket. One declaration can be bound to many peers at once, each connection carrying its own independent view of the properties, and that is the normal case rather than an edge case. This is the part of CNS/CP with no analogue in the protocols you already know, so it is where inherited instincts do the most damage: pub/sub habits produce code that collapses N peers into a single value and quietly discards the thing that mattered.

**Two flags, four behaviours.** `server` states which role **sources** a property; `propagate` states *whether that value broadcasts*. They are independent, and all four combinations occur in published CPs:

| | `propagate` present — **broadcast** to every connection | `propagate` absent — **addressed** to one connection |
|---|---|---|
| **`server` present** — provider-sourced | `padi.light` → `sOut` | `padi.game.beacon` → `granted` |
| **`server` absent** — consumer-sourced | `padi.light` → `cState` | `padi.game.beacon` → `feed` |

`propagate` has nothing to do with being a provider. Either role sources some of the properties, and either role's values may broadcast. Read both flags for every property; never infer one from the role. (`padi.test.propagate` exists to demonstrate this — it carries all four combinations in a single profile.)

**Source is a routing input, not a permission.** When a value changes, the orchestrator looks up the property's source and derives the *opposite* role — provider-sourced values travel to the consumer, consumer-sourced values travel to the provider — and writes it there. If the value came from the side that does not source it, there is no opposite role to compute and propagation simply stops. Nothing is rejected and no error is raised; the value sits on your own key and reaches nobody (SDK companion, gotcha 12).

**Reading: every side can face many.** A consumer bound to three providers receives three values for a provider-written property. A `padi.light` switch bound to three lights is a *provider* receiving three `cState` values. Same problem, either role. Per-connection values live at `…/<role>/<profile>/connections/<connId>/properties/<prop>`, and a connection carries both sides' properties mirrored onto both endpoints.

The mechanism will never resolve disagreement for you (§1.2 rule 4) — it delivers each connection's view faithfully and stops there. Choices taken by working applications: **average, minimum, or maximum** where values are numeric and a summary is meaningful; a **logical OR** where any peer asserting is sufficient, so a light stays lit while anyone is still holding it and tug-of-war is legal by design; a **sum** where contributions accumulate. Choose deliberately and write the choice down, because it *is* your application's semantics.

**Writing: broadcast or address.** Writing to the capability property broadcasts to every connection — for a property flagged `propagate`. Writing to `…/connections/<connId>/properties/<prop>` reaches exactly one peer, which is how a response is returned to whoever asked. Both are available to whichever role owns the property.

**Show the connections; do not collapse them.** The substrate always knows which peer contributed what, so a UI that renders only the aggregate is discarding information it was handed. The working pattern is one visual token per connection — a pill, a dot, a bead in the peer's colour — carrying that connection's live values, alongside a separate token for the broadcast or aggregate view. When a user asks "but who is doing this?", the answer is already in the data.

**Visibility is realm-governed.** Addressed means *delivered to one peer*. It does not mean *hidden from others*: an addressed write lands in a key like any other, and who may read that key is decided by the realm. A realm requiring no token is public. With tokens, the default is privacy within the realm, and a context may carry its own token so it can be protected individually. Treat addressing as delivery, never as confidentiality — let the realm's policy, not a property flag, tell you who can see a value.

---

## 7. Anti-patterns

If you are doing any of these, go back to §2.

- **Connecting by address.** A hardcoded endpoint in application logic is the anti-pattern this architecture exists to replace.
- **A profile that is a point list.** Properties with no stated role pair and no authority scope is a schema wearing a contract's name.
- **Field filtering for permission.** See §3.5.
- **Decomposing by structure.** One CP per air path, per floor, per sub-assembly — where the authority is identical across all of them. See §3.1.
- **Inventing a profile.** If it isn't in the registry, stop. Register it in `padi.test.*` first.
- **Resolving conflicts in the substrate.** See §3.6 and §6.
- **Collapsing N connections into one value.** The substrate knows which peer contributed what; discarding that is a rendering choice, and usually the wrong one. See §6.
- **Deferring the role pair.** "We'll decide who provides and who consumes later" means the contract does not exist yet. Properties cannot be designed against an undecided pair.

---

## 8. References

| Resource | URL |
|---|---|
| **SDK mechanics companion** | https://raw.githubusercontent.com/project-arete/sdk/main/ARETE-SDK.md |
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
