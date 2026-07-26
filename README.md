# Project Arete SDK

Build applications on **CNS/CP** — the governance layer that decides *whether* two
nodes should connect, under what contract, and in what context, before any data
flows.

> ### 🤖 Building with an AI assistant?
>
> **[ARETE.md](ARETE.md)** is a single-file briefing that teaches any AI coding
> assistant — Claude, ChatGPT, Copilot, Cursor — how to build CNS/CP applications
> correctly. Point yours at it before you start:
>
> ```
> https://raw.githubusercontent.com/project-arete/sdk/main/ARETE.md
> ```
>
> Or save it as `ARETE.md` in your project root, where coding agents pick it up on
> their own. It covers the mental model and its vocabulary, the non-negotiables,
> how to resolve a profile from the CP registry, an SDK quickstart, and the
> field-verified traps that otherwise produce plausible-looking but wrong code.
> **[Read it →](ARETE.md)**

## Language Bindings

- [NodeJS](nodejs/)
- [Python](python/)
- [Rust](rust/)

## You will need a realm

Point your application at an orchestrator. [Arete Hosting](https://aretehosting.com)
gives you your own in a few clicks — free right now — or use the public test realm
at `wss://dashboard.test.cns.dev:443`.

Connection Profiles are governed centrally in the [CP registry](https://cp.padi.io).
Resolve a profile there — for example
[`padi.light`](https://cp.padi.io/profiles/padi.light) — before declaring a provider
or consumer for it.

## Learn more

- 🌐 [projectarete.io](https://projectarete.io) — the project website
- 📖 [CNS/CP specification](https://github.com/CNSCP/specification)
