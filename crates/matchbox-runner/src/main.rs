use matchbox_vm::{Chunk, vm::VM};
use anyhow::{Result, bail};
use std::env as std_env;
use std::fs;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

const MAGIC_FOOTER: &[u8; 8] = b"BOXLANG\x01";

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<()> {
    let chunk = load_embedded_bytecode()?;
    let mut vm = VM::new();
    vm.interpret(chunk)?;
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn main() {}

#[cfg(not(target_arch = "wasm32"))]
fn load_embedded_bytecode() -> Result<Chunk> {
    let self_path = std_env::current_exe().map_err(|_| anyhow::anyhow!("Not an executable"))?;
    let bytes = fs::read(self_path)?;
    if bytes.len() < 16 { bail!("Too small"); }
    let footer_start = bytes.len() - 8;
    if &bytes[footer_start..] != MAGIC_FOOTER { bail!("No embedded bytecode found in this runner stub."); }
    let len_start = bytes.len() - 16;
    let mut len_bytes = [0u8; 8];
    len_bytes.copy_from_slice(&bytes[len_start..footer_start]);
    let len = u64::from_le_bytes(len_bytes) as usize;
    let chunk_start = len_start - len;
    let chunk_bytes = &bytes[chunk_start..len_start];
    let chunk: Chunk = bincode::deserialize(chunk_bytes)?;
    Ok(chunk)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct BoxLangVM {
    vm: VM,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl BoxLangVM {
    #[wasm_bindgen(constructor)]
    pub fn new() -> BoxLangVM {
        BoxLangVM { vm: VM::new() }
    }

    pub fn load_bytecode(&mut self, bytes: &[u8]) -> Result<(), String> {
        let chunk: Chunk = bincode::deserialize(bytes).map_err(|e| format!("Error: {}", e))?;
        self.vm.interpret(chunk).map_err(|e| format!("Error: {}", e))?;
        Ok(())
    }

    pub fn call(&mut self, name: &str, args: js_sys::Array) -> Result<JsValue, String> {
        let mut bx_args = Vec::new();
        for i in 0..args.length() {
            bx_args.push(self.vm.js_to_bx(args.get(i)));
        }

        let func = self.vm.get_global(name)
            .ok_or_else(|| format!("Function {} not found", name))?;

        match self.vm.call_function_value(func, bx_args) {
            Ok(val) => Ok(self.vm.bx_to_js(&val)),
            Err(e) => Err(format!("Error: {}", e)),
        }
    }
}
