use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use command_builder::CommandBuilder;
use system_builder::SystemBuilder;
use threadpool::ThreadPool;

use crate::tomlp::TomlParser;

pub mod command_builder;
pub mod command_stage;
pub mod system_builder;
pub mod system_stage;

pub struct Toaster {
    pub toaster_path: PathBuf,
    pub commands: Vec<(String, CommandBuilder)>,
    pub systems: Vec<(String, SystemBuilder)>,
    pub thread_pool: Arc<Mutex<ThreadPool>>,
}

impl Toaster {
    pub fn new(toaster_config_path: PathBuf) -> Self {
        // get the toaster.toml file from the provided path
        let commands_toml =
            std::fs::read_to_string(&toaster_config_path.join("toaster.toml")).unwrap();
        let tparser = TomlParser::new(commands_toml.as_str());
        let (systems, commands) = tparser.parse().unwrap();
        let settings = tparser.get_settings().unwrap();
        let threads = settings.threads;
        Self {
            toaster_path: toaster_config_path,
            commands,
            systems,
            thread_pool: Arc::new(Mutex::new(ThreadPool::new(threads))),
        }
    }

    pub fn run_command(&self, command: &str) {
        let command_builder = self.commands.iter().find(|(name, _)| name == command);

        if let Some((_, command_builder)) = command_builder {
            command_builder.run();
        } else {
            eprintln!("Command '{}' not found.", command);
        }
    }

    pub fn start_system(&mut self, system: &str) {
        println!("Starting system '{}'...", system);
        if let Some((_, system_builder)) = self.systems.iter().find(|(name, _)| name == system) {
            let system_builder = system_builder.clone();
            let thread_pool = Arc::clone(&self.thread_pool);

            println!("Preparing move...");
            thread_pool.lock().unwrap().execute(move || {
                system_builder.start(); // Pass the shutdown signal
            });
        } else {
            eprintln!("System '{}' not found.", system);
        }
    }


    pub fn reload(self) -> Toaster {
        let toaster = Toaster::new(self.toaster_path);
        toaster
    }

}
