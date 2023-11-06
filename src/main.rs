// mod worker;
mod errors;

use core::num;
// use worker::ThreadPool;
use errors::ServerError;
use httparse;
use std::{
    fs::File, io::prelude::*, net::TcpListener, net::TcpStream, str, thread, time::Duration,
};

fn main() -> Result<(), ServerError> {
    let listener = TcpListener::bind("127.0.0.1:7878")
        .map_err(|_| ServerError::Critical("Failed to set up TCP Listener".to_string()))?;
    // let pool = ThreadPool::new(4)?;

    for stream in listener.incoming() {
        println!("Connection received!");
        // Handle connections serially for now
        match stream {
            Ok(s) => {
                handle_connection(s).unwrap();
            },
            Err(e) => {
                eprintln!("Failed to create stream for TCP connection: {}", e);
            }
        }

        // pool.execute(|| {
        //     handle_connection(stream);
        // });
    }

    println!("Shutting down.");

    println!("ABC");

    Ok(())
}

fn read_video_section(file_path: &str, buffer: &mut Vec<u8>) -> Result<usize, ServerError> {
    // let mut buffer = vec![0; BLOCK_SIZE];
    let mut file = File::open(file_path).unwrap();
    // let mut num_block = 0;
    let mut total_bytes_read = 0;
    loop {
        let br = file.read(buffer.as_mut()).unwrap();
        total_bytes_read += br;

        if br == 0 {
            break;
        }
    }
    // println!("{}", num_block);
    Ok(total_bytes_read)
}

fn handle_connection(mut stream: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut req = httparse::Request::new(&mut headers);

    let get = b"GET / HTTP/1.1\r\n";
    // const get_movie = b"GET /movie.mp4 HTTP/1.1\r\n";
    // const sleep = b"GET /sleep HTTP/1.1\r\n";

    println!("{:?}", str::from_utf8(&buffer).unwrap());
    req.parse(&buffer)?;

    const BLOCK_SIZE: usize = 1024 * 400; //376852115; // 383631
    let mut file_buffer = vec![0; BLOCK_SIZE];
    let file_path = "../movie.mp4"; // "../v_day_climb_carry.mp4";
    let file_nbytes = read_video_section(file_path, &mut file_buffer).unwrap();

    println!("Read {} bytes from {}", file_nbytes, file_path);

    // TODO: Handle the partial parse case?
    // if req.parse(&buffer)?.is_partial() {
    // }

    println!("{:?}", req.headers);

    let status_line = if buffer.starts_with(get) {
        "HTTP/1.1 200 OK"
    } else {
        "HTTP/1.1 404 NOT FOUND"
    };

    // let contents = encode(&file_nbytes);
    println!("{}", status_line);

    let response = format!(
        "{}\r\nContent-Type: application/octet-stream\r\nAccept-Ranges: bytes\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\n\r\n",
        status_line,
        file_nbytes,
    );

    println!("{}", response);

    stream.write_all(response.as_bytes()).unwrap();
    let file_bytes_written = stream.write(&file_buffer).unwrap();
    println!("{}", file_bytes_written);
    stream.flush().unwrap();

    Ok(())
}
