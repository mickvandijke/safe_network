use std::collections::BTreeMap;

use crate::common::{Address, Calldata, QuoteHash, QuotePayment, TxHash, U256};
use crate::contract::chunk_payments::{ChunkPayments, MAX_TRANSFERS_PER_TRANSACTION};
use crate::contract::network_token::NetworkToken;
use crate::contract::{chunk_payments, network_token};
use crate::Network;
use alloy::network::{Ethereum, EthereumWallet, NetworkWallet, TransactionBuilder};
use alloy::providers::fillers::{
    ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller, RecommendedFiller, WalletFiller,
};
use alloy::providers::{Identity, Provider, ProviderBuilder, ReqwestProvider};
use alloy::rpc::types::TransactionRequest;
use alloy::signers::local::{LocalSigner, PrivateKeySigner};
use alloy::transports::http::{reqwest, Client, Http};
use alloy::transports::{RpcError, TransportErrorKind};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Private key is invalid")]
    PrivateKeyInvalid,
    #[error(transparent)]
    RpcError(#[from] RpcError<TransportErrorKind>),
    #[error("Network token contract error: {0}")]
    NetworkTokenContract(#[from] network_token::Error),
    #[error("Chunk payments contract error: {0}")]
    ChunkPaymentsContract(#[from] chunk_payments::error::Error),
}

pub struct Wallet {
    wallet: EthereumWallet,
    network: Network,
}

impl Wallet {
    /// Creates a new Wallet object with the specific Network and EthereumWallet.
    pub fn new(network: Network, wallet: EthereumWallet) -> Self {
        Self { wallet, network }
    }

    /// Convenience function that creates a new Wallet with a random EthereumWallet.
    pub fn new_with_random_wallet(network: Network) -> Self {
        Self::new(network, random())
    }

    /// Creates a new Wallet based on the given private_key. It will fail with Error::PrivateKeyInvalid if private_key is invalid.
    pub fn new_from_private_key(network: Network, private_key: &str) -> Result<Self, Error> {
        let wallet = from_private_key(private_key)?;
        Ok(Self::new(network, wallet))
    }

    /// Returns the address of this wallet.
    pub fn address(&self) -> Address {
        wallet_address(&self.wallet)
    }

    /// Returns the raw balance of payment tokens for this wallet.
    pub async fn balance_of_tokens(&self) -> Result<U256, network_token::Error> {
        balance_of_tokens(wallet_address(&self.wallet), &self.network).await
    }

    /// Returns the raw balance of gas tokens for this wallet.
    pub async fn balance_of_gas_tokens(&self) -> Result<U256, network_token::Error> {
        balance_of_gas_tokens(wallet_address(&self.wallet), &self.network).await
    }

    /// Transfer a raw amount of payment tokens to another address.
    pub async fn transfer_tokens(
        &self,
        to: Address,
        amount: U256,
    ) -> Result<TxHash, network_token::Error> {
        transfer_tokens(self.wallet.clone(), &self.network, to, amount).await
    }

    /// Transfer a raw amount of gas tokens to another address.
    pub async fn transfer_gas_tokens(
        &self,
        to: Address,
        amount: U256,
    ) -> Result<TxHash, network_token::Error> {
        transfer_gas_tokens(self.wallet.clone(), &self.network, to, amount).await
    }

    /// Pays for a single quote. Returns transaction hash of the payment.
    pub async fn pay_for_quote(
        &self,
        quote_hash: QuoteHash,
        rewards_addr: Address,
        amount: U256,
    ) -> Result<TxHash, Error> {
        self.pay_for_quotes([(quote_hash, rewards_addr, amount)])
            .await
            .map(|v| v.values().last().cloned().expect("Infallible"))
            .map_err(|err| err.0)
    }

    /// Function for batch payments of quotes. It accepts an iterator of QuotePayment and returns
    /// transaction hashes of the payments by quotes.
    pub async fn pay_for_quotes<I: IntoIterator<Item = QuotePayment>>(
        &self,
        chunk_payments: I,
    ) -> Result<BTreeMap<QuoteHash, TxHash>, PayForQuotesError> {
        pay_for_quotes(self.wallet.clone(), &self.network, chunk_payments).await
    }
}

/// Generate an EthereumWallet with a random private key.
fn random() -> EthereumWallet {
    let signer: PrivateKeySigner = LocalSigner::random();
    EthereumWallet::from(signer)
}

/// Creates a wallet from a private key in HEX format.
fn from_private_key(private_key: &str) -> Result<EthereumWallet, Error> {
    let signer: PrivateKeySigner = private_key.parse().map_err(|_| Error::PrivateKeyInvalid)?;
    Ok(EthereumWallet::from(signer))
}

// TODO(optimization): Find a way to reuse/persist contracts and/or a provider without the wallet nonce going out of sync

#[allow(clippy::type_complexity)]
fn http_provider(
    rpc_url: reqwest::Url,
) -> FillProvider<
    JoinFill<JoinFill<JoinFill<Identity, GasFiller>, NonceFiller>, ChainIdFiller>,
    ReqwestProvider,
    Http<Client>,
    Ethereum,
> {
    ProviderBuilder::new()
        .with_recommended_fillers()
        .on_http(rpc_url)
}

fn http_provider_with_wallet(
    rpc_url: reqwest::Url,
    wallet: EthereumWallet,
) -> FillProvider<
    JoinFill<RecommendedFiller, WalletFiller<EthereumWallet>>,
    ReqwestProvider,
    Http<Client>,
    Ethereum,
> {
    ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_http(rpc_url)
}

/// Returns the address of this wallet.
pub fn wallet_address(wallet: &EthereumWallet) -> Address {
    <EthereumWallet as NetworkWallet<Ethereum>>::default_signer_address(wallet)
}

/// Returns the raw balance of payment tokens for this wallet.
pub async fn balance_of_tokens(
    account: Address,
    network: &Network,
) -> Result<U256, network_token::Error> {
    let provider = http_provider(network.rpc_url().clone());
    let network_token = NetworkToken::new(*network.payment_token_address(), provider);
    network_token.balance_of(account).await
}

/// Returns the raw balance of gas tokens for this wallet.
pub async fn balance_of_gas_tokens(
    account: Address,
    network: &Network,
) -> Result<U256, network_token::Error> {
    let provider = http_provider(network.rpc_url().clone());
    let balance = provider.get_balance(account).await?;
    Ok(balance)
}

/// Approve an address / smart contract to spend this wallet's payment tokens.
async fn approve_to_spend_tokens(
    wallet: EthereumWallet,
    network: &Network,
    spender: Address,
    amount: U256,
) -> Result<TxHash, network_token::Error> {
    let provider = http_provider_with_wallet(network.rpc_url().clone(), wallet);
    let network_token = NetworkToken::new(*network.payment_token_address(), provider);
    network_token.approve(spender, amount).await
}

/// Transfer payment tokens from the supplied wallet to an address.
pub async fn transfer_tokens(
    wallet: EthereumWallet,
    network: &Network,
    receiver: Address,
    amount: U256,
) -> Result<TxHash, network_token::Error> {
    let provider = http_provider_with_wallet(network.rpc_url().clone(), wallet);
    let network_token = NetworkToken::new(*network.payment_token_address(), provider);
    network_token.transfer(receiver, amount).await
}

/// Transfer payment tokens from the supplied wallet to an address.
/// Returns the `To` address and transaction calldata.
pub fn transfer_tokens_calldata(
    network: &Network,
    receiver: Address,
    amount: U256,
) -> (Address, Calldata) {
    let provider = http_provider(network.rpc_url().clone());
    let network_token = NetworkToken::new(*network.payment_token_address(), provider);
    network_token.transfer_calldata(receiver, amount)
}

/// Transfer native/gas tokens from the supplied wallet to an address.
pub async fn transfer_gas_tokens(
    wallet: EthereumWallet,
    network: &Network,
    receiver: Address,
    amount: U256,
) -> Result<TxHash, network_token::Error> {
    let provider = http_provider_with_wallet(network.rpc_url().clone(), wallet);
    let tx = TransactionRequest::default()
        .with_to(receiver)
        .with_value(amount);

    let tx_hash = provider.send_transaction(tx).await?.watch().await?;

    Ok(tx_hash)
}

/// Contains the payment error and the already succeeded batch payments (if any).
#[derive(Debug)]
pub struct PayForQuotesError(pub Error, pub BTreeMap<QuoteHash, TxHash>);

/// Use this wallet to pay for chunks in batched transfer transactions.
/// If the amount of transfers is more than one transaction can contain, the transfers will be split up over multiple transactions.
pub async fn pay_for_quotes<T: IntoIterator<Item = QuotePayment>>(
    wallet: EthereumWallet,
    network: &Network,
    payments: T,
) -> Result<BTreeMap<QuoteHash, TxHash>, PayForQuotesError> {
    let payments: Vec<_> = payments.into_iter().collect();
    let total_amount = payments.iter().map(|(_, _, amount)| amount).sum();

    let mut tx_hashes_by_quote = BTreeMap::new();

    // Approve the contract to spend enough of the client's tokens.
    approve_to_spend_tokens(
        wallet.clone(),
        network,
        *network.chunk_payments_address(),
        total_amount,
    )
    .await
    .map_err(|err| PayForQuotesError(Error::from(err), tx_hashes_by_quote.clone()))?;

    let provider = http_provider_with_wallet(network.rpc_url().clone(), wallet);
    let chunk_payments = ChunkPayments::new(*network.chunk_payments_address(), provider);

    // Divide transfers over multiple transactions if they exceed the max per transaction.
    let chunks = payments.chunks(MAX_TRANSFERS_PER_TRANSACTION);

    for batch in chunks {
        let batch: Vec<QuotePayment> = batch.to_vec();

        let tx_hash = chunk_payments
            .pay_for_quotes(batch.clone())
            .await
            .map_err(|err| PayForQuotesError(Error::from(err), tx_hashes_by_quote.clone()))?;

        for (quote_hash, _, _) in batch {
            tx_hashes_by_quote.insert(quote_hash, tx_hash);
        }
    }

    Ok(tx_hashes_by_quote)
}

#[cfg(test)]
mod tests {
    use crate::common::Amount;
    use crate::testnet::Testnet;
    use crate::wallet::{from_private_key, Wallet};
    use alloy::network::{Ethereum, EthereumWallet, NetworkWallet};
    use alloy::primitives::address;

    #[tokio::test]
    async fn test_from_private_key() {
        let private_key = "bf210844fa5463e373974f3d6fbedf451350c3e72b81b3c5b1718cb91f49c33d"; // DevSkim: ignore DS117838
        let wallet = from_private_key(private_key).unwrap();
        let account = <EthereumWallet as NetworkWallet<Ethereum>>::default_signer_address(&wallet);

        // Assert that the addresses are the same, i.e. the wallet was successfully created from the private key
        assert_eq!(
            account,
            address!("1975d01f46D70AAc0dd3fCf942d92650eE63C79A")
        );
    }

    #[tokio::test]
    async fn test_transfer_gas_tokens() {
        let testnet = Testnet::new().await;
        let network = testnet.to_network();
        let wallet =
            Wallet::new_from_private_key(network.clone(), &testnet.default_wallet_private_key())
                .unwrap();
        let receiver_wallet = Wallet::new_with_random_wallet(network);
        let transfer_amount = Amount::from(117);

        let initial_balance = receiver_wallet.balance_of_gas_tokens().await.unwrap();

        assert_eq!(initial_balance, Amount::from(0));

        let _ = wallet
            .transfer_gas_tokens(receiver_wallet.address(), transfer_amount)
            .await
            .unwrap();

        let final_balance = receiver_wallet.balance_of_gas_tokens().await.unwrap();

        assert_eq!(final_balance, transfer_amount);
    }
}
