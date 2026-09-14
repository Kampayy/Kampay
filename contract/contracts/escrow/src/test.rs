#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    token, vec, Address, Env,
};

const DAY: u64 = 86_400;
const GRACE: u64 = 7 * DAY;

struct Fixture {
    env: Env,
    contract: EscrowContractClient<'static>,
    token: token::Client<'static>,
    client: Address,
    provider: Address,
    arbiter: Address,
}

fn setup() -> Fixture {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(1_000_000);

    let client = Address::generate(&env);
    let provider = Address::generate(&env);
    let arbiter = Address::generate(&env);

    let issuer = Address::generate(&env);
    let asset = env.register_stellar_asset_contract_v2(issuer);
    let token_address = asset.address();
    token::StellarAssetClient::new(&env, &token_address).mint(&client, &1_000_000);

    let contract_id = env.register(EscrowContract, ());

    Fixture {
        contract: EscrowContractClient::new(&env, &contract_id),
        token: token::Client::new(&env, &token_address),
        env,
        client,
        provider,
        arbiter,
    }
}

impl Fixture {
    /// Two milestones worth 6_000 and 4_000, both due in 30 days.
    fn milestones(&self) -> Vec<Milestone> {
        let due = self.env.ledger().timestamp() + 30 * DAY;
        vec![
            &self.env,
            Milestone {
                amount: 6_000,
                deadline: due,
                grace_period: GRACE,
                status: MilestoneStatus::Pending,
            },
            Milestone {
                amount: 4_000,
                deadline: due,
                grace_period: GRACE,
                status: MilestoneStatus::Pending,
            },
        ]
    }

    fn open_funded(&self, arbiter: Option<Address>) -> u64 {
        let id = self.contract.create_escrow(
            &self.client,
            &self.provider,
            &self.token.address,
            &self.milestones(),
            &arbiter,
        );
        self.contract.fund(&id);
        id
    }

    fn advance(&self, seconds: u64) {
        let now = self.env.ledger().timestamp();
        self.env.ledger().set_timestamp(now + seconds);
    }
}

#[test]
fn funding_locks_the_full_contract_value() {
    let f = setup();
    let before = f.token.balance(&f.client);

    let id = f.contract.create_escrow(
        &f.client,
        &f.provider,
        &f.token.address,
        &f.milestones(),
        &None,
    );
    assert_eq!(f.contract.get_escrow(&id).status, EscrowStatus::Draft);
    assert_eq!(f.token.balance(&f.client), before, "draft moves no funds");

    let locked = f.contract.fund(&id);

    assert_eq!(locked, 10_000);
    assert_eq!(f.token.balance(&f.client), before - 10_000);
    assert_eq!(f.contract.get_escrow(&id).status, EscrowStatus::Active);
    assert_eq!(f.contract.locked_value(&id), 10_000);
}

#[test]
fn approving_a_milestone_pays_only_that_milestone() {
    let f = setup();
    let id = f.open_funded(None);

    let paid = f.contract.approve_milestone(&id, &0);

    assert_eq!(paid, 6_000);
    assert_eq!(f.token.balance(&f.provider), 6_000);
    assert_eq!(
        f.contract.locked_value(&id),
        4_000,
        "the second milestone stays locked"
    );
    assert_eq!(
        f.contract.get_escrow(&id).status,
        EscrowStatus::Active,
        "escrow stays open while work remains"
    );
}

#[test]
fn escrow_completes_once_every_milestone_settles() {
    let f = setup();
    let id = f.open_funded(None);

    f.contract.approve_milestone(&id, &0);
    f.contract.approve_milestone(&id, &1);

    assert_eq!(f.token.balance(&f.provider), 10_000);
    assert_eq!(f.contract.get_escrow(&id).status, EscrowStatus::Completed);
    assert_eq!(f.contract.locked_value(&id), 0);
}

#[test]
fn provider_can_claim_after_the_grace_period_expires() {
    let f = setup();
    let id = f.open_funded(None);

    assert!(!f.contract.is_claimable(&id, &0), "not yet due");

    f.advance(30 * DAY + 1);
    assert!(
        !f.contract.is_claimable(&id, &0),
        "deadline passed but grace period still running"
    );

    f.advance(GRACE);
    assert!(f.contract.is_claimable(&id, &0));

    let paid = f.contract.claim_expired(&id, &0);
    assert_eq!(paid, 6_000);
    assert_eq!(f.token.balance(&f.provider), 6_000);
}

#[test]
fn claiming_during_the_grace_period_is_rejected() {
    let f = setup();
    let id = f.open_funded(None);
    f.advance(30 * DAY + 1);

    let err = f.contract.try_claim_expired(&id, &0).unwrap_err().unwrap();

    assert_eq!(err, Error::GracePeriodActive);
    assert_eq!(f.token.balance(&f.provider), 0);
}

#[test]
fn a_dispute_freezes_the_milestone() {
    let f = setup();
    let id = f.open_funded(Some(f.arbiter.clone()));

    f.contract.raise_dispute(&id, &0, &f.client);

    assert_eq!(
        f.contract.get_milestone(&id, &0).status,
        MilestoneStatus::Disputed
    );

    // Even a fully expired grace period cannot release a frozen milestone.
    f.advance(30 * DAY + GRACE + 1);
    let err = f.contract.try_claim_expired(&id, &0).unwrap_err().unwrap();
    assert_eq!(err, Error::MilestoneDisputed);
}

