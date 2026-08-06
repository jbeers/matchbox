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
