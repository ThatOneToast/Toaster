use std::{
    fs::{File, OpenOptions},
    io::Write,
    path::PathBuf,
    sync::{Arc, Mutex, RwLock},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use command_builder::CommandBuilder;
use priority::queue::{prelude::Prio, Queue};
use settings::Settings;
use system_builder::SystemBuilder;
use threadpool::ThreadPool;
use tomlp::TomlParser;

pub mod color;
pub mod command_builder;
pub mod command_stage;
pub mod settings;
pub mod system_builder;
pub mod system_stage;
pub mod tomlp;

pub struct Toaster {
    pub toaster_path: PathBuf,
    pub toaster_log_path: PathBuf,
    pub commands: Option<Vec<(String, CommandBuilder)>>,
    pub systems: Option<Vec<(String, SystemBuilder)>>,
    pub settings: Settings,
    pub thread_pool: Arc<Mutex<ThreadPool>>,
    pub output_queue: Arc<RwLock<Queue<String>>>,
    flush_output: Arc<RwLock<bool>>,
    pub last_flush_output: Arc<RwLock<Option<String>>>,
}

impl Toaster {
    pub fn new() -> Self {
        let home_dir = PathBuf::from(std::env::var("HOME").unwrap());
        let path = home_dir.join(".toaster");
        let log_path = path.join("Logs");

        println!("Config Path: {}", path.to_str().unwrap());
        let toaster_toml = path.join("toaster.toml");
        if !toaster_toml.exists() {
            println!("Creating toaster.toml...");
            std::fs::create_dir_all(toaster_toml.parent().unwrap()).unwrap();
            let mut file = File::create(&toaster_toml).expect("Failed to create toaster.toml");
            let a = r#"[settings]
threads = 1
default_row_length = 4

# All scheduled commands should be provided under system
# An example of how a sysytem is structured
# [system.testing] # After the `.` is the name of your system.
# description = "testing will send Hola! every 30 seconds"
# shell = "zsh"
# stages = [
#     "echo \"Hola!\" | tee output.log",
#     """ # Multiline stages are supported
#     """ # Multiline stages are supported
# ]
# schedules = [
#     "00:00:00:00:30" # This will be ran every 30 seconds
# ]
[system]

# All commands should be provided under command
# An example of how a command is structured
# [command.testing]
# description = "testing"
# shell = "zsh"
# stages = [
#     "%[color:cyan,o:-s] clear && echo \"===== Dirs ===== \" ",
#     "%[color:cyan,o:l6;] ls",
#     "echo Hello World",
#     "%[color:green] echo ok123"
# ]
[command]


"#;
            file.write_all(a.as_bytes()).unwrap();
            println!("Toaster config located at: {}", path.to_str().unwrap());
        }

        if !log_path.exists() {
            println!("Creating log directory...");
            let res = std::fs::create_dir_all(log_path.to_owned());
            match res {
                Ok(_) => {
                    println!("Log directory created.");
                }
                Err(e) => {
                    eprintln!("Failed to create log directory: {}", e);
                }
            }
        }

        let toml_content = std::fs::read_to_string(&toaster_toml)
            .expect("Failed to read toaster.toml in the given directory.");
        let tparser = TomlParser::new(toml_content.as_str());
        let package = tparser.parse().unwrap();
        let threads = package.2.threads;

        let toaster = Self {
            toaster_path: path,
            toaster_log_path: log_path,
            commands: package.1,
            systems: package.0,
            settings: package.2,
            thread_pool: Arc::new(Mutex::new(ThreadPool::new(threads))),
            output_queue: Arc::new(RwLock::new(Queue::new())),
            flush_output: Arc::new(RwLock::new(false)),
            last_flush_output: Arc::new(RwLock::new(None)),
        };

        toaster
            .output_queue
            .write()
            .unwrap()
            .push(Prio::wlip("SYSTEM: Starting toaster...".to_string()));

        println!("Starting output watcher...");
        toaster.output_watcher();
        std::thread::sleep(Duration::from_millis(3));
        toaster.flush_output();
        return toaster;
    }

    fn output_watcher(&self) {
        let flush_output_signal = self.flush_output.clone();
        let output_queue = self.output_queue.clone();
        let log_path = Arc::new(self.toaster_log_path.clone());
        let last_flush_output = self.last_flush_output.clone();

        std::thread::spawn(move || loop {
            if *flush_output_signal.read().unwrap() {
                let datetime = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                let tod = chrono::DateTime::from_timestamp(datetime.as_secs() as i64, 0)
                    .unwrap()
                    .format("%Y-%m-%d")
                    .to_string();
                let file_path = log_path.join(format!("output-{}.log", tod));
                if !file_path.exists() {
                    File::create(&file_path).expect("Failed to create log file");
                }
                let mut file = OpenOptions::new()
                    .append(true)
                    .open(file_path)
                    .expect("Failed to open log file");

                let mut queue = output_queue.write().unwrap();
                let copy_of_queue = queue.all();

                for output in copy_of_queue {
                    queue.pop_by_id(output.0.identifier);
                    let time = output.0.timestamp;
                    let hh_mm_ss = chrono::DateTime::from_timestamp(time as i64, 0)
                        .unwrap()
                        .format("%H:%M:%S")
                        .to_string();

                    let text = format!("{} :::: {}\n", hh_mm_ss, output.1);
                    let res = file.write_all(text.as_bytes());
                    match res {
                        Ok(_) => {
                            last_flush_output.write().unwrap().replace(text.clone());
                        }
                        Err(e) => {
                            eprintln!("Failed to write to log file: {}", e);
                        }
                    }
                }

                *flush_output_signal.write().unwrap() = false;
            }
            std::thread::sleep(Duration::from_millis(20));
        });
    }

    /// Writes out all outputs into the days log file
    pub fn flush_output(&self) {
        *self.flush_output.write().unwrap() = true;
        std::thread::sleep(Duration::from_millis(20));
    }

    pub fn run_command(&self, command: &str) {
        let command_builder = match self.commands {
            Some(ref commands) => commands.iter().find(|(name, _)| name == command),
            None => {
                eprintln!("Error: No commands are loaded. Please check your configuration.");
                return;
            }
        };

        if command_builder.is_none() {
            eprintln!(
                "Error: Command '{}' not found. Please check if the command name is correct.",
                command
            );
            return;
        }

        if let Some((_, command_builder)) = command_builder {
            command_builder.run();
        } else {
            eprintln!("Command '{}' not found.", command);
        }
    }

    pub fn start_system(&mut self, system: &str) {
        if self.systems.is_none() {
            eprintln!("No systems found.");
            return;
        }
        if let Some((_, system_builder)) = self
            .systems
            .as_ref()
            .unwrap()
            .iter()
            .find(|(name, _)| name == system)
        {
            let system_builder = system_builder.clone();
            let output_queue = self.output_queue.clone();

            self.thread_pool.lock().unwrap().execute(move || {
                system_builder.start(output_queue);
            });
        } else {
            eprintln!("System '{}' not found.", system);
        }
    }

    pub fn systems_names(&self) -> Vec<String> {
        self.systems
            .as_ref()
            .unwrap()
            .iter()
            .map(|(name, _)| name.clone())
            .collect()
    }

    pub fn start_systems(&mut self) {
        if self.systems.is_none() {
            eprintln!("No systems found.");
            return;
        }

        let systems = self.systems_names();

        for system in systems {
            self.start_system(system.as_str());
        }
    }

    pub fn reload(&mut self) {
        let path = self.toaster_path.clone();
        let tomlp = TomlParser::new(path.to_str().unwrap());
        let package = tomlp.parse().unwrap();
        self.commands = package.1;
        self.systems = package.0;
        self.settings = package.2;
        self.thread_pool = Arc::new(Mutex::new(ThreadPool::new(self.settings.threads)));
    }
}
