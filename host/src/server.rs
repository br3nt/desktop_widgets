//! A tiny HTTP endpoint on 127.0.0.1 so outside programs can message widgets
//! (`POST /send` with `{"to":"jarvis","msg":"set","agents":4}`) and so the demo HTML
//! panel has somewhere to load from (`GET /examples/panel-demo.html`).
//! One short-lived thread per request; blocks in accept() when idle (no CPU).

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

pub const PANEL_DEMO: &str = include_str!("../examples/panel-demo.html");

/// Parsed request: method, path, body.
pub fn parse_request(raw: &[u8]) -> Option<(String, String, Vec<u8>)> {
    let head_end = raw.windows(4).position(|w| w == b"\r\n\r\n")?;
    let head = std::str::from_utf8(&raw[..head_end]).ok()?;
    let mut lines = head.split("\r\n");
    let mut first = lines.next()?.split(' ');
    let method = first.next()?.to_string();
    let path = first.next()?.to_string();
    let body = raw[head_end + 4..].to_vec();
    Some((method, path, body))
}

fn content_length(raw: &[u8]) -> Option<usize> {
    let head_end = raw.windows(4).position(|w| w == b"\r\n\r\n")?;
    let head = std::str::from_utf8(&raw[..head_end]).ok()?;
    head.split("\r\n")
        .filter_map(|l| l.split_once(':'))
        .find(|(k, _)| k.trim().eq_ignore_ascii_case("content-length"))
        .and_then(|(_, v)| v.trim().parse().ok())
        .or(Some(0))
}

fn respond(s: &mut TcpStream, status: &str, ctype: &str, body: &[u8]) {
    let _ = write!(s, "HTTP/1.1 {status}\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nConnection: close\r\n\r\n", body.len());
    let _ = s.write_all(body);
}

fn handle(mut s: TcpStream, deliver: &(dyn Fn(serde_json::Value) + Send + Sync)) {
    let _ = s.set_read_timeout(Some(std::time::Duration::from_secs(2)));
    let mut raw = Vec::new();
    let mut buf = [0u8; 4096];
    loop {
        match s.read(&mut buf) {
            Ok(0) | Err(_) => break,
            Ok(n) => raw.extend_from_slice(&buf[..n]),
        }
        if let (Some(len), Some(h)) = (content_length(&raw), raw.windows(4).position(|w| w == b"\r\n\r\n")) {
            if raw.len() >= h + 4 + len {
                break;
            }
        }
        if raw.len() > 1 << 20 {
            break;
        }
    }
    let Some((method, path, body)) = parse_request(&raw) else { return respond(&mut s, "400 Bad Request", "text/plain", b"bad request\n") };
    match (method.as_str(), path.as_str()) {
        ("POST", "/send") => match serde_json::from_slice::<serde_json::Value>(&body) {
            Ok(v) if v.get("to").is_some() && v.get("msg").is_some() => {
                deliver(v);
                respond(&mut s, "202 Accepted", "text/plain", b"queued\n")
            }
            _ => respond(&mut s, "400 Bad Request", "text/plain", b"expected JSON {\"to\":..., \"msg\":..., ...}\n"),
        },
        ("GET", "/examples/panel-demo.html") => respond(&mut s, "200 OK", "text/html; charset=utf-8", PANEL_DEMO.as_bytes()),
        ("GET", "/") => respond(&mut s, "200 OK", "text/plain", b"desktop-widgets endpoint. POST /send {\"to\":\"jarvis\",\"msg\":\"set\",\"agents\":4}\n"),
        _ => respond(&mut s, "404 Not Found", "text/plain", b"not found\n"),
    }
}

/// Bind now (so panels can load immediately), then serve on a background thread.
pub fn start(port: u16, deliver: impl Fn(serde_json::Value) + Send + Sync + 'static) -> Result<(), String> {
    let listener = TcpListener::bind(("127.0.0.1", port)).map_err(|e| e.to_string())?;
    let deliver = std::sync::Arc::new(deliver);
    std::thread::Builder::new()
        .name("endpoint".into())
        .spawn(move || {
            for stream in listener.incoming().flatten() {
                let d = deliver.clone();
                let _ = std::thread::Builder::new().stack_size(64 * 1024).spawn(move || handle(stream, &*d));
            }
        })
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_post_with_body() {
        let raw = b"POST /send HTTP/1.1\r\nHost: x\r\nContent-Length: 2\r\n\r\n{}";
        assert_eq!(content_length(raw), Some(2));
        let (m, p, b) = parse_request(raw).unwrap();
        assert_eq!((m.as_str(), p.as_str(), b.as_slice()), ("POST", "/send", b"{}".as_slice()));
    }

    #[test]
    fn end_to_end_send_and_static_file() {
        use std::sync::mpsc;
        let (tx, rx) = mpsc::channel();
        let tx = std::sync::Mutex::new(tx);
        let port = 47911;
        start(port, move |v| tx.lock().unwrap().send(v).unwrap()).unwrap();
        let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
        let body = r#"{"to":"jarvis","msg":"set","agents":4}"#;
        write!(s, "POST /send HTTP/1.1\r\nContent-Length: {}\r\n\r\n{body}", body.len()).unwrap();
        let mut out = String::new();
        s.read_to_string(&mut out).unwrap();
        assert!(out.starts_with("HTTP/1.1 202"));
        let v = rx.recv_timeout(std::time::Duration::from_secs(2)).unwrap();
        assert_eq!(v["agents"], 4);

        let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
        write!(s, "GET /examples/panel-demo.html HTTP/1.1\r\n\r\n").unwrap();
        let mut out = String::new();
        s.read_to_string(&mut out).unwrap();
        assert!(out.starts_with("HTTP/1.1 200") && out.contains("@container"));
    }
}
