use soroban_sdk::{contracttype, Address};

/// Everything the protocol knows about one account's payment behaviour.
///
/// Only counters live here — never a cached score. The score is derived on read
/// so that a weight change takes effect everywhere at once and nobody has to
/// trust a stored number.
#[contracttype]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CreditRecord {
    /// Timestamp of the first settled payment. Zero means no history.
    pub first_activity: u64,
    /// Timestamp of the most recent settled payment.
    pub last_activity: u64,

    /// Payments settled on or before their due date.
    pub payments_on_time: u32,
    /// Payments settled late.
    pub payments_late: u32,
    /// Lifetime value settled, in the token's smallest unit.
    pub total_volume: i128,

    /// Distinct 30-day periods containing at least one payment. Paired with
    /// tenure this measures whether income actually recurs.
    pub active_periods: u32,
    /// Index of the most recent active period, for de-duplication.
    pub last_period: u64,

    /// Distinct counterparties transacted with. Volume concentrated in a single
    /// counterparty is worth far less than the same volume spread across many.
    pub counterparties: u32,
    /// Running sum of counterparty scores at the time of each payment.
    pub counterparty_score_sum: u64,
    /// Number of samples in `counterparty_score_sum`.
    pub counterparty_samples: u32,

    /// Disputes raised against this account.
    pub disputes_total: u32,
    /// Disputes this account lost.
    pub disputes_lost: u32,

    pub loans_repaid: u32,
    pub loans_defaulted: u32,
}

/// Coarse credit band, used to gate lending terms.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Tier {
    /// Not enough history to score.
    Unrated,
    Bronze,
    Silver,
    Gold,
    Platinum,
}

/// The individual components behind a score, so a user can see *why* they were
/// rated the way they were rather than being handed an opaque number.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScoreBreakdown {
    pub payment_history: u32,
    pub tenure: u32,
    pub consistency: u32,
    pub counterparty_quality: u32,
    pub dispute_record: u32,
    pub loan_repayment: u32,
    pub total: u32,
}

#[contracttype]
pub enum DataKey {
    /// Address allowed to manage reporters.
    Admin,
    /// Contracts permitted to write history (payroll, escrow, lending).
    Reporter(Address),
    /// Credit record by account.
    Record(Address),
    /// Marks that `.0` has already transacted with `.1`, so repeat business
    /// does not inflate the distinct-counterparty count.
    Seen(Address, Address),
}
