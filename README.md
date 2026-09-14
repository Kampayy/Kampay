# KamPay

**Programmable payroll, escrow, and on-chain credit — built on Stellar with Soroban.**

KamPay lets companies lock funds upfront and pay contributors automatically: recurring payroll for
employees, escrow-based milestones for freelancers and contractors. Every payment builds a verifiable
on-chain **credit score**, which unlocks access to under-collateralized loans from lenders in the
KamPay credit market.

No trust assumptions. No chasing invoices. No two-week settlement windows.

---

## Table of Contents

- [The Problem](#the-problem)
- [How KamPay Works](#how-kampay-works)
- [Core Modules](#core-modules)
  - [Payroll Streams](#1-payroll-streams)
  - [Milestone Escrow](#2-milestone-escrow)
  - [Grace Periods & Disputes](#3-grace-periods--disputes)
  - [On-Chain Credit Score](#4-on-chain-credit-score)
  - [Lending Market](#5-lending-market)
- [Why Stellar + Soroban](#why-stellar--soroban)
- [Architecture](#architecture)
- [Repository Layout](#repository-layout)
- [Getting Started](#getting-started)
- [Contract Interfaces](#contract-interfaces)
- [Security Model](#security-model)
- [Roadmap](#roadmap)
- [Contributing](#contributing)
- [License](#license)

---

## The Problem

Global, distributed teams are normal now. The money rails behind them are not.

| Pain | Today | With KamPay |
| --- | --- | --- |
| **Trust** | Freelancer ships first and hopes to get paid; company pays first and hopes work arrives | Funds locked in escrow before work starts, released on verified conditions |
| **Cost** | 3–7% in FX spread, wires, and intermediaries | Sub-cent Stellar network fees |
| **Speed** | 2–5 business days, longer across borders | 3–5 second finality, 24/7 |
| **Transparency** | Contributors can't see whether funds exist | Anyone can verify the locked balance on-chain |
| **Credit** | Years of reliable freelance income counts for nothing at a bank | Every completed payment writes to a portable credit record |

The last row is the one nobody solves. A contractor who has been paid on time for three straight
years has built real creditworthiness — and no way to prove or use it. KamPay turns payment history
into collateral.

---

## How KamPay Works

```
┌───────────────┐        ┌──────────────────────┐        ┌───────────────┐
│    Employer   │──lock─▶│   KamPay Contracts   │──pay──▶│  Contributor  │
│               │        │  (Soroban / Stellar) │        │               │
└───────────────┘        └──────────┬───────────┘        └───────┬───────┘
                                    │                            │
                         emits payment events                    │
                                    │                            ▼
                                    ▼                    ┌───────────────┐
                         ┌──────────────────────┐        │ Credit Score  │
                         │  Reputation Oracle   │───────▶│   (on-chain)  │
                         └──────────────────────┘        └───────┬───────┘
                                                                 │
                                                        borrow against it
                                                                 ▼
                         ┌──────────────────────┐        ┌───────────────┐
                         │    Lender Pool       │◀──────▶│  Loan Vault   │
                         └──────────────────────┘        └───────────────┘
```

1. **Fund.** An employer deposits USDC (or any Stellar asset) into a KamPay vault. The balance is
   publicly verifiable — contributors can confirm the money exists before they start.
2. **Schedule.** Payroll streams pay out on a cadence. Escrow contracts pay out per milestone.
3. **Settle.** Payments execute automatically. No manual approval needed for the happy path.
4. **Score.** Each settled payment updates both parties' on-chain credit records.
5. **Borrow.** Contributors and companies with strong records draw loans against their score, with
   repayment auto-deducted from future payroll.

---

## Core Modules

### 1. Payroll Streams

Recurring, automated compensation for employees and long-term contributors.

- **Upfront funding.** The employer locks the full cycle's payroll before it begins. A contributor
  can verify coverage at any time.
- **Flexible cadence.** Per-second streaming, or discrete weekly / bi-weekly / monthly cycles.
- **Batch payroll.** A single transaction pays an entire team — Soroban's low fees make 500-person
  runs practical.
- **Pausable & amendable.** Raises, role changes, and offboarding are governed contract calls, not
  spreadsheet edits.
- **Withdraw any time.** In streaming mode, contributors pull accrued earnings whenever they want
  instead of waiting for payday.

### 2. Milestone Escrow

Trust-minimized payments for freelancers, agencies, and one-off contracts.

- **Deliverable-scoped.** Split a contract into milestones, each with its own amount and deadline.
- **Locked at signing.** The full contract value is escrowed before work begins.
- **Release conditions.** Client approval, deadline expiry with auto-release, or an oracle attestation.
- **Partial release.** Approve milestone 1 and pay for it while milestone 2 is still in progress.
- **Cancellation terms.** Kill-fee and refund splits are agreed at signing and enforced by the contract.

### 3. Grace Periods & Disputes

Escrow without an escape hatch just relocates the trust problem. KamPay makes the exits explicit.

- **Grace periods.** A configurable window after a deadline before penalties, auto-release, or
  default trigger — real work has real slippage.
- **Dispute window.** Either party can raise a dispute before final release; funds freeze in place.
- **Tiered resolution.**
  1. *Direct settlement* — the parties agree on a split, executed atomically.
  2. *Arbiter* — a neutral address chosen at contract creation rules on the split.
  3. *Panel* — a staked, randomly selected juror set for high-value contracts.
- **Reputation consequences.** Dispute outcomes feed the credit score, so bad-faith behavior carries
  a lasting, portable cost.

### 4. On-Chain Credit Score

The feature that turns KamPay from a payment rail into a financial identity.

Every settled payment, honored deadline, resolved dispute, and repaid loan writes to a
`CreditRecord` keyed to a Stellar account. The score is derived from:

| Signal | Weight | What it measures |
| --- | --- | --- |
| **Payment history** | 35% | Payments received or made on time, without dispute |
| **Tenure** | 20% | Age of the account's first settled payment |
| **Income consistency** | 15% | Volatility and continuity of inbound payment volume |
| **Counterparty quality** | 10% | Credit standing of the entities you transact with |
| **Dispute record** | 10% | Frequency and outcome of disputes raised against you |
| **Loan repayment** | 10% | On-time repayment of prior KamPay loans |

Properties that matter:

- **Portable.** The record lives on Stellar, not in KamPay's database. Any protocol can read it.
- **Verifiable.** Every input is a public on-chain event; the score is recomputable from scratch.
- **Two-sided.** Employers are scored too. A company that consistently pays late is visible to
  contributors before they sign.
- **Privacy-aware.** The raw score is public; underlying income figures can be disclosed selectively
  via signed attestations rather than exposed in full.
- **Sybil-resistant.** Score accrues from *counterparty-weighted* volume, so wash-paying yourself
  between fresh accounts earns nothing.

### 5. Lending Market

Credit scores are only useful if something consumes them.

- **Under-collateralized loans.** Borrow against future payroll and a proven history instead of
  posting 150% collateral like every other DeFi lender.
- **Score-tiered terms.** Higher scores unlock larger principals, longer terms, and lower rates.
- **Payroll-linked repayment.** Repayments are deducted from incoming payroll streams at the contract
  level — the lender's strongest guarantee, and the reason rates can be low.
- **Salary advances.** Draw a portion of already-accrued but not-yet-vested payroll instantly.
- **Invoice financing.** Advance against escrowed milestones that are locked but not yet released.
- **Open lender pools.** Anyone can supply capital to a tranche and earn yield scaled to the risk
  band they choose.
- **Transparent defaults.** Default is recorded on-chain and permanently affects the borrower's score.

---

## Why Stellar + Soroban

KamPay is **Stellar-only**. That is a deliberate narrowing, not a limitation.

- **Fees that make micro-payroll viable.** Per-second streaming and 500-person batch runs are
  economically absurd on chains with dollar-level gas. On Stellar they cost fractions of a cent.
- **3–5 second finality.** Payday is instant, not eventually-consistent.
- **Native stablecoins.** Circle-issued USDC on Stellar, plus a deep pool of tokenized fiat.
- **Real on/off ramps.** Stellar Anchors give contributors an actual path from USDC to local
  currency in their own bank account — the part most payroll crypto projects hand-wave.
- **Soroban.** A Rust/WASM contract environment with metered execution, structured storage TTLs, and
  a genuinely good testing story.
- **Built for payments.** Stellar was designed as a payment network, not repurposed into one.

Focusing on one chain means one canonical credit record instead of a fragmented score scattered
across ecosystems. Multi-chain reads may come later; the ledger of record stays Stellar.

---

## Architecture

### Contracts (Soroban / Rust)

| Contract | Responsibility |
| --- | --- |
| `payroll` | Streams, cycles, batch disbursement, pause/amend/terminate |
| `escrow` | Milestone definition, funding, conditional release, cancellation |
| `dispute` | Freeze, grace-period logic, arbiter and panel resolution |
| `credit` | `CreditRecord` storage, score computation, attestation issuance |
| `lending` | Loan origination, lender pools, repayment hooks, liquidation |
| `registry` | Contract discovery, upgrade authority, protocol parameters |

Contracts are composed, not monolithic: `payroll` and `escrow` emit settlement events that `credit`
consumes; `lending` reads `credit` and installs a repayment hook back into `payroll`.

### Frontend

- Employer console — fund vaults, run payroll, open escrows, manage disputes
- Contributor dashboard — earnings, withdrawals, credit score, loan offers
- Lender terminal — supply capital, pick risk tranches, monitor the book
- Freighter / Albedo / xBull wallet support, plus Stellar's passkey-based smart wallets

---

## Repository Layout

```
kampay/
├── contract/          # Soroban smart contracts (Rust)
│   ├── payroll/
│   ├── escrow/
│   ├── dispute/
│   ├── credit/
│   ├── lending/
│   └── registry/
├── Frontend/          # Web application
└── README.md
```

---

## Getting Started

### Prerequisites

```bash
# Rust + the wasm target
rustup target add wasm32-unknown-unknown

# Stellar CLI
cargo install --locked stellar-cli

# Node (frontend)
node --version   # v20 or later
```

### Set up a testnet identity

```bash
stellar keys generate --global deployer --network testnet
stellar keys fund deployer --network testnet
stellar keys address deployer
```

### Build and test the contracts

```bash
cd contract
stellar contract build
cargo test
```

### Deploy to testnet

```bash
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/payroll.wasm \
  --source deployer \
  --network testnet
```

### Run the frontend

```bash
cd Frontend
npm install
cp .env.example .env.local   # set contract IDs and network passphrase
npm run dev
```

---

## Contract Interfaces

Illustrative signatures — see each crate for the authoritative interface.

**Payroll**

```rust
fn create_stream(employer: Address, worker: Address, token: Address,
                 rate_per_second: i128, start: u64, end: u64) -> u64;
fn fund_stream(stream_id: u64, amount: i128);
fn withdraw(stream_id: u64, worker: Address) -> i128;
fn batch_pay(employer: Address, payees: Vec<(Address, i128)>);
fn pause_stream(stream_id: u64, caller: Address);
```

**Escrow**

```rust
fn create_escrow(client: Address, provider: Address, token: Address,
                 milestones: Vec<Milestone>, arbiter: Option<Address>) -> u64;
fn fund(escrow_id: u64);
fn approve_milestone(escrow_id: u64, index: u32, client: Address);
fn claim_expired(escrow_id: u64, index: u32);   // auto-release after grace period
fn raise_dispute(escrow_id: u64, index: u32, caller: Address);
```

**Credit**

```rust
fn score_of(account: Address) -> u32;                 // 0–1000
fn record_of(account: Address) -> CreditRecord;
fn attest(account: Address, fields: Vec<Field>) -> Attestation;
```

**Lending**

```rust
fn quote(borrower: Address, amount: i128, term_days: u32) -> LoanTerms;
fn borrow(borrower: Address, amount: i128, term_days: u32) -> u64;
fn repay(loan_id: u64, amount: i128);
fn supply(lender: Address, tranche: RiskTier, amount: i128);
```

---

## Security Model

- **Funds are always accounted for.** Every unit of value sits in exactly one of: employer vault,
  escrow lock, frozen dispute pool, or contributor balance. No commingling.
- **Authorization is explicit.** Every state-changing call uses Soroban's `require_auth`; no implicit
  caller trust anywhere.
- **Upgrades are governed.** Contract upgrades run through a timelocked registry with a public
  delay window, so users can exit before a change takes effect.
- **Storage TTL is managed.** Long-lived records (credit history, active loans) are explicitly
  extended so critical state cannot expire out from under a user.
- **Oracles are minimized.** Release conditions prefer on-chain facts and party signatures over
  external feeds. Where an oracle is used, it is named at contract creation and cannot be swapped.
- **Not yet audited.** KamPay is pre-audit software. Do not use it with production funds.

---

## Roadmap

**Phase 1 — Payments**
- [ ] Payroll streams and batch disbursement
- [ ] Milestone escrow with conditional release
- [ ] Employer console and contributor dashboard

**Phase 2 — Trust**
- [ ] Grace periods and dispute freezing
- [ ] Arbiter resolution
- [ ] Staked juror panels for high-value contracts

**Phase 3 — Credit**
- [ ] `CreditRecord` and score computation
- [ ] Two-sided (employer and contributor) scoring
- [ ] Selective-disclosure attestations

**Phase 4 — Lending**
- [ ] Salary advances against accrued payroll
- [ ] Under-collateralized loans with payroll-linked repayment
- [ ] Open lender pools with risk tranches
- [ ] Invoice financing against escrowed milestones

**Phase 5 — Reach**
- [ ] Stellar Anchor integrations for local-currency off-ramps
- [ ] Passkey smart wallets for onboarding without seed phrases
- [ ] Public credit-record SDK so other protocols can consume KamPay scores

---

## Contributing

Contributions are welcome.

1. Fork the repo and branch from `main`.
2. Keep contract changes accompanied by tests — `cargo test` must pass.
3. Run `cargo fmt` and `cargo clippy` before opening a PR.
4. Describe the security implications of any change to fund custody or scoring logic.

For substantial features, open an issue to discuss the design first.

---

## License

MIT — see [LICENSE](LICENSE).
