//! Score computation.
//!
//! Every input is a public on-chain counter and every weight is a constant in
//! this file, so any third party can recompute a score from scratch and get the
//! same answer. That is the whole point — a score nobody can audit is just a
//! credit bureau with extra steps.

use crate::types::{CreditRecord, ScoreBreakdown, Tier};

/// Scores are expressed out of 1000.
pub const MAX_SCORE: u32 = 1_000;

/// A 30-day accounting period.
pub const PERIOD: u64 = 30 * 86_400;

/// Settled payments required before an account is scored at all. Below this a
/// record is `Unrated` rather than optimistically scored on tenure alone.
pub const MIN_PAYMENTS_TO_SCORE: u32 = 3;

// Component weights, in percent. Must sum to 100.
const W_PAYMENT_HISTORY: u32 = 35;
const W_TENURE: u32 = 20;
const W_CONSISTENCY: u32 = 15;
const W_COUNTERPARTY: u32 = 10;
const W_DISPUTES: u32 = 10;
const W_LOANS: u32 = 10;

/// Tenure needed for full marks: 36 periods ≈ 3 years.
const TENURE_PERIODS_FOR_MAX: u32 = 36;

/// Score assigned where an account has no history in a component yet. Neutral
/// rather than zero, so never having borrowed is not treated as defaulting.
const NEUTRAL: u32 = 500;

/// Cost of each lost dispute, in points off the dispute component.
const DISPUTE_LOSS_PENALTY: u32 = 250;

/// Distinct counterparties needed for full marks on diversity.
const COUNTERPARTIES_FOR_MAX: u32 = 5;

/// Total settled payments, on time or otherwise.
pub fn payment_count(record: &CreditRecord) -> u32 {
    record.payments_on_time + record.payments_late
}

/// Share of payments that settled on time.
fn payment_history(record: &CreditRecord) -> u32 {
    let total = payment_count(record);
    if total == 0 {
        return 0;
    }
    record.payments_on_time * MAX_SCORE / total
}

/// How long this account has been transacting, saturating at three years.
fn tenure(now: u64, record: &CreditRecord) -> u32 {
    if record.first_activity == 0 || now <= record.first_activity {
        return 0;
    }
    let periods = ((now - record.first_activity) / PERIOD) as u32;
    core::cmp::min(MAX_SCORE, periods * MAX_SCORE / TENURE_PERIODS_FOR_MAX)
}

/// Fraction of elapsed periods that actually contained a payment. Catches the
/// difference between steady income and one big burst two years ago.
fn consistency(now: u64, record: &CreditRecord) -> u32 {
    if record.first_activity == 0 {
        return 0;
    }
    let elapsed = if now > record.first_activity {
        ((now - record.first_activity) / PERIOD) as u32
    } else {
        0
    };
    let total_periods = elapsed + 1;
    core::cmp::min(MAX_SCORE, record.active_periods * MAX_SCORE / total_periods)
}

/// How credible this account's transaction graph looks.
///
/// Half of it is **diversity** — how many distinct counterparties, saturating at
/// five. Half is **standing** — the average score of those counterparties at the
/// time they transacted, with unrated counterparties counted as [`NEUTRAL`]
/// rather than zero, because a new employer is unknown, not delinquent.
///
/// Diversity is what makes a closed loop expensive: two accounts paying each
/// other forever cap out at a fifth of the diversity half, however much volume
/// they push through.
///
/// This is a brake, not a wall. A determined attacker can open five funded
/// accounts and pay themselves, and the graph alone cannot distinguish that
/// from five genuine clients. What it costs them is real capital in motion and
/// counterparties whose own standing never rises above neutral — so the ceiling
/// is mediocre, not good. Strong scores still require partners who are
/// themselves well-rated, which is recursively expensive to fake.
fn counterparty_quality(record: &CreditRecord) -> u32 {
    if record.counterparty_samples == 0 {
        return NEUTRAL;
    }

    let diversity = core::cmp::min(record.counterparties, COUNTERPARTIES_FOR_MAX) * MAX_SCORE
        / COUNTERPARTIES_FOR_MAX;
    let standing = (record.counterparty_score_sum / record.counterparty_samples as u64) as u32;

    (diversity + standing) / 2
}

/// Clean until proven otherwise; each lost dispute is expensive.
fn dispute_record(record: &CreditRecord) -> u32 {
    if record.disputes_total == 0 {
        return MAX_SCORE;
    }
    MAX_SCORE.saturating_sub(record.disputes_lost * DISPUTE_LOSS_PENALTY)
}

/// Repayment rate on prior KamPay loans; neutral for accounts that never borrowed.
fn loan_repayment(record: &CreditRecord) -> u32 {
    let total = record.loans_repaid + record.loans_defaulted;
    if total == 0 {
        return NEUTRAL;
    }
    record.loans_repaid * MAX_SCORE / total
}

/// Full component breakdown plus the weighted total.
pub fn breakdown(now: u64, record: &CreditRecord) -> ScoreBreakdown {
    let payment_history = payment_history(record);
    let tenure = tenure(now, record);
    let consistency = consistency(now, record);
    let counterparty_quality = counterparty_quality(record);
    let dispute_record = dispute_record(record);
    let loan_repayment = loan_repayment(record);

    let total = if payment_count(record) < MIN_PAYMENTS_TO_SCORE {
        0
    } else {
        (payment_history * W_PAYMENT_HISTORY
            + tenure * W_TENURE
            + consistency * W_CONSISTENCY
            + counterparty_quality * W_COUNTERPARTY
            + dispute_record * W_DISPUTES
            + loan_repayment * W_LOANS)
            / 100
    };

    ScoreBreakdown {
        payment_history,
        tenure,
        consistency,
        counterparty_quality,
        dispute_record,
        loan_repayment,
        total,
    }
}

pub fn score(now: u64, record: &CreditRecord) -> u32 {
    breakdown(now, record).total
}

/// A counterparty's score as it should be recorded against the other side.
///
/// Accounts too new to rate contribute [`NEUTRAL`], not zero — otherwise
/// working with new clients would look identical to working with bad ones.
pub fn standing(now: u64, record: &CreditRecord) -> u32 {
    if payment_count(record) < MIN_PAYMENTS_TO_SCORE {
        NEUTRAL
    } else {
        score(now, record)
    }
}

pub fn tier(now: u64, record: &CreditRecord) -> Tier {
    if payment_count(record) < MIN_PAYMENTS_TO_SCORE {
        return Tier::Unrated;
    }
    match score(now, record) {
        0..=399 => Tier::Bronze,
        400..=599 => Tier::Silver,
        600..=799 => Tier::Gold,
        _ => Tier::Platinum,
    }
}

#[cfg(test)]
mod weights {
    use super::*;

    #[test]
    fn component_weights_sum_to_one_hundred() {
        assert_eq!(
            W_PAYMENT_HISTORY + W_TENURE + W_CONSISTENCY + W_COUNTERPARTY + W_DISPUTES + W_LOANS,
            100
        );
    }
}
