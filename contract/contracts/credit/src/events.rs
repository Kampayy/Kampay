//! Contract events emitted by the credit contract.

use soroban_sdk::{contractevent, Address};

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaymentRecorded {
    #[topic]
    pub payer: Address,
    #[topic]
    pub payee: Address,
    pub amount: i128,
    pub on_time: bool,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DisputeRecorded {
    #[topic]
    pub account: Address,
    pub lost: bool,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoanRecorded {
    #[topic]
    pub borrower: Address,
    pub repaid: bool,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReporterChanged {
    #[topic]
    pub reporter: Address,
    pub authorized: bool,
}
