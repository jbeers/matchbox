use crate::types::{BxNativeFunction, BxVM, BxValue};

fn is_closure_bif(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() {
        return Ok(BxValue::new_bool(false));
    }
    Ok(BxValue::new_bool(vm.value_matches_type_name(args[0], "function")))
}

fn is_custom_function_bif(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() {
        return Ok(BxValue::new_bool(false));
    }
    Ok(BxValue::new_bool(vm.value_matches_type_name(args[0], "function")))
}

fn is_date_object_bif(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() {
        return Ok(BxValue::new_bool(false));
    }
    let result = vm.type_name_from_value(args[0])
        .map(|name| name.eq_ignore_ascii_case("datetime"))
        .unwrap_or(false);
    Ok(BxValue::new_bool(result))
}

fn is_debug_mode_bif(_vm: &mut dyn BxVM, _args: &[BxValue]) -> Result<BxValue, String> {
    Ok(BxValue::new_bool(false))
}

fn is_defined_bif(_vm: &mut dyn BxVM, _args: &[BxValue]) -> Result<BxValue, String> {
    Ok(BxValue::new_bool(false))
}

fn is_empty_bif(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() {
        return Ok(BxValue::new_bool(true));
    }
    let val = args[0];
    if val.is_null() { return Ok(BxValue::new_bool(true)); }
    if vm.is_string_value(val) { return Ok(BxValue::new_bool(vm.to_string(val).is_empty())); }
    if vm.is_array_value(val) {
        let len = val.as_gc_id().map(|id| vm.get_len(id)).unwrap_or(0);
        return Ok(BxValue::new_bool(len == 0));
    }
    if vm.is_struct_value(val) {
        let len = val.as_gc_id().map(|id| vm.get_len(id)).unwrap_or(0);
        return Ok(BxValue::new_bool(len == 0));
    }
    if val.is_number() { return Ok(BxValue::new_bool(val.as_number() == 0.0)); }
    if val.is_bool() { return Ok(BxValue::new_bool(!val.as_bool())); }
    Ok(BxValue::new_bool(false))
}

fn is_file_object_bif(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() { return Ok(BxValue::new_bool(false)); }
    let result = vm.type_name_from_value(args[0])
        .map(|name| name.eq_ignore_ascii_case("file"))
        .unwrap_or(false);
    Ok(BxValue::new_bool(result))
}

fn is_ipv6_bif(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() { return Ok(BxValue::new_bool(false)); }
    let s = vm.to_string(args[0]);
    Ok(BxValue::new_bool(!s.is_empty() && s.parse::<std::net::Ipv6Addr>().is_ok()))
}

fn is_leap_year_bif(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() { return Err("isLeapYear() requires a year argument".to_string()); }
    let val = args[0];
    let year = if val.is_number() { val.as_number() as i32 }
    else if val.is_int() { val.as_int() }
    else { vm.to_string(val).trim().parse::<i32>().map_err(|_| format!("isLeapYear() expected numeric, got '{}'", vm.to_string(val)))? };
    Ok(BxValue::new_bool((year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)))
}

fn is_local_host_bif(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() { return Ok(BxValue::new_bool(false)); }
    let s = vm.to_string(args[0]);
    if s.is_empty() { return Ok(BxValue::new_bool(false)); }
    let lower = s.to_ascii_lowercase();
    if lower == "localhost" || lower == "127.0.0.1" || lower == "::1" || lower == "0:0:0:0:0:0:0:1" {
        return Ok(BxValue::new_bool(true));
    }
    if let Ok(addr) = s.parse::<std::net::IpAddr>() { return Ok(BxValue::new_bool(addr.is_loopback())); }
    Ok(BxValue::new_bool(false))
}

fn is_query_bif(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() { return Ok(BxValue::new_bool(false)); }
    let val = args[0];
    if !val.is_ptr() { return Ok(BxValue::new_bool(false)); }
    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Some(id) = val.as_gc_id() {
            if vm.native_object_query_row_count(id).is_some() { return Ok(BxValue::new_bool(true)); }
        }
    }
    let _ = vm;
    Ok(BxValue::new_bool(false))
}

