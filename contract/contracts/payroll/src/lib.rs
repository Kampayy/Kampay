#![no_std]
//! # KamPay Payroll
//!
//! Recurring salary streams funded upfront by an employer and withdrawable at any
//! time by the worker.
//!
//! A stream accrues linearly at `rate_per_second` between `start` and `end`. The
//! worker may withdraw whatever has accrued but not yet been paid out, capped by
//! the amount the employer has actually deposited — so a contributor can always
//! verify on-chain that the money backing their next paycheck exists.

use soroban_sdk::{contract, contracterror, contractimpl, token, Address, Env, Vec};

mod events;
pub use events::*;

mod types;
pub use types::*;

mod test;

/// ~5 seconds per ledger.
const DAY_LEDGERS: u32 = 17_280;
/// Extend stream records to ~120 days whenever they are touched.
const STREAM_TTL: u32 = DAY_LEDGERS * 120;
/// Bump the TTL once it drops below ~90 days.
const STREAM_TTL_THRESHOLD: u32 = DAY_LEDGERS * 90;

#[contract]
pub struct PayrollContract;

#[contractimpl]
impl PayrollContract {
    // ---------------------------------------------------------------- streams

    /// Open a salary stream. Does not move funds — call `fund` next.
    ///
    /// Returns the new stream id.
    pub fn create_stream(
        env: Env,
        employer: Address,
        worker: Address,
        token: Address,
        rate_per_second: i128,
        start: u64,
        end: u64,
    ) -> Result<u64, Error> {
        employer.require_auth();

        if rate_per_second <= 0 {
            return Err(Error::InvalidRate);
        }
        if end <= start {
            return Err(Error::InvalidTimeRange);
        }
        if employer == worker {
            return Err(Error::SelfPayment);
        }

        let stream_id = next_stream_id(&env);
        let stream = Stream {
            employer: employer.clone(),
            worker: worker.clone(),
            token,
            rate_per_second,
            start,
            end,
            deposited: 0,
            withdrawn: 0,
            status: StreamStatus::Active,
            paused_at: 0,
            paused_seconds: 0,
        };

        save_stream(&env, stream_id, &stream);
        index_push(&env, &DataKey::EmployerStreams(employer.clone()), stream_id);
        index_push(&env, &DataKey::WorkerStreams(worker.clone()), stream_id);

        StreamCreated {
            stream_id,
            employer,
            worker,
            token: stream.token,
            rate_per_second,
            start,
            end,
        }
        .publish(&env);

        Ok(stream_id)
    }

    /// Deposit `amount` into a stream. Anyone may top a stream up, but the funds
    /// are irrevocably committed to the worker's accrual schedule.
    pub fn fund(env: Env, stream_id: u64, from: Address, amount: i128) -> Result<(), Error> {
        from.require_auth();

        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        let mut stream = load_stream(&env, stream_id)?;
        if stream.status == StreamStatus::Cancelled {
            return Err(Error::StreamCancelled);
        }

        token::Client::new(&env, &stream.token).transfer(
            &from,
            &env.current_contract_address(),
            &amount,
        );

        stream.deposited += amount;
        save_stream(&env, stream_id, &stream);

        StreamFunded {
            stream_id,
            from,
            amount,
            total_deposited: stream.deposited,
        }
        .publish(&env);

        Ok(())
    }

    /// Withdraw everything the worker has accrued and not yet been paid.
    ///
    /// Returns the amount transferred.
    pub fn withdraw(env: Env, stream_id: u64) -> Result<i128, Error> {
        let mut stream = load_stream(&env, stream_id)?;
        stream.worker.require_auth();

        let amount = withdrawable_of(&env, &stream);
        if amount <= 0 {
            return Err(Error::NothingToWithdraw);
        }

        stream.withdrawn += amount;
        save_stream(&env, stream_id, &stream);

        token::Client::new(&env, &stream.token).transfer(
            &env.current_contract_address(),
            &stream.worker,
            &amount,
        );

        Withdrawn {
            stream_id,
            worker: stream.worker,
            amount,
            total_withdrawn: stream.withdrawn,
        }
        .publish(&env);

        Ok(amount)
    }

