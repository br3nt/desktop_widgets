//! A tiny HTTP endpoint on 127.0.0.1 so outside programs can message widgets
//! (`POST /send` with `Content-Type: application/json` and a body like
//! `{"to":"jarvis","msg":"set","agents":4}`) and so the demo HTML panel has somewhere to
//! load from (`GET /examples/panel-demo.html`).
//!
//! Web pages can't use it: requests carrying a foreign `Origin`, a non-JSON body type
//! (what a no-cors browser post can send), or a `Host` other than 127.0.0.1/localhost
//! (DNS rebinding) are refused, and no CORS headers are sent. Requests are handled one at
//! a time with a 2 s deadline; the thread sleeps in accept() when idle (no CPU).

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::{Duration, Instant};

pub const PANEL_DEMO: &str = include_str!("../examples/panel-demo.html");
const MAX_REQUEST: usize = 64 * 1024;

fn head_of(raw: &[u8]) -> Option<(&str, usize)> {
    let end = raw.windows(4).position(|w| w == b"\r\n\r\n")?;
    Some((std::str::from_utf8(&raw[..end]).ok()?, end + 4))
}

fn header<'a>(head: &'a str, name: &str) -> Option<&'a str> {
    head.split("\r\n").skip(1).filter_map(|l| l.split_once(':')).find(|(k, _)| k.trim().eq_ignore_ascii_case(name)).map(|(_, v)| v.trim())
}

/// Parsed request: method, path, body.
pub fn parse_request(raw: &[u8]) -> Option<(String, String, Vec<u8>)> {
    let (head, body_at) = head_of(raw)?;
    let mut first = head.split("\r\n").next()?.split(' ');
    let method = first.next()?.to_string();
    let path = first.next()?.to_string();
    Some((method, path, raw[body_at..].to_vec()))
}

fn content_length(raw: &[u8]) -> Option<usize> {
    let (head, _) = head_of(raw)?;
    Some(header(head, "content-length").and_then(|v| v.parse().ok()).unwrap_or(0))
}

/// Is this request from a local program (not a web page)? `json` = it must carry a JSON body.
pub fn allowed(raw: &[u8], port: u16, json: bool) -> Result<(), &'static str> {
    let (head, _) = head_of(raw).ok_or("bad request")?;
    let host = header(head, "host").ok_or("missing Host")?;
    let local = [format!("127.0.0.1:{port}"), format!("localhost:{port}"), "127.0.0.1".into(), "localhost".into()];
    if !local.iter().any(|h| h.eq_ignore_ascii_case(host)) {
        return Err("forbidden host");
    }
    if let Some(origin) = header(head, "origin") {
        if origin != format!("http://127.0.0.1:{port}") && origin != format!("http://localhost:{port}") {
            return Err("cross-origin requests are not accepted");
        }
    }
    if json {
        let ct = header(head, "content-type").unwrap_or("");
        if !ct.split(';').next().unwrap_or("").trim().eq_ignore_ascii_case("application/json") {
            return Err("Content-Type must be application/json");
        }
    }
    Ok(())
}

