use super::address::{addr_to_str, str_to_addr};
use crate::client::payments::PaymentOption;
use libp2p::Multiaddr;
use wasm_bindgen::prelude::*;

#[cfg(feature = "vault")]
use super::vault::UserData;

#[wasm_bindgen(js_name = Client)]
pub struct JsClient(super::Client);

#[wasm_bindgen]
pub struct AttoTokens(sn_evm::AttoTokens);
#[wasm_bindgen]
impl AttoTokens {
    #[wasm_bindgen(js_name = toString)]
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

#[wasm_bindgen(js_class = Client)]
impl JsClient {
    #[wasm_bindgen]
    pub async fn connect(peers: Vec<String>) -> Result<JsClient, JsError> {
        let peers = peers
            .into_iter()
            .map(|peer| peer.parse())
            .collect::<Result<Vec<Multiaddr>, _>>()?;

        let client = super::Client::connect(&peers).await?;

        Ok(JsClient(client))
    }

    #[wasm_bindgen(js_name = chunkPut)]
    pub async fn chunk_put(&self, _data: Vec<u8>, _wallet: &JsWallet) -> Result<String, JsError> {
        async { unimplemented!() }.await
    }

    #[wasm_bindgen(js_name = chunkGet)]
    pub async fn chunk_get(&self, addr: String) -> Result<Vec<u8>, JsError> {
        let addr = str_to_addr(&addr)?;
        let chunk = self.0.chunk_get(addr).await?;

        Ok(chunk.value().to_vec())
    }

    #[wasm_bindgen(js_name = dataPut)]
    pub async fn data_put(&self, data: Vec<u8>, wallet: &JsWallet) -> Result<String, JsError> {
        let data = crate::Bytes::from(data);
        let xorname = self
            .0
            .data_put(data, PaymentOption::from(&wallet.0))
            .await?;

        Ok(addr_to_str(xorname))
    }

    #[wasm_bindgen(js_name = dataGet)]
    pub async fn data_get(&self, addr: String) -> Result<Vec<u8>, JsError> {
        let addr = str_to_addr(&addr)?;
        let data = self.0.data_get(addr).await?;

        Ok(data.to_vec())
    }

    #[wasm_bindgen(js_name = dataCost)]
    pub async fn data_cost(&self, data: Vec<u8>) -> Result<AttoTokens, JsValue> {
        let data = crate::Bytes::from(data);
        let cost = self.0.data_cost(data).await.map_err(JsError::from)?;

        Ok(AttoTokens(cost))
    }
}

mod archive {
    use super::*;
    use crate::client::{address::str_to_addr, archive::Archive};
    use std::path::PathBuf;

    #[wasm_bindgen(js_name = Archive)]
    pub struct JsArchive(Archive);

    #[wasm_bindgen(js_class = Archive)]
    impl JsArchive {
        #[wasm_bindgen(constructor)]
        pub fn new() -> Self {
            Self(Archive::new())
        }

        #[wasm_bindgen(js_name = addNewFile)]
        pub fn add_new_file(&mut self, path: String, data_addr: String) -> Result<(), JsError> {
            let path = PathBuf::from(path);
            let data_addr = str_to_addr(&data_addr)?;
            self.0.add_new_file(path, data_addr);

            Ok(())
        }

        #[wasm_bindgen(js_name = renameFile)]
        pub fn rename_file(&mut self, old_path: String, new_path: String) -> Result<(), JsError> {
            let old_path = PathBuf::from(old_path);
            let new_path = PathBuf::from(new_path);
            self.0.rename_file(&old_path, &new_path)?;

            Ok(())
        }

        #[wasm_bindgen]
        pub fn map(&self) -> Result<JsValue, JsError> {
            let files = serde_wasm_bindgen::to_value(self.0.map())?;
            Ok(files)
        }
    }

    #[wasm_bindgen(js_class = Client)]
    impl JsClient {
        #[wasm_bindgen(js_name = archiveGet)]
        pub async fn archive_get(&self, addr: String) -> Result<JsArchive, JsError> {
            let addr = str_to_addr(&addr)?;
            let archive = self.0.archive_get(addr).await?;
            let archive = JsArchive(archive);

            Ok(archive)
        }

        #[wasm_bindgen(js_name = archivePut)]
        pub async fn archive_put(
            &self,
            archive: &JsArchive,
            wallet: &JsWallet,
        ) -> Result<String, JsError> {
            let addr = self.0.archive_put(archive.0.clone(), &wallet.0).await?;

            Ok(addr_to_str(addr))
        }
    }
}

#[cfg(feature = "vault")]
mod vault {
    use super::*;
    use crate::client::external_signer::pay_for_quotes_calldata;
    use crate::client::vault::key::{blst_to_blsttc, derive_secret_key_from_seed};
    use crate::EvmNetwork;
    use sn_evm::QuotePayment;
    use wasm_bindgen::prelude::wasm_bindgen;
    use wasm_bindgen::{JsError, JsValue};

