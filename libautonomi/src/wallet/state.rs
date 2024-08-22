use std::collections::HashMap;
use crate::wallet::MemWallet;
use sn_transfers::bls::SecretKey;
use sn_transfers::{CashNote, HotWallet, Spend, UniquePubkey};

pub struct WalletState {
    secret_key: SecretKey,
    cash_notes: HashMap<UniquePubkey, CashNote>,
    spends: HashMap<UniquePubkey, Spend>,
}

impl From<&MemWallet> for WalletState {
    fn from(value: &MemWallet) -> Self {
        let secret_key = value.hot_wallet.key().secret_key().to_owned();

        let cash_notes = value.available_cash_notes.values().collect();

        let mut spends = HashMap::new();

        [&value.pending_spends, &value.confirmed_spends].iter().for_each(|&spend_map| {
            for (key, spend) in spend_map {
                spends.insert(key.clone(), spend.clone());
            }
        });
        
        Self {
            secret_key,
            cash_notes,
            spends,
        }
    }
}

impl From<WalletState> for MemWallet {
    fn from(value: WalletState) -> Self {
        let hot_wallet = HotWallet {};
        
        let pending_spends

        Self {
            hot_wallet,
            available_cash_notes: Default::default(),
            pending_spends: Default::default(),
            confirmed_spends: Default::default(),
        }
    }
}
