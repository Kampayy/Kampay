#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    Address, Env,
};

const MONTH: u64 = 30 * 86_400;
const YEAR: u64 = 12 * MONTH;

struct Fixture {
    env: Env,
    contract: CreditContractClient<'static>,
    reporter: Address,
}

fn setup() -> Fixture {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(1_700_000_000);

    let admin = Address::generate(&env);
    let reporter = Address::generate(&env);

    let contract_id = env.register(CreditContract, ());
    let contract = CreditContractClient::new(&env, &contract_id);
    contract.initialize(&admin);
    contract.set_reporter(&reporter, &true);

    Fixture {
        env,
        contract,
        reporter,
    }
}

impl Fixture {
    fn pay(&self, payer: &Address, payee: &Address, amount: i128, on_time: bool) {
        self.contract
            .record_payment(&self.reporter, payer, payee, &amount, &on_time);
    }

    fn advance(&self, seconds: u64) {
        let now = self.env.ledger().timestamp();
        self.env.ledger().set_timestamp(now + seconds);
    }

    /// Build a worker with `count` on-time payments, one per month, each from a
    /// distinct employer.
    fn seasoned_worker(&self, count: u32) -> Address {
        let worker = Address::generate(&self.env);
        for _ in 0..count {
            let employer = Address::generate(&self.env);
            self.pay(&employer, &worker, 5_000, true);
            self.advance(MONTH);
        }
        worker
    }
}

#[test]
fn a_fresh_account_is_unrated() {
    let f = setup();
    let account = Address::generate(&f.env);

    assert_eq!(f.contract.score_of(&account), 0);
    assert_eq!(f.contract.tier_of(&account), Tier::Unrated);
    assert_eq!(f.contract.record_of(&account).payments_on_time, 0);
}

#[test]
fn scoring_is_withheld_until_there_is_enough_history() {
    let f = setup();
    let worker = Address::generate(&f.env);
    let employer = Address::generate(&f.env);

    f.pay(&employer, &worker, 1_000, true);
    f.pay(&employer, &worker, 1_000, true);

    assert_eq!(
        f.contract.score_of(&worker),
        0,
        "two payments is not a credit history"
    );
    assert_eq!(f.contract.tier_of(&worker), Tier::Unrated);

    f.pay(&employer, &worker, 1_000, true);

    assert!(
        f.contract.score_of(&worker) > 0,
        "the third payment makes the account ratable"
    );
    assert_ne!(f.contract.tier_of(&worker), Tier::Unrated);
}

#[test]
fn payments_are_recorded_against_both_parties() {
    let f = setup();
    let employer = Address::generate(&f.env);
    let worker = Address::generate(&f.env);

    f.pay(&employer, &worker, 2_500, true);

    let employer_record = f.contract.record_of(&employer);
    let worker_record = f.contract.record_of(&worker);

    assert_eq!(employer_record.payments_on_time, 1);
    assert_eq!(employer_record.total_volume, 2_500);
    assert_eq!(worker_record.payments_on_time, 1);
    assert_eq!(worker_record.total_volume, 2_500);
}

#[test]
fn late_payments_hurt_the_employer_that_caused_them() {
    let f = setup();

    let punctual = Address::generate(&f.env);
    let tardy = Address::generate(&f.env);
    for _ in 0..4 {
        f.pay(&punctual, &Address::generate(&f.env), 1_000, true);
        f.pay(&tardy, &Address::generate(&f.env), 1_000, false);
        f.advance(MONTH);
    }

    let good = f.contract.breakdown_of(&punctual);
    let bad = f.contract.breakdown_of(&tardy);

    assert_eq!(good.payment_history, 1_000);
    assert_eq!(bad.payment_history, 0);
    assert!(
        good.total > bad.total,
        "paying on time must outrank paying late: {} vs {}",
        good.total,
        bad.total
    );
}

#[test]
fn tenure_grows_with_time_and_saturates_at_three_years() {
    let f = setup();
    let worker = f.seasoned_worker(4);

    let early = f.contract.breakdown_of(&worker).tenure;

    f.advance(YEAR);
    let after_a_year = f.contract.breakdown_of(&worker).tenure;
    assert!(
        after_a_year > early,
        "tenure should grow: {early} -> {after_a_year}"
    );

    f.advance(10 * YEAR);
    assert_eq!(
        f.contract.breakdown_of(&worker).tenure,
        MAX_SCORE,
        "tenure saturates rather than growing forever"
    );
}

