use crate::types::{BxVM, BxValue};
use std::io::{self, Write};

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
        io::stderr().flush().map_err(|e| e.to_string())?;
    } else {
        vm.write_output(&output);
        io::stdout().flush().map_err(|e| e.to_string())?;
    }

    Ok(BxValue::new_null())
}
