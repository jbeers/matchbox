use std::fmt;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Debug, Clone, PartialEq)]
pub enum BxValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
    Array(Rc<RefCell<Vec<BxValue>>>),
    Struct(Rc<RefCell<HashMap<String, BxValue>>>),
    CompiledFunction(Rc<BxCompiledFunction>),
    Class(Rc<RefCell<BxClass>>),
    Instance(Rc<RefCell<BxInstance>>),
}

impl fmt::Display for BxValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BxValue::String(s) => write!(f, "{}", s),
            BxValue::Number(n) => write!(f, "{}", n),
            BxValue::Boolean(b) => write!(f, "{}", b),
            BxValue::Null => write!(f, "null"),
            BxValue::Array(arr) => {
                let items: Vec<String> = arr.borrow().iter().map(|v| v.to_string()).collect();
                write!(f, "[{}]", items.join(", "))
            }
            BxValue::Struct(s) => {
                let items: Vec<String> = s.borrow().iter().map(|(k, v)| {
                    // Prevent recursion if value is the same struct
                    if let BxValue::Struct(inner_s) = v {
                        if Rc::ptr_eq(s, inner_s) {
                            return format!("{}: <recursive struct>", k);
                        }
                    }
                    format!("{}: {}", k, v)
                }).collect();
                write!(f, "{{{}}}", items.join(", "))
            }
            BxValue::CompiledFunction(func) => write!(f, "<compiled function {}>", func.name),
            BxValue::Class(class) => write!(f, "<class {}>", class.borrow().name),
            BxValue::Instance(inst) => write!(f, "<instance of {}>", inst.borrow().class.borrow().name),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BxCompiledFunction {
    pub name: String,
    pub arity: usize,
    pub chunk: crate::vm::chunk::Chunk,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BxClass {
    pub name: String,
    pub constructor: crate::vm::chunk::Chunk,
    pub methods: HashMap<String, Rc<BxCompiledFunction>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BxInstance {
    pub class: Rc<RefCell<BxClass>>,
    pub this: Rc<RefCell<HashMap<String, BxValue>>>,
    pub variables: Rc<RefCell<HashMap<String, BxValue>>>,
}
