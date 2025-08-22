use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    println!("Server running at http://127.0.0.1:7878");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| {
                    if let Err(e) = handle_connection(stream) {
                        eprintln!("Failed to handle connection: {}", e);
                    }
                });
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream) -> std::io::Result<()> {
    let mut buffer = [0; 1024];
    let bytes_read = stream.read(&mut buffer)?;

    if bytes_read == 0 {
        return Ok(());
    }

    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    let lines: Vec<&str> = request.lines().collect();

    let (status_line, filename) = if let Some(first_line) = lines.first() {
        let parts: Vec<&str> = first_line.split_whitespace().collect();
        if parts.len() >= 2 {
            match parts[1] {
                "/" => ("HTTP/1.1 200 OK", "index.html"),
                "/index.html" => ("HTTP/1.1 200 OK", "index.html"),
                _ => ("HTTP/1.1 404 NOT FOUND", "404.html"),
            }
        } else {
            ("HTTP/1.1 400 BAD REQUEST", "404.html")
        }
    } else {
        ("HTTP/1.1 400 BAD REQUEST", "404.html")
    };

    let contents = fs::read_to_string(filename).unwrap_or_else(|_| {
        format!(
            "<html><body><h1>Error</h1><p>Could not load {}</p></body></html>",
            filename
        )
    });

    let response = format!(
        "{}\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    );

    stream.write_all(response.as_bytes())?;
    stream.flush()?;

    Ok(())
}
