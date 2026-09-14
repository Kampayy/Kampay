//! Contract events emitted by the escrow contract.
//!
//! Release and dispute outcomes are the settlement signals the `credit`
//! contract scores, so treat this shape as protocol surface.

use soroban_sdk::{contractevent, Address};

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EscrowCreated {
    #[topic]
    pub escrow_id: u64,
    #[topic]
    pub client: Address,
    #[topic]
    pub provider: Address,
    pub token: Address,
    pub milestone_count: u32,
    pub total_value: i128,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EscrowFunded {
    #[topic]
    pub escrow_id: u64,
    pub amount: i128,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MilestoneReleased {
    #[topic]
    pub escrow_id: u64,
    #[topic]
    pub provider: Address,
    pub index: u32,
    pub amount: i128,
    /// True when released by grace-period expiry rather than client approval.
    pub auto_released: bool,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DisputeRaised {
    #[topic]
    pub escrow_id: u64,
    #[topic]
    pub by: Address,
    pub index: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DisputeResolved {
    #[topic]
    pub escrow_id: u64,
    pub index: u32,
    pub to_provider: i128,
    pub to_client: i128,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EscrowCancelled {
    #[topic]
    pub escrow_id: u64,
    pub refunded: i128,
}
