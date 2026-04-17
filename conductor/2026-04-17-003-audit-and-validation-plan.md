# Plan: Validation, Review, and Audit for Arrow Integration

## Overview
This plan specifies the continuous validation and auditing process for the Arrow Zero-Copy Integration. It leverages specialized sub-agents to ensure that high-stakes architectural and security decisions are correctly implemented.

## Audit Phases

### Phase 1: Core Normalization Audit
- **Goal:** Verify that all 50+ indicators have been correctly refactored into `_raw` slice-based functions without regressions.
- **Agents:**
  - `compound-engineering:review:architecture-strategist`: Verify the `_raw` function contract consistency across the library.
  - `compound-engineering:review:correctness-reviewer`: Ensure that existing tests are updated to verify the new `_raw` logic.
- **Timing:** Immediately after Unit 1.1 completion.

### Phase 2: Arrow Implementation Audit
- **Goal:** Audit the POC implementation of `sma_arrow` for alignment, offset handling, and null safety.
- **Agents:**
  - `compound-engineering:review:security-sentinel`: Stress-test the Arrow buffer slicing and `MutableBuffer` usage for potential overflows or out-of-bounds access.
  - `compound-engineering:review:performance-oracle`: Verify that the implementation is indeed zero-copy and maintains 64-byte alignment.
- **Timing:** Immediately after Unit 2.1 completion.

### Phase 3: Language Binding Security Audit
- **Goal:** Validate the PyCapsule handshake in Python and the `WasmBuffer` protocol in WASM.
- **Agents:**
  - `compound-engineering:review:security-reviewer`: Review the `PyCapsule` destructor logic and the `WasmBuffer` memory safety (avoiding stale JS views).
  - `compound-engineering:review:api-contract-reviewer`: Verify that the Python and JS APIs are ergonomic and follow Arrow ecosystem conventions.
- **Timing:** After Unit 2.2 and 2.3 completion.

### Phase 4: Macro Generation & Final Pass
- **Goal:** Audit the macro-generated code for correctness across all 50+ indicators.
- **Agents:**
  - `compound-engineering:review:pattern-recognition-specialist`: Ensure that the macro correctly handles all indicator variants (single vs multi-output).
  - `compound-engineering:review:adversarial-reviewer`: Attempt to break the system using extreme inputs (e.g., arrays with offsets matching total length, empty arrays).
- **Timing:** Final project wrap-up (Unit 4.1).

## Reporting
Each audit phase will produce a `docs/audits/YYYY-MM-DD-audit-<phase>-results.md` report. Any P0 or P1 findings will block the next implementation unit.
