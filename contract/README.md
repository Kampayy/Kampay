# KamPay Contracts

Soroban smart contracts powering KamPay payroll, escrow, and on-chain credit on Stellar.

## Workspace

| Crate | Responsibility |
| --- | --- |
| [`payroll`](contracts/payroll) | Recurring salary streams, funding, withdrawals, batch disbursement |
| [`escrow`](contracts/escrow) | Milestone-scoped escrow with conditional release and grace periods |
| [`credit`](contracts/credit) | On-chain credit records and score computation |

## Build

```bash
stellar contract build
```

WASM artifacts land in `target/wasm32v1-none/release/`.

## Test

```bash
cargo test
```

## Deploy to testnet

```bash
stellar keys generate --global deployer --network testnet
stellar keys fund deployer --network testnet

stellar contract deploy \
  --wasm target/wasm32v1-none/release/payroll.wasm \
  --source deployer \
  --network testnet
```

## Conventions

- Every state-changing entry point calls `require_auth` on the acting address.
- Amounts are `i128` in the token's smallest unit; no floating point anywhere.
- Timestamps are `u64` ledger seconds (`env.ledger().timestamp()`).
- Errors are `#[contracterror]` enums with stable discriminants — never panic with a string.
