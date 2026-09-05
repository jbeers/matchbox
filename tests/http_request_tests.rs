#![cfg(all(feature = "bif-http", not(target_arch = "wasm32")))]

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::process::{Command, Output};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

fn serve(
    listener: TcpListener,
    maximum_requests: usize,
    respond: impl Fn(&str, &mut TcpStream) + Send + 'static,
) -> (SocketAddr, JoinHandle<Vec<String>>) {
    let address = listener.local_addr().unwrap();
    listener.set_nonblocking(true).unwrap();
    let worker = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut requests = Vec::new();
        while requests.len() < maximum_requests && Instant::now() < deadline {
            let (mut socket, _) = match listener.accept() {
                Ok(connection) => connection,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(10));
                    continue;
                }
                Err(error) => panic!("accept HTTP request: {error}"),
            };
            socket
                .set_read_timeout(Some(Duration::from_secs(2)))
                .unwrap();
            let mut header = Vec::new();
            while !header.ends_with(b"\r\n\r\n") {
                let mut byte = [0];
                socket.read_exact(&mut byte).unwrap();
                header.push(byte[0]);
                assert!(header.len() < 8192, "HTTP test header exceeded limit");
            }
            let mut request = String::from_utf8(header).unwrap();
            let length = request
                .lines()
                .find_map(|line| {
                    let (name, value) = line.split_once(':')?;
                    name.eq_ignore_ascii_case("content-length")
                        .then(|| value.trim().parse::<usize>().unwrap())
                })
                .unwrap_or(0);
            let mut body = vec![0; length];
            socket.read_exact(&mut body).unwrap();
            request.push_str(&String::from_utf8(body).unwrap());
            respond(&request, &mut socket);
            requests.push(request);
        }
        requests
    });
    (address, worker)
}

fn clear_proxies(command: &mut Command) {
    for key in [
        "HTTP_PROXY",
        "HTTPS_PROXY",
        "ALL_PROXY",
        "NO_PROXY",
        "http_proxy",
        "https_proxy",
        "all_proxy",
        "no_proxy",
    ] {
        command.env_remove(key);
    }
}

fn run(source: &str, environment: &[(&str, String)]) -> Output {
    let directory = tempfile::tempdir().unwrap();
    let script = directory.path().join("http_test.bxs");
    std::fs::write(&script, source).unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_matchbox"));
    command.arg(script);
    clear_proxies(&mut command);
    for (key, value) in environment {
        command.env(key, value);
    }
    command.output().unwrap()
}

fn assert_success(output: Output) {
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn http_request_controls_work_in_a_standalone_native_binary() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let directory = tempfile::tempdir().unwrap();
    let script = directory.path().join("probe.bxs");
    let executable = directory
        .path()
        .join(format!("probe{}", std::env::consts::EXE_SUFFIX));
    std::fs::write(
        &script,
        format!(
            r#"
        result = jsonDeserialize(http({{ url: "http://localhost:{port}/", timeout: 2,
            connectionTimeout: 1, redirect: false, noProxy: true, ipv4Only: true }}).get());
        if (result.status != 302 || result.body != "native") throw "native HTTP controls failed";
    "#,
            port = address.port()
        ),
    )
    .unwrap();
    assert_success(
        Command::new(env!("CARGO_BIN_EXE_matchbox"))
            .args(["--target", "native", "--output"])
            .arg(&executable)
            .arg(script)
            .output()
            .unwrap(),
    );
    let (_, server) = serve(listener, 1, |_, socket| {
        socket.write_all(b"HTTP/1.1 302 Found\r\nLocation: /not-followed\r\nContent-Length: 6\r\nConnection: close\r\n\r\nnative").unwrap();
    });
    let mut command = Command::new(executable);
    clear_proxies(&mut command);
    assert_success(
        command
            .env("HTTP_PROXY", "http://127.0.0.1:0")
            .output()
            .unwrap(),
    );
    assert_eq!(server.join().unwrap().len(), 1);
}

