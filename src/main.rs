
use std::path::PathBuf;

use toaster::Toaster;

pub mod toaster;
pub mod args;
pub mod tomlp;
pub mod color;
pub mod settings;

fn main() {
    
    let mut toaster = Toaster::new(
        PathBuf::from("/Users/toast/Documents/git/toaster/"));
    
    let systems = toaster.systems.clone();
    
    // toaster.run_command("testing");
    
    for system in systems {
        toaster.start_system(system.0.as_str());
    }

}
