use crate::types::{BxNativeFunction, BxNativeObject, BxVM, BxValue};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};

static WATCHERS: OnceLock<Mutex<HashMap<String, usize>>> = OnceLock::new();
static NEXT_WATCHER_NAME: AtomicUsize = AtomicUsize::new(1);

fn watchers() -> &'static Mutex<HashMap<String, usize>> {
    WATCHERS.get_or_init(|| Mutex::new(HashMap::new()))
}

#[derive(Debug)]
struct WatcherObject {
    name: String,
    state: WatcherState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WatcherState {
    Created,
    Running,
    Stopped,
}

#[derive(Debug)]
struct WatcherNameObject {
    name: String,
}

impl BxNativeObject for WatcherObject {
    fn get_property(&self, _name: &str) -> BxValue {
        BxValue::new_null()
    }

    fn set_property(&mut self, _name: &str, _value: BxValue) {}

    fn call_method(
        &mut self,
        vm: &mut dyn BxVM,
        id: usize,
        name: &str,
        _args: &[BxValue],
    ) -> Result<BxValue, String> {
        match name.to_ascii_lowercase().as_str() {
            "getname" => {
                let name_id = vm.native_object_new(std::rc::Rc::new(std::cell::RefCell::new(
                    WatcherNameObject {
                        name: self.name.clone(),
                    },
                )));
                Ok(BxValue::new_ptr(name_id))
            }
            "getstateasstring" => Ok(BxValue::new_ptr(vm.string_new(
                match self.state {
                    WatcherState::Created => "CREATED",
                    WatcherState::Running => "RUNNING",
                    WatcherState::Stopped => "STOPPED",
                }
                .to_string(),
            ))),
            "isrunning" => Ok(BxValue::new_bool(self.state == WatcherState::Running)),
            "isstopped" => Ok(BxValue::new_bool(self.state == WatcherState::Stopped)),
            "start" => {
                self.state = WatcherState::Running;
                Ok(BxValue::new_ptr(id))
            }
            "stop" | "shutdown" => {
                self.state = WatcherState::Stopped;
                Ok(BxValue::new_ptr(id))
            }
            "restart" => {
                self.state = WatcherState::Running;
                Ok(BxValue::new_ptr(id))
            }
            _ => Err(format!("Watcher method '{}' not found", name)),
        }
    }
}

impl BxNativeObject for WatcherNameObject {
    fn get_property(&self, _name: &str) -> BxValue {
        BxValue::new_null()
    }

    fn set_property(&mut self, _name: &str, _value: BxValue) {}

