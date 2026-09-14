#![no_std]
//! # KamPay Escrow
//!
//! Milestone-scoped escrow for freelance and contract work.
//!
//! The client locks the full contract value before work begins. Each milestone
//! releases independently on client approval, or unilaterally to the provider
//! once its deadline plus grace period has passed without objection — so an
//! unresponsive client cannot strand a provider's payment indefinitely.
//!
//! Either party may freeze a milestone by raising a dispute, which routes the
//! split decision to the arbiter named at creation.

use soroban_sdk::{contract, contracterror, contractimpl, token, Address, Env, Vec};

mod events;
pub use events::*;

mod types;
pub use types::*;

mod test;

/// ~5 seconds per ledger.
const DAY_LEDGERS: u32 = 17_280;
const ESCROW_TTL: u32 = DAY_LEDGERS * 120;
const ESCROW_TTL_THRESHOLD: u32 = DAY_LEDGERS * 90;

#[contract]
pub struct EscrowContract;

#[contractimpl]
impl EscrowContract {
    /// Draft an escrow agreement. No funds move until `fund` is called, and the
    /// provider should not start work until the status reads `Active`.
    pub fn create_escrow(
        env: Env,
        client: Address,
        provider: Address,
        token: Address,
        milestones: Vec<Milestone>,
        arbiter: Option<Address>,
    ) -> Result<u64, Error> {
        client.require_auth();

        if client == provider {
            return Err(Error::SelfContract);
        }
        if milestones.is_empty() {
            return Err(Error::NoMilestones);
        }

        let mut total: i128 = 0;
        for milestone in milestones.iter() {
            if milestone.amount <= 0 {
                return Err(Error::InvalidAmount);
            }
            if milestone.status != MilestoneStatus::Pending {
                return Err(Error::InvalidMilestoneStatus);
            }
            total += milestone.amount;
        }

        let milestone_count = milestones.len();
        let escrow_id = next_escrow_id(&env);
        let escrow = Escrow {
            client: client.clone(),
            provider: provider.clone(),
            token: token.clone(),
            arbiter,
            milestones,
            status: EscrowStatus::Draft,
            funded: 0,
            released: 0,
        };

        save_escrow(&env, escrow_id, &escrow);
        index_push(&env, &DataKey::ClientEscrows(client.clone()), escrow_id);
        index_push(&env, &DataKey::ProviderEscrows(provider.clone()), escrow_id);

        EscrowCreated {
            escrow_id,
            client,
            provider,
            token,
            milestone_count,
            total_value: total,
        }
        .publish(&env);

        Ok(escrow_id)
    }

    /// Lock the full contract value. This is the signal that work can start.
    pub fn fund(env: Env, escrow_id: u64) -> Result<i128, Error> {
        let mut escrow = load_escrow(&env, escrow_id)?;
        escrow.client.require_auth();

        if escrow.status != EscrowStatus::Draft {
            return Err(Error::AlreadyFunded);
        }

        let total = total_value(&escrow);
        let contract = env.current_contract_address();
        token::Client::new(&env, &escrow.token).transfer(&escrow.client, &contract, &total);

        escrow.funded = total;
        escrow.status = EscrowStatus::Active;
        save_escrow(&env, escrow_id, &escrow);

        EscrowFunded {
            escrow_id,
            amount: total,
        }
        .publish(&env);

        Ok(total)
    }

    /// Client signs off on a deliverable; the milestone pays out immediately.
    pub fn approve_milestone(env: Env, escrow_id: u64, index: u32) -> Result<i128, Error> {
        let mut escrow = load_escrow(&env, escrow_id)?;
        escrow.client.require_auth();

        let amount = release(&env, escrow_id, &mut escrow, index, false)?;
        Ok(amount)
    }

    /// Provider claims a milestone the client neither approved nor disputed
    /// before the deadline and its grace period elapsed.
    pub fn claim_expired(env: Env, escrow_id: u64, index: u32) -> Result<i128, Error> {
        let mut escrow = load_escrow(&env, escrow_id)?;
        escrow.provider.require_auth();

        let milestone = milestone_at(&escrow, index)?;
        let claimable_at = milestone.deadline.saturating_add(milestone.grace_period);
        if env.ledger().timestamp() < claimable_at {
            return Err(Error::GracePeriodActive);
        }

        let amount = release(&env, escrow_id, &mut escrow, index, true)?;
        Ok(amount)
    }

