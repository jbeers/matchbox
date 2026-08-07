use crate::types::{BxNativeFunction, BxVM, BxValue};

pub fn register_net_bifs(bifs: &mut std::collections::HashMap<String, BxNativeFunction>) {
    bifs.insert(
        "getlocalhostip".to_string(),
        get_localhost_ip as BxNativeFunction,
    );
}

fn get_localhost_ip(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    let ip = local_ip_address();
    if args.first().is_some_and(|value| value.as_bool()) {
        let array_id = vm.array_new();
        let ip_id = vm.string_new(ip);
        vm.array_push(array_id, BxValue::new_ptr(ip_id));
        Ok(BxValue::new_ptr(array_id))
    } else {
        Ok(BxValue::new_ptr(vm.string_new(ip)))
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn local_ip_address() -> String {
    std::net::UdpSocket::bind("0.0.0.0:0")
        .and_then(|socket| {
            socket.connect("8.8.8.8:80")?;
            socket.local_addr()
        })
        .map(|address| address.ip().to_string())
        .unwrap_or_else(|_| "127.0.0.1".to_string())
}

#[cfg(target_arch = "wasm32")]
fn local_ip_address() -> String {
    "127.0.0.1".to_string()
}
