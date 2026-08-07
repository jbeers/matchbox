use crate::types::{BxVM, BxValue};

pub fn url_encoded_format(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() {
        return Err("urlEncodedFormat() expects 1 argument".to_string());
    }
    let input = vm.to_string(args[0]);
    let mut encoded = String::new();
    for byte in input.as_bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(*byte as char);
        } else {
            encoded.push_str(&format!("%{:02X}", byte));
        }
    }
    Ok(BxValue::new_ptr(vm.string_new(encoded)))
}

pub fn get_file_from_path(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() {
        return Err("getFileFromPath() expects 1 argument".to_string());
    }
    let path = vm.to_string(args[0]);
    let file = path.rsplit(['/', '\\']).next().unwrap_or_default();
    Ok(BxValue::new_ptr(vm.string_new(file.to_string())))
}

pub fn box_announce(_vm: &mut dyn BxVM, _args: &[BxValue]) -> Result<BxValue, String> {
    Ok(BxValue::new_bool(true))
}

pub fn trace(_vm: &mut dyn BxVM, _args: &[BxValue]) -> Result<BxValue, String> {
    Ok(BxValue::new_null())
}

pub fn write_log(_vm: &mut dyn BxVM, _args: &[BxValue]) -> Result<BxValue, String> {
    Ok(BxValue::new_null())
}

pub fn get_base_tag_data(_vm: &mut dyn BxVM, _args: &[BxValue]) -> Result<BxValue, String> {
    Ok(BxValue::new_null())
}

pub fn get_base_tag_list(vm: &mut dyn BxVM, _args: &[BxValue]) -> Result<BxValue, String> {
    Ok(BxValue::new_ptr(vm.string_new(String::new())))
}

pub fn get_base_template_path(vm: &mut dyn BxVM, _args: &[BxValue]) -> Result<BxValue, String> {
    Ok(BxValue::new_ptr(vm.string_new(String::new())))
}

pub fn get_current_template_path(vm: &mut dyn BxVM, _args: &[BxValue]) -> Result<BxValue, String> {
    Ok(BxValue::new_ptr(vm.string_new(String::new())))
}

pub fn get_box_version_info(vm: &mut dyn BxVM, _args: &[BxValue]) -> Result<BxValue, String> {
    let id = vm.struct_new();
    for (key, value) in [
        ("version", "0.9.0"),
        ("buildDate", ""),
        ("codename", ""),
        ("boxlangId", "matchbox"),
    ] {
        let value_id = vm.string_new(value.to_string());
        vm.struct_set(id, key, BxValue::new_ptr(value_id));
    }
    Ok(BxValue::new_ptr(id))
}

pub fn get_component_list(vm: &mut dyn BxVM, _args: &[BxValue]) -> Result<BxValue, String> {
    Ok(BxValue::new_ptr(vm.struct_new()))
}

pub fn get_function_list(vm: &mut dyn BxVM, _args: &[BxValue]) -> Result<BxValue, String> {
    Ok(BxValue::new_ptr(vm.struct_new()))
}

pub fn get_module_info(vm: &mut dyn BxVM, _args: &[BxValue]) -> Result<BxValue, String> {
    Ok(BxValue::new_ptr(vm.struct_new()))
}

pub fn get_module_list(vm: &mut dyn BxVM, _args: &[BxValue]) -> Result<BxValue, String> {
    Ok(BxValue::new_ptr(vm.struct_new()))
}