#[test]
fn http_invalid_options_fail_before_network_or_file_side_effects() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let address = listener.local_addr().unwrap();
    let directory = tempfile::tempdir().unwrap();
    let download = directory.path().join("existing.txt");
    std::fs::write(&download, "keep existing file").unwrap();
    assert_success(run(
        &format!(
            r#"
        for (option in ["timeout", "connectionTimeout", "redirect", "noProxy", "ipv4Only"]) {{
            values = [null, "test-option-secret", {{}}, []];
            if (option == "timeout" || option == "connectionTimeout") {{
                values.append(false);
                values.append(-1);
                values.append(10 ^ 30);
                values.append(10 ^ 19);
                values.append(10 ^ 400);
                values.append((-1) ^ 0.5);
            }} else {{
                values.append(1);
                values.append("true");
            }}
            for (value in values) {{
                spec = {{ url: "http://{address}/", path: {download} }};
                spec[option] = value;
                message = "";
                try {{ http(spec); }} catch (any error) {{ message = error.message; }}
                if (findNoCase(option, message) == 0 || findNoCase("must be", message) == 0)
                    throw "invalid HTTP option must fail synchronously";
                if (find("test-option-secret", message) > 0) throw "invalid options must not be echoed";
            }}
        }}
    "#,
            download = serde_json::to_string(&download).unwrap()
        ),
        &[],
    ));
    assert_eq!(
        std::fs::read_to_string(download).unwrap(),
        "keep existing file"
    );
    assert_eq!(
        listener.accept().unwrap_err().kind(),
        std::io::ErrorKind::WouldBlock
    );
}