fn is_valid_bif(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 2 { return Err("isValid() requires at least 2 arguments: (type, value)".to_string()); }
    let type_str = vm.to_string(args[0]).to_ascii_lowercase();
    let value = args[1];
    let result = match type_str.as_str() {
        "any" => true,
        "array" => vm.is_array_value(value),
        "binary" => vm.is_bytes(value),
        "boolean" | "bool" => {
            if value.is_bool() || value.is_number() { true }
            else if value.is_null() { false }
            else { matches!(vm.to_string(value).to_ascii_lowercase().as_str(), "true" | "false" | "yes" | "no" | "1" | "0") }
        }
        "date" | "datetime" => vm.type_name_from_value(value).map(|n| n.eq_ignore_ascii_case("datetime")).unwrap_or(false),
        "email" => { let s = vm.to_string(value); s.contains('@') && s.contains('.') }
        "float" => { if value.is_number() { value.as_number().fract() != 0.0 } else { vm.to_string(value).parse::<f64>().map(|n| n.fract() != 0.0).unwrap_or(false) } }
        "function" | "closure" => vm.value_matches_type_name(value, "function"),
        "guid" | "uuid" => { let s = vm.to_string(value).trim().to_string(); s.len() == 36 && { let b = s.as_bytes(); b[8]==b'-' && b[13]==b'-' && b[18]==b'-' && b[23]==b'-' && s.replace('-',"").chars().all(|c| c.is_ascii_hexdigit()) } }
        "hex" => { let s = vm.to_string(value); !s.is_empty() && s.chars().all(|c| c.is_ascii_hexdigit()) }
        "integer" | "int" => { if value.is_int() { true } else if value.is_number() { value.as_number().fract() == 0.0 } else { vm.to_string(value).trim().parse::<i64>().is_ok() } }
        "numeric" | "number" => { if value.is_number() { true } else if value.is_null() { false } else { vm.to_string(value).trim().parse::<f64>().is_ok() } }
        "query" => { if !value.is_ptr() { false } else { #[cfg(not(target_arch = "wasm32"))] { value.as_gc_id().and_then(|id| vm.native_object_query_row_count(id)).is_some() } #[cfg(target_arch = "wasm32")] { false } } }
        "range" => { if args.len() >= 4 { let n = if value.is_number() { value.as_number() } else { return Ok(BxValue::new_bool(false)) }; let mn = if args[2].is_number() { args[2].as_number() } else { return Ok(BxValue::new_bool(false)) }; let mx = if args[3].is_number() { args[3].as_number() } else { return Ok(BxValue::new_bool(false)) }; n >= mn && n <= mx } else { false } }
        "regex" | "regular_expression" => { let p = if args.len() >= 3 { vm.to_string(args[2]) } else { return Err("isValid() regex requires pattern".to_string()) }; regex::Regex::new(&p).map(|re| re.is_match(&vm.to_string(value))).unwrap_or(false) }
        "ssn" | "social_security_number" => { let d: String = vm.to_string(value).chars().filter(|c| c.is_ascii_digit()).collect(); d.len() == 9 }
        "string" => vm.is_string_value(value),
        "struct" => vm.is_struct_value(value),
        "telephone" => { let d: String = vm.to_string(value).chars().filter(|c| c.is_ascii_digit()).collect(); d.len() >= 7 && d.len() <= 15 }
        "url" => { let s = vm.to_string(value); s.starts_with("http://") || s.starts_with("https://") }
        "variablename" => { let s = vm.to_string(value); let mut c = s.chars(); match c.next() { Some(ch) if ch.is_ascii_alphabetic() || ch == '_' || ch == '$' => c.all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '$'), _ => false } }
        "xml" => false,
        "zipcode" => { let d: String = vm.to_string(value).chars().filter(|c| c.is_ascii_digit()).collect(); d.len() == 5 || d.len() == 9 }
        "creditcard" => { let d: String = vm.to_string(value).chars().filter(|c| c.is_ascii_digit()).collect(); d.len() >= 13 && d.len() <= 19 }
        "component" | "class" => { value.is_ptr() && !vm.is_string_value(value) && !vm.is_array_value(value) && !vm.is_struct_value(value) && !vm.is_bytes(value) }
        _ => false,
    };
    Ok(BxValue::new_bool(result))
}

fn is_xml_bif(_vm: &mut dyn BxVM, _args: &[BxValue]) -> Result<BxValue, String> { Ok(BxValue::new_bool(false)) }
fn is_xml_attribute_bif(_vm: &mut dyn BxVM, _args: &[BxValue]) -> Result<BxValue, String> { Ok(BxValue::new_bool(false)) }
fn is_xml_doc_bif(_vm: &mut dyn BxVM, _args: &[BxValue]) -> Result<BxValue, String> { Ok(BxValue::new_bool(false)) }
fn is_xml_element_bif(_vm: &mut dyn BxVM, _args: &[BxValue]) -> Result<BxValue, String> { Ok(BxValue::new_bool(false)) }
fn is_xml_node_bif(_vm: &mut dyn BxVM, _args: &[BxValue]) -> Result<BxValue, String> { Ok(BxValue::new_bool(false)) }
fn is_xml_root_bif(_vm: &mut dyn BxVM, _args: &[BxValue]) -> Result<BxValue, String> { Ok(BxValue::new_bool(false)) }

fn boolean_format_bif(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() { return Err("booleanFormat() requires a value argument".to_string()); }
    let val = args[0];
    if val.is_ptr() && vm.is_string_value(val) && vm.to_string(val).is_empty() {
        return Ok(BxValue::new_ptr(vm.string_new("false".to_string())));
    }
    let is_true = if val.is_bool() { val.as_bool() }
    else if val.is_number() { val.as_number() != 0.0 }
    else if val.is_int() { val.as_int() != 0 }
    else if val.is_null() { false }
    else { let s = vm.to_string(val); !s.is_empty() && s.to_ascii_lowercase() != "false" && s != "0" };
    Ok(BxValue::new_ptr(vm.string_new(if is_true { "true" } else { "false" }.to_string())))
}

fn format_decimal_with_separator(number: f64, decimal_places: usize) -> String {
    let is_neg = number < 0.0;
    let fixed = format!("{:.prec$}", number.abs(), prec = decimal_places);
    let parts: Vec<&str> = fixed.split('.').collect();
    let digits: Vec<char> = parts[0].chars().collect();
    let len = digits.len();
    let mut with_sep = String::new();
    for (i, &ch) in digits.iter().enumerate() {
        if i > 0 && (len - i) % 3 == 0 { with_sep.push(','); }
        with_sep.push(ch);
    }
    let prefix = if is_neg { "-" } else { "" };
    if decimal_places > 0 && parts.len() > 1 { format!("{}{}.{}", prefix, with_sep, parts[1]) }
    else { format!("{}{}", prefix, with_sep) }
}

fn decimal_format_bif(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() { return Err("decimalFormat() requires a number argument".to_string()); }
    let number = if args[0].is_number() { args[0].as_number() }
    else if args[0].is_int() { args[0].as_int() as f64 }
    else { vm.to_string(args[0]).trim().parse::<f64>().map_err(|_| format!("decimalFormat() expected number, got '{}'", vm.to_string(args[0])))? };
    let dp = if args.len() > 1 && args[1].is_number() { args[1].as_number() as usize } else { 2 };
    Ok(BxValue::new_ptr(vm.string_new(format_decimal_with_separator(number, dp))))
}

fn number_format_bif(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() { return Err("numberFormat() requires a number argument".to_string()); }
    let number = if args[0].is_null() { 0.0 }
    else if args[0].is_number() { args[0].as_number() }
    else if args[0].is_int() { args[0].as_int() as f64 }
    else { let s = vm.to_string(args[0]); if s.is_empty() { 0.0 } else { s.trim().parse::<f64>().map_err(|_| format!("numberFormat() expected number, got '{}'", s))? } };
    let mask = if args.len() > 1 && !args[1].is_null() { vm.to_string(args[1]) } else { String::new() };
    let formatted = if mask.is_empty() { format_decimal_with_separator(number, 0) }
    else { apply_number_mask(number, &mask) };
    Ok(BxValue::new_ptr(vm.string_new(formatted)))
}

fn apply_number_mask(number: f64, mask: &str) -> String {
    let mask = mask.trim();
    if mask == "$" { return format!("${}", format_decimal_with_separator(number, 0)); }
    let has_dec = mask.contains('.');
    let dp = if has_dec { mask.split('.').last().map(|s| s.len()).unwrap_or(0) } else { 0 };
    let is_neg = number < 0.0;
    let fixed = format!("{:.prec$}", number.abs(), prec = dp);
    let parts: Vec<&str> = fixed.split('.').collect();
    let has_comma = mask.contains(',');
    let int_fmt = if has_comma {
        let digits: Vec<char> = parts[0].chars().collect();
        let len = digits.len();
        let mut s = String::new();
        for (i, &ch) in digits.iter().enumerate() { if i > 0 && (len - i) % 3 == 0 { s.push(','); } s.push(ch); }
        s
    } else {
        let mw = mask.split('.').next().unwrap_or(mask).chars().filter(|c| *c == '0' || *c == '9').count();
        if mw > parts[0].len() { format!("{:0>width$}", parts[0], width = mw) } else { parts[0].to_string() }
    };
    let result = if dp > 0 && parts.len() > 1 { format!("{}.{}", int_fmt, parts[1]) } else { int_fmt };
    let prefix = if mask.contains('$') { if is_neg { "-$" } else { "$" } } else if is_neg { "-" } else { "" };
    format!("{}{}", prefix, result)
}

pub fn register_type_format_bifs(bifs: &mut std::collections::HashMap<String, BxNativeFunction>) {
    bifs.insert("isclosure".to_string(), is_closure_bif as BxNativeFunction);
    bifs.insert("iscustomfunction".to_string(), is_custom_function_bif as BxNativeFunction);
    bifs.insert("isdateobject".to_string(), is_date_object_bif as BxNativeFunction);
    bifs.insert("isdebugmode".to_string(), is_debug_mode_bif as BxNativeFunction);
    bifs.insert("isdefined".to_string(), is_defined_bif as BxNativeFunction);
    bifs.insert("isempty".to_string(), is_empty_bif as BxNativeFunction);
    bifs.insert("arrayisempty".to_string(), is_empty_bif as BxNativeFunction);
    bifs.insert("isfileobject".to_string(), is_file_object_bif as BxNativeFunction);
    bifs.insert("isipv6".to_string(), is_ipv6_bif as BxNativeFunction);
    bifs.insert("isleapyear".to_string(), is_leap_year_bif as BxNativeFunction);
    bifs.insert("islocalhost".to_string(), is_local_host_bif as BxNativeFunction);
    bifs.insert("isquery".to_string(), is_query_bif as BxNativeFunction);
    bifs.insert("isvalid".to_string(), is_valid_bif as BxNativeFunction);
    bifs.insert("isxml".to_string(), is_xml_bif as BxNativeFunction);
    bifs.insert("isxmlattribute".to_string(), is_xml_attribute_bif as BxNativeFunction);
    bifs.insert("isxmldoc".to_string(), is_xml_doc_bif as BxNativeFunction);
    bifs.insert("isxmlelement".to_string(), is_xml_element_bif as BxNativeFunction);
    bifs.insert("isxmlnode".to_string(), is_xml_node_bif as BxNativeFunction);
    bifs.insert("isxmlroot".to_string(), is_xml_root_bif as BxNativeFunction);
    bifs.insert("booleanformat".to_string(), boolean_format_bif as BxNativeFunction);
    bifs.insert("truefalseformat".to_string(), boolean_format_bif as BxNativeFunction);
    bifs.insert("decimalformat".to_string(), decimal_format_bif as BxNativeFunction);
    bifs.insert("numberformat".to_string(), number_format_bif as BxNativeFunction);
    bifs.insert("lsnumberformat".to_string(), number_format_bif as BxNativeFunction);
}
