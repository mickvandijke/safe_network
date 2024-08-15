use crate::{MainPubkey, MainSecretKey, NanoTokens, SignedSpend, UniquePubkey};
use std::collections::{BTreeMap, BTreeSet};

pub struct Account {
    pub_key: MainPubkey,
    secret_key: MainSecretKey,
    available_cash_notes: BTreeMap<UniquePubkey, NanoTokens>,
    /// These have not yet been successfully sent to the network
    /// and need to be, to reach network validity.
    unconfirmed_spend_requests: BTreeSet<SignedSpend>,
}

impl Account {
    pub fn balance(&self) -> NanoTokens {
        let mut balance = 0;
        for (_unique_pubkey, value) in self.available_cash_notes.iter() {
            balance += value.as_nano();
        }
        NanoTokens::from(balance)
    }
}
