use crate::types::{BxVM, BxValue};
use base64::{Engine as _, engine::general_purpose};
use serde_json::Value as JsonValue;

pub fn data_navigate(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() {
        return Err("dataNavigate() expects at least 1 argument".to_string());
    }
    let data = args[0];

    if args.len() < 2 {
        return Ok(data);
    }

    let path_str = vm.to_string(args[1]);
    let parts: Vec<&str> = path_str.split('.').collect();

    let mut current = data;
    for part in parts {
        if part.is_empty() {
            continue;
        }
        if vm.is_struct_value(current) {
            if let Some(id) = current.as_gc_id() {
                if vm.struct_key_exists(id, part) {
                    current = vm.struct_get(id, part);
                } else {
                    return Ok(BxValue::new_null());
                }
            } else {
                return Ok(BxValue::new_null());
            }
        } else if vm.is_array_value(current) {
            if let Some(id) = current.as_gc_id() {
                if let Ok(idx) = part.parse::<usize>() {
                    if idx >= 1 && idx <= vm.array_len(id) {
                        current = vm.array_get(id, idx - 1);
                    } else {
                        return Ok(BxValue::new_null());
                    }
                } else {
                    return Ok(BxValue::new_null());
                }
            } else {
                return Ok(BxValue::new_null());
            }
        } else {
            return Ok(BxValue::new_null());
        }
    }

    Ok(current)
}

pub fn json_prettify(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() {
        return Err("jsonPrettify() expects 1 argument".to_string());
    }
    let json_str = vm.to_string(args[0]);
    let parsed: JsonValue =
        serde_json::from_str(&json_str).map_err(|e| format!("Invalid JSON: {}", e))?;
    let pretty = serde_json::to_string_pretty(&parsed)
        .map_err(|e| format!("Failed to prettify JSON: {}", e))?;
    let s_id = vm.string_new(pretty);
    Ok(BxValue::new_ptr(s_id))
}

pub fn parse_number(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() {
        return Err("parseNumber() expects at least 1 argument".to_string());
    }
    let num_str = vm.to_string(args[0]).trim().to_string();

    let radix_str = if args.len() > 1 {
        let s = vm.to_string(args[1]).to_lowercase();
        if ["bin", "oct", "dec", "hex"].contains(&s.as_str()) {
            Some(s)
        } else if args.len() > 2 {
            Some(vm.to_string(args[2]).to_lowercase())
        } else {
            None
        }
    } else {
        None
    };

    if let Some(radix) = radix_str {
        let radix_num = match radix.as_str() {
            "bin" => 2,
            "oct" => 8,
            "dec" => 10,
            "hex" => 16,
            _ => return Err(format!("Invalid radix: {}", radix)),
        };
        if radix_num == 10 {
            let n: f64 = num_str
                .parse()
                .map_err(|_| format!("Cannot parse [{}] as a number", num_str))?;
            return Ok(BxValue::new_number(n));
        }
        let n = i64::from_str_radix(&num_str, radix_num)
            .map_err(|_| format!("Cannot parse [{}] with radix {}", num_str, radix))?;
        return Ok(BxValue::new_number(n as f64));
    }

    let n: f64 = num_str
        .parse()
        .map_err(|_| format!("Cannot parse [{}] as a number", num_str))?;
    Ok(BxValue::new_number(n))
}

pub fn to_base64(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() {
        return Err("toBase64() expects 1 argument".to_string());
    }
    let val = args[0];

    let bytes = if vm.is_bytes(val) {
        vm.to_bytes(val)?
    } else {
        vm.to_string(val).into_bytes()
    };

    let encoded = general_purpose::STANDARD.encode(&bytes);
    let s_id = vm.string_new(encoded);
    Ok(BxValue::new_ptr(s_id))
}

pub fn to_binary(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() {
        return Err("toBinary() expects 1 argument".to_string());
    }
    let val = args[0];

    if vm.is_bytes(val) {
        return Ok(val);
    }

    let input = vm.to_string(val).trim().to_string();

    let mut cleaned = input;
    let padding = cleaned.len() % 4;
    if padding != 0 {
        cleaned.push_str(&"=".repeat(4 - padding));
    }

    match general_purpose::STANDARD.decode(&cleaned) {
        Ok(bytes) => {
            let id = vm.bytes_new(bytes);
            Ok(BxValue::new_ptr(id))
        }
        Err(e) => Err(format!("Cannot decode base64: {}", e)),
    }
}

pub fn to_modifiable(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() {
        return Err("toModifiable() expects 1 argument".to_string());
    }
    let mut seen = std::collections::HashMap::new();
    super::duplicate_value(vm, args[0], true, &mut seen)
}

