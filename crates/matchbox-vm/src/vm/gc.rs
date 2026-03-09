use crate::types::{BxValue, BxStruct, BxInstance, BxFuture, BxCompiledFunction, BxClass, BxInterface, BxNativeFunction, BxNativeObject, box_string::BoxString};
use std::rc::Rc;
use std::cell::RefCell;

pub type GcId = usize;

#[derive(Debug, Clone)]
pub enum GcObject {
    String(BoxString),
    Array(Vec<BxValue>),
    Struct(BxStruct),
    Instance(BxInstance),
    Future(BxFuture),
    CompiledFunction(Rc<BxCompiledFunction>),
    NativeFunction(BxNativeFunction),
    Class(Rc<RefCell<BxClass>>),
    Interface(Rc<RefCell<BxInterface>>),
    NativeObject(Rc<RefCell<dyn BxNativeObject>>),
    #[cfg(all(target_arch = "wasm32", feature = "js"))]
    JsValue(wasm_bindgen::JsValue),
    #[cfg(all(target_arch = "wasm32", not(feature = "js")))]
    JsHandle(u32),
}

pub struct Heap {
    objects: Vec<Option<GcObject>>,
    marks: Vec<bool>,
    free_list: Vec<GcId>,
    alloc_count: usize,
}

impl Heap {
    pub fn new() -> Self {
        Heap {
            objects: Vec::with_capacity(1024),
            marks: Vec::with_capacity(1024),
            free_list: Vec::new(),
            alloc_count: 0,
        }
    }

    pub fn alloc(&mut self, obj: GcObject) -> GcId {
        self.alloc_count += 1;
        if let Some(id) = self.free_list.pop() {
            self.objects[id] = Some(obj);
            self.marks[id] = false;
            id
        } else {
            let id = self.objects.len();
            self.objects.push(Some(obj));
            self.marks.push(false);
            id
        }
    }

    pub fn get(&self, id: GcId) -> &GcObject {
        self.objects[id].as_ref().expect("Attempted to access collected object")
    }

    pub fn get_mut(&mut self, id: GcId) -> &mut GcObject {
        self.objects[id].as_mut().expect("Attempted to access collected object")
    }

    pub fn should_collect(&self) -> bool {
        self.alloc_count > 1000 // Basic heuristic: collect every 1000 allocations
    }

    pub fn collect(&mut self, roots: &[BxValue]) {
        self.alloc_count = 0;
        // 1. Mark Phase
        self.marks.fill(false);
        let mut worklist = Vec::new();
        for root in roots {
            self.add_to_worklist(root, &mut worklist);
        }

        while let Some(id) = worklist.pop() {
            if self.marks[id] { continue; }
            self.marks[id] = true;

            match self.objects[id].as_ref().unwrap() {
                GcObject::String(_) | GcObject::NativeFunction(_) | GcObject::Class(_) | GcObject::Interface(_) | GcObject::CompiledFunction(_) | GcObject::NativeObject(_) => {}
                #[cfg(all(target_arch = "wasm32", feature = "js"))]
                GcObject::JsValue(_) => {}
                #[cfg(all(target_arch = "wasm32", not(feature = "js")))]
                GcObject::JsHandle(_) => {}
                GcObject::Array(arr) => {
                    for val in arr {
                        self.add_to_worklist(val, &mut worklist);
                    }
                }
                GcObject::Struct(s) => {
                    for val in &s.properties {
                        self.add_to_worklist(val, &mut worklist);
                    }
                }
                GcObject::Instance(inst) => {
                    for val in &inst.properties {
                        self.add_to_worklist(val, &mut worklist);
                    }
                    for val in inst.variables.borrow().values() {
                        self.add_to_worklist(val, &mut worklist);
                    }
                }
                GcObject::Future(f) => {
                    self.add_to_worklist(&f.value, &mut worklist);
                    if let Some(h) = &f.error_handler {
                        self.add_to_worklist(h, &mut worklist);
                    }
                }
            };
        }

        // 2. Sweep Phase
        for i in 0..self.objects.len() {
            if self.objects[i].is_some() && !self.marks[i] {
                self.objects[i] = None;
                self.free_list.push(i);
            }
        }
    }

    fn add_to_worklist(&self, val: &BxValue, worklist: &mut Vec<GcId>) {
        if let Some(id) = val.as_gc_id() {
            if id < self.objects.len() && self.objects[id].is_some() {
                worklist.push(id);
            }
        }
    }
}
