use crate::types::{BxVM, BxValue};
use toml::Value as TomlValue;

pub fn toml_deserialize(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() != 1 {
        return Err("deserializeTOML() expects 1 string argument".to_string());
    }
    if !vm.is_string_value(args[0]) {
        return Err("deserializeTOML() expects a string argument".to_string());
    }

    let value = toml::from_str(&vm.to_string(args[0]))
        .map_err(|error| format!("Failed to parse TOML: {error}"))?;
    Ok(toml_to_bx(vm, value))
}

fn toml_to_bx(vm: &mut dyn BxVM, value: TomlValue) -> BxValue {
    match value {
        TomlValue::String(value) => BxValue::new_ptr(vm.string_new(value)),
        TomlValue::Integer(value) => BxValue::new_number(value as f64),
        TomlValue::Float(value) => BxValue::new_number(value),
        TomlValue::Boolean(value) => BxValue::new_bool(value),
        TomlValue::Datetime(value) => BxValue::new_ptr(vm.string_new(value.to_string())),
        TomlValue::Array(values) => {
            let id = vm.array_new();
            for value in values {
                let value = toml_to_bx(vm, value);
                vm.array_push(id, value);
            }
            BxValue::new_ptr(id)
        }
        TomlValue::Table(values) => {
            let id = vm.struct_new();
            for (key, value) in values {
                let value = toml_to_bx(vm, value);
                vm.struct_set(id, &key, value);
            }
            BxValue::new_ptr(id)
        }
    }
}
