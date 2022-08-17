use poolio::{PanicSwitch, ThreadPool};

use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

const SIZE: usize = 4;

#[test]
fn test_http() {
    const ADDR: &str = "127.0.0.1:7878";

    thread::spawn(|| http_server(ADDR));

    let pool = threadpool::ThreadPool::new(SIZE);
    for _ in 0..10 {
        pool.execute(|| http_client(ADDR, &Method::Get));
        pool.execute(|| http_client(ADDR, &Method::Head));
        pool.execute(|| http_client(ADDR, &Method::Put));
    }
}

enum Method {
    Get,
    Head,
    Put,
}

fn http_client(addr: &str, method: &Method) {
    let mut stream = TcpStream::connect(addr).unwrap();

    let request = match method {
        Method::Get => request("GET", "/"),
        Method::Head => request("HEAD", "/"),
        Method::Put => request("PUT", "/"),
    };

    stream.write_all(request.as_bytes()).unwrap();
    stream.flush().unwrap();

    let mut buffer = [0; 1024];

    let _ = stream.read(&mut buffer).unwrap();

    let ok = b"HTTP/1.1 200 OK\r\n";
    let err = b"HTTP/1.1 404 NOT FOUND\r\n";

    match method {
        Method::Put | Method::Get => assert!(buffer.starts_with(ok)),
        Method::Head => assert!(buffer.starts_with(err)),
    };
}

fn request(method: &str, url: &str) -> String {
    let request = format!("{} {} HTTP/1.1", method, url);
    let headers = "";
    let body = "";
    format!("{}\r\n{}\r\n{}", request, headers, body)
}

fn http_server(addr: &str) {
    let pool = ThreadPool::new(SIZE, PanicSwitch::Respawn).unwrap();

    let listener = TcpListener::bind(addr).unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];

    let _ = stream.read(&mut buffer).unwrap();

    let get = b"GET / HTTP/1.1\r\n";
    let put = b"PUT / HTTP/1.1\r\n";

    let response = if buffer.starts_with(get) {
        response("HTTP/1.1 200 OK")
    } else if buffer.starts_with(put) {
        thread::sleep(Duration::from_secs(1));
        response("HTTP/1.1 200 OK")
    } else {
        response("HTTP/1.1 404 NOT FOUND")
    };

    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn response(status: &str) -> String {
    let headers = "";
    let body = "";
    format!("{}\r\n{}\r\n{}", status, headers, body)
}
