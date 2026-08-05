use crate::types::{BxVM, BxValue};
use serde_yaml::Value as YamlValue;

pub fn yaml_deserialize(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() != 1 {
        return Err("deserializeYAML() expects 1 string argument".to_string());
    }
    if !vm.is_string_value(args[0]) {
        return Err("deserializeYAML() expects a string argument".to_string());
    }

    let yaml_str = vm.to_string(args[0]);
    let yaml_val: YamlValue =
        serde_yaml::from_str(&yaml_str).map_err(|e| format!("Failed to parse YAML: {}", e))?;

    yaml_to_bx(vm, yaml_val)
}

fn yaml_to_bx(vm: &mut dyn BxVM, val: YamlValue) -> Result<BxValue, String> {
    match val {
        YamlValue::Null => Ok(BxValue::new_null()),
        YamlValue::Bool(b) => Ok(BxValue::new_bool(b)),
        YamlValue::Number(n) => n
            .as_f64()
            .map(BxValue::new_number)
            .ok_or_else(|| "deserializeYAML() encountered an unsupported YAML number".to_string()),
        YamlValue::String(s) => Ok(BxValue::new_ptr(vm.string_new(s))),
        YamlValue::Sequence(seq) => {
            let id = vm.array_new();
            for item in seq {
                let item = yaml_to_bx(vm, item)?;
                vm.array_push(id, item);
            }
            Ok(BxValue::new_ptr(id))
        }
        YamlValue::Mapping(mapping) => {
            let id = vm.struct_new();
            for (key, value) in mapping {
                let key = match key {
                    YamlValue::String(key) => key,
                    _ => {
                        return Err(
                            "deserializeYAML() only supports string mapping keys; found a non-string key"
                                .to_string(),
                        );
                    }
                };
                let value = yaml_to_bx(vm, value)?;
                vm.struct_set(id, &key, value);
            }
            Ok(BxValue::new_ptr(id))
        }
        _ => Err("deserializeYAML() encountered an unsupported YAML value".to_string()),
    }
}
