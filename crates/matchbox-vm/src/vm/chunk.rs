use super::opcode::op;
use crate::types::Constant;
use crate::types::box_string::BoxString;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[cfg(not(target_os = "espidf"))]
pub type RuntimeCaches = Vec<Option<IcEntry>>;

#[cfg(target_os = "espidf")]
pub type RuntimeCaches = Vec<(usize, IcEntry)>;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum ConstantKey {
    Number(u64),
    String(BoxString),
    Boolean(bool),
    Null,
    StringArray(Vec<String>),
    Function(String),
}

impl ConstantKey {
    fn from_constant(c: &Constant) -> Option<Self> {
        match c {
            Constant::Number(f) => Some(ConstantKey::Number(f.to_bits())),
            Constant::String(s) => Some(ConstantKey::String(s.clone())),
            Constant::Boolean(b) => Some(ConstantKey::Boolean(*b)),
            Constant::Null => Some(ConstantKey::Null),
            Constant::StringArray(v) => Some(ConstantKey::StringArray(v.clone())),
            Constant::CompiledFunction(f) => Some(ConstantKey::Function(f.name.clone())),
            Constant::Class(_) | Constant::Interface(_) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Chunk {
    pub code: Vec<u32>,
    pub constants: Vec<Constant>,
    pub lines: Vec<u32>,
    pub filename: String,
    pub source: String,
    #[serde(skip)]
    pub caches: RuntimeCaches,
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
    Polymorphic {
        entries: [(usize, usize); 4],
        count: usize,
    },
    Megamorphic,
    Global {
        index: usize,
    },
}

impl Chunk {
    #[inline]
    fn emit_cache_slot(&mut self) {
        #[cfg(not(target_os = "espidf"))]
        self.caches.push(None);
    }

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

    /// Create a new sub-chunk.
    pub fn new_sub_chunk(&self) -> Self {
        Chunk {
            code: Vec::new(),
            constants: Vec::new(),
            lines: Vec::new(),
            filename: self.filename.clone(),
            source: String::new(),
            caches: Vec::new(),
            constant_map: HashMap::new(),
        }
    }

    /// Emit a zero-operand instruction (1 word).
    #[inline]
    pub fn emit0(&mut self, opcode: u8, line: u32) {
        self.code.push(opcode as u32);
        self.lines.push(line);
        self.emit_cache_slot();
    }

    /// Emit a single-operand instruction (1 word).
    /// Operand `a` occupies bits [31:8]; must fit in 24 bits for correct decoding.
    #[inline]
    pub fn emit1(&mut self, opcode: u8, a: u32, line: u32) {
        self.code.push((opcode as u32) | (a << 8));
        self.lines.push(line);
        self.emit_cache_slot();
    }

    /// Emit a two-operand instruction (2 words).
    /// `a` in word0 bits [31:8], `b` in word1 (full 32 bits).
    #[inline]
    pub fn emit2(&mut self, opcode: u8, a: u32, b: u32, line: u32) {
        self.code.push((opcode as u32) | (a << 8));
        self.lines.push(line);
        self.emit_cache_slot();
        self.code.push(b);
        self.lines.push(line);
        self.emit_cache_slot();
    }

    /// Emit a three-operand instruction (3 words).
    /// `a` in word0 bits [31:8], `b` in word1, `c` in word2 (each full 32 bits except a which is 24-bit).
    #[inline]
    pub fn emit3(&mut self, opcode: u8, a: u32, b: u32, c: u32, line: u32) {
        self.code.push((opcode as u32) | (a << 8));
        self.lines.push(line);
        self.emit_cache_slot();
        self.code.push(b);
        self.lines.push(line);
        self.emit_cache_slot();
        self.code.push(c);
        self.lines.push(line);
        self.emit_cache_slot();
    }

    /// Emit ITER_NEXT: 3 words.
    /// word0 = ITER_NEXT | (collection_slot << 8)
    /// word1 = cursor_slot | (has_index << 31)
    /// word2 = exit_offset (placeholder 0, back-patched later)
    #[inline]
    pub fn emit_iter_next(&mut self, collection: u32, cursor: u32, has_index: bool, line: u32) {
        let word1 = cursor | if has_index { 0x8000_0000u32 } else { 0 };
        self.emit3(op::ITER_NEXT, collection, word1, 0, line);
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
        #[cfg(not(target_os = "espidf"))]
        if self.caches.len() < self.code.len() {
            self.caches.resize(self.code.len(), None);
        }
    }

    pub fn cache_get(&self, ip: usize) -> Option<IcEntry> {
        #[cfg(not(target_os = "espidf"))]
        {
            self.caches.get(ip).and_then(|entry| entry.clone())
        }

        #[cfg(target_os = "espidf")]
        {
            self.caches
                .iter()
                .find_map(|(cached_ip, entry)| (*cached_ip == ip).then(|| entry.clone()))
        }
    }

    pub fn cache_set(&mut self, ip: usize, entry: IcEntry) {
        #[cfg(not(target_os = "espidf"))]
        {
            if self.caches.len() <= ip {
                self.caches.resize(ip + 1, None);
            }
            self.caches[ip] = Some(entry);
        }

        #[cfg(target_os = "espidf")]
        {
            if let Some((_, existing)) = self
                .caches
                .iter_mut()
                .find(|(cached_ip, _)| *cached_ip == ip)
            {
                *existing = entry;
            } else {
                self.caches.push((ip, entry));
            }
        }
    }

    pub fn cache_add_shape(&mut self, ip: usize, shape_id: usize, index: usize) {
        match self.cache_get(ip) {
            None => self.cache_set(ip, IcEntry::Monomorphic { shape_id, index }),
            Some(IcEntry::Monomorphic {
                shape_id: cached_shape,
                index: cached_index,
            }) => {
                if cached_shape == shape_id {
                    return;
                }
                let mut entries = [(0, 0); 4];
                entries[0] = (cached_shape, cached_index);
                entries[1] = (shape_id, index);
                self.cache_set(ip, IcEntry::Polymorphic { entries, count: 2 });
            }
            Some(IcEntry::Polymorphic { mut entries, count }) => {
                if entries[..count]
                    .iter()
                    .any(|(cached_shape, _)| *cached_shape == shape_id)
                {
                    return;
                }
                if count < entries.len() {
                    entries[count] = (shape_id, index);
                    self.cache_set(
                        ip,
                        IcEntry::Polymorphic {
                            entries,
                            count: count + 1,
                        },
                    );
                } else {
                    self.cache_set(ip, IcEntry::Megamorphic);
                }
            }
            Some(IcEntry::Megamorphic | IcEntry::Global { .. }) => {}
        }
    }

    pub fn cache_slice(&self, start: usize, len: usize) -> Vec<Option<IcEntry>> {
        (start..start + len).map(|ip| self.cache_get(ip)).collect()
    }

    pub fn clone_without_runtime_caches(&self) -> Self {
        Chunk {
            code: self.code.clone(),
            constants: self.constants.clone(),
            lines: self.lines.clone(),
            filename: self.filename.clone(),
            source: self.source.clone(),
            caches: Vec::new(),
            constant_map: HashMap::new(),
        }
    }

    pub fn reconstruct_functions(&mut self) {
        // NO-OP in the flat model.
    }
}
