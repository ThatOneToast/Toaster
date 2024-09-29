use std::{
    io::{Read, Write},
    os::unix::net::UnixStream,
    time::Duration,
};

use args::Arguments;

pub mod args;

fn main() {
    let daemon_res = UnixStream::connect("/tmp/toaster.sock");
    let mut daemon = match daemon_res {
        Ok(stream) => stream,
        Err(_) => {
            panic!("Couldn't connect to daemon server; Make sure it's running.");
        }
    };

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
        "--reload" => {
            daemon.write(b"reload").expect("Could not write to socket");

            let mut buffer = [0; 20];
            daemon
                .read(&mut buffer)
                .expect("Could not read from socket");
            let string = String::from_utf8_lossy(&buffer).to_string();

            if string != "ok" {
                panic!("Could not reload daemon got {}", string);
            }
        }
        "--flush" => {
            daemon.write(b"flush").expect("Could not write to socket");

            let mut buffer = [0; 20];
            daemon
                .read(&mut buffer)
                .expect("Could not read from socket");

            let string = String::from_utf8_lossy(&buffer).to_string();

            if string != "ok" {
                panic!("Could not flush daemon got {}", string);
            }
        }
        "--ping" => {
            daemon.write(b"ping").expect("Could not write to socket");

            let mut buffer = [0; 20];
            daemon
                .read(&mut buffer)
                .expect("Could not read from socket");

            let string = String::from_utf8_lossy(&buffer).to_string();

            if string != "pong" {
                panic!("Could not ping daemon got {}", string);
            }
        }
        _ => {
            println!("Usage: toaster --reload | --flush | --ping");
        }
    }

    daemon.flush().expect("Could not flush socket");
}
