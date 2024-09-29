use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
    time::{Duration, UNIX_EPOCH},
};

use priority::queue::{prelude::Prio, Queue};

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

    pub fn start(self, output_queue: Arc<RwLock<Queue<String>>>) {
        let mut last_ran_stages: BTreeMap<u8, u64> = BTreeMap::new();

        loop {
            let current_time_u64 = UNIX_EPOCH.elapsed().unwrap().as_secs();

            for (stage_id, stage) in &self.stages {
                let execution_schedule = stage.schedule.clone();
                let scheduled_time = execution_schedule.get_as_u64();

                if last_ran_stages.get(stage_id).unwrap_or(&0) + scheduled_time <= current_time_u64
                {
                    let command = stage.command.clone();

                    let output_res = std::process::Command::new(self.shell.as_str())
                        .arg("-c")
                        .arg(command.as_str())
                        .output();

                    let output = match output_res {
                        Ok(output) => output,
                        Err(e) => {
                            eprintln!("Failed to run command: {}", e);
                            continue;
                        }
                    };

                    let mut final_output = String::new();
                    final_output.push_str(
                        format!(
                            "{}: Output: {}",
                            self.name.clone(),
                            String::from_utf8(output.stdout).unwrap()
                        )
                        .as_str(),
                    );

                    output_queue.write().unwrap().push(Prio::wlip(final_output));

                    last_ran_stages.insert(*stage_id, current_time_u64);
                }

                std::thread::sleep(Duration::from_millis(100));
            }
        }
    }
}