#[test]
fn http_existing_calls_and_response_shapes_work_with_request_controls() {
    let directory = tempfile::tempdir().unwrap();
    let struct_download = directory.path().join("struct.txt");
    let positional_download = directory.path().join("positional.txt");
    let (address, server) = serve(
        TcpListener::bind("127.0.0.1:0").unwrap(),
        8,
        |request, socket| {
            let response = if request.starts_with("GET /redirect ") {
                "HTTP/1.1 302 Found\r\nLocation: /missing\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            } else if request.starts_with("GET /missing ") {
                "HTTP/1.1 404 Not Found\r\nContent-Length: 7\r\nConnection: close\r\n\r\nmissing"
            } else {
                "HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok"
            };
            socket.write_all(response.as_bytes()).unwrap();
        },
    );
    assert_success(run(
        &format!(
            r#"
        result = jsonDeserialize(http("http://{address}/").get());
        if (result.status != 200 || result.body != "ok" || result.file_content != "ok")
            throw "positional URL and response fields must still work";
        result = jsonDeserialize(http({{ url: "http://{address}/", method: "post",
            headers: {{ Authorization: "Bearer loopback-test-secret", "Content-Type": "application/json" }},
            body: '{{"model":"test"}}' }}).get());
        if (result.status != 200) throw "POST must still work";
        result = jsonDeserialize(http({{ url: "http://localhost:{port}/", path: {struct_download},
            TiMeOuT: 0, ConnectionTimeout: 0, NoProxy: true, IPv4Only: true, Redirect: false }}).get());
        if (result.status != 200 || result.file_path != {struct_download}) throw "struct download failed";
        result = jsonDeserialize(http("http://{address}/", "GET", {positional_download}).get());
        if (result.status != 200 || result.file_path != {positional_download}) throw "positional download failed";
        result = jsonDeserialize(http({{ url: "http://{address}/redirect" }}).get());
        if (result.status != 404 || result.body != "missing") throw "default redirects or error response failed";
        result = jsonDeserialize(http({{ url: "http://{address}/redirect", redirect: true }}).get());
        if (result.status != 404) throw "explicit redirect=true failed";
    "#,
            port = address.port(),
            struct_download = serde_json::to_string(&struct_download).unwrap(),
            positional_download = serde_json::to_string(&positional_download).unwrap()
        ),
        &[],
    ));
    assert_eq!(std::fs::read_to_string(struct_download).unwrap(), "ok");
    assert_eq!(std::fs::read_to_string(positional_download).unwrap(), "ok");
    let requests = server.join().unwrap();
    assert_eq!(requests.len(), 8);
    let post = requests
        .iter()
        .find(|request| request.starts_with("POST "))
        .unwrap();
    assert!(post
        .to_ascii_lowercase()
        .contains("authorization: bearer loopback-test-secret"));
    assert!(post
        .to_ascii_lowercase()
        .contains("content-type: application/json"));
    assert!(post.ends_with(r#"{"model":"test"}"#));
}

#[test]
fn http_redirect_following_is_bounded() {
    let (address, server) = serve(
        TcpListener::bind("127.0.0.1:0").unwrap(),
        11,
        |_, socket| {
            socket.write_all(b"HTTP/1.1 302 Found\r\nLocation: /loop\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap();
        },
    );
    assert_success(run(
        &format!(
            r#"
        message = "";
        try {{ http({{ url: "http://{address}/loop", timeout: 2 }}).get(); }}
        catch (any error) {{ message = error.message; }}
        if (findNoCase("redirect", message) == 0) throw "redirect loops must fail with a redirect error";
    "#
        ),
        &[],
    ));
    assert_eq!(server.join().unwrap().len(), 11);
}

#[test]
fn http_errors_do_not_echo_credentials_or_request_data() {
    assert_success(run(
        r#"
        for (authorization in ["Bearer header-secret", "Bearer header-secret" & chr(10)]) {
            message = "";
            try {
                http({ url: "http://testuser:url-secret@127.0.0.1:0/?key=query-secret",
                    method: "POST", headers: { Authorization: authorization }, body: "body-secret",
                    timeout: 1, noProxy: true }).get();
            } catch (any error) { message = error.message; }
            if (findNoCase("HTTP request failed", message) == 0) throw "request should have failed";
            for (secret in ["header-secret", "url-secret", "query-secret", "body-secret"]) {
                if (find(secret, message) > 0) throw "HTTP diagnostics must not echo request data";
            }
        }
    "#,
        &[],
    ));
}

#[test]
fn http_ipv4_only_rejects_ipv6_including_redirects() {
    let listener = match TcpListener::bind("[::1]:0") {
        Ok(listener) => listener,
        Err(error)
            if matches!(
                error.kind(),
                std::io::ErrorKind::AddrNotAvailable | std::io::ErrorKind::Unsupported
            ) =>
        {
            eprintln!("IPv6 loopback unavailable; skipping IPv6-only assertions: {error}");
            return;
        }
        Err(error) => panic!("bind IPv6 loopback: {error}"),
    };
    let (ipv6, target) = serve(listener, 3, |_, socket| {
        socket
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok")
            .unwrap();
    });
    let (ipv4, redirect) = serve(
        TcpListener::bind("127.0.0.1:0").unwrap(),
        1,
        move |_, socket| {
            write!(socket, "HTTP/1.1 302 Found\r\nLocation: http://{ipv6}/\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap();
        },
    );
    assert_success(run(
        &format!(
            r#"
        result = jsonDeserialize(http({{ url: "http://{ipv6}/", noProxy: true }}).get());
        if (result.status != 200) throw "IPv6 loopback must work by default";
        for (url in ["http://{ipv6}/", "http://{ipv4}/"]) {{
            rejected = false;
            try {{
                http({{ url: url, ipv4Only: true, noProxy: true, timeout: 2 }}).get();
            }} catch (any error) {{
                rejected = findNoCase("HTTP request failed", error.message) > 0;
            }}
            if (!rejected) throw "ipv4Only must not connect to IPv6";
        }}
    "#
        ),
        &[],
    ));
    assert_eq!(redirect.join().unwrap().len(), 1);
    assert_eq!(
        target.join().unwrap().len(),
        1,
        "IPv4-only calls must not reach the IPv6 listener"
    );
}

#[test]
fn http_no_proxy_bypasses_environment_proxies_per_request() {
    let (address, direct) = serve(TcpListener::bind("127.0.0.1:0").unwrap(), 1, |_, socket| {
        socket
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 6\r\nConnection: close\r\n\r\ndirect")
            .unwrap();
    });
    let (proxy_address, proxy) = serve(
        TcpListener::bind("127.0.0.1:0").unwrap(),
        2,
        |_, socket| {
            socket.write_all(b"HTTP/1.1 407 Proxy Authentication Required\r\nContent-Length: 5\r\nConnection: close\r\n\r\nproxy").unwrap();
        },
    );
    let environment: Vec<_> = [
        "HTTP_PROXY",
        "HTTPS_PROXY",
        "ALL_PROXY",
        "http_proxy",
        "https_proxy",
        "all_proxy",
    ]
    .map(|key| (key, format!("http://{proxy_address}")))
    .into();
    assert_success(run(
        &format!(
            r#"
        result = jsonDeserialize(http({{ url: "http://{address}/", timeout: 2 }}).get());
        if (result.status != 407 || result.body != "proxy") throw "default must honor proxy settings";
        result = jsonDeserialize(http({{ url: "http://{address}/", noProxy: true, timeout: 2,
            headers: {{ Authorization: "Bearer loopback-test-secret" }} }}).get());
        if (result.status != 200 || result.body != "direct") throw "noProxy must connect directly";
        result = jsonDeserialize(http({{ url: "http://{address}/", noProxy: false, timeout: 2 }}).get());
        if (result.status != 407) throw "noProxy must not change proxies for later requests";
    "#
        ),
        &environment,
    ));
    let direct_requests = direct.join().unwrap();
    assert_eq!(direct_requests.len(), 1);
    assert!(direct_requests[0]
        .to_ascii_lowercase()
        .contains("authorization: bearer loopback-test-secret"));
    let proxy_requests = proxy.join().unwrap();
    assert_eq!(proxy_requests.len(), 2);
    assert!(proxy_requests
        .iter()
        .all(|request| !request.contains("loopback-test-secret")));
}

#[test]
fn http_connection_timeout_bounds_a_stalled_tls_handshake() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let server = thread::spawn(move || {
        let (_socket, _) = listener.accept().unwrap();
        thread::sleep(Duration::from_millis(500));
    });
    assert_success(run(
        &format!(
            r#"
        timedOut = false;
        try {{
            http({{ url: "https://{address}/", connectionTimeout: 0.1, timeout: 2 }}).get();
        }} catch (any error) {{
            timedOut = findNoCase("timed out", error.message) > 0;
        }}
        if (!timedOut) throw "connectionTimeout must bound the TLS handshake";
    "#
        ),
        &[],
    ));
    server.join().unwrap();
}

#[test]
fn http_timeout_covers_headers_and_the_entire_response_body() {
    let directory = tempfile::tempdir().unwrap();
    let download = directory.path().join("download.txt");
    for phase in ["headers", "body", "download"] {
        let (address, server) = serve(
            TcpListener::bind("127.0.0.1:0").unwrap(),
            1,
            move |_, socket| {
                thread::sleep(Duration::from_millis(if phase == "headers" {
                    400
                } else {
                    150
                }));
                let _ = socket.write_all(
                    b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\n",
                );
                if phase != "headers" {
                    thread::sleep(Duration::from_millis(150));
                }
                let _ = socket.write_all(b"ok");
            },
        );
        let path = if phase == "download" {
            format!(", path: {}", serde_json::to_string(&download).unwrap())
        } else {
            String::new()
        };
        assert_success(run(
            &format!(
                r#"
            timedOut = false;
            try {{
                http({{ url: "http://{address}/", timeout: 0.25 {path} }}).get();
            }} catch (any error) {{
                timedOut = findNoCase("timed out", error.message) > 0;
            }}
            if (!timedOut) throw "HTTP {phase} must observe the total request timeout";
        "#
            ),
            &[],
        ));
        assert_eq!(server.join().unwrap().len(), 1);
    }
}

#[test]
fn http_redirect_can_be_disabled() {
    let (address, server) = serve(
        TcpListener::bind("127.0.0.1:0").unwrap(),
        2,
        |request, socket| {
            let response = if request.starts_with("GET /redirect ") {
                "HTTP/1.1 302 Found\r\nLocation: /target\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            } else {
                "HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok"
            };
            socket.write_all(response.as_bytes()).unwrap();
        },
    );
    let output = run(
        &format!(
            r#"
        result = jsonDeserialize(http({{ url: "http://{address}/redirect", redirect: false }}).get());
        if (result.status != 302) throw "redirect=false must return the original response";
    "#
        ),
        &[],
    );
    assert_success(output);
    assert_eq!(
        server.join().unwrap().len(),
        1,
        "redirect target must not be requested"
    );
}
