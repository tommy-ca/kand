---
title: feat: Port Composite Indicators and Implement WASM memory-growth
type: feat
status: active
date: 2026-04-18
---

# Port Composite Indicators and Implement WASM Memory-Growth

## Overview

This phase focuses on the final remaining tasks for the V6 modernization:
1. Porting composite indicators (e.g., `BBANDS`, `MACD`, `STOCH`, `VEGAS`) to the V5 durability standard.
2. Implementing automated WASM memory-growth versioning for resilience.

## Problem Frame

Composite indicators present unique challenges because they contain nested states (e.g., `MACD` contains 3 `StatefulEMA` instances). The `kand_indicator_multi!` macro is optimized for primitive scalar states (`f64`) and cannot natively serialize nested structs into Arrow columns without significant macro bloat. Therefore, we must implement these using the **Explicit Commit Pattern** manually for the `Stateful` and `Batch` tiers, ensuring atomic updates.

Additionally, WASM memory growth needs automated versioning to ensure memory boundaries are respected during resizing, providing resilience in web environments.

## Requirements Trace

- R1. Port `BBANDS`, `STOCH`, and `VEGAS` to `Stateful` and `Batch` structures.
- R2. Use the **Explicit Commit Pattern** (clone nested states, apply updates tentatively, and commit only on success) to guarantee atomicity.
- R3. Implement WASM memory-growth resilience logic.

## Scope Boundaries

- `MACD` already uses the manual explicit commit pattern, so we will review and refine it if needed.
- `BBANDS` uses a sliding window for SMA and Variance.
- `STOCH` requires sliding windows for price channels.
- `VEGAS` is a composite of 4 EMAs.
- The `kand_indicator!` macro will NOT be used for composites if manual implementation provides better safety and serialization control.

## Implementation Units

- [ ] **Unit 1: Port VEGAS Composite Indicator**

**Goal:** Implement `StatefulVEGAS` and `BatchVEGAS` manually using the Explicit Commit Pattern.

**Files:**
- Modify: `kand/src/ta/ohlcv/vegas.rs`

**Approach:**
- `StatefulVEGAS` will contain 4 `StatefulEMA` instances.
- In `next()`, clone the 4 EMAs. Call `next()` on the clones. If successful, assign the clones back to `self`.
- Implement `to_record_batch` by extracting the internal EMA states and serializing them.

- [ ] **Unit 2: Port BBANDS Indicator**

**Goal:** Implement `StatefulBBANDS` and `BatchBBANDS` manually.

**Files:**
- Modify: `kand/src/ta/ohlcv/bbands.rs`

**Approach:**
- `BBANDS` needs a sliding window of prices to calculate SMA and standard deviation.
- We can implement this manually with `__kand_window`, `__kand_cursor`, and `__kand_count`.
- No nested state is strictly necessary if we track `sum` and `sum_sq` manually.

- [ ] **Unit 3: Port STOCH Indicator**

**Goal:** Implement `StatefulSTOCH` and `BatchSTOCH` manually.

**Files:**
- Modify: `kand/src/ta/ohlcv/stoch.rs`

**Approach:**
- Requires tracking historical Highs and Lows over a window, plus sliding sums for FastK and SlowK smoothing.
- Manual circular buffers for `high`, `low`, `close` and the internal SMA states.

- [ ] **Unit 4: WASM Memory-Growth Versioning**

**Goal:** Add resilient WASM memory growth limits.

**Files:**
- Modify: `kand-wasm/src/lib.rs` (or relevant WASM module)

**Approach:**
- Ensure any `Vec` or Arrow buffer allocation triggers explicit memory growth checks if necessary.
- Wait, Rust's standard library allocator in `wasm32-unknown-unknown` handles memory growth automatically via `memory.grow`. We just need to ensure `wee_alloc` or `dlmalloc` is configured properly or we add a versioning hook for the JS side to detect buffer invalidation.
