pub mod chunk;
pub mod opcode;

use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use anyhow::{Result, bail};
use crate::types::{BxValue, BxCompiledFunction, BxInstance};
use self::chunk::Chunk;
use self::opcode::OpCode;

struct CallFrame {
    function: Rc<BxCompiledFunction>,
    ip: usize,
    stack_base: usize,
    receiver: Option<Rc<RefCell<BxInstance>>>,
    handlers: Vec<usize>, // IP targets for catch blocks
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
            stack: Vec::with_capacity(256),
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
            receiver: None,
            handlers: Vec::new(),
        };
        
        self.frames.push(frame);
        
        self.run()
    }

    fn run(&mut self) -> Result<BxValue> {
        loop {
            let instruction = self.read_instruction().clone();
            
            match instruction {
                OpCode::OpReturn => {
                    let frame = self.frames.pop().unwrap();
                    let result = if self.stack.len() > frame.stack_base {
                        self.stack.pop().unwrap()
                    } else {
                        BxValue::Null
                    };
                    
                    if self.frames.is_empty() {
                        return Ok(result);
                    }
                    
                    self.stack.truncate(frame.stack_base);
                    
                    if frame.function.name.ends_with(".constructor") {
                        let instance = self.stack.pop().unwrap();
                        self.stack.push(instance);
                    } else {
                        self.stack.pop();
                        self.stack.push(result);
                    }
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
                        _ => { self.throw_error("Operands must be two numbers or two strings.")?; continue; },
                    }
                }
                OpCode::OpSubtract => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    if let (BxValue::Number(a), BxValue::Number(b)) = (a, b) {
                        self.stack.push(BxValue::Number(a - b));
                    } else {
                        self.throw_error("Operands must be two numbers.")?;
                        continue;
                    }
                }
                OpCode::OpMultiply => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    if let (BxValue::Number(a), BxValue::Number(b)) = (a, b) {
                        self.stack.push(BxValue::Number(a * b));
                    } else {
                        self.throw_error("Operands must be two numbers.")?;
                        continue;
                    }
                }
                OpCode::OpDivide => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    if let (BxValue::Number(a), BxValue::Number(b)) = (a, b) {
                        if b == 0.0 { self.throw_error("Division by zero")?; continue; }
                        else { self.stack.push(BxValue::Number(a / b)); }
                    } else {
                        self.throw_error("Operands must be two numbers.")?;
                        continue;
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
                    self.stack.push(BxValue::Array(Rc::new(RefCell::new(items))));
                }
                OpCode::OpStruct(count) => {
                    let mut items = HashMap::new();
                    for _ in 0..count {
                        let value = self.stack.pop().unwrap();
                        let key = self.stack.pop().unwrap().to_string().to_lowercase();
                        items.insert(key, value);
                    }
                    self.stack.push(BxValue::Struct(Rc::new(RefCell::new(items))));
                }
                OpCode::OpIndex => {
                    let index_val = self.stack.pop().unwrap();
                    let base_val = self.stack.pop().unwrap();
                    match base_val {
                        BxValue::Array(arr) => {
                            if let BxValue::Number(n) = index_val {
                                let idx = n as usize;
                                let arr = arr.borrow();
                                if idx < 1 || idx > arr.len() {
                                    self.throw_error(&format!("Array index out of bounds: {}", idx))?;
                                    continue;
                                } else {
                                    self.stack.push(arr[idx - 1].clone());
                                }
                            } else {
                                self.throw_error("Array index must be a number")?;
                                continue;
                            }
                        }
                        BxValue::Struct(s) => {
                            let key = index_val.to_string().to_lowercase();
                            self.stack.push(s.borrow().get(&key).cloned().unwrap_or(BxValue::Null));
                        }
                        _ => { self.throw_error("Invalid access: base must be array or struct")?; continue; }
                    }
                }
                OpCode::OpSetIndex => {
                    let val = self.stack.pop().unwrap();
                    let index_val = self.stack.pop().unwrap();
                    let base_val = self.stack.pop().unwrap();
                    
                    match base_val {
                        BxValue::Array(arr) => {
                            if let BxValue::Number(n) = index_val {
                                let idx = n as usize;
                                let mut arr = arr.borrow_mut();
                                if idx < 1 || idx > arr.len() {
                                    self.throw_error(&format!("Array index out of bounds: {}", idx))?;
                                    continue;
                                } else {
                                    arr[idx - 1] = val.clone();
                                    self.stack.push(val);
                                }
                            } else {
                                self.throw_error("Array index must be a number")?;
                                continue;
                            }
                        }
                        BxValue::Struct(s) => {
                            let key = index_val.to_string().to_lowercase();
                            s.borrow_mut().insert(key, val.clone());
                            self.stack.push(val);
                        }
                        BxValue::Instance(inst) => {
                            let key = index_val.to_string().to_lowercase();
                            inst.borrow().this.borrow_mut().insert(key, val.clone());
                            self.stack.push(val);
                        }
                        _ => { self.throw_error("Invalid indexed assignment")?; continue; }
                    }
                }
                OpCode::OpMember(idx) => {
                    let name = self.read_string_constant(idx).to_lowercase();
                    let base_val = self.stack.pop().unwrap();
                    match base_val {
                        BxValue::Struct(s) => {
                            self.stack.push(s.borrow().get(&name).cloned().unwrap_or(BxValue::Null));
                        }
                        BxValue::Instance(inst) => {
                            if let Some(val) = inst.borrow().this.borrow().get(&name) {
                                self.stack.push(val.clone());
                            } else {
                                if let Some(method) = inst.borrow().class.borrow().methods.get(&name) {
                                    self.stack.push(BxValue::CompiledFunction(Rc::clone(method)));
                                } else {
                                    self.stack.push(BxValue::Null);
                                }
                            }
                        }
                        _ => { self.throw_error("Member access only supported on structs and instances")?; continue; }
                    }
                }
                OpCode::OpSetMember(idx) => {
                    let name = self.read_string_constant(idx).to_lowercase();
                    let val = self.stack.pop().unwrap();
                    let base_val = self.stack.pop().unwrap();
                    
                    match base_val {
                        BxValue::Struct(s) => {
                            s.borrow_mut().insert(name, val.clone());
                            self.stack.push(val);
                        }
                        BxValue::Instance(inst) => {
                            inst.borrow().this.borrow_mut().insert(name, val.clone());
                            self.stack.push(val);
                        }
                        _ => { self.throw_error("Member assignment only supported on structs and instances")?; continue; }
                    }
                }
                OpCode::OpInvoke(idx, arg_count) => {
                    let name = self.read_string_constant(idx).to_lowercase();
                    let mut args = Vec::with_capacity(arg_count);
                    for _ in 0..arg_count {
                        args.push(self.stack.pop().unwrap());
                    }
                    args.reverse();
                    
                    let receiver_val = self.stack.pop().unwrap();
                    match receiver_val {
                        BxValue::Instance(inst) => {
                            let method = {
                                let inst_borrow = inst.borrow();
                                if let Some(BxValue::CompiledFunction(f)) = inst_borrow.this.borrow().get(&name) {
                                    Some(Rc::clone(f))
                                } else if let Some(f) = inst_borrow.class.borrow().methods.get(&name) {
                                    Some(Rc::clone(f))
                                } else {
                                    None
                                }
                            };
                            
                            if let Some(func) = method {
                                if arg_count != func.arity {
                                    self.throw_error(&format!("Expected {} arguments but got {}.", func.arity, arg_count))?;
                                    continue;
                                } else {
                                    for arg in args { self.stack.push(arg); }
                                    let frame = CallFrame {
                                        function: func,
                                        ip: 0,
                                        stack_base: self.stack.len() - arg_count,
                                        receiver: Some(inst),
                                        handlers: Vec::new(),
                                    };
                                    self.frames.push(frame);
                                }
                            } else {
                                self.throw_error(&format!("Method {} not found.", name))?;
                                continue;
                            }
                        }
                        BxValue::Struct(s) => {
                            if let Some(BxValue::CompiledFunction(func)) = s.borrow().get(&name) {
                                if arg_count != func.arity {
                                    self.throw_error(&format!("Expected {} arguments but got {}.", func.arity, arg_count))?;
                                    continue;
                                } else {
                                    for arg in args { self.stack.push(arg); }
                                    let frame = CallFrame {
                                        function: Rc::clone(func),
                                        ip: 0,
                                        stack_base: self.stack.len() - arg_count,
                                        receiver: None,
                                        handlers: Vec::new(),
                                    };
                                    self.frames.push(frame);
                                }
                            } else {
                                self.throw_error(&format!("Member {} not found or not callable.", name))?;
                                continue;
                            }
                        }
                        _ => { self.throw_error("Can only invoke methods on instances and structs.")?; continue; }
                    }
                }
                OpCode::OpCall(arg_count) => {
                    let func_val = self.stack[self.stack.len() - 1 - arg_count].clone();
                    match func_val {
                        BxValue::CompiledFunction(func) => {
                            if arg_count != func.arity {
                                self.throw_error(&format!("Expected {} arguments but got {}.", func.arity, arg_count))?;
                                continue;
                            } else {
                                let frame = CallFrame {
                                    function: Rc::clone(&func),
                                    ip: 0,
                                    stack_base: self.stack.len() - arg_count,
                                    receiver: self.frames.last().unwrap().receiver.clone(),
                                    handlers: Vec::new(),
                                };
                                self.frames.push(frame);
                            }
                        }
                        _ => { self.throw_error("Can only call functions.")?; continue; }
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
                        _ => { self.throw_error("Comparison only supported for numbers currently")?; continue; }
                    }
                }
                OpCode::OpLessEqual => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    match (a, b) {
                        (BxValue::Number(a), BxValue::Number(b)) => self.stack.push(BxValue::Boolean(a <= b)),
                        _ => { self.throw_error("Comparison only supported for numbers currently")?; continue; }
                    }
                }
                OpCode::OpGreater => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    match (a, b) {
                        (BxValue::Number(a), BxValue::Number(b)) => self.stack.push(BxValue::Boolean(a > b)),
                        _ => { self.throw_error("Comparison only supported for numbers currently")?; continue; }
                    }
                }
                OpCode::OpGreaterEqual => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    match (a, b) {
                        (BxValue::Number(a), BxValue::Number(b)) => self.stack.push(BxValue::Boolean(a >= b)),
                        _ => { self.throw_error("Comparison only supported for numbers currently")?; continue; }
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
                                let arr = arr.borrow();
                                if cursor_val < arr.len() {
                                    (false, Some(arr[cursor_val].clone()), Some(BxValue::Number(cursor_val as f64 + 1.0)))
                                } else {
                                    (true, None, None)
                                }
                            }
                            BxValue::Struct(s) => {
                                let s = s.borrow();
                                let mut keys: Vec<_> = s.keys().collect();
                                keys.sort();
                                if cursor_val < keys.len() {
                                    let key = keys[cursor_val];
                                    let val = s.get(key).unwrap();
                                    (false, Some(BxValue::String(key.clone())), Some(val.clone()))
                                } else {
                                    (true, None, None)
                                }
                            }
                            _ => { 
                                self.throw_error("Iteration only supported for arrays and structs")?;
                                (true, None, None) // Dummy return, IP already changed
                            }
                        }
                    };

                    if !self.frames.is_empty() { // Check if we re-entered loop because of dummy above? No, throw changes IP.
                        if is_done {
                            self.frames.last_mut().unwrap().ip += offset;
                        } else {
                            if let BxValue::Number(ref mut n) = self.stack[cursor_idx] {
                                *n += 1.0;
                            }
                            self.stack.push(next_val.unwrap());
                            if push_index {
                                self.stack.push(next_idx.unwrap());
                            }
                        }
                    }
                }
                OpCode::OpNew(arg_count) => {
                    let class_idx = self.stack.len() - 1 - arg_count;
                    let class_val = self.stack[class_idx].clone();
                    if let BxValue::Class(class) = class_val {
                        let this_scope = Rc::new(RefCell::new(HashMap::new()));
                        let variables_scope = Rc::new(RefCell::new(HashMap::new()));
                        
                        let instance = Rc::new(RefCell::new(BxInstance {
                            class: Rc::clone(&class),
                            this: this_scope.clone(),
                            variables: variables_scope.clone(),
                        }));
                        
                        self.stack[class_idx] = BxValue::Instance(Rc::clone(&instance));

                        let frame = CallFrame {
                            function: Rc::new(BxCompiledFunction {
                                name: format!("{}.constructor", class.borrow().name),
                                arity: 0,
                                chunk: class.borrow().constructor.clone(),
                            }),
                            ip: 0,
                            stack_base: class_idx + 1,
                            receiver: Some(Rc::clone(&instance)),
                            handlers: Vec::new(),
                        };
                        self.frames.push(frame);
                    } else {
                        self.throw_error("Can only instantiate classes.")?;
                        continue;
                    }
                }
                OpCode::OpGetPrivate(idx) => {
                    let name = self.read_string_constant(idx).to_lowercase();
                    if let Some(ref receiver) = self.frames.last().unwrap().receiver {
                        if name == "this" {
                            self.stack.push(BxValue::Instance(Rc::clone(receiver)));
                        } else if name == "variables" {
                            let vars = Rc::clone(&receiver.borrow().variables);
                            self.stack.push(BxValue::Struct(vars));
                        } else {
                            let val = receiver.borrow().variables.borrow().get(&name).cloned().unwrap_or(BxValue::Null);
                            self.stack.push(val);
                        }
                    } else {
                        self.throw_error("'variables' scope only available in classes.")?;
                        continue;
                    }
                }
                OpCode::OpSetPrivate(idx) => {
                    let name = self.read_string_constant(idx).to_lowercase();
                    let val = self.stack.last().unwrap().clone();
                    if let Some(ref receiver) = self.frames.last().unwrap().receiver {
                        receiver.borrow().variables.borrow_mut().insert(name, val);
                    } else {
                        self.throw_error("'variables' scope only available in classes.")?;
                        continue;
                    }
                }
                OpCode::OpPushHandler(offset) => {
                    let target_ip = self.frames.last().unwrap().ip + offset;
                    self.frames.last_mut().unwrap().handlers.push(target_ip);
                }
                OpCode::OpPopHandler => {
                    self.frames.last_mut().unwrap().handlers.pop();
                }
                OpCode::OpThrow => {
                    let val = self.stack.pop().unwrap();
                    self.throw_value(val)?;
                }
            }
        }
    }

    fn throw_error(&mut self, msg: &str) -> Result<()> {
        let val = BxValue::String(msg.to_string());
        self.throw_value(val)
    }

    fn throw_value(&mut self, val: BxValue) -> Result<()> {
        while !self.frames.is_empty() {
            let frame_idx = self.frames.len() - 1;
            if !self.frames[frame_idx].handlers.is_empty() {
                let handler_ip = self.frames[frame_idx].handlers.pop().unwrap();
                let stack_base = self.frames[frame_idx].stack_base;
                self.frames[frame_idx].ip = handler_ip;
                
                self.stack.truncate(stack_base);
                self.stack.push(val);
                return Ok(());
            }
            self.frames.pop();
        }
        bail!("Unhandled exception: {}", val);
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
