use super::address::{addr_to_str, str_to_addr};
#[cfg(feature = "vault")]
use super::vault::UserData;
use crate::client::data_private::PrivateDataAccess;
use crate::client::payment::Receipt;
use libp2p::Multiaddr;
use serde_wasm_bindgen::Serializer;
use sn_protocol::storage::Chunk;
use wasm_bindgen::prelude::*;

/// Same as `serde_wasm_bindgen::to_value`, but also supports large numbers.
fn to_value<T: serde::ser::Serialize + ?Sized>(value: &T) -> Result<JsValue, JsError> {
    let serializer = Serializer::new().serialize_large_number_types_as_bigints(true);
    Ok(value.serialize(&serializer)?)
}

/// The `Client` object allows interaction with the network to store and retrieve data.
///
/// To connect to the network, see {@link Client.connect}.
///
/// # Example
///
/// ```js
/// let client = await Client.connect(["/ip4/127.0.0.1/tcp/36075/ws/p2p/12D3KooWALb...BhDAfJY"]);
/// const dataAddr = await client.dataPut(new Uint8Array([0, 1, 2, 3]), wallet);
///
/// const archive = new Archive();
/// archive.addNewFile("foo", dataAddr);
///
/// const archiveAddr = await client.archivePut(archive, wallet);
/// const archiveFetched = await client.archiveGet(archiveAddr);
/// ```
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

#[wasm_bindgen(js_name = Chunk)]
pub struct JsChunk(Chunk);

#[wasm_bindgen(js_class = Chunk)]
impl JsChunk {
    /// Returns the bytes.
    #[wasm_bindgen]
    pub fn bytes(&self) -> Vec<u8> {
        self.0.value.to_vec()
    }

    /// Returns the XOR name.
    #[wasm_bindgen]
    pub fn xor_name(&self) -> String {
        self.0.address.xorname().to_string()
    }
}

#[wasm_bindgen(js_class = Client)]
impl JsClient {
    /// Connect to the network via the given peers.
    ///
    /// # Example
    ///
    /// ```js
    /// let client = await Client.connect(["/ip4/127.0.0.1/tcp/36075/ws/p2p/12D3KooWALb...BhDAfJY"]);
    /// ```
    #[wasm_bindgen]
    pub async fn connect(peers: Vec<String>) -> Result<JsClient, JsError> {
        let peers = peers
            .into_iter()
            .map(|peer| peer.parse())
            .collect::<Result<Vec<Multiaddr>, _>>()?;

        let client = super::Client::connect(&peers).await?;

        Ok(JsClient(client))
    }

    /// Upload a chunk to the network.
    ///
    /// Returns the hex encoded address of the chunk.
    ///
    /// This is not yet implemented.
    #[wasm_bindgen(js_name = chunkPut)]
    pub async fn chunk_put(&self, _data: Vec<u8>, _wallet: &JsWallet) -> Result<String, JsError> {
        async { unimplemented!() }.await
    }

    /// Fetch the chunk from the network.
    #[wasm_bindgen(js_name = chunkGet)]
    pub async fn chunk_get(&self, addr: String) -> Result<Vec<u8>, JsError> {
        let addr = str_to_addr(&addr)?;
        let chunk = self.0.chunk_get(addr).await?;

        Ok(chunk.value().to_vec())
    }

    /// Upload data to the network.
    ///
    /// Returns the hex encoded address of the data.
    #[wasm_bindgen(js_name = dataPut)]
    pub async fn data_put(&self, data: Vec<u8>, wallet: &JsWallet) -> Result<String, JsError> {
        let data = crate::Bytes::from(data);
        let xorname = self.0.data_put(data, (&wallet.0).into()).await?;

        Ok(addr_to_str(xorname))
    }

