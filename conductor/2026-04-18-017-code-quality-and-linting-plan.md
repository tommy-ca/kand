# Implementation Plan: Code Quality, Linting & Pre-commit Hooks

## Overview
This plan focuses on improving the code quality of the `kand` project by resolving compiler warnings, standardizing code formatting, and establishing a robust pre-commit workflow for both Rust and Python.

## Problem Frame
- **Compiler Warnings**: Multiple unused imports, naming convention violations, and unused fields clutter the build output.
- **Formatting**: No enforced code style across the workspace.
- **Automation**: No automated checks before commits, leading to potential regressions in code quality.

## Requirements Trace
- **R1. Warning Resolution**: 100% warning-free build for `cargo check --workspace --features arrow`.
- **R2. Rust Quality**: Standardize formatting with `cargo fmt` and enforce linting with `cargo clippy`.
- **R3. Python Quality**: Integrate `ruff` for extremely fast Python linting and formatting.
- **R4. Pre-commit Hooks**: Implement `.pre-commit-config.yaml` to run all checks automatically.
- **R5. CI Alignment**: Ensure CI workflows are updated to match these local standards.

## Implementation Units

### Phase 1: Warning Resolution & Rust Cleanup
- [ ] Unit 1.1: Resolve unused imports and variables in `kand` and `kand-py`.
- [ ] Unit 1.2: Fix naming conventions for `BatchSMA_py` etc. in `kand-py` to follow CamelCase.
- [ ] Unit 1.3: Run `cargo fix --lib -p kand` where applicable.

### Phase 2: Python Tooling Integration
- [ ] Unit 2.1: Add `ruff` and `pytest` to `pyproject.toml`.
- [ ] Unit 2.2: Configure `ruff` in `pyproject.toml`.
- [ ] Unit 2.3: Run `ruff check --fix` and `ruff format` on `kand-py/python`.

### Phase 3: Pre-commit Workflow
- [ ] Unit 3.1: Create `.pre-commit-config.yaml` with hooks for:
    - `trailing-whitespace`, `end-of-file-fixer`, `check-yaml`.
    - `cargo fmt`, `cargo clippy`.
    - `ruff` (lint and format).
- [ ] Unit 3.2: Install and run pre-commit on all files.

### Phase 4: Documentation & Audit
- [ ] Unit 4.1: Update `CONTRIBUTING.md` with new quality standards.
- [ ] Unit 4.2: Final semantic audit of the project root.

## Verification
- `cargo check --workspace --features arrow` (0 warnings).
- `pre-commit run --all-files` (All PASS).