    #[wasm_bindgen(js_name = UserData)]
    pub struct JsUserData(UserData);

    #[wasm_bindgen(js_class = UserData)]
    impl JsUserData {
        #[wasm_bindgen(constructor)]
        pub fn new() -> Self {
            Self(UserData::new())
        }

        #[wasm_bindgen(js_name = addFileArchive)]
        pub fn add_file_archive(
            &mut self,
            archive: String,
            name: Option<String>,
        ) -> Result<(), JsError> {
            let archive = str_to_addr(&archive)?;

            let old_name = if let Some(ref name) = name {
                self.0.add_file_archive_with_name(archive, name.clone())
            } else {
                self.0.add_file_archive(archive)
            };

            if let Some(old_name) = old_name {
                tracing::warn!(
                    "Changing name of archive `{archive}` from `{old_name:?}` to `{name:?}`"
                );
            }

            Ok(())
        }

        #[wasm_bindgen(js_name = removeFileArchive)]
        pub fn remove_file_archive(&mut self, archive: String) -> Result<(), JsError> {
            let archive = str_to_addr(&archive)?;
            self.0.remove_file_archive(archive);

            Ok(())
        }

        #[wasm_bindgen(js_name = fileArchives)]
        pub fn file_archives(&self) -> Result<JsValue, JsError> {
            let archives = serde_wasm_bindgen::to_value(&self.0.file_archives)?;
            Ok(archives)
        }
    }

    #[wasm_bindgen(js_class = Client)]
    impl JsClient {
        #[wasm_bindgen(js_name = getUserDataFromVault)]
        pub async fn get_user_data_from_vault(
            &self,
            secret_key: &SecretKeyJs,
        ) -> Result<JsUserData, JsError> {
            let user_data = self.0.get_user_data_from_vault(&secret_key.0).await?;

            Ok(JsUserData(user_data))
        }

        #[wasm_bindgen(js_name = putUserDataToVault)]
        pub async fn put_user_data_to_vault(
            &self,
            user_data: &JsUserData,
            wallet: &JsWallet,
            secret_key: &SecretKeyJs,
        ) -> Result<(), JsError> {
            self.0
                .put_user_data_to_vault(&secret_key.0, &wallet.0, user_data.0.clone())
                .await?;

            Ok(())
        }
    }

    #[wasm_bindgen(js_name = vaultKeyFromSignature)]
    pub fn vault_key_from_signature(signature: Vec<u8>) -> Result<SecretKeyJs, JsError> {
        let blst_key = derive_secret_key_from_seed(&signature)?;
        let vault_sk = blst_to_blsttc(&blst_key)?;
        Ok(SecretKeyJs(vault_sk))
    }
}

#[cfg(feature = "external-signer")]
mod external_signer {
    use super::*;
    use crate::client::payments::{PaymentOption, Receipt};
    use crate::receipt_from_quotes_and_payments;
    use sn_evm::external_signer::{approve_to_spend_tokens_calldata, pay_for_quotes_calldata};
    use sn_evm::EvmNetwork;
    use sn_evm::ProofOfPayment;
    use sn_evm::QuotePayment;
    use sn_evm::{Amount, PaymentQuote};
    use sn_evm::{EvmAddress, QuoteHash, TxHash};
    use std::collections::{BTreeMap, HashMap};
    use wasm_bindgen::prelude::wasm_bindgen;
    use wasm_bindgen::{JsError, JsValue};
    use xor_name::XorName;

    #[wasm_bindgen(js_class = Client)]
    impl JsClient {
        #[wasm_bindgen(js_name = getQuotes)]
        pub async fn get_quotes_for_data(&self, data: Vec<u8>) -> Result<JsValue, JsError> {
            let data = crate::Bytes::from(data);
            let result = self.0.get_quotes_for_data(data).await?;
            let js_value = serde_wasm_bindgen::to_value(&result)?;
            Ok(js_value)
        }

        #[wasm_bindgen(js_name = dataPutWithReceipt)]
        pub async fn data_put_with_receipt(
            &self,
            data: Vec<u8>,
            receipt: JsValue,
        ) -> Result<String, JsError> {
            let data = crate::Bytes::from(data);
            let receipt: Receipt = serde_wasm_bindgen::from_value(receipt)?;

            let xorname = self
                .0
                .data_put(data, PaymentOption::Receipt(receipt))
                .await?;

            Ok(addr_to_str(xorname))
        }

