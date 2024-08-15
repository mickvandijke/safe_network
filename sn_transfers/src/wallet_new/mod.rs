use crate::wallet_new::account::Account;
use bls::Signature;
use std::collections::BTreeMap;

pub mod account;
pub mod mnemonic;

/// Eg. "m/0"
pub type DerivationPath = String;

pub trait Signer {
    fn sign<T: AsRef<[u8]>>(&self, data: T) -> Signature;
}

pub struct Wallet {
    mnemonic: bip39::Mnemonic,
    accounts: BTreeMap<DerivationPath, Account>,
}

impl Wallet {}
