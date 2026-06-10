use crate::types::{BxNativeFunction, BxVM, BxValue};
use std::collections::HashMap;

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

fn watcher_exists(_vm: &mut dyn BxVM, _args: &[BxValue]) -> Result<BxValue, String> {
    Ok(BxValue::new_bool(false))
}

fn watcher_get(_vm: &mut dyn BxVM, _args: &[BxValue]) -> Result<BxValue, String> {
    Ok(BxValue::new_null())
}

fn watcher_get_all(vm: &mut dyn BxVM, _args: &[BxValue]) -> Result<BxValue, String> {
    let id = vm.array_new();
    Ok(BxValue::new_ptr(id))
}

fn watcher_list(vm: &mut dyn BxVM, _args: &[BxValue]) -> Result<BxValue, String> {
    let id = vm.array_new();
    Ok(BxValue::new_ptr(id))
}

fn watcher_new(_vm: &mut dyn BxVM, _args: &[BxValue]) -> Result<BxValue, String> {
    Err("Filesystem watcher is not available in this build".to_string())
}

fn watcher_restart(_vm: &mut dyn BxVM, _args: &[BxValue]) -> Result<BxValue, String> {
    Err("Filesystem watcher is not available in this build".to_string())
}

fn watcher_shutdown(_vm: &mut dyn BxVM, _args: &[BxValue]) -> Result<BxValue, String> {
    Err("Filesystem watcher is not available in this build".to_string())
}

fn watcher_shutdown_all(_vm: &mut dyn BxVM, _args: &[BxValue]) -> Result<BxValue, String> {
    Ok(BxValue::new_null())
}

fn watcher_start(_vm: &mut dyn BxVM, _args: &[BxValue]) -> Result<BxValue, String> {
    Err("Filesystem watcher is not available in this build".to_string())
}

fn watcher_stop(_vm: &mut dyn BxVM, _args: &[BxValue]) -> Result<BxValue, String> {
    Err("Filesystem watcher is not available in this build".to_string())
}

fn watcher_stop_all(_vm: &mut dyn BxVM, _args: &[BxValue]) -> Result<BxValue, String> {
    Ok(BxValue::new_null())
}