    /// Freeze a milestone pending arbitration. Either party may call this while
    /// the milestone is still pending.
    pub fn raise_dispute(env: Env, escrow_id: u64, index: u32, by: Address) -> Result<(), Error> {
        by.require_auth();

        let mut escrow = load_escrow(&env, escrow_id)?;
        if by != escrow.client && by != escrow.provider {
            return Err(Error::NotAParty);
        }
        if escrow.status != EscrowStatus::Active {
            return Err(Error::EscrowNotActive);
        }
        if escrow.arbiter.is_none() {
            return Err(Error::NoArbiter);
        }

        let mut milestone = milestone_at(&escrow, index)?;
        if milestone.status != MilestoneStatus::Pending {
            return Err(Error::InvalidMilestoneStatus);
        }

        milestone.status = MilestoneStatus::Disputed;
        escrow.milestones.set(index, milestone);
        save_escrow(&env, escrow_id, &escrow);

        DisputeRaised {
            escrow_id,
            by,
            index,
        }
        .publish(&env);

        Ok(())
    }

    /// Arbiter rules on a frozen milestone, splitting it between the parties.
    ///
    /// `to_provider` is the provider's share; the remainder returns to the client.
    pub fn resolve_dispute(
        env: Env,
        escrow_id: u64,
        index: u32,
        to_provider: i128,
    ) -> Result<(), Error> {
        let mut escrow = load_escrow(&env, escrow_id)?;
        let arbiter = escrow.arbiter.clone().ok_or(Error::NoArbiter)?;
        arbiter.require_auth();

        let mut milestone = milestone_at(&escrow, index)?;
        if milestone.status != MilestoneStatus::Disputed {
            return Err(Error::NotDisputed);
        }
        if to_provider < 0 || to_provider > milestone.amount {
            return Err(Error::InvalidSplit);
        }

        let to_client = milestone.amount - to_provider;
        let token_client = token::Client::new(&env, &escrow.token);
        let contract = env.current_contract_address();

        if to_provider > 0 {
            token_client.transfer(&contract, &escrow.provider, &to_provider);
            escrow.released += to_provider;
        }
        if to_client > 0 {
            token_client.transfer(&contract, &escrow.client, &to_client);
        }

        milestone.status = if to_provider > 0 {
            MilestoneStatus::Released
        } else {
            MilestoneStatus::Refunded
        };
        escrow.milestones.set(index, milestone);
        settle_if_complete(&mut escrow);
        save_escrow(&env, escrow_id, &escrow);

        DisputeResolved {
            escrow_id,
            index,
            to_provider,
            to_client,
        }
        .publish(&env);

        Ok(())
    }

    /// Mutually cancel the agreement. Every pending milestone is refunded to the
    /// client. Requires both signatures, so neither side can walk away alone.
    pub fn cancel(env: Env, escrow_id: u64) -> Result<i128, Error> {
        let mut escrow = load_escrow(&env, escrow_id)?;
        escrow.client.require_auth();
        escrow.provider.require_auth();

        if escrow.status == EscrowStatus::Cancelled || escrow.status == EscrowStatus::Completed {
            return Err(Error::EscrowNotActive);
        }

        let mut refund: i128 = 0;
        for index in 0..escrow.milestones.len() {
            let mut milestone = escrow.milestones.get(index).unwrap();
            if milestone.status == MilestoneStatus::Pending {
                refund += milestone.amount;
                milestone.status = MilestoneStatus::Refunded;
                escrow.milestones.set(index, milestone);
            }
        }

        if refund > 0 && escrow.funded > 0 {
            let contract = env.current_contract_address();
            token::Client::new(&env, &escrow.token).transfer(&contract, &escrow.client, &refund);
        }

        escrow.status = EscrowStatus::Cancelled;
        save_escrow(&env, escrow_id, &escrow);

        EscrowCancelled {
            escrow_id,
            refunded: refund,
        }
        .publish(&env);

        Ok(refund)
    }

    // ----------------------------------------------------------------- views

    pub fn get_escrow(env: Env, escrow_id: u64) -> Result<Escrow, Error> {
        load_escrow(&env, escrow_id)
    }

    pub fn get_milestone(env: Env, escrow_id: u64, index: u32) -> Result<Milestone, Error> {
        milestone_at(&load_escrow(&env, escrow_id)?, index)
    }

    /// Whether the provider could claim this milestone right now without the
    /// client's approval.
    pub fn is_claimable(env: Env, escrow_id: u64, index: u32) -> Result<bool, Error> {
        let escrow = load_escrow(&env, escrow_id)?;
        let milestone = milestone_at(&escrow, index)?;
        let claimable_at = milestone.deadline.saturating_add(milestone.grace_period);
        Ok(escrow.status == EscrowStatus::Active
            && milestone.status == MilestoneStatus::Pending
            && env.ledger().timestamp() >= claimable_at)
    }