#[test]
fn consistency_rewards_recurring_income_over_a_single_burst() {
    let f = setup();

    // Steady: one payment every month for six months.
    let steady = Address::generate(&f.env);
    for _ in 0..6 {
        f.pay(&Address::generate(&f.env), &steady, 1_000, true);
        f.advance(MONTH);
    }

    // Bursty: six payments in one month, then silence for the same span.
    let bursty = Address::generate(&f.env);
    for _ in 0..6 {
        f.pay(&Address::generate(&f.env), &bursty, 1_000, true);
    }
    f.advance(6 * MONTH);

    let steady_score = f.contract.breakdown_of(&steady).consistency;
    let bursty_score = f.contract.breakdown_of(&bursty).consistency;

    assert!(
        steady_score > bursty_score,
        "recurring income should beat a single burst: {steady_score} vs {bursty_score}"
    );
}

#[test]
fn counterparty_diversity_is_counted_once_per_partner() {
    let f = setup();
    let worker = Address::generate(&f.env);
    let employer = Address::generate(&f.env);

    f.pay(&employer, &worker, 1_000, true);
    f.pay(&employer, &worker, 1_000, true);
    f.pay(&employer, &worker, 1_000, true);

    assert_eq!(
        f.contract.record_of(&worker).counterparties,
        1,
        "repeat business with one partner is still one partner"
    );

    f.pay(&Address::generate(&f.env), &worker, 1_000, true);
    assert_eq!(f.contract.record_of(&worker).counterparties, 2);
}

#[test]
fn a_closed_payment_loop_scores_worse_than_diversified_history() {
    let f = setup();

    // A sybil ring: two accounts paying each other in a tight loop, at twenty
    // times the honest worker's volume.
    let sybil_a = Address::generate(&f.env);
    let sybil_b = Address::generate(&f.env);
    for _ in 0..12 {
        f.pay(&sybil_a, &sybil_b, 100_000, true);
        f.pay(&sybil_b, &sybil_a, 100_000, true);
        f.advance(MONTH);
    }

    // An honest worker: the same cadence, spread across distinct clients.
    let honest = f.seasoned_worker(12);

    let sybil = f.contract.breakdown_of(&sybil_a);
    let real = f.contract.breakdown_of(&honest);

    assert!(
        sybil.counterparty_quality < real.counterparty_quality,
        "a two-account loop must score worse on counterparty quality: {} vs {}",
        sybil.counterparty_quality,
        real.counterparty_quality
    );
    assert!(
        sybil.total < real.total,
        "volume alone must not buy a score: sybil {} vs honest {}",
        sybil.total,
        real.total
    );
}

#[test]
fn volume_is_tracked_but_never_scored() {
    let f = setup();

    let whale = Address::generate(&f.env);
    let minnow = Address::generate(&f.env);
    for _ in 0..4 {
        f.pay(&Address::generate(&f.env), &whale, 10_000_000, true);
        f.pay(&Address::generate(&f.env), &minnow, 1, true);
        f.advance(MONTH);
    }

    assert_eq!(f.contract.record_of(&whale).total_volume, 40_000_000);
    assert_eq!(f.contract.record_of(&minnow).total_volume, 4);
    assert_eq!(
        f.contract.score_of(&whale),
        f.contract.score_of(&minnow),
        "moving more money is not the same as being creditworthy"
    );
}

#[test]
fn an_unrated_counterparty_counts_as_neutral_not_delinquent() {
    let f = setup();

    // Every client here is brand new, so none of them can be rated yet.
    let worker = f.seasoned_worker(5);

    let standing = f.contract.breakdown_of(&worker).counterparty_quality;
    assert!(
        standing >= 500,
        "dealing with new clients must not be penalised like dealing with bad \
         ones, got {standing}"
    );
}

#[test]
fn losing_disputes_degrades_the_record() {
    let f = setup();
    let account = f.seasoned_worker(6);
    let clean = f.contract.breakdown_of(&account).dispute_record;
    assert_eq!(clean, MAX_SCORE, "no disputes means a clean record");

    f.contract.record_dispute(&f.reporter, &account, &true);
    let after_one = f.contract.breakdown_of(&account).dispute_record;
    assert!(after_one < clean);

    f.contract.record_dispute(&f.reporter, &account, &true);
    assert!(f.contract.breakdown_of(&account).dispute_record < after_one);
}

