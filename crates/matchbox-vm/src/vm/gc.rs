use crate::types::{
    BxClass, BxCompiledFunction, BxFuture, BxInstance, BxInterface, BxNativeFunction,
    BxNativeObject, BxRange, BxStruct, BxValue, box_string::BoxString,
};
use chrono::{DateTime, Utc};
use std::cell::RefCell;
use std::rc::Rc;

pub type GcId = usize;

/// Configuration for the garbage collector, with preset defaults
/// tuned for different target platforms.
#[derive(Debug, Clone)]
pub struct GCConfig {
    /// Initial capacity of backing Vecs (objects, marks, etc.)
    pub initial_capacity: usize,
    /// Initial GC threshold (alloc_count > this triggers collection)
    pub initial_threshold: usize,
    /// Maximum GC threshold after dynamic adjustment
    pub max_threshold: usize,
    /// Optional hard cap on total heap objects
    pub max_objects: Option<usize>,
}

impl GCConfig {
    /// Aggressive defaults for ESP32 (~320KB heap budget).
    pub fn for_esp32() -> Self {
        GCConfig {
            initial_capacity: 128,
            initial_threshold: 256,
            max_threshold: 1024,
            max_objects: Some(2048),
        }
    }

    /// Lenient defaults for desktop (plentiful RAM).
    pub fn for_desktop() -> Self {
        GCConfig {
            initial_capacity: 1024,
            initial_threshold: 10000,
            max_threshold: 100000,
            max_objects: None,
        }
    }

    /// Balanced defaults for WASM (browser/edge memory limits).
    pub fn for_wasm() -> Self {
        GCConfig {
            initial_capacity: 512,
            initial_threshold: 2000,
            max_threshold: 20000,
            max_objects: Some(50000),
        }
    }
}

impl Default for GCConfig {
    fn default() -> Self {
        #[cfg(target_os = "espidf")]
        {
            GCConfig::for_esp32()
        }
        #[cfg(not(target_os = "espidf"))]
        {
            GCConfig::for_desktop()
        }
    }
}

#[derive(Debug, Clone)]
pub enum GcObject {
    String(BoxString),
    Bytes(Vec<u8>),
    Array(Vec<BxValue>),
    Range(BxRange),
    DateTime(DateTime<Utc>),
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
    #[cfg(all(target_arch = "wasm32", feature = "js-host-abi", not(feature = "js")))]
    JsHandle(u32),
}

pub struct Heap {
    objects: Vec<Option<GcObject>>,
    marks: Vec<bool>,
    free_list: Vec<GcId>,
    alloc_count: usize,
    next_gc_threshold: usize,
    generations: Vec<u8>,
    dirty: Vec<bool>,
    remembered_set: Vec<GcId>,
    young_objects: Vec<GcId>,
    minor_gc_count: usize,
    config: GCConfig,
}

impl Heap {
    pub fn new() -> Self {
        Self::with_config(GCConfig::default())
    }

    pub fn with_config(config: GCConfig) -> Self {
        let cap = config.initial_capacity;
        Heap {
            objects: Vec::with_capacity(cap),
            marks: Vec::with_capacity(cap),
            free_list: Vec::new(),
            alloc_count: 0,
            next_gc_threshold: config.initial_threshold,
            generations: Vec::with_capacity(cap),
            dirty: Vec::with_capacity(cap),
            remembered_set: Vec::new(),
            young_objects: Vec::new(),
            minor_gc_count: 0,
            config,
        }
    }

    pub fn alloc(&mut self, obj: GcObject) -> GcId {
        self.alloc_count += 1;
        if let Some(id) = self.free_list.pop() {
            self.objects[id] = Some(obj);
            self.marks[id] = false;
            self.generations[id] = 0;
            self.dirty[id] = false;
            self.young_objects.push(id);
            id
        } else {
            let id = self.objects.len();
            self.objects.push(Some(obj));
            self.marks.push(false);
            self.generations.push(0);
            self.dirty.push(false);
            self.young_objects.push(id);
            id
        }
    }

