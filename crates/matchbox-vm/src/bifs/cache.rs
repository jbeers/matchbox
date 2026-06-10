use crate::types::{BxNativeFunction, BxVM, BxValue};
use std::collections::HashMap;

pub fn register_cache_bifs(bifs: &mut HashMap<String, BxNativeFunction>) {
    bifs.insert("cache".to_string(), cache as BxNativeFunction);
    bifs.insert("cacheFilter".to_string(), cache_filter as BxNativeFunction);
    bifs.insert("cacheNames".to_string(), cache_names as BxNativeFunction);
    bifs.insert("cacheProviders".to_string(), cache_providers as BxNativeFunction);
    bifs.insert("cacheService".to_string(), cache_service as BxNativeFunction);
}

fn cache(_vm: &mut dyn BxVM, _args: &[BxValue]) -> Result<BxValue, String> {
    Err("Cache service is not available in this build".to_string())
}

fn cache_filter(vm: &mut dyn BxVM, _args: &[BxValue]) -> Result<BxValue, String> {
    let id = vm.array_new();
    Ok(BxValue::new_ptr(id))
}

fn cache_names(vm: &mut dyn BxVM, _args: &[BxValue]) -> Result<BxValue, String> {
    let id = vm.array_new();
    Ok(BxValue::new_ptr(id))
}

fn cache_providers(vm: &mut dyn BxVM, _args: &[BxValue]) -> Result<BxValue, String> {
    let id = vm.array_new();
    Ok(BxValue::new_ptr(id))
}

fn cache_service(_vm: &mut dyn BxVM, _args: &[BxValue]) -> Result<BxValue, String> {
    Err("Cache service is not available in this build".to_string())
}
