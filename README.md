# Project Chimera (Simplified Platform)

Project Chimera is a Rust-based playground for building and testing a
multi-agent orchestration stack.  This repository historically tracked
an ambitious "eight layer" architecture; the codebase has now been
streamlined so the core concepts are easy to explore without pulling in
large model dependencies or unfinished scaffolding.

## What's Included

- **Agent management** – strongly typed agent definitions and a manager
  that records basic activity metrics.
- **Task orchestration** – a lightweight orchestrator that assigns
  tasks to registered agents and tracks their lifecycle.
- **Inference façade** – a dependency-free mock inference engine that
  demonstrates request handling without requiring GPU bindings.
- **Training façade** – an asynchronous simulator that walks through the
  motions of LoRA training while writing artefacts to disk.
- **Platform services** – reusable rate limiting and audit logging
  utilities, plus helper functions for configuration and metrics.
- **Binaries** – example `agent`, `router`, and `trainer` programs
  showing how the library pieces fit together.

## Getting Started

```bash
# Run unit tests
cargo test

# Start the example agent (listens on 0.0.0.0:8080)
cargo run --bin agent -- --name demo-agent
```

The agent exposes simple JSON endpoints:

- `GET /health` – readiness information and a unix timestamp.
- `POST /predict` – validates the payload, logs an audit event, and
  returns a canned response.
- `GET /status` – static service metadata for dashboards.

## Project Layout

```
src/
  agents.rs          Agent and metrics definitions
  audit_logging.rs   Structured audit log writer
  inference.rs       Stub inference engine
  lib.rs             Platform entry point used by the binaries
  orchestration.rs   Task orchestration helpers
  rate_limiting.rs   In-memory token bucket
  training.rs        Simulated LoRA trainer
  utils/             Shared helpers (config, validation, metrics)
```

Additional directories under `src/layer*` capture legacy experiments.
They remain available but are not required for the streamlined build.

## Development Notes

- The repository no longer depends on Candle, SQLx, or other heavyweight
  crates.  Compilation is fast and works with the default Rust toolchain.
- Validation avoids regular expressions and URL parsers to keep the
  dependency graph shallow.
- The simulated trainer writes artefacts under the configured output
  directory; run tests in a temporary directory or clean up afterwards.

## Contributing

Issues and pull requests are welcome.  Please keep new code covered by
unit tests and prefer small, well-documented modules over sprawling
"future work" stubs.
