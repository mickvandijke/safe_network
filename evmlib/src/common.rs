use alloy::primitives::FixedBytes;

pub type Address = alloy::primitives::Address;
pub type Hash = FixedBytes<32>;
pub type TxHash = alloy::primitives::TxHash;
pub type U256 = alloy::primitives::U256;
pub type QuoteHash = Hash;
pub type Amount = U256;
pub type QuotePayment = (QuoteHash, Address, Amount);
pub type EthereumWallet = alloy::network::EthereumWallet;
pub type Calldata = alloy::primitives::Bytes;
