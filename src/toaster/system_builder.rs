use std::{
    collections::BTreeMap,
    thread,
    time::{Duration, UNIX_EPOCH},
};

use super::system_stage::SStage;

#[derive(Debug, Clone)]
pub struct SystemBuilder {
    pub name: String,
    pub description: String,
    pub stages: BTreeMap<u8, SStage>,
    pub shell: String,
}

impl SystemBuilder {
    pub fn new(name: String, desc: String, shell: String) -> Self {
        Self {
            name: name.to_lowercase(),
            description: desc,
            stages: BTreeMap::new(),
            shell,
        }
    }

    pub fn add_stage(&mut self, stage: SStage) {
        let stage_id = self.stages.len() as u8 + 1;
        self.stages.insert(stage_id, stage);
    }

    pub fn get_stages(&self) -> Vec<&SStage> {
        self.stages.iter().map(|(_, v)| v).collect::<Vec<&SStage>>()
    }

    pub fn start(self) {
        let mut last_ran_stages: BTreeMap<u8, u64> = BTreeMap::new();

        loop {
            let current_time_u64 = UNIX_EPOCH.elapsed().unwrap().as_secs();

            for (stage_id, stage) in &self.stages {
                let execution_schedule = stage.schedule.clone();
                let scheduled_time = execution_schedule.get_as_u64();

                // Check if it's time to run the command
                if last_ran_stages.get(stage_id).unwrap_or(&0) + scheduled_time <= current_time_u64
                {
                    let command = stage.command.clone();
                    println!("Running command: {}", command);

                    let output = std::process::Command::new(self.shell.as_str())
                        .arg("-c")
                        .arg(command.as_str())
                        .spawn()
                        .expect("Failed to execute command");
                    
                    let output_result = output.wait_with_output().expect("Failed to wait on command");
                    
                    println!("Command output: {}", String::from_utf8_lossy(&output_result.stdout));
                    

                    // Update the last ran time for this stage
                    last_ran_stages.insert(*stage_id, current_time_u64);
                }
            }

            // Sleep briefly to prevent tight looping
            thread::sleep(Duration::from_millis(100));
        }
    }
}