#[test]
fn arbiter_splits_a_disputed_milestone() {
    let f = setup();
    let id = f.open_funded(Some(f.arbiter.clone()));
    let client_before = f.token.balance(&f.client);

    f.contract.raise_dispute(&id, &0, &f.provider);
    f.contract.resolve_dispute(&id, &0, &2_500);

    assert_eq!(f.token.balance(&f.provider), 2_500);
    assert_eq!(
        f.token.balance(&f.client),
        client_before + 3_500,
        "the remainder returns to the client"
    );
    assert_eq!(
        f.contract.get_milestone(&id, &0).status,
        MilestoneStatus::Released
    );
}

#[test]
fn arbiter_can_rule_entirely_for_the_client() {
    let f = setup();
    let id = f.open_funded(Some(f.arbiter.clone()));
    let client_before = f.token.balance(&f.client);

    f.contract.raise_dispute(&id, &0, &f.client);
    f.contract.resolve_dispute(&id, &0, &0);

    assert_eq!(f.token.balance(&f.provider), 0);
    assert_eq!(f.token.balance(&f.client), client_before + 6_000);
    assert_eq!(
        f.contract.get_milestone(&id, &0).status,
        MilestoneStatus::Refunded
    );
}

#[test]
fn mutual_cancellation_refunds_pending_milestones_only() {
    let f = setup();
    let id = f.open_funded(None);
    f.contract.approve_milestone(&id, &0);
    let client_before = f.token.balance(&f.client);

    let refunded = f.contract.cancel(&id);

    assert_eq!(refunded, 4_000, "only the untouched milestone comes back");
    assert_eq!(f.token.balance(&f.client), client_before + 4_000);
    assert_eq!(
        f.token.balance(&f.provider),
        6_000,
        "released work stays paid"
    );
    assert_eq!(f.contract.get_escrow(&id).status, EscrowStatus::Cancelled);
}

#[test]
fn escrows_are_indexed_for_both_parties() {
    let f = setup();
    let id = f.open_funded(None);

    assert_eq!(f.contract.escrows_of_client(&f.client), vec![&f.env, id]);
    assert_eq!(
        f.contract.escrows_of_provider(&f.provider),
        vec![&f.env, id]
    );
}

// ------------------------------------------------------------ failure cases

#[test]
fn rejects_an_escrow_with_no_milestones() {
    let f = setup();
    let err = f
        .contract
        .try_create_escrow(
            &f.client,
            &f.provider,
            &f.token.address,
            &Vec::new(&f.env),
            &None,
        )
        .unwrap_err()
        .unwrap();
    assert_eq!(err, Error::NoMilestones);
}

#[test]
fn rejects_releasing_from_an_unfunded_escrow() {
    let f = setup();
    let id = f.contract.create_escrow(
        &f.client,
        &f.provider,
        &f.token.address,
        &f.milestones(),
        &None,
    );
    let err = f
        .contract
        .try_approve_milestone(&id, &0)
        .unwrap_err()
        .unwrap();
    assert_eq!(err, Error::EscrowNotActive);
}

#[test]
fn rejects_double_funding() {
    let f = setup();
    let id = f.open_funded(None);
    let err = f.contract.try_fund(&id).unwrap_err().unwrap();
    assert_eq!(err, Error::AlreadyFunded);
}

#[test]
fn rejects_approving_a_settled_milestone() {
    let f = setup();
    let id = f.open_funded(None);
    f.contract.approve_milestone(&id, &0);
    let err = f
        .contract
        .try_approve_milestone(&id, &0)
        .unwrap_err()
        .unwrap();
    assert_eq!(err, Error::MilestoneSettled);
}

#[test]
fn rejects_a_dispute_from_an_outsider() {
    let f = setup();
    let id = f.open_funded(Some(f.arbiter.clone()));
    let stranger = Address::generate(&f.env);

    let err = f
        .contract
        .try_raise_dispute(&id, &0, &stranger)
        .unwrap_err()
        .unwrap();
    assert_eq!(err, Error::NotAParty);
}

#[test]
fn rejects_a_dispute_when_no_arbiter_was_named() {
    let f = setup();
    let id = f.open_funded(None);
    let err = f
        .contract
        .try_raise_dispute(&id, &0, &f.client)
        .unwrap_err()
        .unwrap();
    assert_eq!(err, Error::NoArbiter);
}

#[test]
fn rejects_a_split_larger_than_the_milestone() {
    let f = setup();
    let id = f.open_funded(Some(f.arbiter.clone()));
    f.contract.raise_dispute(&id, &0, &f.client);

    let err = f
        .contract
        .try_resolve_dispute(&id, &0, &9_999)
        .unwrap_err()
        .unwrap();
    assert_eq!(err, Error::InvalidSplit);
}

#[test]
fn rejects_resolving_a_milestone_that_is_not_disputed() {
    let f = setup();
    let id = f.open_funded(Some(f.arbiter.clone()));
    let err = f
        .contract
        .try_resolve_dispute(&id, &0, &100)
        .unwrap_err()
        .unwrap();
    assert_eq!(err, Error::NotDisputed);
}

#[test]
fn rejects_an_unknown_milestone_index() {
    let f = setup();
    let id = f.open_funded(None);
    let err = f
        .contract
        .try_approve_milestone(&id, &99)
        .unwrap_err()
        .unwrap();
    assert_eq!(err, Error::MilestoneNotFound);
}