    /// Upload private data to the network.
    ///
    /// Returns the `PrivateDataAccess` chunk of the data.
    #[wasm_bindgen(js_name = privateDataPut)]
    pub async fn private_data_put(
        &self,
        data: Vec<u8>,
        wallet: &JsWallet,
    ) -> Result<JsValue, JsError> {
        let data = crate::Bytes::from(data);
        let private_data_access = self.0.private_data_put(data, (&wallet.0).into()).await?;
        let js_value = serde_wasm_bindgen::to_value(&private_data_access)?;

        Ok(js_value)
    }

    /// Upload private data to the network.
    /// Uses a `Receipt` as payment.
    ///
    /// Returns the `PrivateDataAccess` chunk of the data.
    #[wasm_bindgen(js_name = privateDataPutWithReceipt)]
    pub async fn private_data_put_with_receipt(
        &self,
        data: Vec<u8>,
        receipt: JsValue,
    ) -> Result<JsValue, JsError> {
        let data = crate::Bytes::from(data);
        let receipt: Receipt = serde_wasm_bindgen::from_value(receipt)?;
        let private_data_access = self.0.private_data_put(data, receipt.into()).await?;
        let js_value = serde_wasm_bindgen::to_value(&private_data_access)?;

        Ok(js_value)
    }

    /// Fetch the data from the network.
    #[wasm_bindgen(js_name = dataGet)]
    pub async fn data_get(&self, addr: String) -> Result<Vec<u8>, JsError> {
        let addr = str_to_addr(&addr)?;
        let data = self.0.data_get(addr).await?;

        Ok(data.to_vec())
    }

    /// Fetch the data from the network.
    #[wasm_bindgen(js_name = privateDataGet)]
    pub async fn private_data_get(&self, private_data_access: JsValue) -> Result<Vec<u8>, JsError> {
        let private_data_access: PrivateDataAccess =
            serde_wasm_bindgen::from_value(private_data_access)?;
        let data = self.0.private_data_get(private_data_access).await?;

        Ok(data.to_vec())
    }

    /// Get the cost of uploading data to the network.
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
    use wasm_bindgen::JsError;

    /// Structure mapping paths to data addresses.
    #[wasm_bindgen(js_name = Archive)]
    pub struct JsArchive(Archive);

    #[wasm_bindgen(js_class = Archive)]
    impl JsArchive {
        /// Create a new archive.
        #[wasm_bindgen(constructor)]
        pub fn new() -> Self {
            Self(Archive::new())
        }

        /// Add a new file to the archive.
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

        /// Serialize to bytes.
        #[wasm_bindgen(js_name = bytes)]
        pub fn into_bytes(&self) -> Result<Vec<u8>, JsError> {
            let root_serialized = rmp_serde::to_vec(&self.0)?;
            Ok(root_serialized)
        }
    }

    #[wasm_bindgen(js_class = Client)]
    impl JsClient {
        /// Fetch an archive from the network.
        #[wasm_bindgen(js_name = archiveGet)]
        pub async fn archive_get(&self, addr: String) -> Result<JsArchive, JsError> {
            let addr = str_to_addr(&addr)?;
            let archive = self.0.archive_get(addr).await?;
            let archive = JsArchive(archive);

            Ok(archive)
        }

        /// Upload an archive to the network.
        ///
        /// Returns the hex encoded address of the archive.
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

mod archive_private {
    use super::*;
    use crate::client::archive_private::{PrivateArchive, PrivateArchiveAccess};
    use crate::client::data_private::PrivateDataAccess;
    use crate::client::payment::Receipt;
    use std::path::PathBuf;
    use wasm_bindgen::{JsError, JsValue};

    /// Structure mapping paths to data addresses.
    #[wasm_bindgen(js_name = PrivateArchive)]
    pub struct JsPrivateArchive(PrivateArchive);

    #[wasm_bindgen(js_class = PrivateArchive)]
    impl JsPrivateArchive {
        /// Create a new private archive.
        #[wasm_bindgen(constructor)]
        pub fn new() -> Self {
            Self(PrivateArchive::new())
        }

