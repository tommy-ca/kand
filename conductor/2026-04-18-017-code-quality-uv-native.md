# Implementation Plan: Code Quality, Linting & Pre-commit Workflow (uv Native)

## Overview
This plan establishes a unified code quality standard for the `kand` ecosystem, integrating Rust and Python toolchains with automated pre-commit hooks.

## Problem Frame
- **Formatting**: Inconsistent code style across the multi-language repository.
- **Linting**: No enforced standards for Python code quality.
- **Automation**: Manual execution of quality checks is prone to human error.
- **Tooling**: Desire to use `uv` native commands for Python dependency and tool management.

## Requirements Trace
- **R1. Warning-Free Build**: Maintain 0 warnings in Rust and Python.
- **R2. Rust Standards**: Enforce `cargo fmt` and `clippy`.
- **R3. Python Standards**: Enforce `ruff` (lint/format) via `uv`.
- **R4. Automated Quality**: Establish `pre-commit` hooks that run on every commit.

## Implementation Units

### Phase 1: Rust Standardization
- [x] Unit 1.1: Resolve all compiler warnings (Completed).
- [ ] Unit 1.2: Run `cargo fmt --all`.
- [ ] Unit 1.3: Run `cargo clippy --workspace --features arrow -- -D warnings`.

### Phase 2: Python Toolchain (uv)
- [ ] Unit 2.1: Initialize `uv` environment.
- [ ] Unit 2.2: Run `uv run ruff check --fix .`.
- [ ] Unit 2.3: Run `uv run ruff format .`.

### Phase 3: Pre-commit Configuration
- [ ] Unit 3.1: Create `.pre-commit-config.yaml`.
- [ ] Unit 3.2: Configure hooks to use `uv run` for Python tools where appropriate.
- [ ] Unit 3.3: Install and verify hooks.

### Phase 4: Final Workspace Audit
- [ ] Unit 4.1: Update `CONTRIBUTING.md` with the new workflow.
- [ ] Unit 4.2: Final semantic commit.

## Verification
- `cargo check --workspace --features arrow` -> 0 warnings.
- `uv run ruff check .` -> 0 issues.
- `pre-commit run --all-files` -> All PASS.
