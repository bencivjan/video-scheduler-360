// mod worker;
mod errors;

use core::num;
// use worker::ThreadPool;
use errors::ServerError;
use httparse;
use std::{
    fs::File, fs::metadata, io::{prelude::*, SeekFrom}, net::TcpListener, net::TcpStream, str, thread, time::Duration,
};

struct Chunk {
    bytes: Vec<u8>,
    content_start: u64,
    content_end: u64,
    content_length: u64
}

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

    println!("ABC");

    Ok(())
}

// fn read_video_section(file_path: &str, buffer: &mut Vec<u8>) -> Result<usize, ServerError> {
//     // let mut buffer = vec![0; BLOCK_SIZE];
//     let mut file = File::open(file_path).unwrap();
//     // let mut num_block = 0;
//     // let mut total_bytes_read = 0;
//     loop {
//     let br = file.read(buffer.as_mut()).unwrap();
//     total_bytes_read += br;

//     if br == 0 {
//         break;
//     }
//     }
//     println!("{}", num_block);
//     Ok(br)
// }

fn handle_connection(mut stream: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    // Testing stuff below
    let file_path = "../v_day_climb_carry.mp4"; // "../movie.mp4";
    let mut file = File::open(file_path).unwrap();
    let mut file_bytes_written = 0;
    const BLOCK_SIZE: usize = 1024 * 400; // 376852115;
    let mut file_buffer = vec![0; BLOCK_SIZE];

    let get = b"GET / HTTP/1.1\r\n";

    let status_line = "HTTP/1.1 206 Partial Content";

    let response = format!(
        "{}\r\n\
        Accept-Ranges: bytes \r\n\
        Content-Range: bytes {}-{}/{} \r\n\
        Content-Type: application/octet-stream \r\n\
        Content-Length: {} \r\n\
        Access-Control-Allow-Origin: * \r\n\r\n",
        status_line,
        0,
        376852115 - 1,
        376852115,
        376852115
    );

    stream.write_all(response.as_bytes()).unwrap();

    loop {
        let bytes_read = file.read(&mut file_buffer).unwrap();
        file_bytes_written += bytes_read;
        if bytes_read == 0 {
            break; // End of file
        }
        println!("About to write {} bytes", bytes_read);
        match stream.write(&file_buffer[..bytes_read]) {
            Ok(bw) => println!("Write {} bytes", bw),
            Err(e) => {
                println!("=Error=: {e}");
                break;
            }
        }
    }
    println!("bytes written to stream: {}", file_bytes_written);
    stream.flush().unwrap();

    Ok(())

    // ====================================================================================

    // let mut req_buffer = [0; 1024];
    // stream.read(&mut req_buffer).unwrap();
    // let mut headers = [httparse::EMPTY_HEADER; 64];
    // let mut req = httparse::Request::new(&mut headers);

    // // println!("{:?}", str::from_utf8(&req_buffer).unwrap());
    // req.parse(&req_buffer)?;

    // // println!("{:?}", req);

    // const BLOCK_SIZE: usize = 1024 * 400; // 376852115;
    // let file_buffer = vec![0; BLOCK_SIZE];
    // let file_path = "../v_day_climb_carry.mp4"; // "../movie.mp4";

    // let mut file = File::open(file_path)?;
    // let content_length = metadata(file_path)?.len();

    // let mut chunk = Chunk {
    //     bytes: file_buffer,
    //     content_start: 0,
    //     content_end: 0,
    //     content_length,
    // };

    // // TODO: Handle chunk requests from client
    // let mut start = 0;
    // let mut end = 0;
    // if let Some(user_agent) = req.headers.iter().find(|h| h.name.to_lowercase() == "range") {
    //     // Parse range request
    //     let range_str = std::str::from_utf8(user_agent.value).unwrap();
    //     (start, end) = parse_byte_range_request(range_str, 0, content_length).unwrap();
    // }
    
    // // loop {
    // file.seek(SeekFrom::Start(start)).unwrap();
    // let br = file.read(chunk.bytes.as_mut()).unwrap() as u64;

    // // update chunk fields
    // chunk.content_start = start; // chunk.content_end;
    // chunk.content_end = start + BLOCK_SIZE as u64 - 1; // chunk.content_start + br - 1;

    // // if br == 0 {
    // //     break;
    // // }
    // println!("Bytes read from file: {}", br);

    // // process the data
    // stream_data(&mut stream, &req_buffer, &mut chunk)?;
    // // }



    // // let file_nbytes = read_video_section(file_path, &mut file_buffer).unwrap();

    // // println!("Read {} bytes from {}", file_nbytes, file_path);

    // // TODO: Handle the partial parse case?
    // // if req.parse(&buffer)?.is_partial() {
    // // }

    // // println!("{:?}", req.headers);

    // Ok(())
}

fn stream_data(stream: &mut TcpStream, request: &[u8], chunk: &mut Chunk) -> Result<(), Box<dyn std::error::Error>> {
    let get = b"GET / HTTP/1.1\r\n";
    let nbytes: usize = (chunk.content_end - chunk.content_start + 1).try_into().unwrap();

    let status_line = if request.starts_with(get) {
        "HTTP/1.1 206 Partial Content" // "HTTP/1.1 200 OK"
    } else {
        "HTTP/1.1 404 NOT FOUND"
    };

    let response = format!(
        "{}\r\n\
        Accept-Ranges: bytes \r\n\
        Content-Range: bytes {}-{}/{} \r\n\
        Content-Type: application/octet-stream \r\n\
        Content-Length: {} \r\n\
        Access-Control-Allow-Origin: * \r\n\r\n",
        status_line,
        chunk.content_start,
        chunk.content_end,
        chunk.content_length,
        chunk.content_length
    );

    println!("{}", response);
    println!("nbytes: {}", nbytes);

    stream.write_all(response.as_bytes()).unwrap();
    let file_bytes_written = stream.write(&chunk.bytes[0..nbytes]).unwrap();

    Ok(())
}

fn parse_byte_range_request(range_str: &str, start_default: u64, end_default: u64) -> Result<(u64, u64), Box<dyn std::error::Error>> {
    // Split the string by '-' and collect the parts into a vector of strings
    let parts: Vec<&str> = range_str.split('-').collect();

    if parts.len() != 2 {
        // This is brutal, change later
        return Err(Box::new(ServerError::CustomError("Invalid range format.".to_string())));
    }

    // Attempt to parse the two parts into integers
    let start = parts[0].trim_start_matches("bytes=").parse::<u64>().unwrap_or(start_default);
    let end = parts[1].parse::<u64>().unwrap_or(end_default);

    Ok((start, end))
}