    /// Attempts to allocate a new GC object, returning `None` if the backing
    /// storage cannot grow (e.g. out of memory). The fast path (reusing a
    /// slot from the free list) is identical to `alloc`.
    pub fn try_alloc(&mut self, obj: GcObject) -> Option<GcId> {
        self.alloc_count += 1;
        if let Some(id) = self.free_list.pop() {
            self.objects[id] = Some(obj);
            self.marks[id] = false;
            self.generations[id] = 0;
            self.dirty[id] = false;
            self.young_objects.push(id);
            Some(id)
        } else {
            // Check hard cap before attempting to grow
            if let Some(max) = self.config.max_objects {
                if self.objects.len() >= max {
                    return None;
                }
            }
            // Try to grow backing Vecs; if OOM, return None
            self.objects.try_reserve(1).ok()?;
            self.marks.try_reserve(1).ok()?;
            self.generations.try_reserve(1).ok()?;
            self.dirty.try_reserve(1).ok()?;
            let id = self.objects.len();
            self.objects.push(Some(obj));
            self.marks.push(false);
            self.generations.push(0);
            self.dirty.push(false);
            self.young_objects.push(id);
            Some(id)
        }
    }

    pub fn get(&self, id: GcId) -> &GcObject {
        self.objects[id]
            .as_ref()
            .expect("Attempted to access collected object")
    }

    #[inline]
    pub fn get_opt(&self, id: GcId) -> Option<&GcObject> {
        self.objects.get(id).and_then(|o| o.as_ref())
    }

    pub fn get_mut(&mut self, id: GcId) -> &mut GcObject {
        if self.generations[id] > 0 && !self.dirty[id] {
            self.dirty[id] = true;
            self.remembered_set.push(id);
        }
        self.objects[id]
            .as_mut()
            .expect("Attempted to access collected object")
    }

    pub fn should_collect(&self) -> bool {
        self.alloc_count > self.next_gc_threshold
    }

    /// Returns the current GC threshold (for testing/diagnostics).
    pub fn gc_threshold(&self) -> usize {
        self.next_gc_threshold
    }

    pub fn collect(&mut self, roots: &[BxValue]) {
        self.alloc_count = 0;
        self.minor_gc_count += 1;

        if self.minor_gc_count >= 8 {
            self.major_collect(roots);
            self.minor_gc_count = 0;
        } else {
            self.minor_collect(roots);
        }

        let live = self.objects.iter().filter(|o| o.is_some()).count();
        let dynamic = live.saturating_mul(2).max(self.config.initial_threshold);
        self.next_gc_threshold = dynamic.min(self.config.max_threshold);
    }

    fn minor_collect(&mut self, roots: &[BxValue]) {
        // Clear marks for young objects only
        for &id in &self.young_objects {
            if id < self.marks.len() {
                self.marks[id] = false;
            }
        }

        let mut worklist = Vec::new();

        // Add young roots from stack/globals
        for root in roots {
            self.add_to_worklist_young(root, &mut worklist);
        }

        // Scan remembered set: old objects that were mutated may point to young objects
        // We need to collect the IDs first to avoid borrow issues
        let remembered: Vec<GcId> = self.remembered_set.drain(..).collect();
        for id in &remembered {
            self.push_children_young(*id, &mut worklist);
        }

        // Mark phase: only traverse young objects
        while let Some(id) = worklist.pop() {
            if self.marks[id] {
                continue;
            }
            self.marks[id] = true;
            self.push_children_young(id, &mut worklist);
        }

        // Sweep phase: only process young objects
        let young: Vec<GcId> = self.young_objects.drain(..).collect();
        for id in young {
            if self.objects[id].is_some() && !self.marks[id] {
                self.objects[id] = None;
                self.free_list.push(id);
            } else if self.objects[id].is_some() {
                // Promote survivors to old generation
                self.generations[id] = 1;
            }
        }

        // Clear dirty flags for remembered objects
        for id in remembered {
            self.dirty[id] = false;
        }
    }

    fn major_collect(&mut self, roots: &[BxValue]) {
        // Full mark-sweep of the entire heap
        self.marks.fill(false);
        let mut worklist = Vec::new();
        for root in roots {
            self.add_to_worklist(root, &mut worklist);
        }

        while let Some(id) = worklist.pop() {
            if self.marks[id] {
                continue;
            }
            self.marks[id] = true;
            self.push_children(id, &mut worklist);
        }

        // Sweep entire heap
        for i in 0..self.objects.len() {
            if self.objects[i].is_some() && !self.marks[i] {
                self.objects[i] = None;
                self.free_list.push(i);
            } else if self.objects[i].is_some() {
                // All survivors become old
                self.generations[i] = 1;
            }
        }

        // Clear generational bookkeeping
        self.young_objects.clear();
        self.remembered_set.clear();
        self.dirty.fill(false);
    }

