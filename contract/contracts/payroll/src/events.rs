//! Contract events emitted by the payroll contract.
//!
//! These are the settlement signals the `credit` contract consumes to build a
//! worker's payment history, so their shape is part of the protocol's public
//! surface — change them deliberately.

use soroban_sdk::{contractevent, Address};

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StreamCreated {
    #[topic]
    pub stream_id: u64,
    #[topic]
    pub employer: Address,
    #[topic]
    pub worker: Address,
    pub token: Address,
    pub rate_per_second: i128,
    pub start: u64,
    pub end: u64,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StreamFunded {
    #[topic]
    pub stream_id: u64,
    pub from: Address,
    pub amount: i128,
    pub total_deposited: i128,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Withdrawn {
    #[topic]
    pub stream_id: u64,
    #[topic]
    pub worker: Address,
    pub amount: i128,
    pub total_withdrawn: i128,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StreamPaused {
    #[topic]
    pub stream_id: u64,
    pub at: u64,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StreamResumed {
    #[topic]
    pub stream_id: u64,
    pub paused_seconds: u64,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StreamCancelled {
    #[topic]
    pub stream_id: u64,
    pub paid_to_worker: i128,
    pub refunded_to_employer: i128,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchPaid {
    #[topic]
    pub employer: Address,
    pub token: Address,
    pub payee_count: u32,
    pub total: i128,
}
