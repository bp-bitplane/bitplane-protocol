---
status: accepted
date: 2026-10-18
---

# Rust types are the protocol schema

Protocol v1 is defined as Rust types in the `bitplane-protocol` crate: serde types and constants, no logic. Every other form of the schema is generated from them. TypeScript types come from **specta**, behind a cargo feature and pinned to the exact version bitplane-cc already uses (`=2.0.0-rc.25`), so protocol types can go into cc's `bindings.ts` through the toolchain cc already runs. If specta's release-candidate churn starts to hurt, **ts-rs 12** is the drop-in fallback; it produced the same shapes. A **schemars JSON Schema** is generated too and committed, checked in CI like cc's bindings, as documentation and a diffable record of wire changes between versions. Nothing is ever generated from it.

`u64` and `usize` fields carry an explicit TS type annotation (`number`), because `JSON.parse` returns a number however the Rust side types it. Enums carry `#[serde(other)] Unknown`, so a client reading a newer daemon degrades instead of failing.

`plane.dispatch` carries `bp`'s `Request` and `Response` as opaque JSON in Rust: sessiond pipes them to `bp` and never links `bitplane-core`. In TS the method is generic over the `Request`, `Response` and `ErrorEnvelope` types that bitplane-cli generates from `bitplane-core` with its own feature-gated `specta::Type` derive, so the engine's types are declared once, in the engine.

Binary frames are outside every generator: they are specified in prose with test vectors and implemented by small hand-written codecs in Rust and TS (ADR 0024).

## Considered Options

- **Schema first (hand-written JSON Schema → typify and json-schema-to-typescript).** Rejected: the round trip lost serde's tags. typify turned internally tagged enums into untagged `VariantN` structs, json-schema-to-typescript dropped the discriminant next to `$ref`, and JSON Schema has no generics, so `Outcome<T>` and `PerMember<T>` were monomorphised.
- **typeshare.** Rejected: it fails on `#[serde(flatten)]` and on internally tagged enums with newtype variants, both of which `bp`'s wire types use.
- **ts-rs as the primary generator.** Kept as the fallback only: it would be a second generator beside cc's specta, and it silently types `u64` as `bigint`, where specta refuses until the field is annotated.
- **Re-declaring the engine types in `bitplane-protocol`.** Rejected: two copies drift, as cc's hand-copied `bp` types already have.

## Consequences

- bitplane-cli gains a feature-gated `specta::Type` derive on its wire types and publishes the generated TS.
- A wire change shows up as a diff of the committed schema and TS in review.
- Specta stays pinned in lockstep with cc; bumping it is a change to both.
- Research: [Source of truth for the protocol schema](https://github.com/bp-bitplane/bitplane-protocol/issues/2).
