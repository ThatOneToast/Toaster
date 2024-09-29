use std::{
    io::{Read, Write},
    os::unix::net::{UnixListener, UnixStream},
    sync::{Arc, RwLock},
};

use lib::{system_builder::SystemBuilder, Toaster};

fn main() {
    let toaster = Arc::new(RwLock::new(Toaster::new()));
    std::thread::sleep(std::time::Duration::from_millis(100));
    let systems: Arc<RwLock<Option<Vec<(String, SystemBuilder)>>>> =
        Arc::new(RwLock::new(toaster.read().unwrap().systems.clone()));
    let daemon_res = UnixListener::bind("/tmp/toaster.sock");

    let daemon = match daemon_res {
        Ok(d) => {d},
        Err(_) => {
            let res = std::fs::remove_file("/tmp/toaster.sock");
            if res.is_err() {
                panic!("Failed to remove socket file. and establish daemon something went wrong, considering restarting your compouter.");
            }
            
            UnixListener::bind("/tmp/toaster.sock").unwrap()
        }
    };
    
    daemon.set_nonblocking(false).expect("Failed to set blocking");

    let thread_daemon: Arc<RwLock<UnixListener>> = Arc::new(RwLock::new(daemon));
    let thread_toaster: Arc<RwLock<Toaster>> = Arc::clone(&toaster);
    let thread_systems: Arc<RwLock<Option<Vec<(String, SystemBuilder)>>>> = Arc::clone(&systems);
    
    let mut last_flush = std::time::Instant::now();

    run_systems(systems.clone(), toaster.clone());

    println!("Starting daemon...");
    for stream in thread_daemon.read().unwrap().incoming() {
        println!("New connection...");
        let mut stream: UnixStream = stream.unwrap();
    
        let mut buf = [0; 20];
        stream.read(&mut buf).unwrap();
        
        println!("Buf {:?}", buf);
        
        let string = String::from_utf8_lossy(&buf).to_string();
        
        println!("Received command: {}", string);
        
        if last_flush.elapsed().as_secs() > 45 && string != "flush" && string != "reload" {
            println!("Auto-Flushing output...");
            thread_toaster.write().unwrap().flush_output();
            last_flush = std::time::Instant::now();
        };
        
        match string.as_str() {
            "reload" => {
                println!("Reloading...");
                thread_toaster.write().unwrap().reload();
                let new_systems = thread_toaster.read().unwrap().systems.clone();
                *thread_systems.write().unwrap() = new_systems;
                last_flush = std::time::Instant::now();
        
                stream.write(b"ok").unwrap();
                run_systems(thread_systems.clone(), thread_toaster.clone());
                println!("Reloaded.");
            },
            "flush" => {
                println!("Flushing output...");
                thread_toaster.write().unwrap().flush_output();
                println!("Flushed output.");
                last_flush = std::time::Instant::now();
                stream.write(b"ok").unwrap();
            },
            "ping" => {
                stream.write(b"pong").unwrap();
            },
            _ => {
                let msg = format!("Invalid command: {}", string);
                println!("Bad command got from client: {}", msg);
                stream.write(msg.as_bytes()).unwrap();
            }
        }
    
        
        stream.flush().unwrap();
    }


}

fn run_systems(
    systems: Arc<RwLock<Option<Vec<(String, SystemBuilder)>>>>,
    toaster: Arc<RwLock<Toaster>>,
) {
    let c_systems = systems.read().unwrap().clone();
    let mut toaster = toaster.write().unwrap();
    match c_systems {
        Some(systems) => {
            for sys in systems {
                println!("Starting system '{}'...", sys.0);
                toaster.start_system(sys.0.as_str());
            }
        },
        None => {println!("No systems found.");}

    }

}
