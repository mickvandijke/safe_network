use crate::client::data::PayError;
use crate::Client;
use sn_evm::{EvmWallet, ProofOfPayment};
use std::collections::HashMap;
use xor_name::XorName;

/// Contains the proof of payment for XOR addresses.
pub type Receipt = HashMap<XorName, ProofOfPayment>;

/// Payment options for data payments.
pub enum PaymentOption {
    Wallet(EvmWallet),
    Receipt(Receipt),
}

impl From<&EvmWallet> for PaymentOption {
    fn from(value: &EvmWallet) -> Self {
        PaymentOption::Wallet(value.clone())
    }
}

impl Client {
    pub(crate) async fn pay_for_content_addrs(
        &self,
        payment_option: PaymentOption,
        content_addrs: impl Iterator<Item = XorName>,
    ) -> Result<Receipt, PayError> {
        match payment_option {
            PaymentOption::Wallet(wallet) => {
                let (proofs, _) = self.pay(content_addrs, &wallet).await?;
                Ok(proofs)
            }
            PaymentOption::Receipt(receipt) => Ok(receipt),
        }
    }
}