    /// Suspend accrual. Time spent paused never accrues to the worker.
    pub fn pause(env: Env, stream_id: u64) -> Result<(), Error> {
        let mut stream = load_stream(&env, stream_id)?;
        stream.employer.require_auth();

        match stream.status {
            StreamStatus::Paused => return Err(Error::StreamPaused),
            StreamStatus::Cancelled => return Err(Error::StreamCancelled),
            StreamStatus::Active => {}
        }

        stream.status = StreamStatus::Paused;
        stream.paused_at = env.ledger().timestamp();
        save_stream(&env, stream_id, &stream);

        StreamPaused {
            stream_id,
            at: stream.paused_at,
        }
        .publish(&env);

        Ok(())
    }

    /// Resume accrual after a pause.
    pub fn resume(env: Env, stream_id: u64) -> Result<(), Error> {
        let mut stream = load_stream(&env, stream_id)?;
        stream.employer.require_auth();

        if stream.status != StreamStatus::Paused {
            return Err(Error::StreamNotPaused);
        }

        let now = env.ledger().timestamp();
        let pause_start = core::cmp::max(stream.paused_at, stream.start);
        let resume_at = core::cmp::min(now, stream.end);
        if resume_at > pause_start {
            stream.paused_seconds = stream.paused_seconds.saturating_add(resume_at - pause_start);
        }

        stream.status = StreamStatus::Active;
        stream.paused_at = 0;
        save_stream(&env, stream_id, &stream);

        StreamResumed {
            stream_id,
            paused_seconds: stream.paused_seconds,
        }
        .publish(&env);

        Ok(())
    }

    /// Settle a stream early: the worker is paid everything accrued to this
    /// moment and the employer is refunded the untouched remainder.
    pub fn cancel(env: Env, stream_id: u64) -> Result<(), Error> {
        let mut stream = load_stream(&env, stream_id)?;
        stream.employer.require_auth();

        if stream.status == StreamStatus::Cancelled {
            return Err(Error::StreamCancelled);
        }

        let owed = withdrawable_of(&env, &stream);
        let refund = stream.deposited - stream.withdrawn - owed;
        let client = token::Client::new(&env, &stream.token);
        let contract = env.current_contract_address();

        if owed > 0 {
            client.transfer(&contract, &stream.worker, &owed);
            stream.withdrawn += owed;
        }
        if refund > 0 {
            client.transfer(&contract, &stream.employer, &refund);
            stream.deposited -= refund;
        }

        stream.status = StreamStatus::Cancelled;
        save_stream(&env, stream_id, &stream);

        StreamCancelled {
            stream_id,
            paid_to_worker: owed,
            refunded_to_employer: refund,
        }
        .publish(&env);

        Ok(())
    }

    // ------------------------------------------------------------- batch pay

    /// One-shot disbursement to many payees in a single transaction.
    ///
    /// Stellar's fees make this practical for whole-team payroll runs.
    pub fn batch_pay(
        env: Env,
        employer: Address,
        token_address: Address,
        payments: Vec<Payment>,
    ) -> Result<i128, Error> {
        employer.require_auth();

        if payments.is_empty() {
            return Err(Error::EmptyBatch);
        }

        let client = token::Client::new(&env, &token_address);
        let mut total: i128 = 0;

        for payment in payments.iter() {
            if payment.amount <= 0 {
                return Err(Error::InvalidAmount);
            }
            client.transfer(&employer, &payment.to, &payment.amount);
            total += payment.amount;
        }

        BatchPaid {
            employer,
            token: token_address,
            payee_count: payments.len(),
            total,
        }
        .publish(&env);

        Ok(total)
    }

    // ----------------------------------------------------------------- views

    pub fn get_stream(env: Env, stream_id: u64) -> Result<Stream, Error> {
        load_stream(&env, stream_id)
    }

    /// Amount the worker could withdraw right now.
    pub fn withdrawable(env: Env, stream_id: u64) -> Result<i128, Error> {
        let stream = load_stream(&env, stream_id)?;
        Ok(withdrawable_of(&env, &stream))
    }