#[test]
fn winning_a_dispute_costs_nothing() {
    let f = setup();
    let account = f.seasoned_worker(6);

    f.contract.record_dispute(&f.reporter, &account, &false);

    let record = f.contract.record_of(&account);
    assert_eq!(record.disputes_total, 1);
    assert_eq!(record.disputes_lost, 0);
    assert_eq!(
        f.contract.breakdown_of(&account).dispute_record,
        MAX_SCORE,
        "being disputed and vindicated is not a penalty"
    );
}

#[test]
fn never_having_borrowed_is_neutral_not_a_default() {
    let f = setup();
    let account = f.seasoned_worker(6);

    let no_history = f.contract.breakdown_of(&account).loan_repayment;
    assert_eq!(no_history, 500, "no loan history scores neutral");

    f.contract.record_loan(&f.reporter, &account, &true);
    assert!(
        f.contract.breakdown_of(&account).loan_repayment > no_history,
        "repaying a loan should beat never having borrowed"
    );
}

#[test]
fn defaulting_wrecks_the_loan_component() {
    let f = setup();
    let account = f.seasoned_worker(6);

    f.contract.record_loan(&f.reporter, &account, &false);

    assert_eq!(f.contract.breakdown_of(&account).loan_repayment, 0);
}

#[test]
fn tiers_track_the_score() {
    let f = setup();
    let worker = f.seasoned_worker(12);
    f.advance(3 * YEAR);

    let score = f.contract.score_of(&worker);
    let tier = f.contract.tier_of(&worker);

    let expected = match score {
        0..=399 => Tier::Bronze,
        400..=599 => Tier::Silver,
        600..=799 => Tier::Gold,
        _ => Tier::Platinum,
    };
    assert_eq!(tier, expected, "tier disagreed with score {score}");
}

#[test]
fn a_long_clean_history_reaches_the_top_tiers() {
    let f = setup();
    let worker = f.seasoned_worker(36);

    let score = f.contract.score_of(&worker);
    assert!(
        score >= 600,
        "three years of punctual, diversified income should rate well, got {score}"
    );
}

#[test]
fn the_score_is_recomputable_from_the_public_record() {
    let f = setup();
    let worker = f.seasoned_worker(8);

    let breakdown = f.contract.breakdown_of(&worker);
    let recomputed = (breakdown.payment_history * 35
        + breakdown.tenure * 20
        + breakdown.consistency * 15
        + breakdown.counterparty_quality * 10
        + breakdown.dispute_record * 10
        + breakdown.loan_repayment * 10)
        / 100;

    assert_eq!(
        breakdown.total, recomputed,
        "the published weights must reproduce the published total"
    );
    assert_eq!(f.contract.score_of(&worker), breakdown.total);
}

// ------------------------------------------------------------ authorization

#[test]
fn rejects_reports_from_unauthorized_addresses() {
    let f = setup();
    let stranger = Address::generate(&f.env);
    let a = Address::generate(&f.env);
    let b = Address::generate(&f.env);

    let err = f
        .contract
        .try_record_payment(&stranger, &a, &b, &1_000, &true)
        .unwrap_err()
        .unwrap();

    assert_eq!(err, Error::NotAuthorized);
}

#[test]
fn a_revoked_reporter_can_no_longer_write() {
    let f = setup();
    let a = Address::generate(&f.env);
    let b = Address::generate(&f.env);

    f.contract.set_reporter(&f.reporter, &false);

    let err = f
        .contract
        .try_record_payment(&f.reporter, &a, &b, &1_000, &true)
        .unwrap_err()
        .unwrap();
    assert_eq!(err, Error::NotAuthorized);
    assert!(!f.contract.is_reporter(&f.reporter));
}

#[test]
fn rejects_double_initialization() {
    let f = setup();
    let err = f
        .contract
        .try_initialize(&Address::generate(&f.env))
        .unwrap_err()
        .unwrap();
    assert_eq!(err, Error::AlreadyInitialized);
}

#[test]
fn rejects_self_payment() {
    let f = setup();
    let account = Address::generate(&f.env);

    let err = f
        .contract
        .try_record_payment(&f.reporter, &account, &account, &1_000, &true)
        .unwrap_err()
        .unwrap();
    assert_eq!(err, Error::SelfPayment);
}

#[test]
fn rejects_non_positive_amounts() {
    let f = setup();
    let a = Address::generate(&f.env);
    let b = Address::generate(&f.env);

    let err = f
        .contract
        .try_record_payment(&f.reporter, &a, &b, &0, &true)
        .unwrap_err()
        .unwrap();
    assert_eq!(err, Error::InvalidAmount);
}
