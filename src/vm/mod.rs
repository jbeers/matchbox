pub mod chunk;
pub mod opcode;

use std::collections::HashMap;
use std::rc::Rc;
use anyhow::{Result, bail};
use crate::types::{BxValue, BxCompiledFunction};
use self::chunk::Chunk;
use self::opcode::OpCode;

struct CallFrame {
    function: Rc<BxCompiledFunction>,
    ip: usize,
    stack_base: usize,
}

pub struct VM {
    frames: Vec<CallFrame>,
    stack: Vec<BxValue>,
    globals: HashMap<String, BxValue>,
}

impl VM {
    pub fn new() -> Self {
        VM {
            frames: Vec::with_capacity(64),
            stack: Vec::with_capacity(256), // Pre-allocate some stack space
            globals: HashMap::new(),
        }
    }

    pub fn interpret(&mut self, chunk: Chunk) -> Result<BxValue> {
        let function = Rc::new(BxCompiledFunction {
            name: "script".to_string(),
            arity: 0,
            chunk,
        });
        
        let frame = CallFrame {
            function,
            ip: 0,
            stack_base: 0,
        };
        
        self.frames.push(frame);
        
        self.run()
    }

    fn run(&mut self) -> Result<BxValue> {
        loop {
            let instruction = self.read_instruction().clone();
            
            match instruction {
                OpCode::OpIterNext(collection_slot, cursor_slot, offset, push_index) => {
                    let base = self.frames.last().unwrap().stack_base;
                    let collection_idx = base + collection_slot;
                    let cursor_idx = base + cursor_slot;
                    
                    let (is_done, next_val, next_idx) = {
                        let cursor_val = match &self.stack[cursor_idx] {
                            BxValue::Number(n) => *n as usize,
                            _ => bail!("Internal VM error: iterator cursor is not a number"),
                        };
                        
                        match &self.stack[collection_idx] {
                            BxValue::Array(arr) => {
                                if cursor_val < arr.len() {
                                    (false, Some(arr[cursor_val].clone()), Some(BxValue::Number(cursor_val as f64 + 1.0)))
                                } else {
                                    (true, None, None)
                                }
                            }
                            BxValue::Struct(s) => {
                                let mut keys: Vec<_> = s.keys().collect();
                                keys.sort(); // Deterministic order for iteration
                                if cursor_val < keys.len() {
                                    let key = keys[cursor_val];
                                    let val = s.get(key).unwrap();
                                    // Item is the key, index is the value in struct iteration for BoxLang parity
                                    (false, Some(BxValue::String(key.clone())), Some(val.clone()))
                                } else {
                                    (true, None, None)
                                }
                            }
                            _ => bail!("Iteration only supported for arrays and structs"),
                        }
                    };

                    if is_done {
                        self.frames.last_mut().unwrap().ip += offset;
                    } else {
                        // Update cursor in place
                        if let BxValue::Number(ref mut n) = self.stack[cursor_idx] {
                            *n += 1.0;
                        }
                        
                        let next_idx_val = next_idx.unwrap();
                        let next_val_actual = next_val.unwrap();

                        // Push item (value)
                        self.stack.push(next_val_actual);
                        // Push index (key) if requested
                        if push_index {
                            self.stack.push(next_idx_val);
                        }
                    }
                }
                OpCode::OpReturn => {
                    let result = self.stack.pop().unwrap_or(BxValue::Null);
                    let frame = self.frames.pop().unwrap();
                    
                    if self.frames.is_empty() {
                        return Ok(result);
                    }
                    
                    // Truncate stack to remove arguments and the function object
                    self.stack.truncate(frame.stack_base - 1);
                    self.stack.push(result);
                }
                OpCode::OpConstant(idx) => {
                    let constant = self.read_constant(idx);
                    self.stack.push(constant);
                }
                OpCode::OpAdd => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    match (a, b) {
                        (BxValue::Number(a), BxValue::Number(b)) => self.stack.push(BxValue::Number(a + b)),
                        (BxValue::String(a), BxValue::String(b)) => self.stack.push(BxValue::String(format!("{}{}", a, b))),
                        _ => bail!("Operands must be two numbers or two strings."),
                    }
                }
                OpCode::OpSubtract => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    if let (BxValue::Number(a), BxValue::Number(b)) = (a, b) {
                        self.stack.push(BxValue::Number(a - b));
                    } else {
                        bail!("Operands must be two numbers.");
                    }
                }
                OpCode::OpMultiply => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    if let (BxValue::Number(a), BxValue::Number(b)) = (a, b) {
                        self.stack.push(BxValue::Number(a * b));
                    } else {
                        bail!("Operands must be two numbers.");
                    }
                }
                OpCode::OpDivide => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    if let (BxValue::Number(a), BxValue::Number(b)) = (a, b) {
                        if b == 0.0 { bail!("Division by zero"); }
                        self.stack.push(BxValue::Number(a / b));
                    } else {
                        bail!("Operands must be two numbers.");
                    }
                }
                OpCode::OpStringConcat => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(BxValue::String(format!("{}{}", a, b)));
                }
                OpCode::OpPrint(count) => {
                    let mut args = Vec::with_capacity(count);
                    for _ in 0..count {
                        args.push(self.stack.pop().unwrap());
                    }
                    args.reverse();
                    let out = args.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(" ");
                    print!("{}", out);
                }
                OpCode::OpPrintln(count) => {
                    let mut args = Vec::with_capacity(count);
                    for _ in 0..count {
                        args.push(self.stack.pop().unwrap());
                    }
                    args.reverse();
                    let out = args.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(" ");
                    println!("{}", out);
                }
                OpCode::OpPop => {
                    self.stack.pop();
                }
                OpCode::OpDefineGlobal(idx) => {
                    let name = self.read_string_constant(idx);
                    let val = self.stack.pop().unwrap();
                    self.globals.insert(name.to_lowercase(), val);
                }
                OpCode::OpGetGlobal(idx) => {
                    let name = self.read_string_constant(idx);
                    if let Some(val) = self.globals.get(&name.to_lowercase()) {
                        self.stack.push(val.clone());
                    } else {
                        self.stack.push(BxValue::Null); 
                    }
                }
                OpCode::OpSetGlobal(idx) => {
                    let name = self.read_string_constant(idx);
                    let val = self.stack.last().unwrap().clone();
                    self.globals.insert(name.to_lowercase(), val);
                }
                OpCode::OpGetLocal(slot) => {
                    let base = self.frames.last().unwrap().stack_base;
                    let val = self.stack[base + slot].clone();
                    self.stack.push(val);
                }
                OpCode::OpSetLocal(slot) => {
                    let base = self.frames.last().unwrap().stack_base;
                    let val = self.stack.last().unwrap().clone();
                    self.stack[base + slot] = val;
                }
                OpCode::OpArray(count) => {
                    let mut items = Vec::with_capacity(count);
                    for _ in 0..count {
                        items.push(self.stack.pop().unwrap());
                    }
                    items.reverse();
                    self.stack.push(BxValue::Array(items));
                }
                OpCode::OpStruct(count) => {
                    let mut items = HashMap::new();
                    for _ in 0..count {
                        let value = self.stack.pop().unwrap();
                        let key = self.stack.pop().unwrap().to_string().to_lowercase();
                        items.insert(key, value);
                    }
                    self.stack.push(BxValue::Struct(items));
                }
                OpCode::OpIndex => {
                    let index_val = self.stack.pop().unwrap();
                    let base_val = self.stack.pop().unwrap();
                    match (base_val, index_val) {
                        (BxValue::Array(arr), BxValue::Number(n)) => {
                            let idx = n as usize;
                            if idx < 1 || idx > arr.len() {
                                bail!("Array index out of bounds: {}", idx);
                            }
                            self.stack.push(arr[idx - 1].clone());
                        }
                        (BxValue::Struct(s), key_val) => {
                            let key = key_val.to_string().to_lowercase();
                            self.stack.push(s.get(&key).cloned().unwrap_or(BxValue::Null));
                        }
                        _ => bail!("Invalid access: base must be array or struct"),
                    }
                }
                OpCode::OpMember(idx) => {
                    let name = self.read_string_constant(idx).to_lowercase();
                    let base_val = self.stack.pop().unwrap();
                    match base_val {
                        BxValue::Struct(s) => {
                            self.stack.push(s.get(&name).cloned().unwrap_or(BxValue::Null));
                        }
                        _ => bail!("Member access only supported on structs"),
                    }
                }
                OpCode::OpCall(arg_count) => {
                    let func_val = self.stack[self.stack.len() - 1 - arg_count].clone();
                    match func_val {
                        BxValue::CompiledFunction(func) => {
                            if arg_count != func.arity {
                                bail!("Expected {} arguments but got {}.", func.arity, arg_count);
                            }
                            let frame = CallFrame {
                                function: Rc::clone(&func),
                                ip: 0,
                                stack_base: self.stack.len() - arg_count,
                            };
                            self.frames.push(frame);
                        }
                        _ => bail!("Can only call functions."),
                    }
                }
                OpCode::OpEqual => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(BxValue::Boolean(a == b));
                }
                OpCode::OpNotEqual => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(BxValue::Boolean(a != b));
                }
                OpCode::OpLess => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    match (a, b) {
                        (BxValue::Number(a), BxValue::Number(b)) => self.stack.push(BxValue::Boolean(a < b)),
                        _ => bail!("Comparison only supported for numbers currently"),
                    }
                }
                OpCode::OpLessEqual => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    match (a, b) {
                        (BxValue::Number(a), BxValue::Number(b)) => self.stack.push(BxValue::Boolean(a <= b)),
                        _ => bail!("Comparison only supported for numbers currently"),
                    }
                }
                OpCode::OpGreater => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    match (a, b) {
                        (BxValue::Number(a), BxValue::Number(b)) => self.stack.push(BxValue::Boolean(a > b)),
                        _ => bail!("Comparison only supported for numbers currently"),
                    }
                }
                OpCode::OpGreaterEqual => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    match (a, b) {
                        (BxValue::Number(a), BxValue::Number(b)) => self.stack.push(BxValue::Boolean(a >= b)),
                        _ => bail!("Comparison only supported for numbers currently"),
                    }
                }
                OpCode::OpJump(offset) => {
                    self.frames.last_mut().unwrap().ip += offset;
                }
                OpCode::OpJumpIfFalse(offset) => {
                    if !self.is_truthy(self.stack.last().unwrap()) {
                        self.frames.last_mut().unwrap().ip += offset;
                    }
                }
                OpCode::OpLoop(offset) => {
                    self.frames.last_mut().unwrap().ip -= offset;
                }
            }
        }
    }

    fn is_truthy(&self, val: &BxValue) -> bool {
        match val {
            BxValue::Boolean(b) => *b,
            BxValue::Null => false,
            BxValue::Number(n) => *n != 0.0,
            BxValue::String(s) => !s.is_empty() && s.to_lowercase() != "false",
            _ => true,
        }
    }

    fn read_instruction(&mut self) -> &OpCode {
        let frame = self.frames.last_mut().unwrap();
        let op = &frame.function.chunk.code[frame.ip];
        frame.ip += 1;
        op
    }

    fn read_constant(&self, idx: usize) -> BxValue {
        let frame = self.frames.last().unwrap();
        frame.function.chunk.constants[idx].clone()
    }

    fn read_string_constant(&self, idx: usize) -> String {
        let frame = self.frames.last().unwrap();
        match &frame.function.chunk.constants[idx] {
            BxValue::String(s) => s.clone(),
            _ => panic!("Constant at index {} is not a string", idx),
        }
    }
}

