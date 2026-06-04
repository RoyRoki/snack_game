pub mod game;
pub mod storage;
pub mod types;

#[cfg(not(target_arch = "wasm32"))]
pub mod render;
#[cfg(not(target_arch = "wasm32"))]
pub mod menu;

#[cfg(feature = "wasm")]
mod wasm_api;
#[cfg(feature = "wasm")]
pub use wasm_api::WasmGame;
