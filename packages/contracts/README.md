# @backit/contracts

Soroban smart contracts for BACKit prediction market platform.

## Prerequisites

- Rust stable toolchain
- Soroban CLI: `cargo install --locked stellar-cli --features opt`

## Getting Started

```bash
# Build contracts
pnpm build

# Run tests
pnpm test

# Format code
pnpm format

# Lint code
pnpm lint
```

## Contract Structure

- **call_registry** - Main contract for creating and managing prediction calls
- **outcome_manager** - Contract for verifying and settling outcomes with ed25519 signatures

## Available Scripts

- `pnpm build` - Build Soroban contracts
- `pnpm test` - Run Rust tests
- `pnpm lint` - Run cargo clippy
- `pnpm format` - Format Rust code with cargo fmt
- `pnpm clean` - Clean build artifacts
