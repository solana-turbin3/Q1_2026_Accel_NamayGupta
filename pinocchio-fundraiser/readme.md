# Pinocchio Fundraiser

A Solana fundraiser program migrated from Anchor to **Pinocchio**, with design changes aimed at lower compute usage and smaller binary size.

## Overview

This program implements a crowdfunding flow: makers initialize fundraisers with a target amount and duration; contributors donate tokens up to a per-contributor cap (10%); once the target is met, the maker can claim funds via **Check**; if the fundraiser ends without meeting the target, contributors can **Refund** their tokens.

## Migration & Optimizations

### From Anchor to Pinocchio

- **No IDL** – instruction data is serialized manually with [wincode](https://crates.io/crates/wincode) instead of Borsh
- **`#![no_std]`** – no standard library for smaller footprint
- **`no_allocator!()`** – no heap allocation; no bump allocator or dynamic allocation
- **Compact state** – `repr(C)` structs with fixed layouts (Fundraiser: 90 bytes, Contributor: 9 bytes)
- **Zero-copy deserialization** – account data is read directly via raw pointers instead of full deserialization

### Design Changes

| Change | Benefit |
|--------|---------|
| **ATAs assumed at client** | Vault, contributor ATA, and maker ATA are created by the client before calling the program. No ATA CPI in the program. |
| **Minimal instruction accounts** | Only required accounts are passed; no token program, ATA program, or system program unless needed for CPIs |
| **Contributor PDA created on first contribute** | `init_if_needed`-style behavior: contributor account is created lazily on first contribution |
| **No discriminators for account types** | Accounts are identified by position; client is responsible for correct ordering |
| **Single discriminator byte** | 0=Initialize, 1=Contribute, 2=Check, 3=Refund – no multi-byte discriminator |

### Instructions

1. **Initialize** – Maker creates a fundraiser (PDA) with target amount and duration
2. **Contribute** – Contributor sends tokens to the vault; contributor PDA created if needed
3. **Check** – Maker claims vault when target is met
4. **Refund** – Contributor reclaims tokens when the fundraiser ends and target is not met

## Build & Deploy

```bash
cargo build-sbf
solana program deploy target/deploy/pinocchio_fundraiser.so --url devnet
```

## Tests

```bash
cargo build-sbf
cargo test -- --nocapture
```

## Compute Unit Usage

Measured with LiteSVM:

| Instruction  | Compute Units |
|-------------|---------------|
| Initialize  | 2,006         |
| Contribute  | 8,112         |
| Check       | 6,165         |
| Refund      | 6,733         |


