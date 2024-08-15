// Copyright 2024 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::error::{Result, TransferError};
use crate::MainSecretKey;
use bls::SecretKey;
use curv::elliptic::curves::ECScalar;
use rand::RngCore;

pub fn random_eip2333_mnemonic() -> Result<bip39::Mnemonic> {
    let mut entropy = [1u8; 32];
    let rng = &mut rand::rngs::OsRng;

    rng.fill_bytes(&mut entropy);

    let mnemonic = bip39::Mnemonic::from_entropy(&entropy)
        .map_err(|_error| TransferError::FailedToParseEntropy)?;

    Ok(mnemonic)
}

/// Derive a root wallet secret key from the mnemonic for the account.
pub fn root_sk_from_mnemonic(mnemonic: bip39::Mnemonic, passphrase: &str) -> Result<MainSecretKey> {
    let seed = mnemonic.to_seed(passphrase);
    let root_sk = eip2333::derive_master_sk(&seed)
        .map_err(|_err| TransferError::InvalidMnemonicSeedPhrase)?;
    let key_bytes = root_sk.serialize();
    let sk =
        SecretKey::from_bytes(key_bytes.into()).map_err(|_err| TransferError::InvalidKeyBytes)?;
    Ok(MainSecretKey::new(sk))
}
