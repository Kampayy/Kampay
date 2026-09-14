# KamPay

**Programmable payroll, escrow, and on-chain credit — built on Stellar with Soroban.**

KamPay lets companies lock funds upfront and pay contributors automatically: recurring payroll for
employees, escrow-based milestones for freelancers and contractors. Every payment builds a verifiable
on-chain **credit score**, which unlocks access to under-collateralized loans from lenders in the
KamPay credit market.

No trust assumptions. No chasing invoices. No two-week settlement windows.

> **Status:** early development. The payroll, escrow, and credit contracts are written, tested
> (57 tests), and building to WASM; the lending contract is designed but not yet implemented, and
> nothing has been deployed to mainnet or audited. See the [roadmap](#roadmap) for what is done and
> what is not.

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
- [Testing](#testing)
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
- **Release conditions.** Client approval, or deadline expiry with auto-release once the grace
  period lapses. Oracle-attested release is planned, not built.
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
| **Income consistency** | 15% | Share of elapsed 30-day periods that contained a payment |
| **Counterparty quality** | 10% | Half counterparty diversity, half their own credit standing |
| **Dispute record** | 10% | Disputes *lost* — being disputed and vindicated costs nothing |
| **Loan repayment** | 10% | On-time repayment of prior KamPay loans |

An account stays **Unrated** until it has at least three settled payments, so a
fresh address cannot coast on tenure alone. Scores map to tiers — Bronze,
Silver, Gold, Platinum — which gate lending terms.

Properties that matter:

- **Portable.** The record lives on Stellar, not in KamPay's database. Any protocol can read it.
- **Recomputable.** Only counters are stored, never a cached score. Every input is a public event
  and every weight is a constant in [`scoring.rs`](contract/contracts/credit/src/scoring.rs), so a
  third party can derive the same number from scratch. `breakdown_of()` returns each component, so
  a user can see what is holding their rating back instead of getting an opaque verdict.
- **Two-sided.** Employers are scored too. A company that consistently pays late is visible to
  contributors before they sign.
- **Charitable to newcomers.** A counterparty with too little history to rate counts as *neutral*,
  not zero — working with new clients must not look like working with bad ones.
- **Volume-blind.** Total volume is tracked for display and loan sizing but is deliberately not a
  scoring input. Moving more money cannot buy a rating.

On sybil resistance, honestly: counterparty diversity is half the graph score, so a closed loop of
two accounts paying each other caps out low however much volume they push through. That is a brake,
not a wall. A determined attacker can fund several accounts and pay themselves; what it costs them
is real capital in motion and counterparties whose own standing never rises above neutral, so the
ceiling is mediocre rather than good. The limitation is documented in the contract rather than
papered over.

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

Shipped, tested, and building to WASM today:

| Crate | Responsibility |
| --- | --- |
| [`payroll`](contract/contracts/payroll) | Salary streams, funding, withdrawals, pause/resume/cancel, batch disbursement |
| [`escrow`](contract/contracts/escrow) | Milestone definition, funding, conditional release, grace-period auto-claim, disputes |
| [`credit`](contract/contracts/credit) | Credit records, score computation, reporter allowlist |

Planned, not yet written:

| Crate | Responsibility |
| --- | --- |
| `lending` | Loan origination, lender pools, payroll repayment hooks, liquidation |
| `registry` | Contract discovery, timelocked upgrade authority, protocol parameters |

Dispute handling currently lives inside `escrow` rather than in a separate
crate; it moves out when staked juror panels land.

Contracts are composed, not monolithic: `payroll` and `escrow` emit settlement
events that `credit` consumes; `lending` will read `credit` and install a
repayment hook back into `payroll`.

### Frontend

Next.js 16 (App Router) with TypeScript and Tailwind v4. The marketing homepage
is built; the authenticated surfaces are next:

- Marketing homepage — **built**
- Employer console — fund vaults, run payroll, open escrows, manage disputes
- Contributor dashboard — earnings, withdrawals, credit score, loan offers
- Lender terminal — supply capital, pick risk tranches, monitor the book
- Freighter / Albedo / xBull wallet support, plus Stellar's passkey smart wallets

---

## Repository Layout

```
kampay/
├── contract/                   # Soroban workspace (Rust)
│   ├── Cargo.toml              # workspace manifest, shared release profile
│   ├── rust-toolchain.toml     # pins the wasm32v1-none target
│   └── contracts/
│       ├── payroll/            # salary streams + batch pay
│       ├── escrow/             # milestone escrow + disputes
│       └── credit/             # credit records + scoring
├── Frontend/                   # Next.js 16 app
│   └── src/
│       ├── app/                # App Router entry, layout, global tokens
│       └── components/         # homepage sections and site chrome
└── README.md
```

## Getting Started

### Prerequisites

```bash
# Rust plus the Soroban wasm target
rustup target add wasm32v1-none

# Stellar CLI (v23 or later)
cargo install --locked stellar-cli

node --version   # v20 or later
```

### Build and test the contracts

```bash
cd contract
cargo test              # 57 tests across the three crates
stellar contract build  # -> target/wasm32v1-none/release/*.wasm
```

Lints and formatting are expected to be clean:

```bash
cargo clippy --all-targets
cargo fmt --check
```

### Deploy to testnet

```bash
stellar keys generate --global deployer --network testnet
stellar keys fund deployer --network testnet

stellar contract deploy \
  --wasm target/wasm32v1-none/release/payroll.wasm \
  --source deployer \
  --network testnet
```

The `credit` contract needs one extra step after deployment — it only accepts
history from an allowlist, so initialise it and register the contracts that are
permitted to report:

```bash
stellar contract invoke --id <CREDIT_ID> --source deployer --network testnet \
  -- initialize --admin <ADMIN_ADDRESS>

stellar contract invoke --id <CREDIT_ID> --source deployer --network testnet \
  -- set_reporter --reporter <PAYROLL_ID> --authorized true
```

### Run the frontend

```bash
cd Frontend
npm install
npm run dev     # http://localhost:3000
```

```bash
npm run build   # production build
npm run lint
```

## Contract Interfaces

Actual signatures, abridged. See each crate for the full surface and its errors.

**Payroll** — [`contract/contracts/payroll`](contract/contracts/payroll)

```rust
fn create_stream(employer: Address, worker: Address, token: Address,
                 rate_per_second: i128, start: u64, end: u64) -> Result<u64, Error>;
fn fund(stream_id: u64, from: Address, amount: i128) -> Result<(), Error>;
fn withdraw(stream_id: u64) -> Result<i128, Error>;
fn pause(stream_id: u64) -> Result<(), Error>;
fn resume(stream_id: u64) -> Result<(), Error>;
fn cancel(stream_id: u64) -> Result<(), Error>;
fn batch_pay(employer: Address, token: Address,
             payments: Vec<Payment>) -> Result<i128, Error>;

// views
fn withdrawable(stream_id: u64) -> Result<i128, Error>;
fn runway(stream_id: u64) -> Result<u64, Error>;   // funded seconds remaining
```

**Escrow** — [`contract/contracts/escrow`](contract/contracts/escrow)

```rust
fn create_escrow(client: Address, provider: Address, token: Address,
                 milestones: Vec<Milestone>,
                 arbiter: Option<Address>) -> Result<u64, Error>;
fn fund(escrow_id: u64) -> Result<i128, Error>;
fn approve_milestone(escrow_id: u64, index: u32) -> Result<i128, Error>;
fn claim_expired(escrow_id: u64, index: u32) -> Result<i128, Error>;
fn raise_dispute(escrow_id: u64, index: u32, by: Address) -> Result<(), Error>;
fn resolve_dispute(escrow_id: u64, index: u32, to_provider: i128) -> Result<(), Error>;
fn cancel(escrow_id: u64) -> Result<i128, Error>;   // requires both signatures

// views
fn is_claimable(escrow_id: u64, index: u32) -> Result<bool, Error>;
fn locked_value(escrow_id: u64) -> Result<i128, Error>;
```

**Credit** — [`contract/contracts/credit`](contract/contracts/credit)

```rust
fn initialize(admin: Address) -> Result<(), Error>;
fn set_reporter(reporter: Address, authorized: bool) -> Result<(), Error>;

// reporting — allowlisted contracts only
fn record_payment(reporter: Address, payer: Address, payee: Address,
                  amount: i128, on_time: bool) -> Result<(), Error>;
fn record_dispute(reporter: Address, account: Address, lost: bool) -> Result<(), Error>;
fn record_loan(reporter: Address, borrower: Address, repaid: bool) -> Result<(), Error>;

// views
fn score_of(account: Address) -> u32;                  // 0-1000
fn breakdown_of(account: Address) -> ScoreBreakdown;   // per-component scores
fn tier_of(account: Address) -> Tier;
fn record_of(account: Address) -> CreditRecord;        // the raw counters
```

Selective-disclosure attestations are on the roadmap and not yet implemented.

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
  external feeds. An escrow's arbiter is named at creation and cannot be swapped afterwards, so
  neither side can install a friendly judge mid-contract.
- **Credit history is allowlisted.** Only reporter contracts registered by the admin can write to a
  `CreditRecord`. Everything else is read-only.
- **Not yet audited.** KamPay is pre-audit software. Do not use it with production funds.

---

## Testing

```bash
cd contract && cargo test
```

| Crate | Tests | Covers |
| --- | --- | --- |
| `payroll` | 15 | Linear accrual, funding caps, pause semantics, cancellation splits, end-date bounding, batch pay, every error path |
| `escrow` | 19 | Funding, partial release, grace-period expiry, dispute freezing, arbiter splits, mutual cancellation, every error path |
| `credit` | 23 | Score gating, two-sided recording, tenure saturation, consistency, counterparty diversity, dispute and loan effects, reporter authorization |

Some tests exist specifically to pin down behaviour that is easy to get backwards:

- `accrual_is_capped_by_what_was_actually_deposited` — an underfunded stream cannot promise more
  than it holds.
- `paused_time_never_accrues` — pausing must not silently back-pay when resumed.
- `a_closed_payment_loop_scores_worse_than_diversified_history` — the sybil brake actually bites.
  An earlier version of the scoring model failed this test by rewarding a two-account loop over an
  honest worker; the metric was reworked rather than the assertion weakened.
- `volume_is_tracked_but_never_scored` — moving more money must not buy a rating.
- `the_score_is_recomputable_from_the_public_record` — the published weights reproduce the
  published total.

---

## Roadmap

**Phase 1 — Payments**
- [x] Payroll streams with per-second accrual, pause/resume, and early settlement
- [x] Batch disbursement for whole-team runs
- [x] Milestone escrow with conditional release
- [x] Marketing homepage
- [ ] Employer console and contributor dashboard
- [ ] Wallet integration (Freighter, Albedo, xBull)
- [ ] Testnet deployment with published contract IDs

**Phase 2 — Trust**
- [x] Grace periods and provider auto-claim
- [x] Dispute freezing and arbiter resolution
- [ ] Staked juror panels for high-value contracts
- [ ] Extract dispute logic into its own crate

**Phase 3 — Credit**
- [x] `CreditRecord` counters and score computation
- [x] Two-sided scoring for employers and contributors
- [x] Per-component breakdown so a score can be explained
- [ ] Wire payroll and escrow settlement into the credit contract as reporters
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
- [ ] Third-party security audit

## Contributing

Contributions are welcome.

1. Fork the repo and branch from `main`.
2. Keep contract changes accompanied by tests — `cargo test` must pass.
3. Run `cargo fmt` and `cargo clippy --all-targets` before opening a PR; both are expected clean.
4. For frontend changes, `npm run build` and `npm run lint` must pass.
5. Describe the security implications of any change to fund custody or scoring logic.

If you change a scoring weight in `scoring.rs`, update the table in this README and the two places
the frontend mirrors it — the homepage credit card and the credit section.

For substantial features, open an issue to discuss the design first.

---

## License

MIT — see [LICENSE](LICENSE).
