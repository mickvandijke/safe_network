use crate::common::{Address, Calldata, TxHash, U256};
use crate::contract::network_token::NetworkTokenContract::NetworkTokenContractInstance;
use alloy::providers::{Network, Provider};
use alloy::sol;
use alloy::transports::{RpcError, Transport, TransportErrorKind};

sol!(
    #[allow(clippy::too_many_arguments)]
    #[allow(missing_docs)]
    #[sol(rpc)]
    NetworkTokenContract,
    "artifacts/AutonomiNetworkToken.json"
);

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    ContractError(#[from] alloy::contract::Error),
    #[error(transparent)]
    RpcError(#[from] RpcError<TransportErrorKind>),
}

pub struct NetworkToken<T: Transport + Clone, P: Provider<T, N>, N: Network> {
    pub contract: NetworkTokenContractInstance<T, P, N>,
}

impl<T, P, N> NetworkToken<T, P, N>
where
    T: Transport + Clone,
    P: Provider<T, N>,
    N: Network,
{
    /// Create a new NetworkToken contract instance.
    pub fn new(contract_address: Address, provider: P) -> Self {
        let contract = NetworkTokenContract::new(contract_address, provider);
        NetworkToken { contract }
    }

    /// Deploys the AutonomiNetworkToken smart contract to the network of the provider.
    /// ONLY DO THIS IF YOU KNOW WHAT YOU ARE DOING!
    pub async fn deploy(provider: P) -> Self {
        let contract = NetworkTokenContract::deploy(provider)
            .await
            .expect("Could not deploy contract");
        NetworkToken { contract }
    }

    pub fn set_provider(&mut self, provider: P) {
        let address = *self.contract.address();
        self.contract = NetworkTokenContract::new(address, provider);
    }

    /// Get the raw token balance of an address.
    pub async fn balance_of(&self, account: Address) -> Result<U256, Error> {
        let balance = self.contract.balanceOf(account).call().await?._0;
        Ok(balance)
    }

    /// Approve spender to spend a raw amount of tokens.
    pub async fn approve(&self, spender: Address, value: U256) -> Result<TxHash, Error> {
        let tx_hash = self
            .contract
            .approve(spender, value)
            .send()
            .await?
            .watch()
            .await?;

        Ok(tx_hash)
    }

    /// Approve spender to spend a raw amount of tokens.
    /// Returns the `To` address and transaction calldata.
    pub fn approve_calldata(&self, spender: Address, value: U256) -> (Address, Calldata) {
        let calldata = self.contract.approve(spender, value).calldata().to_owned();
        (*self.contract.address(), calldata)
    }

    /// Transfer a raw amount of tokens.
    pub async fn transfer(&self, receiver: Address, amount: U256) -> Result<TxHash, Error> {
        let tx_hash = self
            .contract
            .transfer(receiver, amount)
            .send()
            .await?
            .watch()
            .await?;

        Ok(tx_hash)
    }

    /// Transfer a raw amount of tokens.
    /// Returns the `To` address and transaction calldata.
    pub fn transfer_calldata(&self, receiver: Address, amount: U256) -> (Address, Calldata) {
        let calldata = self
            .contract
            .transfer(receiver, amount)
            .calldata()
            .to_owned();

        (*self.contract.address(), calldata)
    }
}
