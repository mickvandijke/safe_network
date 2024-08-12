// Copyright 2024 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

//! > **Core functionalities for interacting with the SAFE Network**
//!
//! The `sn_client` crate is a part of the [Safe Network](https://safenetwork.tech/) (SN),
//! and plays a crucial role in this ecosystem by serving as the client library that allows
//! applications and users to interact with the Safe Network, and build applications that
//! leverage the Safe Network's capabilities, providing a high-level API that simplifies the development process.
//!
//! Here are the key functionalities provided by this crate:
//!
//! 1. **Network Communication**: It handles communication with the Safe Network, enabling clients to
//!    send and receive messages from the decentralized nodes that make up the network.
//!
//! 2. **Data Storage and Retrieval**: to store and retrieve data on the Safe Network.
//!    This includes both private and public data, ensuring privacy and security.
//!
//! 3. **Authentication and Access Control**: It provides mechanisms for authenticating users and
//!    managing access to data, ensuring that only authorized users can access sensitive information.
//!
//! 4. **File Management**: The crate supports operations related to file management, such as uploading,
//!    downloading, and managing files and directories on the Safe Network.
//!
//! 5. **Token Management**: It includes functionality for managing Safe Network tokens, which can be
//!    used for various purposes within the network, including paying for storage and services.
//!
//! ## Quick links
//! - [Crates.io](https://crates.io/crates/sn_client)
//! - [Forum](https://forum.autonomi.community/)
//! - [Issues on GitHub](https://github.com/maidsafe/safe_network/issues)
//!

#[macro_use]
extern crate tracing;

pub mod acc_packet;
pub mod api;
mod audit;
mod chunks;
mod error;
mod event;
mod faucet;
mod files;
mod folders;
mod register;
mod uploader;
mod wallet;

/// Test utils
#[cfg(feature = "test-utils")]
pub mod test_utils;

// re-export used crates to make them available to app builders
// this ensures the version of the crates used by the app builders are the same as the ones used by the client
// so they don't run into issues with incompatible types due to different versions of the same crate
pub use sn_networking as networking;
pub use sn_protocol as protocol;
pub use sn_registers as registers;
pub use sn_transfers as transfers;

const MAX_CONCURRENT_TASKS: usize = 4096;

pub use self::{
    audit::{DagError, SpendDag, SpendDagGet, SpendFault},
    error::Error,
    event::{ClientEvent, ClientEventsBroadcaster, ClientEventsReceiver},
    faucet::fund_faucet_from_genesis_wallet,
    files::{
        download::{FilesDownload, FilesDownloadEvent},
        FilesApi, BATCH_SIZE,
    },
    folders::{FolderEntry, FoldersApi, Metadata},
    register::ClientRegister,
    uploader::{UploadCfg, UploadEvent, UploadSummary, Uploader},
    wallet::{send, StoragePaymentResult, WalletClient},
};
pub(crate) use error::Result;

use sn_networking::Network;
use std::sync::Arc;

#[cfg(target_arch = "wasm32")]
use console_error_panic_hook;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use web_sys::console;

// This is like the `main` function, except for JavaScript.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub async fn main_js() -> std::result::Result<(), JsValue> {
    // This provides better error messages in debug mode.
    // It's disabled in release mode so it doesn't bloat up the file size.
    // #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    console::log_1(&JsValue::from_str("Hello safe world!"));

    // Tracing
    // TODO: dont log _everything_
    // right now it logs all libp2p entirely.
    tracing_wasm::set_as_global_default();

    Ok(())
}

/// A quick client that only takes some peers to connect to
#[wasm_bindgen]
#[cfg(target_arch = "wasm32")]
pub async fn get_data(peer: &str, data_address: &str) -> std::result::Result<(), JsError> {
    let bytes = hex::decode(&data_address).expect("Input address is not a hex string");
    let xor_name = xor_name::XorName(
        bytes
            .try_into()
            .expect("Failed to parse XorName from hex string"),
    );

    use sn_protocol::storage::ChunkAddress;
    console::log_1(&JsValue::from_str(peer));

    let the_peer = sn_peers_acquisition::parse_peer_addr(peer)?;

    console::log_1(&JsValue::from_str(&format!(
        "Provided Peer was {the_peer:?}"
    )));

    // TODO: We need to tidy this up, the client loops forever in the browser, and eventually crashes
    // it does _do things_ but errors surface, and even after getting data, it continues...
    let client = Client::quick_start(Some(vec![the_peer]))
        .await
        .map_err(|e| JsError::new(&format!("Client could not start: {e:?}")))?;

    console::log_1(&JsValue::from_str("Client started {chunk:?}"));

    let chunk = client
        .get_chunk(ChunkAddress::new(xor_name), false, None)
        .await
        .map_err(|e| JsError::new(&format!("Client get data failed: {e:?}")))?;

    console::log_1(&JsValue::from_str(&format!("Data found {chunk:?}")));

    Ok(())
}