        /// Add a new file to the private archive.
        #[wasm_bindgen(js_name = addNewFile)]
        pub fn add_new_file(&mut self, path: String, data_map: JsValue) -> Result<(), JsError> {
            let path = PathBuf::from(path);
            let data_map: PrivateDataAccess = serde_wasm_bindgen::from_value(data_map)?;
            self.0.add_new_file(path, data_map);

            Ok(())
        }

        #[wasm_bindgen]
        pub fn map(&self) -> Result<JsValue, JsError> {
            let files = serde_wasm_bindgen::to_value(self.0.map())?;
            Ok(files)
        }

        /// Serialize to bytes.
        #[wasm_bindgen(js_name = bytes)]
        pub fn into_bytes(&self) -> Result<Vec<u8>, JsError> {
            let root_serialized = rmp_serde::to_vec(&self.0)?;
            Ok(root_serialized)
        }
    }

    #[wasm_bindgen(js_class = Client)]
    impl JsClient {
        /// Fetch a private archive from the network.
        #[wasm_bindgen(js_name = privateArchiveGet)]
        pub async fn private_archive_get(
            &self,
            private_archive_access: JsValue,
        ) -> Result<JsPrivateArchive, JsError> {
            let private_archive_access: PrivateArchiveAccess =
                serde_wasm_bindgen::from_value(private_archive_access)?;
            let archive = self.0.private_archive_get(private_archive_access).await?;
            let archive = JsPrivateArchive(archive);

            Ok(archive)
        }

        /// Upload a private archive to the network.
        ///
        /// Returns the `PrivateArchiveAccess` chunk of the archive.
        #[wasm_bindgen(js_name = privateArchivePut)]
        pub async fn private_archive_put(
            &self,
            archive: &JsPrivateArchive,
            wallet: &JsWallet,
        ) -> Result<JsValue, JsError> {
            let private_archive_access = self
                .0
                .private_archive_put(archive.0.clone(), (&wallet.0).into())
                .await?;

            let js_value = serde_wasm_bindgen::to_value(&private_archive_access)?;

            Ok(js_value)
        }

        /// Upload a private archive to the network.
        /// Uses a `Receipt` as payment.
        ///
        /// Returns the `PrivateArchiveAccess` chunk of the archive.
        #[wasm_bindgen(js_name = privateArchivePutWithReceipt)]
        pub async fn private_archive_put_with_receipt(
            &self,
            archive: &JsPrivateArchive,
            receipt: JsValue,
        ) -> Result<JsValue, JsError> {
            let receipt: Receipt = serde_wasm_bindgen::from_value(receipt)?;

            let private_archive_access = self
                .0
                .private_archive_put(archive.0.clone(), receipt.into())
                .await?;

            let js_value = serde_wasm_bindgen::to_value(&private_archive_access)?;

            Ok(js_value)
        }
    }
}

#[cfg(feature = "vault")]
mod vault {
    use super::*;
    use crate::client::address::addr_to_str;
    use crate::client::archive_private::PrivateArchiveAccess;
    use crate::client::payment::Receipt;
    use crate::client::vault::user_data::USER_DATA_VAULT_CONTENT_IDENTIFIER;
    use crate::client::vault::VaultContentType;
    use sn_protocol::storage::Scratchpad;
    use wasm_bindgen::{JsError, JsValue};

    /// Structure to keep track of uploaded archives, registers and other data.
    #[wasm_bindgen(js_name = UserData)]
    pub struct JsUserData(UserData);

    #[wasm_bindgen(js_class = UserData)]
    impl JsUserData {
        /// Create a new user data structure.
        #[wasm_bindgen(constructor)]
        pub fn new() -> Self {
            Self(UserData::new())
        }

        /// Store an archive address in the user data with an optional name.
        ///
        /// # Example
        ///
        /// ```js
        /// userData.addFileArchive(archiveAddr, "foo");
        /// ```
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

