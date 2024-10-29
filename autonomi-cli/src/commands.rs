// Copyright 2024 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod file;
mod register;
mod vault;
mod wallet;

use clap::Subcommand;
use color_eyre::Result;

use crate::opt::Opt;

#[derive(Subcommand, Debug)]
pub enum SubCmd {
    /// Operations related to file handling.
    File {
        #[command(subcommand)]
        command: FileCmd,
    },

    /// Operations related to register management.
    Register {
        #[command(subcommand)]
        command: RegisterCmd,
    },

    /// Operations related to vault management.
    Vault {
        #[command(subcommand)]
        command: VaultCmd,
    },

    /// Operations related to wallet management.
    Wallet {
        #[command(subcommand)]
        command: WalletCmd,
    }
}

#[derive(Subcommand, Debug)]
pub enum FileCmd {
    /// Estimate cost to upload a file.
    Cost {
        /// The file to estimate cost for.
        file: String,
    },

    /// Upload a file and pay for it. Data on the Network is private by default.
    Upload {
        /// The file to upload.
        file: String,
        /// Upload the file as public. Everyone can see public data on the Network.
        #[arg(short, long)]
        public: bool,
    },

    /// Download a file from the given address.
    Download {
        /// The address of the file to download.
        addr: String,
        /// The destination file path.
        dest_file: String,
    },

    /// List previous uploads
    List,
}

#[derive(Subcommand, Debug)]
pub enum RegisterCmd {
    /// Generate a new register key.
    GenerateKey {
        /// Overwrite existing key if it exists
        /// Warning: overwriting the existing key will result in loss of access to any existing registers created using that key
        #[arg(short, long)]
        overwrite: bool,
    },

    /// Estimate cost to register a name.
    Cost {
        /// The name to register.
        name: String,
    },

    /// Create a new register with the given name and value.
    Create {
        /// The name of the register.
        name: String,
        /// The value to store in the register.
        value: String,
        /// Create the register with public write access.
        #[arg(long, default_value = "false")]
        public: bool,
    },

    /// Edit an existing register.
    Edit {
        /// Use the name of the register instead of the address
        /// Note that only the owner of the register can use this shorthand as the address can be generated from the name and register key.
        #[arg(short, long)]
        name: bool,
        /// The address of the register
        /// With the name option on the address will be used as a name
        address: String,
        /// The new value to store in the register.
        value: String,
    },

    /// Get the value of a register.
    Get {
        /// Use the name of the register instead of the address
        /// Note that only the owner of the register can use this shorthand as the address can be generated from the name and register key.
        #[arg(short, long)]
        name: bool,
        /// The address of the register
        /// With the name option on the address will be used as a name
        address: String,
    },

    /// List previous registers
    List,
}

#[derive(Subcommand, Debug)]
pub enum VaultCmd {
    /// Estimate cost to create a vault.
    Cost,

    /// Create a vault at a deterministic address based on your `SECRET_KEY`.
    /// Pushing an encrypted backup of your local user data to the network
    Create,

    /// Load an existing vault from the network.
    /// Use this when loading your user data to a new device.
    /// You need to have your original `SECRET_KEY` to load the vault.
    Load,

    /// Sync vault with the network, including registers and files.
    /// Loads existing user data from the network and merges it with your local user data.
    /// Pushes your local user data to the network.
    Sync {
        /// Force push your local user data to the network.
        /// This will overwrite any existing data in your vault.
        #[arg(short, long)]
        force: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum WalletCmd {
    /// Create a wallet
    // #[command(subcommand)]
    Create {
        #[arg(long)]
        encrypt: Option<String>,
        #[arg(long)]
        password: Option<String>,
        #[arg(long)]
        private_key: Option<String>,

    },

    /// Check the balance of the wallet
    Balance,
}

pub async fn handle_subcommand(opt: Opt) -> Result<()> {
    let peers = crate::access::network::get_peers(opt.peers);
    let cmd = opt.command;

    match cmd {
        SubCmd::File { command } => match command {
            FileCmd::Cost { file } => file::cost(&file, peers.await?).await,
            FileCmd::Upload { file, public } => file::upload(&file, public, peers.await?).await,
            FileCmd::Download { addr, dest_file } => {
                file::download(&addr, &dest_file, peers.await?).await
            }
            FileCmd::List => file::list(),
        },
        SubCmd::Register { command } => match command {
            RegisterCmd::GenerateKey { overwrite } => register::generate_key(overwrite),
            RegisterCmd::Cost { name } => register::cost(&name, peers.await?).await,
            RegisterCmd::Create {
                name,
                value,
                public,
            } => register::create(&name, &value, public, peers.await?).await,
            RegisterCmd::Edit {
                address,
                name,
                value,
            } => register::edit(address, name, &value, peers.await?).await,
            RegisterCmd::Get { address, name } => register::get(address, name, peers.await?).await,
            RegisterCmd::List => register::list(),
        },
        SubCmd::Vault { command } => match command {
            VaultCmd::Cost => vault::cost(peers.await?).await,
            VaultCmd::Create => vault::create(peers.await?).await,
            VaultCmd::Load => vault::load(peers.await?).await,
            VaultCmd::Sync { force } => vault::sync(peers.await?, force).await,
        },
        SubCmd::Wallet { command } => match command {
            WalletCmd::Create {
                encrypt,
                password,
                private_key,

            } => {
                wallet::create( encrypt, password,private_key)
            },
            WalletCmd::Balance => Ok(wallet::balance()?),
        }
    }
}