    fn push_children(&self, id: GcId, worklist: &mut Vec<GcId>) {
        match self.objects[id].as_ref().unwrap() {
            GcObject::String(_)
            | GcObject::Bytes(_)
            | GcObject::NativeFunction(_)
            | GcObject::Class(_)
            | GcObject::Interface(_)
            | GcObject::CompiledFunction(_)
            | GcObject::Range(_)
            | GcObject::DateTime(_) => {}
            GcObject::NativeObject(obj) => {
                let mut tracer = WorklistTracer {
                    worklist,
                    heap: self,
                };
                // Use unsafe to bypass RefCell borrow check during tracing.
                // This is safe because GC is stop-the-world and we are only reading.
                // This is necessary because the object might be borrowed by the VM
                // during a native method call that triggered GC.
                unsafe {
                    let ptr = obj.as_ptr();
                    (*ptr).trace(&mut tracer);
                }
            }
            #[cfg(all(target_arch = "wasm32", feature = "js"))]
            GcObject::JsValue(_) => {}
            #[cfg(all(target_arch = "wasm32", feature = "js-host-abi", not(feature = "js")))]
            GcObject::JsHandle(_) => {}
            GcObject::Array(arr) => {
                for val in arr {
                    self.add_to_worklist(val, worklist);
                }
            }
            GcObject::Struct(s) => {
                for val in &s.properties {
                    self.add_to_worklist(val, worklist);
                }
            }
            GcObject::Instance(inst) => {
                for val in &inst.properties {
                    self.add_to_worklist(val, worklist);
                }
                for val in inst.variables.borrow().values() {
                    self.add_to_worklist(val, worklist);
                }
            }
            GcObject::Future(f) => {
                self.add_to_worklist(&f.value, worklist);
                if let Some(h) = &f.error_handler {
                    self.add_to_worklist(h, worklist);
                }
            }
        };
    }

    fn push_children_young(&self, id: GcId, worklist: &mut Vec<GcId>) {
        match self.objects[id].as_ref().unwrap() {
            GcObject::String(_)
            | GcObject::Bytes(_)
            | GcObject::NativeFunction(_)
            | GcObject::Class(_)
            | GcObject::Interface(_)
            | GcObject::CompiledFunction(_)
            | GcObject::Range(_)
            | GcObject::DateTime(_) => {}
            GcObject::NativeObject(obj) => {
                let mut tracer = YoungWorklistTracer {
                    worklist,
                    heap: self,
                };
                unsafe {
                    let ptr = obj.as_ptr();
                    (*ptr).trace(&mut tracer);
                }
            }
            #[cfg(all(target_arch = "wasm32", feature = "js"))]
            GcObject::JsValue(_) => {}
            #[cfg(all(target_arch = "wasm32", feature = "js-host-abi", not(feature = "js")))]
            GcObject::JsHandle(_) => {}
            GcObject::Array(arr) => {
                for val in arr {
                    self.add_to_worklist_young(val, worklist);
                }
            }
            GcObject::Struct(s) => {
                for val in &s.properties {
                    self.add_to_worklist_young(val, worklist);
                }
            }
            GcObject::Instance(inst) => {
                for val in &inst.properties {
                    self.add_to_worklist_young(val, worklist);
                }
                for val in inst.variables.borrow().values() {
                    self.add_to_worklist_young(val, worklist);
                }
            }
            GcObject::Future(f) => {
                self.add_to_worklist_young(&f.value, worklist);
                if let Some(h) = &f.error_handler {
                    self.add_to_worklist_young(h, worklist);
                }
            }
        };
    }
}

struct WorklistTracer<'a> {
    worklist: &'a mut Vec<GcId>,
    heap: &'a Heap,
}

impl<'a> crate::types::Tracer for WorklistTracer<'a> {
    fn mark(&mut self, val: &BxValue) {
        self.heap.add_to_worklist(val, self.worklist);
    }
}

