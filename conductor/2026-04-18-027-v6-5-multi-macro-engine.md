---
title: feat: Implement kand_indicator_multi! for complex indicators
type: feat
status: active
date: 2026-04-18
origin: docs/brainstorms/2026-04-18-kand-indicator-multi-requirements.md
---

# Implement kand_indicator_multi! for complex indicators

## Overview

We need to implement `kand_indicator_multi!` to handle indicators with multi-output requirements (e.g., `MACD`, `BBANDS`) and complex multi-input sliding windows (e.g., `CCI`, `MFI`, `DX`). The current V6.3 macro handles scalar outputs and single-buffer sliding windows well but fails when inputs must be tracked individually or multiple results are produced.

## Problem Frame

As discovered in V6.4 porting, indicators like `CCI` require tracking `(high, low, close)` over a sliding window. Flattening these into a single `__kand_window` tuple breaks Arrow `FixedSizeListArray` serialization. Additionally, indicators like `DX` output multiple values (`DX`, `+DI`, `-DI`). We need a new macro that can generate parallel circular buffers and support `Outputs: (O1, O2)` tuples.

## Requirements Trace

- R1. Generate `Stateful` and `Batch` structs that support multiple outputs from the `next` and `next_batch` methods.
- R2. Support parallel circular buffers (e.g., `__kand_window_high`, `__kand_window_low`) rather than a single tuple buffer.
- R3. Serialize these parallel buffers as separate `FixedSizeListArray` columns with the `__kand_` prefix for V5.2 compliance.
- R4. Provide a clear syntax for defining `outputs`.

## Scope Boundaries

- This plan focuses on creating the `kand_indicator_multi!` macro and updating the `BatchIndicator` trait if necessary (or creating a `BatchIndicatorMulti`).
- It will port `CCI`, `MFI`, and `DX` using the new macro.
- It will NOT port composite indicators like `MACD` yet (that will be a follow-up phase).

## Context & Research

### Relevant Code and Patterns

- `kand/src/helper/arrow_macro.rs`: The current macro engine lives here. We will add `kand_indicator_multi!` alongside it.
- `kand/src/ta/traits.rs`: `BatchIndicator` currently returns `TAArrowArray`. We may need a trait that returns a tuple of `TAArrowArray`s, or just implement specific methods. Wait, `kand_arrow_wrapper_multi!` returns a tuple `(TAArrowArray, TAArrowArray, ...)`. We should align `BatchIndicatorMulti` with this.

## Key Technical Decisions

- **Decision 1: Parallel Circular Buffers**: Instead of a `Vec<(TAFloat, TAFloat)>`, the stateful struct will hold `window_high: Vec<TAFloat>`, `window_low: Vec<TAFloat>`, etc.
  - *Rationale*: Arrow does not easily support list-of-structs or list-of-tuples without complex `StructArray` builders. Separate `FixedSizeListArray`s of `Float64` are much faster and match our V5.2 durability standard natively.

- **Decision 2: Variable Outputs**: The macro will accept an `outputs: { out1: TAFloat, out2: TAFloat }` syntax.
  - *Rationale*: This clearly defines the struct fields for output buffers in `Batch` mode and guides the tuple return type `Result<(TAFloat, TAFloat), KandError>`.

## Open Questions

### Resolved During Planning

- **Trait Compatibility**: How do we handle `BatchIndicator` returning multiple arrays?
  - *Resolution*: We will introduce `IndicatorMulti` and `BatchIndicatorMulti` traits or simply implement inherent methods for multi-output indicators if trait bloat is an issue. Since `kand` already uses traits heavily, adding `IndicatorMulti` and `BatchIndicatorMulti` is the idiomatic path.

### Deferred to Implementation

- **Exact Macro Syntax**: The exact parsing of token trees for parallel buffers will be refined during implementation.

## Implementation Units

- [ ] **Unit 1: Define Multi-Output Traits**

**Goal:** Create `IndicatorMulti` and `BatchIndicatorMulti` traits in `kand/src/ta/traits.rs`.

**Requirements:** R1

**Dependencies:** None

**Files:**
- Modify: `kand/src/ta/traits.rs`

**Approach:**
- Define `trait IndicatorMulti { type Input; type Output; fn next(&mut self, input: Self::Input) -> Result<Self::Output, KandError>; }` (Actually, we can just reuse `Indicator` with `type Output = (TAFloat, TAFloat)`).
- Wait, `BatchIndicator` has `type Output = TAArrowArray`. We can just set `type Output = (TAArrowArray, TAArrowArray)`. So we don't even need new traits, just use the existing ones with tuple types!

**Verification:**
- Code compiles.

- [ ] **Unit 2: Implement `kand_indicator_multi!` Macro**

**Goal:** Write the macro in `arrow_macro.rs` to handle parallel sliding windows and multiple outputs.

**Requirements:** R1, R2, R3, R4

**Dependencies:** Unit 1

**Files:**
- Modify: `kand/src/helper/arrow_macro.rs`

**Approach:**
- Add a new `macro_rules! kand_indicator_multi`.
- Parse `outputs: { $($out_name:ident : $out_type:ty),+ }`.
- Parse `inputs` to generate parallel `__kand_window_$input_name`.
- Implement `to_record_batch` to serialize each parallel window as a `FixedSizeListArray`.

**Test scenarios:**
- Write a dummy indicator in tests to verify multi-input parallel buffers and multiple outputs compile and serialize properly.

**Verification:**
- The macro expands correctly and tests pass.

- [ ] **Unit 3: Port CCI, MFI, DX**

**Goal:** Utilize the new macro to port the complex indicators deferred from V6.4.

**Requirements:** R1, R2

**Dependencies:** Unit 2

**Files:**
- Modify: `kand/src/ta/ohlcv/cci.rs`
- Modify: `kand/src/ta/ohlcv/mfi.rs`
- Modify: `kand/src/ta/ohlcv/dx.rs`

**Approach:**
- Replace manual `Stateful` and `Batch` implementations with `kand_indicator_multi!`.
- Adjust `next` logic to use the new parallel buffers.

**Verification:**
- `cargo test` and `cargo bench` show parity and no performance regressions.

## System-Wide Impact

- **Interaction graph**: Minimal. Extends macro engine capabilities.
- **State lifecycle risks**: Parallel buffers must be kept in sync using a shared `__kand_cursor`.

## Risks & Dependencies

- Macro complexity in Rust can be difficult to debug. We will use `cargo expand` if necessary to troubleshoot token parsing.