    fn call_method(
        &mut self,
        vm: &mut dyn BxVM,
        _id: usize,
        name: &str,
        _args: &[BxValue],
    ) -> Result<BxValue, String> {
        match name.to_ascii_lowercase().as_str() {
            "getname" => Ok(BxValue::new_ptr(vm.string_new(self.name.clone()))),
            _ => Err(format!("Watcher name method '{}' not found", name)),
        }
    }
}

pub fn register_watcher_bifs(bifs: &mut HashMap<String, BxNativeFunction>) {
    bifs.insert("watcherExists".to_string(), watcher_exists as BxNativeFunction);
    bifs.insert("watcherGet".to_string(), watcher_get as BxNativeFunction);
    bifs.insert("watcherGetAll".to_string(), watcher_get_all as BxNativeFunction);
    bifs.insert("watcherList".to_string(), watcher_list as BxNativeFunction);
    bifs.insert("watcherNew".to_string(), watcher_new as BxNativeFunction);
    bifs.insert("watcherRestart".to_string(), watcher_restart as BxNativeFunction);
    bifs.insert("watcherShutdown".to_string(), watcher_shutdown as BxNativeFunction);
    bifs.insert("watcherShutdownAll".to_string(), watcher_shutdown_all as BxNativeFunction);
    bifs.insert("watcherStart".to_string(), watcher_start as BxNativeFunction);
    bifs.insert("watcherStop".to_string(), watcher_stop as BxNativeFunction);
    bifs.insert("watcherStopAll".to_string(), watcher_stop_all as BxNativeFunction);
}

fn watcher_name(vm: &dyn BxVM, args: &[BxValue]) -> Result<String, String> {
    args.first()
        .copied()
        .map(|value| vm.to_string(value))
        .ok_or_else(|| "Watcher name is required".to_string())
}

fn watcher_id(vm: &dyn BxVM, args: &[BxValue]) -> Result<usize, String> {
    let name = watcher_name(vm, args)?;
    watchers()
        .lock()
        .map_err(|_| "Watcher registry is poisoned".to_string())?
        .get(&name)
        .copied()
        .ok_or_else(|| format!("Watcher '{}' does not exist", name))
}

fn watcher_exists(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    let name = watcher_name(vm, args)?;
    let exists = watchers()
        .lock()
        .map_err(|_| "Watcher registry is poisoned".to_string())?
        .contains_key(&name);
    Ok(BxValue::new_bool(exists))
}

fn watcher_get(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    Ok(BxValue::new_ptr(watcher_id(vm, args)?))
}

fn watcher_get_all(vm: &mut dyn BxVM, _args: &[BxValue]) -> Result<BxValue, String> {
    let result_id = vm.struct_new();
    let entries = watchers()
        .lock()
        .map_err(|_| "Watcher registry is poisoned".to_string())?
        .clone();
    for (name, id) in entries {
        vm.struct_set(result_id, &name, BxValue::new_ptr(id));
    }
    Ok(BxValue::new_ptr(result_id))
}

fn watcher_list(vm: &mut dyn BxVM, _args: &[BxValue]) -> Result<BxValue, String> {
    let result_id = vm.array_new();
    let entries = watchers()
        .lock()
        .map_err(|_| "Watcher registry is poisoned".to_string())?;
    for name in entries.keys() {
        let name_id = vm.string_new(name.clone());
        vm.array_push(result_id, BxValue::new_ptr(name_id));
    }
    Ok(BxValue::new_ptr(result_id))
}

fn watcher_new(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 3 {
        return Err("watcherNew() expects name, paths, and listener".to_string());
    }
    if !vm.is_array_value(args[1]) && !vm.is_string_value(args[1]) {
        return Err("watcherNew() paths must be an array or string".to_string());
    }
    if !vm.value_matches_type_name(args[2], "function") && !vm.is_struct_value(args[2]) {
        return Err("watcherNew() listener must be a function or struct".to_string());
    }

    let mut name = vm.to_string(args[0]);
    if name.is_empty() {
        name = format!(
            "watcher-{}",
            NEXT_WATCHER_NAME.fetch_add(1, Ordering::Relaxed)
        );
    }
    let force = args.get(3).is_some_and(|value| value.as_bool());
    let mut registry = watchers()
        .lock()
        .map_err(|_| "Watcher registry is poisoned".to_string())?;
    if registry.contains_key(&name) {
        if !force {
            return Err(format!("Watcher '{}' already exists", name));
        }
        registry.remove(&name);
    }
    let id = vm.native_object_new(std::rc::Rc::new(std::cell::RefCell::new(WatcherObject {
        name: name.clone(),
        state: WatcherState::Created,
    })));
    registry.insert(name, id);
    Ok(BxValue::new_ptr(id))
}

fn watcher_restart(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    let id = watcher_id(vm, args)?;
    vm.native_object_call_method(id, "restart", &[])
}

fn watcher_shutdown(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    let name = watcher_name(vm, args)?;
    let id = watchers()
        .lock()
        .map_err(|_| "Watcher registry is poisoned".to_string())?
        .remove(&name)
        .ok_or_else(|| format!("Watcher '{}' does not exist", name))?;
    let _ = vm.native_object_call_method(id, "shutdown", &[]);
    Ok(BxValue::new_bool(true))
}

fn watcher_shutdown_all(vm: &mut dyn BxVM, _args: &[BxValue]) -> Result<BxValue, String> {
    let _ = vm;
    watchers()
        .lock()
        .map_err(|_| "Watcher registry is poisoned".to_string())?
        .clear();
    Ok(BxValue::new_null())
}

fn watcher_start(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    let id = watcher_id(vm, args)?;
    vm.native_object_call_method(id, "start", &[])
}

fn watcher_stop(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    let id = watcher_id(vm, args)?;
    vm.native_object_call_method(id, "stop", &[])
}

fn watcher_stop_all(vm: &mut dyn BxVM, _args: &[BxValue]) -> Result<BxValue, String> {
    let entries = watchers()
        .lock()
        .map_err(|_| "Watcher registry is poisoned".to_string())?
        .values()
        .copied()
        .collect::<Vec<_>>();
    for id in entries {
        let _ = vm.native_object_call_method(id, "stop", &[]);
    }
    Ok(BxValue::new_null())
}