        /// Store a private archive data map in the user data with an optional name.
        ///
        /// # Example
        ///
        /// ```js
        /// userData.addPrivateFileArchive(privateArchiveAccess, "foo");
        /// ```
        #[wasm_bindgen(js_name = addPrivateFileArchive)]
        pub fn add_private_file_archive(
            &mut self,
            private_archive_access: JsValue,
            name: Option<String>,
        ) -> Result<(), JsError> {
            let private_archive_access: PrivateArchiveAccess =
                serde_wasm_bindgen::from_value(private_archive_access)?;

            let old_name = if let Some(ref name) = name {
                self.0
                    .add_private_file_archive_with_name(private_archive_access, name.clone())
            } else {
                self.0.add_private_file_archive(private_archive_access)
            };

            if let Some(old_name) = old_name {
                tracing::warn!(
                    "Changing name of private archive from `{old_name:?}` to `{name:?}`"
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

        #[wasm_bindgen(js_name = removePrivateFileArchive)]
        pub fn remove_private_file_archive(
            &mut self,
            private_archive_access: JsValue,
        ) -> Result<(), JsError> {
            let private_archive_access: PrivateArchiveAccess =
                serde_wasm_bindgen::from_value(private_archive_access)?;

            self.0.remove_private_file_archive(private_archive_access);

            Ok(())
        }

        #[wasm_bindgen(js_name = fileArchives)]
        pub fn file_archives(&self) -> Result<JsValue, JsError> {
            let archives = serde_wasm_bindgen::to_value(&self.0.file_archives)?;
            Ok(archives)
        }

        #[wasm_bindgen(js_name = privateFileArchives)]
        pub fn private_file_archives(&self) -> Result<JsValue, JsError> {
            let archives = serde_wasm_bindgen::to_value(&self.0.private_file_archives)?;
            Ok(archives)
        }
    }

    #[wasm_bindgen(js_name = Scratchpad)]
    pub struct JsScratchpad(Scratchpad);

    #[wasm_bindgen(js_class = Scratchpad)]
    impl JsScratchpad {
        /// Returns a VEC with the XOR name.
        #[wasm_bindgen(js_name = xorName)]
        pub fn xor_name(&self) -> Option<String> {
            self.0
                .network_address()
                .as_xorname()
                .map(|xor_name| addr_to_str(xor_name))
        }
    }

    #[wasm_bindgen(js_class = Client)]
    impl JsClient {
        /// Fetch the user data from the vault.
        ///
        /// # Example
        ///
        /// ```js
        /// const secretKey = genSecretKey();
        /// const userData = await client.getUserDataFromVault(secretKey);
        /// ```
        #[wasm_bindgen(js_name = getUserDataFromVault)]
        pub async fn get_user_data_from_vault(
            &self,
            secret_key: &SecretKeyJs,
        ) -> Result<JsUserData, JsError> {
            let user_data = self.0.get_user_data_from_vault(&secret_key.0).await?;

            Ok(JsUserData(user_data))
        }

        /// Put the user data to the vault.
        ///
        /// # Example
        ///
        /// ```js
        /// const secretKey = genSecretKey();
        /// await client.putUserDataToVault(userData, wallet, secretKey);
        /// ```
        #[wasm_bindgen(js_name = putUserDataToVault)]
        pub async fn put_user_data_to_vault(
            &self,
            user_data: &JsUserData,
            wallet: &JsWallet,
            secret_key: &SecretKeyJs,
        ) -> Result<(), JsError> {
            self.0
                .put_user_data_to_vault(&secret_key.0, (&wallet.0).into(), user_data.0.clone())
                .await?;

            Ok(())
        }

        /// Put the user data to the vault.
        ///
        /// # Example
        ///
        /// ```js
        /// const secretKey = genSecretKey();
        /// await client.putUserDataToVaultWithReceipt(userData, receipt, secretKey);
        /// ```
        #[wasm_bindgen(js_name = putUserDataToVaultWithReceipt)]
        pub async fn put_user_data_to_vault_with_receipt(
            &self,
            user_data: &JsUserData,
            receipt: JsValue,
            secret_key: &SecretKeyJs,
        ) -> Result<(), JsError> {
            let receipt: Receipt = serde_wasm_bindgen::from_value(receipt)?;

            self.0
                .put_user_data_to_vault(&secret_key.0, receipt.into(), user_data.0.clone())
                .await?;

            Ok(())
        }

        /// Returns an existing scratchpad or creates a new one if it does not exist.
        #[wasm_bindgen(js_name = getOrCreateScratchpad)]
        pub async fn get_or_create_scratchpad(
            &self,
            secret_key: &SecretKeyJs,
            vault_content_type: JsValue,
        ) -> Result<JsValue, JsError> {
            let vault_content_type: VaultContentType =
                serde_wasm_bindgen::from_value(vault_content_type)?;

            let result = self
                .0
                .get_or_create_scratchpad(&secret_key.0, vault_content_type)
                .await?;

            let js_value = serde_wasm_bindgen::to_value(&result)?;

            Ok(js_value)
        }

        /// Returns an existing user data scratchpad or creates a new one if it does not exist.
        #[wasm_bindgen(js_name = getOrCreateUserDataScratchpad)]
        pub async fn get_or_create_user_data_scratchpad(
            &self,
            secret_key: &SecretKeyJs,
        ) -> Result<JsScratchpad, JsError> {
            let vault_content_type = *USER_DATA_VAULT_CONTENT_IDENTIFIER;

            let (scratchpad, _is_new) = self
                .0
                .get_or_create_scratchpad(&secret_key.0, vault_content_type)
                .await?;

            let js_scratchpad = JsScratchpad(scratchpad);

            Ok(js_scratchpad)
        }
    }
}

#[cfg(feature = "external-signer")]
mod external_signer {
    use super::*;
    use crate::client::address::str_to_addr;
    use crate::client::external_signer::encrypt_data;
    use crate::client::payment::Receipt;
    use crate::receipt_from_quotes_and_payments;
    use sn_evm::external_signer::{approve_to_spend_tokens_calldata, pay_for_quotes_calldata};
    use sn_evm::EvmNetwork;
    use sn_evm::QuotePayment;
    use sn_evm::{Amount, PaymentQuote};
    use sn_evm::{EvmAddress, QuoteHash, TxHash};
    use std::collections::{BTreeMap, HashMap};
    use wasm_bindgen::prelude::wasm_bindgen;
    use wasm_bindgen::{JsError, JsValue};
    use xor_name::XorName;

    #[wasm_bindgen(js_class = Client)]
    impl JsClient {
        /// Get quotes for given chunk addresses.
        ///
        /// # Example
        ///
        /// ```js
        /// const [quotes, quotePayments, free_chunks] = await client.getQuotes(chunkAddresses);
        /// ``
        #[wasm_bindgen(js_name = getQuotes)]
        pub async fn get_quotes(&self, chunk_addresses: Vec<String>) -> Result<JsValue, JsError> {
            let mut xor_addresses: Vec<XorName> = vec![];

            for chunk_address_str in &chunk_addresses {
                let xor_address = str_to_addr(chunk_address_str)?;
                xor_addresses.push(xor_address);
            }

            let result = self
                .0
                .get_quotes_for_content_addresses(xor_addresses.into_iter())
                .await?;

            let js_value = serde_wasm_bindgen::to_value(&result)?;

            Ok(js_value)
        }

        /// Upload data with a receipt.
        ///
        /// # Example
        ///
        /// ```js
        /// const receipt = getReceiptFromQuotesAndPayments(quotes, payments);
        /// const addr = await client.dataPutWithReceipt(data, receipt);
        /// ```
        #[wasm_bindgen(js_name = dataPutWithReceipt)]
        pub async fn data_put_with_receipt(
            &self,
            data: Vec<u8>,
            receipt: JsValue,
        ) -> Result<String, JsError> {
            let data = crate::Bytes::from(data);
            let receipt: Receipt = serde_wasm_bindgen::from_value(receipt)?;
            let xorname = self.0.data_put(data, receipt.into()).await?;
            Ok(addr_to_str(xorname))
        }
    }

    /// Encrypt data.
    ///
    /// # Example
    ///
    /// ```js
    /// const [dataMapChunk, dataChunks, dataMapChunkAddress, dataChunkAddresses] = client.encryptData(data);
    /// ``
    #[wasm_bindgen(js_name = encryptData)]
    pub fn encrypt(data: Vec<u8>) -> Result<JsValue, JsError> {
        let data = crate::Bytes::from(data);
        let result = encrypt_data(data)?;
        let map_xor_name = *result.0.address().xorname();
        let mut xor_names = vec![];

        for chunk in &result.1 {
            xor_names.push(*chunk.name());
        }

        let result = (result.0, result.1, map_xor_name, xor_names);
        let js_value = serde_wasm_bindgen::to_value(&result)?;

        Ok(js_value)
    }

    /// Get the calldata for paying for quotes.
    ///
    /// # Example
    ///
    /// ```js
    /// const [quotes, quotePayments, free_chunks] = await client.getQuotes(data);
    /// const callData = getPayForQuotesCalldata(evmNetwork, quotePayments);
    /// ```
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

    /// Form approve to spend tokens calldata.
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

    /// Generate payment proof.
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

/// # Example
///
/// ```js
/// const secretKey = genSecretKey();
/// await client.putUserDataToVault(userData, wallet, secretKey);
/// const userDataFetched = await client.getUserDataFromVault(secretKey);
/// ```
#[wasm_bindgen(js_name = genSecretKey)]
pub fn gen_secret_key() -> SecretKeyJs {
    let secret_key = bls::SecretKey::random();
    SecretKeyJs(secret_key)
}

/// Get the current `EvmNetwork` that was set using environment variables that were used during the build process of this library.
#[wasm_bindgen(js_name = getEvmNetwork)]
pub fn evm_network() -> Result<JsValue, JsError> {
    let evm_network = evmlib::utils::get_evm_network_from_env()?;
    let js_value = serde_wasm_bindgen::to_value(&evm_network)?;
    Ok(js_value)
}

/// Create an `EvmNetwork` with custom values.
///
/// # Example
///
/// ```js
/// const [quotes, quotePayments, free_chunks] = await client.getQuotes(data);
/// const evmNetwork = getEvmNetworkCustom("http://localhost:4343", "<payment token addr>", "<data payments addr>");
/// const payForQuotesCalldata = getPayForQuotesCalldata(evmNetwork, quotePayments);
/// ```
#[wasm_bindgen(js_name = getEvmNetworkCustom)]
pub fn evm_network_custom(
    rpc_url: String,
    payment_token_address: String,
    data_payments_address: String,
) -> Result<JsValue, JsError> {
    let evm_network =
        evmlib::utils::get_evm_network(&rpc_url, &payment_token_address, &data_payments_address);
    let js_value = serde_wasm_bindgen::to_value(&evm_network)?;
    Ok(js_value)
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

/// Get a funded wallet with a custom network.
#[wasm_bindgen(js_name = getFundedWalletWithCustomNetwork)]
pub fn funded_wallet_with_custom_network(
    network: JsValue,
    private_key: String,
) -> Result<JsWallet, JsError> {
    let network: evmlib::Network = serde_wasm_bindgen::from_value(network)?;
    let wallet = evmlib::wallet::Wallet::new_from_private_key(network, &private_key)?;
    Ok(JsWallet(wallet))
}

/// Enable tracing logging in the console.
///
/// A level could be passed like `trace` or `warn`. Or set for a specific module/crate
/// with `sn_networking=trace,autonomi=info`.
///
/// # Example
///
/// ```js
/// logInit("sn_networking=warn,autonomi=trace");
/// ```
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