fn respond(s: &mut TcpStream, status: &str, ctype: &str, body: &[u8]) {
    let _ = write!(s, "HTTP/1.1 {status}\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len());
    let _ = s.write_all(body);
}

fn handle(mut s: TcpStream, port: u16, deliver: &dyn Fn(serde_json::Value)) {
    let deadline = Instant::now() + Duration::from_secs(2);
    let _ = s.set_write_timeout(Some(Duration::from_secs(2)));
    let mut raw = Vec::new();
    let mut buf = [0u8; 4096];
    loop {
        let left = deadline.saturating_duration_since(Instant::now());
        if left.is_zero() || s.set_read_timeout(Some(left)).is_err() {
            break;
        }
        match s.read(&mut buf) {
            Ok(0) | Err(_) => break,
            Ok(n) => raw.extend_from_slice(&buf[..n]),
        }
        if let (Some(len), Some((_, body_at))) = (content_length(&raw), head_of(&raw)) {
            if raw.len() >= body_at + len {
                break;
            }
        }
        if raw.len() > MAX_REQUEST {
            return respond(&mut s, "413 Payload Too Large", "text/plain", b"too large\n");
        }
    }
    let Some((method, path, body)) = parse_request(&raw) else { return respond(&mut s, "400 Bad Request", "text/plain", b"bad request\n") };
    if let Err(why) = allowed(&raw, port, method == "POST") {
        return respond(&mut s, "403 Forbidden", "text/plain", format!("{why}\n").as_bytes());
    }
    match (method.as_str(), path.as_str()) {
        ("POST", "/send") => match serde_json::from_slice::<serde_json::Value>(&body) {
            Ok(v) if v.get("to").is_some() && v.get("msg").is_some() => {
                deliver(v);
                respond(&mut s, "202 Accepted", "text/plain", b"queued\n")
            }
            _ => respond(&mut s, "400 Bad Request", "text/plain", b"expected JSON {\"to\":..., \"msg\":..., ...}\n"),
        },
        ("GET", "/examples/panel-demo.html") => respond(&mut s, "200 OK", "text/html; charset=utf-8", PANEL_DEMO.as_bytes()),
        ("GET", "/") => respond(&mut s, "200 OK", "text/plain", b"desktop-widgets endpoint. POST /send (application/json) {\"to\":\"jarvis\",\"msg\":\"set\",\"agents\":4}\n"),
        _ => respond(&mut s, "404 Not Found", "text/plain", b"not found\n"),
    }
}

/// Bind now (so panels can load immediately), then serve one request at a time on a thread.
pub fn start(port: u16, deliver: impl Fn(serde_json::Value) + Send + 'static) -> Result<(), String> {
    let listener = TcpListener::bind(("127.0.0.1", port)).map_err(|e| e.to_string())?;
    std::thread::Builder::new()
        .name("endpoint".into())
        .spawn(move || {
            for stream in listener.incoming().flatten() {
                handle(stream, port, &deliver);
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

    fn req(head: &str) -> Vec<u8> {
        format!("{head}\r\n\r\n").into_bytes()
    }

    #[test]
    fn rejects_browser_and_rebinding_requests() {
        let p = 47800;
        let ok = req("POST /send HTTP/1.1\r\nHost: 127.0.0.1:47800\r\nContent-Type: application/json");
        assert_eq!(allowed(&ok, p, true), Ok(()));
        // curl without a Host port and localhost are fine.
        assert_eq!(allowed(&req("POST /send HTTP/1.1\r\nHost: localhost:47800\r\nContent-Type: application/json; charset=utf-8"), p, true), Ok(()));
        // A web page: cross-origin.
        let web = req("POST /send HTTP/1.1\r\nHost: 127.0.0.1:47800\r\nOrigin: https://evil.example\r\nContent-Type: application/json");
        assert!(allowed(&web, p, true).is_err());
        // no-cors form posts can't set a JSON content type.
        let form = req("POST /send HTTP/1.1\r\nHost: 127.0.0.1:47800\r\nContent-Type: text/plain");
        assert!(allowed(&form, p, true).is_err());
        // DNS rebinding: a foreign Host name.
        let rebind = req("POST /send HTTP/1.1\r\nHost: evil.example:47800\r\nContent-Type: application/json");
        assert!(allowed(&rebind, p, true).is_err());
        // GETs of the demo panel don't need a JSON body but still check Host.
        assert_eq!(allowed(&req("GET /x HTTP/1.1\r\nHost: 127.0.0.1:47800"), p, false), Ok(()));
        assert!(allowed(&req("GET /x HTTP/1.1\r\nHost: evil.example"), p, false).is_err());
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
        write!(s, "POST /send HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}", body.len()).unwrap();
        let mut out = String::new();
        s.read_to_string(&mut out).unwrap();
        assert!(out.starts_with("HTTP/1.1 202"));
        let v = rx.recv_timeout(std::time::Duration::from_secs(2)).unwrap();
        assert_eq!(v["agents"], 4);

        let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
        write!(s, "GET /examples/panel-demo.html HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\n\r\n").unwrap();
        let mut out = String::new();
        s.read_to_string(&mut out).unwrap();
        assert!(out.starts_with("HTTP/1.1 200") && out.contains("@container"));
        assert!(!out.contains("Access-Control-Allow-Origin"));

        // A browser no-cors post is refused and never delivered.
        let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
        write!(s, "POST /send HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nOrigin: https://evil.example\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{body}", body.len()).unwrap();
        let mut out = String::new();
        s.read_to_string(&mut out).unwrap();
        assert!(out.starts_with("HTTP/1.1 403"));
        assert!(rx.recv_timeout(std::time::Duration::from_millis(200)).is_err());
    }
}
