use std::collections::HashMap;

pub fn get_stub(target: &str) -> Option<&'static [u8]> {
    let mut stubs: HashMap<&str, &[u8]> = HashMap::new();
    stubs.insert("wasi", include_bytes!("../stubs/runner_stub_wasip1.wasm"));
    stubs.insert("wasm32-wasip1", include_bytes!("../stubs/runner_stub_wasip1.wasm"));
    stubs.insert("host", include_bytes!("../stubs/runner_stub_x86_64-unknown-linux-gnu"));
    stubs.insert("x86_64-unknown-linux-gnu", include_bytes!("../stubs/runner_stub_x86_64-unknown-linux-gnu"));
    stubs.get(target).copied()
}
