mod errors;
mod scheduler;

use errors::ServerError;
use httparse;
use scheduler::*;
use std::cell::RefCell;
use std::{
    env, fs::metadata, fs::File, io::prelude::*, net::TcpListener, net::TcpStream, str, thread,
    time::Instant,
};

// Define a thread-local variable using RefCell
thread_local! {
    pub static THREAD_LOCAL_CONNECTION_COUNTER: RefCell<usize> = RefCell::new(0);
}

fn increment_thread_local_counter() {
    THREAD_LOCAL_CONNECTION_COUNTER.with(|counter| {
        *counter.borrow_mut() += 1;
    });
}

fn decrement_thread_local_counter() {
    THREAD_LOCAL_CONNECTION_COUNTER.with(|counter| {
        if *counter.borrow_mut() > 0 {
            *counter.borrow_mut() -= 1;
        }
    });
}

fn main() -> Result<(), ServerError> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return Err(ServerError::Critical(
            "Specify a scheduling algorithm".to_string(),
        ));
    }
    let scheduling_alg = args[1].as_str();
    println!("{}", args[1]);

    let listener = TcpListener::bind("127.0.0.1:7878")
        .map_err(|_| ServerError::Critical("Failed to set up TCP Listener".to_string()))?;

    // Async stream handling
    let (executor, spawner) = new_executor_and_spawner();
    thread::spawn(move || {
        executor.run();
    });

    // Thread id for debugging purposes
    let mut thread_id = 0;
    const QUANTUM: u128 = 500;

    for stream in listener.incoming() {
        println!("Connection received!");

        thread_id += 1;
        match stream {
            Ok(s) => {
                // I feel like the quantum should be passed to the spawner not handle_connection
                match scheduling_alg {
                    "r" | "w" => {
                        spawner.spawn(handle_connection(s, thread_id, QUANTUM));
                    }
                    "v" => {}
                    _ => {
                        println!("Unsupported scheduling algorithm");
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to create stream for TCP connection: {}", e);
            }
        }
    }

    println!("Shutting down.");

    Ok(())
}

#[allow(unused)]
async fn handle_connection(mut stream: TcpStream, thread_id: usize, quantum_: u128) {
    let mut quantum = quantum_;
    let mut instructor = false;

    let file_path = "../v_day_climb_carry.mp4";

    let mut req_buffer = [0; 1024];
    stream.read(&mut req_buffer).unwrap();
    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut req = httparse::Request::new(&mut headers);

    req.parse(&req_buffer).unwrap();
    match req.path {
        Some("/instructor") => {
            println!("Instructor thread started");
            quantum = quantum * 4;
            instructor = true;
        }
        Some(_) => {}
        None => {}
    };

    // Could probably mess with block size to change performance
    const BLOCK_SIZE: usize = 1024 * 1000;
    let mut file = File::open(file_path).unwrap();
    let content_length = metadata(file_path).unwrap().len();

    println!("Req: {:?}", req);
    // println!("Req string: {}", String::from_utf8_lossy(&req_buffer));
    // println!("Req method: {:?}", req.method);
    let status_line = match req.method {
        Some("GET") => "HTTP/1.1 206 Partial Content",
        _ => "HTTP/1.1 404 NOT FOUND",
    };

    // Increment global TCP connection counter
    increment_thread_local_counter();
    println!(
        "Number of active connections: {:?}",
        THREAD_LOCAL_CONNECTION_COUNTER.with(|counter| { *counter.borrow() })
    );

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

    write_chunks_to_stream(file_path, BLOCK_SIZE, Some(&mut stream), quantum, thread_id).await;

    decrement_thread_local_counter();
    stream.flush().unwrap();
}

#[allow(unused)]
async fn write_chunks_to_stream(
    file_path: &str,
    block_size: usize,
    mut stream_op: Option<&mut TcpStream>,
    quantum: u128,
    thread_id: usize,
) {
    let mut file_buffer = vec![0; block_size];
    let mut quantum_start = Instant::now();
    let mut file_bytes_written = 0;
    let mut total_read_time_us = 0;
    let mut total_write_time_us = 0;
    let mut num_chunks = 0;
    // let total_start_time = Instant::now();

    let mut file: File = File::open(file_path).unwrap();

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

        if let Some(ref mut stream) = stream_op {
            match stream.write(&file_buffer[..bytes_read]) {
                Ok(_) => { /* Maybe want to print some useful info later */ }
                Err(e) => {
                    println!("=Error=: {e}");
                    break;
                }
            }
        };

        end_time = Instant::now();
        total_write_time_us += end_time.duration_since(start_time).as_micros();
        num_chunks += 1;

        if Instant::now().duration_since(quantum_start).as_millis() > quantum {
            // println!("Yielding thread {}", thread_id);
            yield_now().await;
            quantum_start = Instant::now();
        }
    }
    // println!("Total bytes written to stream: {}", file_bytes_written);
    // if num_chunks > 0 {
    //     // println!("Instructor thead: {}", if instructor {"Yes"} else {"No"} );
    //     println!("Total stream execution time: {} s", Instant::now().duration_since(total_start_time).as_secs_f64());
    //     println!(
    //         "Average time for file read operation with {} chunks: {} us",
    //         num_chunks,
    //         total_read_time_us / num_chunks
    //     );
    //     println!(
    //         "Average time for network write operation with {} chunks: {} us",
    //         num_chunks,
    //         total_write_time_us / num_chunks
    //     );
    // }
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

#[cfg(test)]
mod tests {
    use super::*;

    const THREAD_COUNT: usize = 5;
    const QUANTUM: u128 = 4;

    thread_local! {
        static THREAD_LOCAL_VEC: RefCell<Vec<u128>> = RefCell::new(vec![0; THREAD_COUNT]);
    }

    async fn write_chunks_to_stream_timer(
        file_path: &str,
        block_size: usize,
        stream_op: Option<&mut TcpStream>,
        quantum: u128,
        thread_id: usize,
    ) {
        let total_start_time = Instant::now();
        write_chunks_to_stream(file_path, block_size, stream_op, quantum, thread_id).await;
        println!(
            "Thread {} total execution time: {} s",
            thread_id,
            Instant::now()
                .duration_since(total_start_time)
                .as_secs_f64()
        );
        // *execution_time_pointer = Instant::now().duration_since(total_start_time).as_secs_f64();
        THREAD_LOCAL_VEC.with(|vec_cell| {
            let mut vec = vec_cell.borrow_mut();

            let time: u128 = Instant::now().duration_since(total_start_time).as_millis();
            // Write to the elements of the vector
            vec[thread_id] = time;
        });
    }

    #[test]
    fn test_round_robin() {
        // Async stream handling
        let (executor, spawner) = new_executor_and_spawner();

        let file_path = "../v_day_climb_carry.mp4";
        const BLOCK_SIZE: usize = 1024 * 1000;
        // const THREAD_COUNT: usize = 3;

        for i in 0..THREAD_COUNT {
            spawner.spawn(write_chunks_to_stream_timer(
                file_path, BLOCK_SIZE, None, QUANTUM, i,
            ));
        }

        drop(spawner);
        executor.run();

        THREAD_LOCAL_VEC.with(|vec_cell| {
            let vec = vec_cell.borrow();
            let mut sum = 0;
            for time in vec.iter() {
                sum += time;
            }
            // println!("Thread-local vector inside closure: {:?}", *vec);
            println!(
                "Average thread execution time: {} ms",
                sum / THREAD_COUNT as u128
            );
        });
    }

    // #[test]
    // fn test_instructor_observer() {
    //     // Async stream handling
    //     let (executor, spawner) = new_executor_and_spawner();

    //     let file_path = "../v_day_climb_carry.mp4";
    //     const BLOCK_SIZE: usize = 1024 * 1000;
    //     let observer_quantum = 4;
    //     let instructor_quantum = observer_quantum * 4;

    //     // Spawn instructors with a quantum of 4x
    //     for i in 0..1 {
    //         spawner.spawn(write_chunks_to_stream_timer(file_path, BLOCK_SIZE, None, instructor_quantum, i))
    //     }

    //     // Spawn observers
    //     for i in 1..4 {
    //         spawner.spawn(write_chunks_to_stream_timer(file_path, BLOCK_SIZE, None, 4, i));
    //     }
    //     drop(spawner);
    //     executor.run();
    // }
}