    /// Total value still locked in the contract for this agreement.
    pub fn locked_value(env: Env, escrow_id: u64) -> Result<i128, Error> {
        let escrow = load_escrow(&env, escrow_id)?;
        let mut locked: i128 = 0;
        for milestone in escrow.milestones.iter() {
            if milestone.status == MilestoneStatus::Pending
                || milestone.status == MilestoneStatus::Disputed
            {
                locked += milestone.amount;
            }
        }
        Ok(locked)
    }

    pub fn escrows_of_client(env: Env, client: Address) -> Vec<u64> {
        index_get(&env, &DataKey::ClientEscrows(client))
    }

    pub fn escrows_of_provider(env: Env, provider: Address) -> Vec<u64> {
        index_get(&env, &DataKey::ProviderEscrows(provider))
    }
}

// ------------------------------------------------------------------ internals

/// Pay a pending milestone out to the provider.
fn release(
    env: &Env,
    escrow_id: u64,
    escrow: &mut Escrow,
    index: u32,
    auto: bool,
) -> Result<i128, Error> {
    if escrow.status != EscrowStatus::Active {
        return Err(Error::EscrowNotActive);
    }

    let mut milestone = milestone_at(escrow, index)?;
    match milestone.status {
        MilestoneStatus::Released | MilestoneStatus::Refunded => {
            return Err(Error::MilestoneSettled)
        }
        MilestoneStatus::Disputed => return Err(Error::MilestoneDisputed),
        MilestoneStatus::Pending => {}
    }

    let amount = milestone.amount;
    let contract = env.current_contract_address();
    token::Client::new(env, &escrow.token).transfer(&contract, &escrow.provider, &amount);

    milestone.status = MilestoneStatus::Released;
    escrow.milestones.set(index, milestone);
    escrow.released += amount;
    settle_if_complete(escrow);
    save_escrow(env, escrow_id, escrow);

    MilestoneReleased {
        escrow_id,
        provider: escrow.provider.clone(),
        index,
        amount,
        auto_released: auto,
    }
    .publish(env);

    Ok(amount)
}

/// Mark the agreement complete once no milestone is still open.
fn settle_if_complete(escrow: &mut Escrow) {
    let open = escrow
        .milestones
        .iter()
        .any(|m| m.status == MilestoneStatus::Pending || m.status == MilestoneStatus::Disputed);
    if !open {
        escrow.status = EscrowStatus::Completed;
    }
}

fn total_value(escrow: &Escrow) -> i128 {
    escrow.milestones.iter().map(|m| m.amount).sum()
}

fn milestone_at(escrow: &Escrow, index: u32) -> Result<Milestone, Error> {
    escrow.milestones.get(index).ok_or(Error::MilestoneNotFound)
}

fn next_escrow_id(env: &Env) -> u64 {
    let key = DataKey::NextEscrowId;
    let id: u64 = env.storage().instance().get(&key).unwrap_or(0);
    env.storage().instance().set(&key, &(id + 1));
    env.storage()
        .instance()
        .extend_ttl(ESCROW_TTL_THRESHOLD, ESCROW_TTL);
    id
}

fn save_escrow(env: &Env, escrow_id: u64, escrow: &Escrow) {
    let key = DataKey::Escrow(escrow_id);
    env.storage().persistent().set(&key, escrow);
    env.storage()
        .persistent()
        .extend_ttl(&key, ESCROW_TTL_THRESHOLD, ESCROW_TTL);
}

fn load_escrow(env: &Env, escrow_id: u64) -> Result<Escrow, Error> {
    let key = DataKey::Escrow(escrow_id);
    let escrow: Escrow = env
        .storage()
        .persistent()
        .get(&key)
        .ok_or(Error::EscrowNotFound)?;
    env.storage()
        .persistent()
        .extend_ttl(&key, ESCROW_TTL_THRESHOLD, ESCROW_TTL);
    Ok(escrow)
}

fn index_push(env: &Env, key: &DataKey, escrow_id: u64) {
    let mut ids: Vec<u64> = env
        .storage()
        .persistent()
        .get(key)
        .unwrap_or_else(|| Vec::new(env));
    ids.push_back(escrow_id);
    env.storage().persistent().set(key, &ids);
    env.storage()
        .persistent()
        .extend_ttl(key, ESCROW_TTL_THRESHOLD, ESCROW_TTL);
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
    EscrowNotFound = 1,
    MilestoneNotFound = 2,
    NoMilestones = 3,
    InvalidAmount = 4,
    AlreadyFunded = 5,
    EscrowNotActive = 6,
    MilestoneSettled = 7,
    MilestoneDisputed = 8,
    GracePeriodActive = 9,
    NotAParty = 10,
    NoArbiter = 11,
    NotDisputed = 12,
    InvalidSplit = 13,
    SelfContract = 14,
    InvalidMilestoneStatus = 15,
}
