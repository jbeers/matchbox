extern crate self as matchbox_vm;

pub mod bifs;
#[cfg(all(feature = "qoq", not(target_arch = "wasm32")))]
pub mod qoq;
pub mod types;
pub mod vm;

#[cfg(not(target_arch = "wasm32"))]
pub mod datasource;

pub use matchbox_macros::*;
pub use vm::chunk::Chunk;
