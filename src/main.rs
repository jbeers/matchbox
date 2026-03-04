mod ast;
mod bifs;
mod parser;
mod types;
mod vm;
mod compiler;

use std::env as std_env;
use std::fs;
use anyhow::{Result, bail};

fn main() -> Result<()> {
    let args: Vec<String> = std_env::args().collect();
    if args.len() < 2 {
        bail!("Usage: bx-rust <file.bxs>");
    }

    let filename = args.iter().skip(1).find(|a| !a.starts_with("--")).unwrap_or(&args[1]);
    let source = fs::read_to_string(filename)?;

    match parser::parse(&source) {
        Ok(ast) => {
            let compiler = compiler::Compiler::new();
            match compiler.compile(&ast) {
                Ok(chunk) => {
                    let mut vm = vm::VM::new();
                    
                    // Register BIFs
                    for (name, val) in bifs::register_all() {
                        vm.globals.insert(name, val);
                    }

                    match vm.interpret(chunk) {
                        Ok(_) => {}
                        Err(e) => eprintln!("VM Runtime Error: {}", e),
                    }
                }
                Err(e) => eprintln!("Compiler Error: {}", e),
            }
        }
        Err(e) => {
            eprintln!("Parse Error: {}", e);
        }
    }

    Ok(())
}
