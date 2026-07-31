use crate::types::{BxVM, BxValue};

pub fn system_output(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() {
        return Err("systemOutput() expects an object to write".to_string());
    }

    let mut output = vm.to_string(args[0]);
    if args.get(1).map(|value| value.as_bool()).unwrap_or(false) {
        output.push('\n');
    }

    if args.get(2).map(|value| value.as_bool()).unwrap_or(false) {
        eprint!("{output}");
    } else {
        vm.write_output(&output);
    }

    Ok(BxValue::new_null())
}