pub fn to_numeric(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() {
        return Err("toNumeric() expects at least 1 argument".to_string());
    }
    let val = args[0];

    if val.is_number() {
        return Ok(val);
    }
    if val.is_bool() {
        return Ok(BxValue::new_number(if val.as_bool() { 1.0 } else { 0.0 }));
    }

    let num_str = vm.to_string(val).trim().to_string();

    if args.len() > 1 {
        let radix_val = args[1];
        let radix_num = if radix_val.is_number() {
            radix_val.as_number() as u32
        } else {
            let r = vm.to_string(radix_val).to_lowercase();
            match r.as_str() {
                "bin" => 2,
                "oct" => 8,
                "dec" => 10,
                "hex" => 16,
                _ => {
                    if let Ok(n) = r.parse::<u32>() {
                        n
                    } else {
                        return Err(format!("Invalid radix [{}]", r));
                    }
                }
            }
        };

        if !(2..=36).contains(&radix_num) {
            return Err("Radix must be between 2 and 36".to_string());
        }

        if radix_num == 10 {
            let n: f64 = num_str
                .parse()
                .map_err(|_| format!("Cannot parse [{}] as a number", num_str))?;
            return Ok(BxValue::new_number(n));
        }

        let n = i64::from_str_radix(&num_str, radix_num)
            .map_err(|_| format!("Cannot parse [{}] with radix {}", num_str, radix_num))?;
        return Ok(BxValue::new_number(n as f64));
    }

    let n: f64 = num_str
        .parse()
        .map_err(|_| format!("Cannot convert [{}] to numeric", num_str))?;
    Ok(BxValue::new_number(n))
}

pub fn to_script(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 2 {
        return Err("toScript() expects 2 arguments: (value, jsVarName)".to_string());
    }
    let val = args[0];
    let js_var = vm.to_string(args[1]);

    if val.is_null() {
        let s_id = vm.string_new(format!("{} = null;", js_var));
        return Ok(BxValue::new_ptr(s_id));
    }

    let str_val = vm.to_string(val);
    if val.is_bool() {
        let js_val = if val.as_bool() { "true" } else { "false" };
        let s_id = vm.string_new(format!("{} = {};", js_var, js_val));
        return Ok(BxValue::new_ptr(s_id));
    }

    if val.is_number() {
        let s_id = vm.string_new(format!("{} = {};", js_var, str_val));
        return Ok(BxValue::new_ptr(s_id));
    }

    if vm.is_string_value(val) {
        let escaped = str_val.replace('\\', "\\\\").replace('\'', "\\'");
        let s_id = vm.string_new(format!("{} = '{}';", js_var, escaped));
        return Ok(BxValue::new_ptr(s_id));
    }

    let json_val = bx_to_json_for_script(vm, val);
    let json_str = serde_json::to_string(&json_val)
        .map_err(|e| format!("Failed to serialize to JSON: {}", e))?;
    let s_id = vm.string_new(format!("{} = {};", js_var, json_str));
    Ok(BxValue::new_ptr(s_id))
}

fn bx_to_json_for_script(vm: &dyn BxVM, val: BxValue) -> JsonValue {
    if val.is_null() {
        JsonValue::Null
    } else if val.is_bool() {
        JsonValue::Bool(val.as_bool())
    } else if val.is_number() {
        JsonValue::Number(serde_json::Number::from_f64(val.as_number()).unwrap())
    } else if vm.is_struct_value(val) {
        if let Some(id) = val.as_gc_id() {
            let mut map = serde_json::Map::new();
            for key in vm.struct_key_array(id) {
                let item = vm.struct_get(id, &key);
                map.insert(key, bx_to_json_for_script(vm, item));
            }
            JsonValue::Object(map)
        } else {
            JsonValue::Null
        }
    } else if vm.is_array_value(val) {
        if let Some(id) = val.as_gc_id() {
            let mut vec = Vec::new();
            for i in 0..vm.array_len(id) {
                let item = vm.array_get(id, i);
                vec.push(bx_to_json_for_script(vm, item));
            }
            JsonValue::Array(vec)
        } else {
            JsonValue::Null
        }
    } else if let Some(id) = val.as_gc_id() {
        JsonValue::String(vm.to_string(BxValue::new_ptr(id)))
    } else {
        JsonValue::Null
    }
}

pub fn to_unmodifiable(_vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() {
        return Err("toUnmodifiable() expects 1 argument".to_string());
    }
    Ok(args[0])
}
