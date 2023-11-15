// mod worker;
mod errors;
mod scheduler;

use core::num;
// use worker::ThreadPool;
use errors::ServerError;
use httparse;
use std::{
    fs::metadata, fs::File, io::prelude::*, net::TcpListener, net::TcpStream, str, thread,
    time::Duration, time::Instant,
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
            }
            Err(e) => {
                eprintln!("Failed to create stream for TCP connection: {}", e);
            }
        }

        // pool.execute(|| {
        //     handle_connection(stream);
        // });
    }

    println!("Shutting down.");

    Ok(())
}

fn handle_connection(mut stream: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let file_path = "../v_day_climb_carry.mp4";
    // let file_path = "../bears.mp4";

    let mut req_buffer = [0; 1024];
    stream.read(&mut req_buffer).unwrap();
    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut req = httparse::Request::new(&mut headers);

    req.parse(&req_buffer)?;

    // Could probably mess with block size to change performance
    const BLOCK_SIZE: usize = 1024 * 400;
    let mut file_buffer = vec![0; BLOCK_SIZE];
    let mut file = File::open(file_path)?;
    let content_length = metadata(file_path)?.len();
    let mut file_bytes_written = 0;

    let status_line = match req.method {
        Some("GET") => "HTTP/1.1 206 Partial Content",
        _ => "HTTP/1.1 404 NOT FOUND",
    };

    // TODO: Handle chunk requests from client
    let mut start = 0;
    let mut end = 0;
    if let Some(user_agent) = req
        .headers
        .iter()
        .find(|h| h.name.to_lowercase() == "range")
    {
        // Parse range request
        let range_str = std::str::from_utf8(user_agent.value).unwrap();
        (start, end) = parse_byte_range_request(range_str, 0, content_length).unwrap();
    }

    let response = format!(
        "{}\r\n\
        Accept-Ranges: bytes \r\n\
        Content-Range: bytes {}-{}/{} \r\n\
        Content-Type: application/octet-stream \r\n\
        Content-Length: {} \r\n\
        Access-Control-Allow-Origin: * \r\n\r\n",
        status_line,
        0,
        content_length - 1,
        content_length,
        content_length
    );

    stream.write_all(response.as_bytes()).unwrap();

    let mut total_read_time_us = 0;
    let mut total_write_time_us = 0;
    let mut num_chunks = 0;
    loop {
        let mut start_time = Instant::now();
        let bytes_read = file.read(&mut file_buffer).unwrap();
        file_bytes_written += bytes_read;
        if bytes_read == 0 {
            break; // End of file
        }
        let mut end_time = Instant::now();
        total_read_time_us += end_time.duration_since(start_time).as_micros();

        start_time = Instant::now();
        match stream.write(&file_buffer[..bytes_read]) {
            Ok(_) => { /* Maybe want to print some useful info later */ }
            Err(e) => {
                println!("=Error=: {e}");
                break;
            }
        }
        end_time = Instant::now();
        total_write_time_us += end_time.duration_since(start_time).as_micros();
        num_chunks += 1;
    }
    println!("Total bytes written to stream: {}", file_bytes_written);
    if num_chunks > 0 {
        println!(
            "Average time for file read operation with {} chunks: {} us",
            num_chunks,
            total_read_time_us / num_chunks
        );
        println!(
            "Average time for network write operation with {} chunks: {} us",
            num_chunks,
            total_write_time_us / num_chunks
        );
    }
    stream.flush().unwrap();

    Ok(())
}

fn parse_byte_range_request(
    range_str: &str,
    start_default: u64,
    end_default: u64,
) -> Result<(u64, u64), Box<dyn std::error::Error>> {
    // Split the string by '-' and collect the parts into a vector of strings
    let parts: Vec<&str> = range_str.split('-').collect();

    if parts.len() != 2 {
        // This is brutal, change later
        return Err(Box::new(ServerError::CustomError(
            "Invalid range format.".to_string(),
        )));
    }

    // Attempt to parse the two parts into integers
    let start = parts[0]
        .trim_start_matches("bytes=")
        .parse::<u64>()
        .unwrap_or(start_default);
    let end = parts[1].parse::<u64>().unwrap_or(end_default);

    Ok((start, end))
}
