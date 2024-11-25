#[cfg_attr(target_arch = "wasm32", path = "wasm32.rs")]
#[cfg_attr(not(target_arch = "wasm32"), path = "other.rs")]
pub(crate) mod mod_impl;
mod wasm32;

pub(crate) use mod_impl::build_transport;