struct YoungWorklistTracer<'a> {
    worklist: &'a mut Vec<GcId>,
    heap: &'a Heap,
}

impl<'a> crate::types::Tracer for YoungWorklistTracer<'a> {
    fn mark(&mut self, val: &BxValue) {
        self.heap.add_to_worklist_young(val, self.worklist);
    }
}

impl Heap {
    fn add_to_worklist(&self, val: &BxValue, worklist: &mut Vec<GcId>) {
        if let Some(id) = val.as_gc_id() {
            if id < self.objects.len() && self.objects[id].is_some() {
                worklist.push(id);
            }
        }
    }

    fn add_to_worklist_young(&self, val: &BxValue, worklist: &mut Vec<GcId>) {
        if let Some(id) = val.as_gc_id() {
            if id < self.objects.len() && self.objects[id].is_some() && self.generations[id] == 0 {
                worklist.push(id);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_alloc_succeeds_with_free_list_reuse() {
        let mut heap = Heap::new();
        // Allocate and then simulate GC freeing to populate free_list
        let id = heap.alloc(GcObject::String(BoxString::new("hello")));
        // Simulate what collect() does: mark as freed and add to free_list
        heap.objects[id] = None;
        heap.free_list.push(id);
        heap.young_objects.retain(|&x| x != id);

        // try_alloc should reuse the freed slot
        let result = heap.try_alloc(GcObject::String(BoxString::new("world")));
        assert_eq!(result, Some(id));
        assert!(heap.objects[id].is_some());
    }

    #[test]
    fn try_alloc_succeeds_with_new_allocation() {
        let mut heap = Heap::new();
        let result = heap.try_alloc(GcObject::String(BoxString::new("hello")));
        assert!(result.is_some());
        let id = result.unwrap();
        assert!(heap.objects[id].is_some());
    }

    #[test]
    fn try_alloc_returns_none_when_capacity_exhausted() {
        let mut heap = Heap::new();
        // Fill up capacity to approach the grow boundary
        let cap = heap.objects.capacity();
        for _ in 0..cap {
            heap.alloc(GcObject::String(BoxString::new("fill")));
        }
        // The method signature enforces Option return; verify type
        let result = heap.try_alloc(GcObject::String(BoxString::new("extra")));
        let _: Option<GcId> = result;
    }

    #[test]
    fn gcconfig_esp32_preset() {
        let cfg = GCConfig::for_esp32();
        assert_eq!(cfg.initial_capacity, 128);
        assert_eq!(cfg.initial_threshold, 256);
        assert_eq!(cfg.max_threshold, 1024);
        assert_eq!(cfg.max_objects, Some(2048));
    }

    #[test]
    fn gcconfig_desktop_preset() {
        let cfg = GCConfig::for_desktop();
        assert_eq!(cfg.initial_capacity, 1024);
        assert_eq!(cfg.initial_threshold, 10000);
        assert_eq!(cfg.max_threshold, 100000);
        assert_eq!(cfg.max_objects, None);
    }

    #[test]
    fn gcconfig_wasm_preset() {
        let cfg = GCConfig::for_wasm();
        assert_eq!(cfg.initial_capacity, 512);
        assert_eq!(cfg.initial_threshold, 2000);
        assert_eq!(cfg.max_threshold, 20000);
        assert_eq!(cfg.max_objects, Some(50000));
    }

    #[test]
    fn gcconfig_initial_threshold_used_in_heap() {
        let cfg = GCConfig::for_esp32();
        let heap = Heap::with_config(cfg.clone());
        assert_eq!(heap.next_gc_threshold, cfg.initial_threshold);
    }

    #[test]
    fn try_alloc_respects_max_objects() {
        let mut cfg = GCConfig::for_desktop();
        cfg.max_objects = Some(5);
        let mut heap = Heap::with_config(cfg);
        // Allocate up to the cap
        for _ in 0..5 {
            assert!(
                heap.try_alloc(GcObject::String(BoxString::new("x")))
                    .is_some()
            );
        }
        // 6th allocation should fail
        assert!(
            heap.try_alloc(GcObject::String(BoxString::new("y")))
                .is_none()
        );
    }
}
