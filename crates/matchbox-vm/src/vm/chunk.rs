use std::collections::HashMap;
use crate::types::Constant;
use crate::types::box_string::BoxString;
use super::opcode::OpCode;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum ConstantKey {
    Number(u64),
    String(BoxString),
    Boolean(bool),
    Null,
    StringArray(Vec<String>),
}

impl ConstantKey {
    fn from_constant(c: &Constant) -> Option<Self> {
        match c {
            Constant::Number(f) => Some(ConstantKey::Number(f.to_bits())),
            Constant::String(s) => Some(ConstantKey::String(s.clone())),
            Constant::Boolean(b) => Some(ConstantKey::Boolean(*b)),
            Constant::Null => Some(ConstantKey::Null),
            Constant::StringArray(v) => Some(ConstantKey::StringArray(v.clone())),
            Constant::CompiledFunction(_) | Constant::Class(_) | Constant::Interface(_) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Chunk {
    pub code: Vec<OpCode>,
    pub constants: Vec<Constant>,
    pub lines: Vec<u32>,
    pub filename: String,
    pub source: String,
    #[serde(skip)]
    pub caches: Vec<Option<IcEntry>>,
    #[serde(skip)]
    constant_map: HashMap<ConstantKey, u32>,
}

impl Default for Chunk {
    fn default() -> Self {
        Chunk {
            code: Vec::new(),
            constants: Vec::new(),
            lines: Vec::new(),
            filename: String::new(),
            source: String::new(),
            caches: Vec::new(),
            constant_map: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum IcEntry {
    Monomorphic {
        shape_id: usize,
        index: usize,
    },
    Global {
        index: usize,
    },
    // We can add Polymorphic here later
}

impl Chunk {
    pub fn new(filename: &str) -> Self {
        Chunk {
            code: Vec::new(),
            constants: Vec::new(),
            lines: Vec::new(),
            filename: filename.to_string(),
            source: String::new(),
            caches: Vec::new(),
            constant_map: HashMap::new(),
        }
    }

    pub fn write(&mut self, opcode: OpCode, line: u32) {
        self.code.push(opcode);
        self.lines.push(line);
        self.caches.push(None);
    }

    pub fn add_constant(&mut self, value: Constant) -> u32 {
        if let Some(key) = ConstantKey::from_constant(&value) {
            if let Some(&idx) = self.constant_map.get(&key) {
                return idx;
            }
            let idx = self.constants.len() as u32;
            self.constants.push(value);
            self.constant_map.insert(key, idx);
            idx
        } else {
            self.constants.push(value);
            (self.constants.len() - 1) as u32
        }
    }

    pub fn ensure_caches(&mut self) {
        if self.caches.len() < self.code.len() {
            self.caches.resize(self.code.len(), None);
        }
    }
}
