#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    token, vec, Address, Env,
};

const DAY: u64 = 86_400;
/// 1 unit per second — a round number keeps the assertions readable.
const RATE: i128 = 1;

struct Fixture {
    env: Env,
    client: PayrollContractClient<'static>,
    token: token::Client<'static>,
    employer: Address,
    worker: Address,
}

fn setup() -> Fixture {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(1_000);

    let employer = Address::generate(&env);
    let worker = Address::generate(&env);

    let issuer = Address::generate(&env);
    let asset = env.register_stellar_asset_contract_v2(issuer);
    let token_address = asset.address();
    token::StellarAssetClient::new(&env, &token_address).mint(&employer, &1_000_000);

    let contract_id = env.register(PayrollContract, ());
    let client = PayrollContractClient::new(&env, &contract_id);

    Fixture {
        token: token::Client::new(&env, &token_address),
        env,
        client,
        employer,
        worker,
    }
}

impl Fixture {
    fn open_funded_stream(&self, deposit: i128) -> u64 {
        let start = self.env.ledger().timestamp();
        let id = self.client.create_stream(
            &self.employer,
            &self.worker,
            &self.token.address,
            &RATE,
            &start,
            &(start + 30 * DAY),
        );
        self.client.fund(&id, &self.employer, &deposit);
        id
    }

    fn advance(&self, seconds: u64) {
        let now = self.env.ledger().timestamp();
        self.env.ledger().set_timestamp(now + seconds);
    }
}

#[test]
fn stream_accrues_linearly() {
    let f = setup();
    let id = f.open_funded_stream(100_000);

    assert_eq!(f.client.withdrawable(&id), 0, "nothing accrues at t=0");

    f.advance(1_000);
    assert_eq!(f.client.withdrawable(&id), 1_000);

    f.advance(500);
    assert_eq!(f.client.withdrawable(&id), 1_500);
}

#[test]
fn withdraw_pays_the_worker_and_advances_the_watermark() {
    let f = setup();
    let id = f.open_funded_stream(100_000);
    f.advance(2_000);

    let paid = f.client.withdraw(&id);

    assert_eq!(paid, 2_000);
    assert_eq!(f.token.balance(&f.worker), 2_000);
    assert_eq!(f.client.withdrawable(&id), 0, "watermark advanced");

    f.advance(1_000);
    assert_eq!(f.client.withdrawable(&id), 1_000, "accrual continues");
}

#[test]
fn accrual_is_capped_by_what_was_actually_deposited() {
    let f = setup();
    // Funded for 500 seconds of a 30-day stream.
    let id = f.open_funded_stream(500);

    f.advance(5_000);

    assert_eq!(
        f.client.withdrawable(&id),
        500,
        "an underfunded stream cannot pay out more than it holds"
    );
    assert_eq!(f.client.withdraw(&id), 500);
}

#[test]
fn paused_time_never_accrues() {
    let f = setup();
    let id = f.open_funded_stream(100_000);

    f.advance(1_000);
    f.client.pause(&id);

    f.advance(5_000);
    assert_eq!(
        f.client.withdrawable(&id),
        1_000,
        "nothing accrues while paused"
    );

    f.client.resume(&id);
    f.advance(1_000);
    assert_eq!(
        f.client.withdrawable(&id),
        2_000,
        "accrual resumes without back-paying the pause"
    );
}

#[test]
fn cancel_splits_funds_between_worker_and_employer() {
    let f = setup();
    let deposit = 100_000;
    let id = f.open_funded_stream(deposit);
    let employer_after_funding = f.token.balance(&f.employer);

    f.advance(3_000);
    f.client.cancel(&id);

    assert_eq!(f.token.balance(&f.worker), 3_000, "worker paid what accrued");
    assert_eq!(
        f.token.balance(&f.employer),
        employer_after_funding + (deposit - 3_000),
        "employer refunded the remainder"
    );
    assert_eq!(f.client.get_stream(&id).status, StreamStatus::Cancelled);
}

#[test]
fn stream_stops_accruing_after_its_end_date() {
    let f = setup();
    let start = f.env.ledger().timestamp();
    let id = f.client.create_stream(
        &f.employer,
        &f.worker,
        &f.token.address,
        &RATE,
        &start,
        &(start + 1_000),
    );
    f.client.fund(&id, &f.employer, &100_000);

    f.advance(10_000);

    assert_eq!(
        f.client.withdrawable(&id),
        1_000,
        "accrual is bounded by the end timestamp, not the balance"
    );
}

#[test]
fn runway_reports_remaining_funded_seconds() {
    let f = setup();
    let id = f.open_funded_stream(5_000);

    assert_eq!(f.client.runway(&id), 5_000);

    f.advance(2_000);
    assert_eq!(f.client.runway(&id), 3_000);
}

#[test]
fn batch_pay_disburses_to_every_payee() {
    let f = setup();
    let a = Address::generate(&f.env);
    let b = Address::generate(&f.env);

    let total = f.client.batch_pay(
        &f.employer,
        &f.token.address,
        &vec![
            &f.env,
            Payment { to: a.clone(), amount: 700 },
            Payment { to: b.clone(), amount: 300 },
        ],
    );

    assert_eq!(total, 1_000);
    assert_eq!(f.token.balance(&a), 700);
    assert_eq!(f.token.balance(&b), 300);
}

#[test]
fn streams_are_indexed_for_both_parties() {
    let f = setup();
    let id = f.open_funded_stream(1_000);

    assert_eq!(f.client.streams_of_employer(&f.employer), vec![&f.env, id]);
    assert_eq!(f.client.streams_of_worker(&f.worker), vec![&f.env, id]);
}

// ------------------------------------------------------------ failure cases

#[test]
fn rejects_inverted_time_range() {
    let f = setup();
    let start = f.env.ledger().timestamp();
    let err = f
        .client
        .try_create_stream(
            &f.employer,
            &f.worker,
            &f.token.address,
            &RATE,
            &(start + 1_000),
            &start,
        )
        .unwrap_err()
        .unwrap();
    assert_eq!(err, Error::InvalidTimeRange);
}

#[test]
fn rejects_non_positive_rate() {
    let f = setup();
    let start = f.env.ledger().timestamp();
    let err = f
        .client
        .try_create_stream(
            &f.employer,
            &f.worker,
            &f.token.address,
            &0,
            &start,
            &(start + DAY),
        )
        .unwrap_err()
        .unwrap();
    assert_eq!(err, Error::InvalidRate);
}

#[test]
fn rejects_withdraw_with_nothing_accrued() {
    let f = setup();
    let id = f.open_funded_stream(1_000);
    let err = f.client.try_withdraw(&id).unwrap_err().unwrap();
    assert_eq!(err, Error::NothingToWithdraw);
}

#[test]
fn rejects_double_pause() {
    let f = setup();
    let id = f.open_funded_stream(1_000);
    f.client.pause(&id);
    let err = f.client.try_pause(&id).unwrap_err().unwrap();
    assert_eq!(err, Error::StreamPaused);
}

#[test]
fn rejects_funding_a_cancelled_stream() {
    let f = setup();
    let id = f.open_funded_stream(1_000);
    f.client.cancel(&id);
    let err = f
        .client
        .try_fund(&id, &f.employer, &500)
        .unwrap_err()
        .unwrap();
    assert_eq!(err, Error::StreamCancelled);
}

#[test]
fn rejects_unknown_stream() {
    let f = setup();
    let err = f.client.try_get_stream(&404).unwrap_err().unwrap();
    assert_eq!(err, Error::StreamNotFound);
}
