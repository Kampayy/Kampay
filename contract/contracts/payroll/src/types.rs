use soroban_sdk::{contracttype, Address};

/// Lifecycle of a salary stream.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StreamStatus {
    /// Accruing normally.
    Active,
    /// Accrual suspended; paused time is excluded from the worker's earnings.
    Paused,
    /// Settled early. Terminal state.
    Cancelled,
}

/// A linear salary stream between an employer and a worker.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Stream {
    pub employer: Address,
    pub worker: Address,
    /// Address of the payment token (typically USDC on Stellar).
    pub token: Address,
    /// Accrual rate in the token's smallest unit, per second.
    pub rate_per_second: i128,
    /// Ledger timestamp accrual begins.
    pub start: u64,
    /// Ledger timestamp accrual stops.
    pub end: u64,
    /// Total ever deposited, minus anything refunded on cancellation.
    pub deposited: i128,
    /// Total ever paid out to the worker.
    pub withdrawn: i128,
    pub status: StreamStatus,
    /// When the current pause began; 0 when not paused.
    pub paused_at: u64,
    /// Accumulated seconds excluded from accrual by completed pauses.
    pub paused_seconds: u64,
}

/// One line in a batch payroll run.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Payment {
    pub to: Address,
    pub amount: i128,
}

#[contracttype]
pub enum DataKey {
    /// Monotonic stream id counter.
    NextStreamId,
    /// Stream record by id.
    Stream(u64),
    /// Stream ids opened by an employer.
    EmployerStreams(Address),
    /// Stream ids paying a worker.
    WorkerStreams(Address),
}
