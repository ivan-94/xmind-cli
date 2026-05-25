# Technical Design

## Purpose

This directory translates the product and command reference contracts into an implementable Rust architecture. It defines the technical stack, crate layout, module boundaries, data flow, quality gates, testing strategy, E2E strategy, and AI-native feedback requirements.

The implementation should optimize for correctness, deterministic machine output, safe workbook mutation, and fast agent feedback loops before adding broad XMind feature coverage.

## Reading Order

1. [architecture.md](architecture.md)
2. [tech-stack.md](tech-stack.md)
3. [crate-layout.md](crate-layout.md)
4. [data-model.md](data-model.md)
5. [command-runtime.md](command-runtime.md)
6. [xmind-storage.md](xmind-storage.md)
7. [patch-engine.md](patch-engine.md)
8. [output-and-errors.md](output-and-errors.md)
9. [quality-gates.md](quality-gates.md)
10. [testing-strategy.md](testing-strategy.md)
11. [e2e-test-plan.md](e2e-test-plan.md)
12. [release-policy.md](release-policy.md)
13. [implementation-roadmap.md](implementation-roadmap.md)

## Engineering Posture

The CLI is currently a single Rust binary package with internal modules for CLI parsing, application orchestration, domain logic, infrastructure, and renderers. The binary entrypoint stays thin: parse command-line input, call module services, and render output.

Core rules:

- Use strong domain types instead of stringly-typed internals.
- Parse and validate before mutation.
- Mutate a workbook model, not raw XMind storage files.
- Preserve unknown workbook data by default.
- Emit one stable JSON envelope for agents.
- Treat every write as transactional: build, validate, then atomically replace.
- Make every failure actionable with structured error context.
