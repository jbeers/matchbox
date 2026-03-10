use matchbox_vm::types::{BxNativeFunction, BxValue, BxVM};
use std::collections::HashMap;

pub fn cube(_vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() != 1 {
        return Err("cube requires exactly 1 argument".to_string());
    }
    let n = args[0].as_number();
    Ok(BxValue::new_number(n * n * n))
}

pub fn register_bifs() -> HashMap<String, BxNativeFunction> {
    let mut map = HashMap::new();
    map.insert("cube".to_string(), cube as BxNativeFunction);
    map
}