    /// Total accrued over the life of the stream, paid or not.
    pub fn accrued(env: Env, stream_id: u64) -> Result<i128, Error> {
        let stream = load_stream(&env, stream_id)?;
        Ok(accrued_of(&env, &stream))
    }

    /// Seconds of funding left at the current rate. Zero means the stream is
    /// running on empty and the worker should be warned.
    pub fn runway(env: Env, stream_id: u64) -> Result<u64, Error> {
        let stream = load_stream(&env, stream_id)?;
        let unfunded = stream.deposited - accrued_of(&env, &stream);
        if unfunded <= 0 {
            return Ok(0);
        }
        Ok((unfunded / stream.rate_per_second) as u64)
    }

    pub fn streams_of_employer(env: Env, employer: Address) -> Vec<u64> {
        index_get(&env, &DataKey::EmployerStreams(employer))
    }

    pub fn streams_of_worker(env: Env, worker: Address) -> Vec<u64> {
        index_get(&env, &DataKey::WorkerStreams(worker))
    }
}

// ------------------------------------------------------------------ internals

/// Seconds of accrual earned so far, excluding any time spent paused.
fn elapsed_of(env: &Env, stream: &Stream) -> u64 {
    let now = env.ledger().timestamp();
    if now <= stream.start {
        return 0;
    }

    let capped = core::cmp::min(now, stream.end);
    let gross = capped - stream.start;

    let mut paused = stream.paused_seconds;
    if stream.status == StreamStatus::Paused {
        let pause_start = core::cmp::max(stream.paused_at, stream.start);
        if capped > pause_start {
            paused = paused.saturating_add(capped - pause_start);
        }
    }

    gross.saturating_sub(paused)
}

/// Total value accrued, never exceeding what the employer actually deposited.
fn accrued_of(env: &Env, stream: &Stream) -> i128 {
    let earned = (elapsed_of(env, stream) as i128) * stream.rate_per_second;
    core::cmp::min(earned, stream.deposited)
}

fn withdrawable_of(env: &Env, stream: &Stream) -> i128 {
    accrued_of(env, stream) - stream.withdrawn
}

fn next_stream_id(env: &Env) -> u64 {
    let key = DataKey::NextStreamId;
    let id: u64 = env.storage().instance().get(&key).unwrap_or(0);
    env.storage().instance().set(&key, &(id + 1));
    env.storage().instance().extend_ttl(STREAM_TTL_THRESHOLD, STREAM_TTL);
    id
}

fn save_stream(env: &Env, stream_id: u64, stream: &Stream) {
    let key = DataKey::Stream(stream_id);
    env.storage().persistent().set(&key, stream);
    env.storage()
        .persistent()
        .extend_ttl(&key, STREAM_TTL_THRESHOLD, STREAM_TTL);
}

fn load_stream(env: &Env, stream_id: u64) -> Result<Stream, Error> {
    let key = DataKey::Stream(stream_id);
    let stream: Stream = env
        .storage()
        .persistent()
        .get(&key)
        .ok_or(Error::StreamNotFound)?;
    env.storage()
        .persistent()
        .extend_ttl(&key, STREAM_TTL_THRESHOLD, STREAM_TTL);
    Ok(stream)
}

fn index_push(env: &Env, key: &DataKey, stream_id: u64) {
    let mut ids: Vec<u64> = env
        .storage()
        .persistent()
        .get(key)
        .unwrap_or_else(|| Vec::new(env));
    ids.push_back(stream_id);
    env.storage().persistent().set(key, &ids);
    env.storage()
        .persistent()
        .extend_ttl(key, STREAM_TTL_THRESHOLD, STREAM_TTL);
}

fn index_get(env: &Env, key: &DataKey) -> Vec<u64> {
    env.storage()
        .persistent()
        .get(key)
        .unwrap_or_else(|| Vec::new(env))
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    StreamNotFound = 1,
    InvalidTimeRange = 2,
    InvalidRate = 3,
    InvalidAmount = 4,
    StreamPaused = 5,
    StreamNotPaused = 6,
    StreamCancelled = 7,
    NothingToWithdraw = 8,
    EmptyBatch = 9,
    SelfPayment = 10,
}
