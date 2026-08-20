#[cfg(feature = "bif-http")]
use crate::types::{BxVM, BxValue, NativeFutureValue};

#[cfg(all(feature = "bif-http", not(target_arch = "wasm32")))]
use std::fs::File;
#[cfg(all(feature = "bif-http", not(target_arch = "wasm32")))]
use std::io::copy;

#[cfg(all(feature = "bif-http", target_arch = "wasm32", feature = "js"))]
use web_sys::{Request, RequestInit, RequestMode};

#[cfg(feature = "bif-http")]
fn request_headers(vm: &mut dyn BxVM, spec: usize) -> Vec<(String, String)> {
    let headers = vm.struct_get(spec, "headers");
    let Some(id) = headers.as_gc_id() else {
        return Vec::new();
    };
    if !vm.is_struct_value(headers) {
        return Vec::new();
    }
    vm.struct_key_array(id)
        .into_iter()
        .map(|key| {
            let value = vm.to_string(vm.struct_get(id, &key));
            (key, value)
        })
        .collect()
}

#[cfg(feature = "bif-http")]
fn request_body(vm: &mut dyn BxVM, spec: usize) -> Option<String> {
    if !vm.struct_key_exists(spec, "body") {
        return None;
    }
    let body = vm.struct_get(spec, "body");
    if body.is_null() {
        return None;
    }
    Some(vm.to_string(body))
}

#[cfg(all(feature = "bif-http", not(target_arch = "wasm32")))]
fn send_http(
    url: &str,
    method: &str,
    headers: &[(String, String)],
    body: Option<String>,
) -> Result<reqwest::blocking::Response, String> {
    let client = reqwest::blocking::Client::new();
    let mut request = match method {
        "GET" => client.get(url),
        "POST" => client.post(url),
        "PUT" => client.put(url),
        "DELETE" => client.delete(url),
        _ => return Err(format!("Unsupported HTTP method: {method}")),
    };
    for (name, value) in headers {
        request = request.header(name.as_str(), value.as_str());
    }
    if let Some(body) = body {
        request = request.body(body);
    }
    request
        .send()
        .map_err(|e| format!("HTTP request failed: {e}"))
}

#[cfg(feature = "bif-http")]
pub fn http_bif(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    let mut url = String::new();
    let mut method = "GET".to_string();
    #[cfg(not(target_arch = "wasm32"))]
    let mut path = None;
    let mut headers = Vec::new();
    let mut body = None;

    if args.len() == 1 && args[0].as_gc_id().is_some() {
        let id = args[0].as_gc_id().unwrap();
        if vm.struct_key_exists(id, "url") {
            url = vm.to_string(vm.struct_get(id, "url"));
            let m = vm.to_string(vm.struct_get(id, "method"));
            if !m.is_empty() && m != "null" {
                method = m.to_uppercase();
            }
            let p = vm.to_string(vm.struct_get(id, "path"));
            if !p.is_empty() && p != "null" {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    path = Some(p);
                }
            }
            headers = request_headers(vm, id);
            body = request_body(vm, id);
        } else {
            url = vm.to_string(args[0]);
        }
    } else if !args.is_empty() {
        url = vm.to_string(args[0]);
        if args.len() > 1 {
            method = vm.to_string(args[1]).to_uppercase();
        }
        if args.len() > 2 {
            #[cfg(not(target_arch = "wasm32"))]
            {
                path = Some(vm.to_string(args[2]));
            }
        }
    }

    if url.is_empty() || url == "null" {
        return Err("http() requires a URL".to_string());
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let handle = vm.native_future_new();
        let future = handle.future();
        std::thread::spawn(move || {
            let result = (|| {
                let mut response = send_http(&url, &method, &headers, body)?;
                let status = response.status().as_u16();
                if let Some(p) = path {
                    let mut file = File::create(&p)
                        .map_err(|e| format!("Failed to create file: {e}"))?;
                    copy(&mut response, &mut file)
                        .map_err(|e| format!("Failed to download file: {e}"))?;
                    Ok(serde_json::json!({ "status": status, "file_path": p }))
                } else {
                    let text = response
                        .text()
                        .map_err(|e| format!("Failed to read response body: {e}"))?;
                    Ok(serde_json::json!({
                        "status": status,
                        "file_content": text,
                        "body": text
                    }))
                }
            })();
            match result {
                Ok(value) => {
                    let _ = handle.resolve(NativeFutureValue::String(value.to_string()));
                }
                Err(error) => {
                    let _ = handle.reject(NativeFutureValue::Error { message: error });
                }
            }
        });
        Ok(future)
    }

    #[cfg(all(target_arch = "wasm32", feature = "js"))]
    {
        let _ = (headers, body);
        let opts = RequestInit::new();
        opts.set_method(&method);
        opts.set_mode(RequestMode::Cors);

        let request = Request::new_with_str_and_init(&url, &opts)
            .map_err(|e| format!("Failed to create request: {:?}", e))?;

        let window = web_sys::window().ok_or("No global window object found")?;
        let _request_promise = window.fetch_with_request(&request);

        Err("http() on WASM (fetch) is only supported in async contexts. Return values are not yet synchronous on the web.".to_string())
    }

    #[cfg(all(target_arch = "wasm32", not(feature = "js")))]
    {
        let _ = (method, headers, body);
        Err("http() on wasm requires the `js` feature for browser fetch support.".to_string())
    }
}

#[cfg(all(test, feature = "bif-http", not(target_arch = "wasm32")))]
mod tests {
    use super::send_http;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    #[test]
    fn posts_headers_and_body() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut sock, _) = listener.accept().unwrap();
            let mut buf = vec![0_u8; 8192];
            let n = sock.read(&mut buf).unwrap();
            let req = String::from_utf8_lossy(&buf[..n]).into_owned();
            sock.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok")
                .unwrap();
            req
        });

        let response = send_http(
            &format!("http://{addr}/v1/chat/completions"),
            "POST",
            &[
                ("Authorization".into(), "Bearer testdev".into()),
                ("Content-Type".into(), "application/json".into()),
            ],
            Some(r#"{"model":"x"}"#.into()),
        )
        .unwrap();
        assert_eq!(response.status().as_u16(), 200);
        assert_eq!(response.text().unwrap(), "ok");

        let req = server.join().unwrap();
        let lower = req.to_ascii_lowercase();
        assert!(lower.contains("authorization: bearer testdev"), "{req}");
        assert!(lower.contains("content-type: application/json"), "{req}");
        assert!(req.contains(r#"{"model":"x"}"#), "{req}");
    }
}
