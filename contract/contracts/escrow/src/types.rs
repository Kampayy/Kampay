use soroban_sdk::{contracttype, Address, Vec};

/// Lifecycle of a whole escrow agreement.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EscrowStatus {
    /// Created but not yet funded. No work should start here.
    Draft,
    /// Fully funded; the provider can safely begin.
    Active,
    /// Every milestone reached a terminal state.
    Completed,
    /// Mutually cancelled; unreleased funds returned to the client.
    Cancelled,
}

/// Lifecycle of a single milestone.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MilestoneStatus {
    /// Awaiting client approval.
    Pending,
    /// Paid out to the provider.
    Released,
    /// Frozen pending arbitration.
    Disputed,
    /// Returned to the client.
    Refunded,
}

/// One deliverable and the money locked against it.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Milestone {
    /// Amount locked for this deliverable, in the token's smallest unit.
    pub amount: i128,
    /// Timestamp by which the client is expected to approve or dispute.
    pub deadline: u64,
    /// Seconds after `deadline` before the provider may claim unilaterally.
    /// This is the slack that keeps honest-but-late review from being punitive.
    pub grace_period: u64,
    pub status: MilestoneStatus,
}

/// An escrow agreement between a client and a service provider.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Escrow {
    pub client: Address,
    pub provider: Address,
    /// Address of the payment token (typically USDC on Stellar).
    pub token: Address,
    /// Neutral party who can rule on disputes. Fixed at creation and
    /// immutable, so neither side can swap in a friendly arbiter later.
    pub arbiter: Option<Address>,
    pub milestones: Vec<Milestone>,
    pub status: EscrowStatus,
    /// Total transferred in at funding time.
    pub funded: i128,
    /// Total paid out to the provider so far.
    pub released: i128,
}

#[contracttype]
pub enum DataKey {
    /// Monotonic escrow id counter.
    NextEscrowId,
    /// Escrow record by id.
    Escrow(u64),
    /// Escrow ids opened by a client.
    ClientEscrows(Address),
    /// Escrow ids where an address is the provider.
    ProviderEscrows(Address),
}
