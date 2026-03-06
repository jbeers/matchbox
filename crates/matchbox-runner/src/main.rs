use matchbox_vm::{Chunk, vm::VM};
use anyhow::{Result, bail};
use std::env as std_env;
use std::fs;

const MAGIC_FOOTER: &[u8; 8] = b"BOXLANG\x01";

fn main() -> Result<()> {
    let chunk = load_embedded_bytecode()?;
    let mut vm = VM::new();
    vm.interpret(chunk)?;
    Ok(())
}

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
