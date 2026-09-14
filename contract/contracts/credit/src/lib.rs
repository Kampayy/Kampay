#![no_std]
//! # KamPay Credit
//!
//! On-chain credit records built from settled KamPay payments.
//!
//! The payroll, escrow, and lending contracts report settlement outcomes here.
//! Those counters are public, and the scoring weights are constants in
//! [`scoring`], so anyone can recompute a score independently — the contract
//! stores history, never a cached verdict.
//!
//! Scoring is two-sided: an employer who consistently pays late carries that on
//! their own record, visible to any contributor before they sign.

use soroban_sdk::{contract, contracterror, contractimpl, Address, Env};

mod events;
pub use events::*;

mod scoring;
pub use scoring::{MAX_SCORE, MIN_PAYMENTS_TO_SCORE, PERIOD};

mod types;
pub use types::*;

mod test;

/// ~5 seconds per ledger.
const DAY_LEDGERS: u32 = 17_280;
/// Credit history is long-lived by nature — always bump it to the maximum.
const RECORD_TTL: u32 = DAY_LEDGERS * 120;
const RECORD_TTL_THRESHOLD: u32 = DAY_LEDGERS * 90;

#[contract]
pub struct CreditContract;

#[contractimpl]
impl CreditContract {
    /// Set the admin that manages the reporter allowlist.
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .extend_ttl(RECORD_TTL_THRESHOLD, RECORD_TTL);
        Ok(())
    }

    /// Allow or revoke a contract's ability to write credit history.
    ///
    /// Only the payroll, escrow, and lending contracts should ever hold this.
    pub fn set_reporter(env: Env, reporter: Address, authorized: bool) -> Result<(), Error> {
        admin(&env)?.require_auth();

        env.storage()
            .persistent()
            .set(&DataKey::Reporter(reporter.clone()), &authorized);
        env.storage().persistent().extend_ttl(
            &DataKey::Reporter(reporter.clone()),
            RECORD_TTL_THRESHOLD,
            RECORD_TTL,
        );

        ReporterChanged {
            reporter,
            authorized,
        }
        .publish(&env);

        Ok(())
    }

    // ------------------------------------------------------------- reporting

    /// Record a settled payment against both parties.
    ///
    /// `on_time` reflects whether the payer met the agreed date — it lands on
    /// the payer's history as reliability and on the payee's as income quality.
    pub fn record_payment(
        env: Env,
        reporter: Address,
        payer: Address,
        payee: Address,
        amount: i128,
        on_time: bool,
    ) -> Result<(), Error> {
        require_reporter(&env, &reporter)?;

        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }
        if payer == payee {
            return Err(Error::SelfPayment);
        }

        let now = env.ledger().timestamp();

        // Snapshot each side's standing *before* this payment, so the two
        // updates cannot bootstrap each other within a single call.
        let payer_standing = scoring::standing(now, &load_record(&env, &payer));
        let payee_standing = scoring::standing(now, &load_record(&env, &payee));

        credit_side(&env, now, &payer, &payee, amount, on_time, payee_standing);
        credit_side(&env, now, &payee, &payer, amount, on_time, payer_standing);

        PaymentRecorded {
            payer,
            payee,
            amount,
            on_time,
        }
        .publish(&env);

        Ok(())
    }

    /// Record the outcome of a dispute raised against `account`.
    pub fn record_dispute(
        env: Env,
        reporter: Address,
        account: Address,
        lost: bool,
    ) -> Result<(), Error> {
        require_reporter(&env, &reporter)?;

        let mut record = load_record(&env, &account);
        record.disputes_total += 1;
        if lost {
            record.disputes_lost += 1;
        }
        save_record(&env, &account, &record);

        DisputeRecorded { account, lost }.publish(&env);

        Ok(())
    }

    /// Record a loan reaching a terminal state.
    pub fn record_loan(
        env: Env,
        reporter: Address,
        borrower: Address,
        repaid: bool,
    ) -> Result<(), Error> {
        require_reporter(&env, &reporter)?;

        let mut record = load_record(&env, &borrower);
        if repaid {
            record.loans_repaid += 1;
        } else {
            record.loans_defaulted += 1;
        }
        save_record(&env, &borrower, &record);

        LoanRecorded { borrower, repaid }.publish(&env);

        Ok(())
    }

    // ----------------------------------------------------------------- views

    /// Credit score out of 1000. Zero until the account has enough history.
    pub fn score_of(env: Env, account: Address) -> u32 {
        let now = env.ledger().timestamp();
        scoring::score(now, &load_record(&env, &account))
    }

    /// Per-component scores behind the total, so a user can see what is holding
    /// their rating back.
    pub fn breakdown_of(env: Env, account: Address) -> ScoreBreakdown {
        let now = env.ledger().timestamp();
        scoring::breakdown(now, &load_record(&env, &account))
    }

    pub fn tier_of(env: Env, account: Address) -> Tier {
        let now = env.ledger().timestamp();
        scoring::tier(now, &load_record(&env, &account))
    }

    /// Raw counters. Every score is recomputable from these.
    pub fn record_of(env: Env, account: Address) -> CreditRecord {
        load_record(&env, &account)
    }

    pub fn is_reporter(env: Env, reporter: Address) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::Reporter(reporter))
            .unwrap_or(false)
    }

    pub fn get_admin(env: Env) -> Result<Address, Error> {
        admin(&env)
    }
}

