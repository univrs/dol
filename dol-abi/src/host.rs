//! Host interface for WASM modules

use crate::message::{Message, Response};
use wasm_bindgen::prelude::*;

/// Host interface trait for WASM modules
///
/// This trait defines the contract between a DOL WASM module and its host runtime.
pub trait HostInterface {
    /// Send a message to the host
    fn send_message(&self, message: &Message) -> crate::Result<Response>;

    /// Initialize the module
    fn init(&mut self) -> crate::Result<()>;

    /// Shutdown the module
    fn shutdown(&mut self) -> crate::Result<()>;
}

/// JavaScript FFI for host calls
#[wasm_bindgen]
extern "C" {
    /// Send a message to the host and get a response
    #[wasm_bindgen(js_namespace = vudo, js_name = sendMessage)]
    pub fn send_message(msg: &str) -> String;

    /// Initialize the host interface
    #[wasm_bindgen(js_namespace = vudo, js_name = initialize)]
    pub fn initialize() -> JsValue;

    /// Get the host version
    #[wasm_bindgen(js_namespace = vudo, js_name = getVersion)]
    pub fn get_version() -> String;
}
