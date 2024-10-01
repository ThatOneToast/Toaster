use std::{
    io::{BufRead, Read, Write}, os::unix::net::UnixStream, path::PathBuf, time::Duration
};

use args::Arguments;
use chrono::Datelike;

pub mod args;

fn main() {
    let daemon_res = UnixStream::connect("/tmp/toaster.sock");
    let mut daemon = match daemon_res {
        Ok(stream) => stream,
        Err(_) => {
            panic!("Couldn't connect to daemon server; Make sure it's running.");
        }
    };
    
    let home = PathBuf::from(std::env::var("HOME").unwrap());
    let toaster_path = home.join(".toaster");
    let log_path = toaster_path.join("Logs");
    
    daemon
        .set_nonblocking(false)
        .expect("Could not set blocking");
    daemon
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("Could not set read timeout");
    daemon
        .set_write_timeout(Some(Duration::from_secs(2)))
        .expect("Could not set write timeout");

    let mut args = Arguments::new();
    let command = args.next().expect("No command given");

    match command.as_str() {
        "logs" => {
            let next = args.next().expect("no flag provided");
            match next.as_str() {
                "latest" => {
                    let year = chrono::Local::now().year();
                    let month = chrono::Local::now().month();
                    let day = chrono::Local::now().day();
                    
                    let output_file_name = format!("output-{}-{}-{}.log", year, month, day);
                    let output_file_path = log_path.join(output_file_name);
                    let file = std::fs::File::open(output_file_path).expect("Could not open file");
                    let buffer = std::io::BufReader::new(file);
                    buffer.lines().for_each(|line| {
                        println!("{}", line.unwrap());
                    });
                },
                _ => {
                    let output_file_name = format!("output-{}.log", next);
                    let output_file_path = log_path.join(output_file_name);
                    let file = std::fs::File::open(output_file_path).expect("Could not open file");
                    let buffer = std::io::BufReader::new(file);
                    buffer.lines().for_each(|line| {
                        println!("{}", line.unwrap());
                    });
                }
            }
        },
        "--reload" => {
            daemon.write(b"reload").expect("Could not write to socket");

            let mut buffer = [0; 25];
            daemon
                .read(&mut buffer)
                .expect("Could not read from socket");
            let string = String::from_utf8_lossy(&buffer).to_string();

            if string != "ok" {
                panic!("Could not reload daemon got {}", string);
            } else {
                println!("Reloaded!");
            }
        }
        "--flush" => {
            daemon.write(b"flush").expect("Could not write to socket");

            let mut buffer = [0; 25];
            let len = daemon
                .read(&mut buffer)
                .expect("Could not read from socket");

            let string = String::from_utf8_lossy(&buffer.to_vec().as_slice()[..len]).to_string();

            if string != "ok" {
                panic!("Could not flush daemon got {}", string);
            } else {
                println!("Flushed!");
            }
        }
        "--ping" => {
            daemon.write(b"ping").expect("Could not write to socket");

            let mut buffer = [0; 25];
            let len = daemon.read(&mut buffer).expect("Could not read from socket");

            let string = String::from_utf8_lossy(&buffer.to_vec().as_slice()[..len]).to_string();

            if string != "pong" {
                panic!("Could not ping daemon got {}", string);
            } else {
                println!("Pong!");
            }
            
        }
        _ => {
            println!("Usage: toaster --reload | --flush | --ping");
        }
    }

    daemon.flush().expect("Could not flush socket");
}