// ------------------------------------------------------------------ internals

/// Apply one payment to one party's record.
///
/// `counterparty_standing` is the other side's standing as of before this
/// payment — [`scoring::standing`], so unrated partners register as neutral.
fn credit_side(
    env: &Env,
    now: u64,
    account: &Address,
    counterparty: &Address,
    amount: i128,
    on_time: bool,
    counterparty_standing: u32,
) {
    let mut record = load_record(env, account);

    if record.first_activity == 0 {
        record.first_activity = now;
    }
    record.last_activity = now;

    if on_time {
        record.payments_on_time += 1;
    } else {
        record.payments_late += 1;
    }
    record.total_volume += amount;

    // Count each 30-day period once, however many payments it contained.
    let period = now / scoring::PERIOD;
    if record.active_periods == 0 || period > record.last_period {
        record.active_periods += 1;
        record.last_period = period;
    }

    // Only the first payment with a given counterparty adds to diversity.
    let seen_key = DataKey::Seen(account.clone(), counterparty.clone());
    if !env.storage().persistent().has(&seen_key) {
        record.counterparties += 1;
        env.storage().persistent().set(&seen_key, &true);
        env.storage()
            .persistent()
            .extend_ttl(&seen_key, RECORD_TTL_THRESHOLD, RECORD_TTL);
    }

    record.counterparty_score_sum += counterparty_standing as u64;
    record.counterparty_samples += 1;

    save_record(env, account, &record);
}

fn admin(env: &Env) -> Result<Address, Error> {
    env.storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(Error::NotInitialized)
}

fn require_reporter(env: &Env, reporter: &Address) -> Result<(), Error> {
    reporter.require_auth();
    let authorized: bool = env
        .storage()
        .persistent()
        .get(&DataKey::Reporter(reporter.clone()))
        .unwrap_or(false);
    if authorized {
        Ok(())
    } else {
        Err(Error::NotAuthorized)
    }
}

fn load_record(env: &Env, account: &Address) -> CreditRecord {
    let key = DataKey::Record(account.clone());
    match env.storage().persistent().get::<_, CreditRecord>(&key) {
        Some(record) => {
            env.storage()
                .persistent()
                .extend_ttl(&key, RECORD_TTL_THRESHOLD, RECORD_TTL);
            record
        }
        None => CreditRecord::default(),
    }
}

fn save_record(env: &Env, account: &Address, record: &CreditRecord) {
    let key = DataKey::Record(account.clone());
    env.storage().persistent().set(&key, record);
    env.storage()
        .persistent()
        .extend_ttl(&key, RECORD_TTL_THRESHOLD, RECORD_TTL);
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    NotAuthorized = 3,
    InvalidAmount = 4,
    SelfPayment = 5,
}
