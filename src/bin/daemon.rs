use std::{
    io::{Read, Write},
    os::unix::net::{UnixListener, UnixStream},
    sync::{Arc, RwLock},
    time::Instant,
};

use lib::Toaster;

fn daemon() -> UnixListener {
    let daemon_res = UnixListener::bind("/tmp/toaster.sock");

    let daemon = match daemon_res {
        Ok(d) => d,
        Err(_) => {
            let res = std::fs::remove_file("/tmp/toaster.sock");
            if res.is_err() {
                panic!("Failed to remove socket file.");
            }

            UnixListener::bind("/tmp/toaster.sock").unwrap()
        }
    };

    return daemon;
}

fn main() {
    let toaster = Arc::new(RwLock::new(Toaster::new()));
    let mut last_flush = Instant::now();
    std::thread::sleep(std::time::Duration::from_millis(100));

    {
        toaster.write().unwrap().start_systems();
    }

    let daemon = daemon();

    daemon
        .set_nonblocking(false)
        .expect("Failed to set blocking");

    for stream in daemon.incoming() {
        if last_flush.elapsed().as_secs() > 45 {
            println!("Auto-Flushing output...");
            toaster.write().unwrap().flush_output();
            last_flush = Instant::now();
        }
        let stream: UnixStream = stream.unwrap();
        let toaster = Arc::clone(&toaster);
        std::thread::spawn(move || {
            handle_client(stream, toaster);
        });
    }
}

fn handle_client(mut stream: UnixStream, toaster: Arc<RwLock<Toaster>>) {
    let mut buf = [0; 20];
    let len = stream.read(&mut buf).unwrap();

    let string = String::from_utf8_lossy(&buf.to_vec().as_slice()[..len]).to_string();
    println!("Received command: {}", string);
    let str = string.as_str();

    match str {
        "reload" => {
            println!("Reloading...");
            toaster.write().unwrap().reload();
        }
        "flush" => {
            println!("Flushing output...");
            toaster.write().unwrap().flush_output();
            println!("Flushed output.");
            stream.write(b"ok").unwrap();
        }
        "ping" => {
            stream.write(b"pong").unwrap();
        }
        _ => {
            let msg = format!("Invalid command: {}", string);
            println!("Bad command got from client: {}", msg);
            stream.write(msg.as_bytes()).unwrap();
        }
    }
}