        #[cfg(feature = "vault")]
        #[wasm_bindgen(js_name = putUserDataToVaultWithReceipt)]
        pub async fn put_user_data_to_vault_with_receipt(
            &self,
            user_data: &JsUserData,
            secret_key: &SecretKeyJs,
            receipt: JsValue,
        ) -> Result<(), JsError> {
            let receipt: Receipt = serde_wasm_bindgen::from_value(receipt)?;

            self.0
                .put_user_data_to_vault(&secret_key.0, &wallet.0, user_data.0.clone())
                .await?;

            Ok(())
        }
    }

    #[wasm_bindgen(js_name = getPayForQuotesCalldata)]
    pub fn get_pay_for_quotes_calldata(
        network: JsValue,
        payments: JsValue,
    ) -> Result<JsValue, JsError> {
        let network: EvmNetwork = serde_wasm_bindgen::from_value(network)?;
        let payments: Vec<QuotePayment> = serde_wasm_bindgen::from_value(payments)?;
        let calldata = pay_for_quotes_calldata(&network, payments.into_iter())?;
        let js_value = serde_wasm_bindgen::to_value(&calldata)?;
        Ok(js_value)
    }

    #[wasm_bindgen(js_name = getApproveToSpendTokensCalldata)]
    pub fn get_approve_to_spend_tokens_calldata(
        network: JsValue,
        spender: JsValue,
        amount: JsValue,
    ) -> Result<JsValue, JsError> {
        let network: EvmNetwork = serde_wasm_bindgen::from_value(network)?;
        let spender: EvmAddress = serde_wasm_bindgen::from_value(spender)?;
        let amount: Amount = serde_wasm_bindgen::from_value(amount)?;
        let calldata = approve_to_spend_tokens_calldata(&network, spender, amount);
        let js_value = serde_wasm_bindgen::to_value(&calldata)?;
        Ok(js_value)
    }

    #[wasm_bindgen(js_name = getReceiptFromQuotesAndPayments)]
    pub fn get_receipt_from_quotes_and_payments(
        quotes: JsValue,
        payments: JsValue,
    ) -> Result<JsValue, JsError> {
        let quotes: HashMap<XorName, PaymentQuote> = serde_wasm_bindgen::from_value(quotes)?;
        let payments: BTreeMap<QuoteHash, TxHash> = serde_wasm_bindgen::from_value(payments)?;
        let receipt = receipt_from_quotes_and_payments(&quotes, &payments);
        let js_value = serde_wasm_bindgen::to_value(&receipt)?;
        Ok(js_value)
    }
}

#[wasm_bindgen(js_name = SecretKey)]
pub struct SecretKeyJs(bls::SecretKey);

#[wasm_bindgen(js_name = genSecretKey)]
pub fn gen_secret_key() -> SecretKeyJs {
    let secret_key = bls::SecretKey::random();
    SecretKeyJs(secret_key)
}

#[wasm_bindgen(js_name = Wallet)]
pub struct JsWallet(evmlib::wallet::Wallet);

/// Get a funded wallet for testing. This either uses a default private key or the `EVM_PRIVATE_KEY`
/// environment variable that was used during the build process of this library.
#[wasm_bindgen(js_name = getFundedWallet)]
pub fn funded_wallet() -> JsWallet {
    let network = evmlib::utils::get_evm_network_from_env()
        .expect("Failed to get EVM network from environment variables");
    if matches!(network, evmlib::Network::ArbitrumOne) {
        panic!("You're trying to use ArbitrumOne network. Use a custom network for testing.");
    }
    // Default deployer wallet of the testnet.
    const DEFAULT_WALLET_PRIVATE_KEY: &str =
        "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";

    let private_key = std::env::var("SECRET_KEY").unwrap_or(DEFAULT_WALLET_PRIVATE_KEY.to_string());

    let wallet = evmlib::wallet::Wallet::new_from_private_key(network, &private_key)
        .expect("Invalid private key");

    JsWallet(wallet)
}

/// Enable tracing logging in the console.
///
/// A level could be passed like `trace` or `warn`. Or set for a specific module/crate
/// with `sn_networking=trace,autonomi=info`.
#[wasm_bindgen(js_name = logInit)]
pub fn log_init(directive: String) {
    use tracing_subscriber::prelude::*;

    console_error_panic_hook::set_once();

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false) // Only partially supported across browsers
        .without_time() // std::time is not available in browsers
        .with_writer(tracing_web::MakeWebConsoleWriter::new()); // write events to the console
    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(tracing_subscriber::EnvFilter::new(directive))
        .init();
}
