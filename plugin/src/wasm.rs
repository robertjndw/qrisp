use wasm_minimal_protocol::*;

initiate_protocol!();

#[wasm_func]
pub fn generate(data: &[u8], opts: &[u8]) -> Result<Vec<u8>, String> {
    crate::generate(data, opts)
}

#[wasm_func]
pub fn matrix(data: &[u8], opts: &[u8]) -> Result<Vec<u8>, String> {
    crate::matrix(data, opts)
}
