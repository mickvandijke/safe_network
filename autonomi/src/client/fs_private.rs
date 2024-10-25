// Copyright 2024 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

// Copyright 2024 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::client::Client;
use bytes::Bytes;
use sn_evm::EvmWallet;
use std::path::PathBuf;

use super::archive_private::{PrivateArchive, PrivateArchiveAccess};
use super::data_private::PrivateDataAccess;
use super::fs::{FileDownloadError, FileUploadError};

impl Client {
    /// Download a private file from network to local file system
    pub async fn private_file_download(
        &self,
        data_access: PrivateDataAccess,
        to_dest: PathBuf,
    ) -> Result<(), FileDownloadError> {
        let data = self.private_data_get(data_access).await?;
        if let Some(parent) = to_dest.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(to_dest, data).await?;
        Ok(())
    }

    /// Download a private directory from network to local file system
    pub async fn private_dir_download(
        &self,
        archive_access: PrivateArchiveAccess,
        to_dest: PathBuf,
    ) -> Result<(), FileDownloadError> {
        let archive = self.private_archive_get(archive_access).await?;
        for (path, addr, _meta) in archive.iter() {
            self.private_file_download(addr.clone(), to_dest.join(path))
                .await?;
        }
        Ok(())
    }

    /// Upload a private directory to the network. The directory is recursively walked.
    /// Reads all files, splits into chunks, uploads chunks, uploads private archive, returns [`PrivateArchiveAccess`] (pointing to the private archive)
    pub async fn private_dir_upload(
        &self,
        dir_path: PathBuf,
        wallet: &EvmWallet,
    ) -> Result<PrivateArchiveAccess, FileUploadError> {
        let mut archive = PrivateArchive::new();

        for entry in walkdir::WalkDir::new(dir_path) {
            let entry = entry?;

            if !entry.file_type().is_file() {
                continue;
            }

            let path = entry.path().to_path_buf();
            tracing::info!("Uploading file: {path:?}");
            #[cfg(feature = "loud")]
            println!("Uploading file: {path:?}");
            let file = self.private_file_upload(path.clone(), wallet).await?;

            let metadata = super::fs::metadata_from_entry(&entry);

            archive.add_file(path, file, metadata);
        }

        let archive_serialized = archive.into_bytes()?;

        let arch_addr = self
            .private_data_put(archive_serialized, wallet.into())
            .await?;

        Ok(arch_addr)
    }

    /// Upload a private file to the network.
    /// Reads file, splits into chunks, uploads chunks, uploads datamap, returns [`PrivateDataAccess`] (pointing to the datamap)
    async fn private_file_upload(
        &self,
        path: PathBuf,
        wallet: &EvmWallet,
    ) -> Result<PrivateDataAccess, FileUploadError> {
        let data = tokio::fs::read(path).await?;
        let data = Bytes::from(data);
        let addr = self.private_data_put(data, wallet.into()).await?;
        Ok(addr)
    }
}
