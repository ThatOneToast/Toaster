use std::{collections::BTreeMap, fmt::{Display, Formatter}, process::Command};


use crate::{color::Color, tomlp::SortRules};

use super::command_stage::Stage;


#[derive(Debug, Clone)]
pub struct CommandBuilder {
    pub name: String,
    pub description: String,
    pub stages: BTreeMap<u8, Stage>,
    pub shell: String,
}

impl CommandBuilder {
    pub fn new(name: String, desc: String, shell: String) -> Self {
        Self {
            name: name.to_lowercase(),
            description: desc,
            stages: BTreeMap::new(),
            shell,
        }
    }

    pub fn add_stage(&mut self, command: String, colored: Color, sorted: Option<SortRules>) {
        let stage = self.stages.len() as u8 + 1;
        self.stages
            .insert(stage, Stage::new(command, colored, sorted));
    }

    pub fn get_stages(&self) -> Vec<&Stage> {
        self.stages.iter().map(|(_, v)| v).collect::<Vec<&Stage>>()
    }

    fn get_map(&self) -> &BTreeMap<u8, Stage> {
        &self.stages
    }

    pub fn run(&self) {
        let stages = self.get_map();

        for (_, stage) in stages {
            let output = Command::new(self.shell.as_str())
                .arg("-c")
                .arg(stage.command.to_owned())
                .output()
                .expect("Failed to run command");

            let output_str = String::from_utf8_lossy(&output.stdout);
            let error_str = String::from_utf8_lossy(&output.stderr);

            // Apply color to the entire output string if there are no errors
            let color_code = stage.color.ansi_code();

            // Print the output with color
            if !output_str.is_empty() {
                if stage.sorted.is_some() && stage.sorted.as_ref().unwrap().sorting {
                    if let Some(sort_rules) = &stage.sorted {
                        let elements_per_line = sort_rules.items_per_line;

                        let mut elements: Vec<&str> = output_str.split_whitespace().collect();

                        if sort_rules.sorting {
                            elements.sort();
                            
                            let mut grouped_lines = Vec::new();
                            for chunk in elements.chunks(elements_per_line) {
                                let line = chunk.join(", ");
                                grouped_lines.push(line);
                            }
    
                            for line in grouped_lines {
                                println!("{}{}{}", color_code, line, "\x1b[0m");
                            }
                        }
                    }
                } else {
                    if !output_str.is_empty() {
                        println!("{}{}{}", color_code, output_str, "\x1b[0m");
                    }
                }
            } else {
                eprintln!("Failed to run command");
            }

            // Print error messages in red, if any
            if !error_str.is_empty() {
                eprint!("\x1b[31m{}\x1b[0m", error_str); // Print errors in red
            }
        }
    }
}

impl Display for CommandBuilder {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut string = String::new();
        string.push_str(&format!("Command Name: {}\n", self.name));
        string.push_str(&format!("Description: {}\n", self.description));
        for stage in &self.stages {
            string.push_str(&format!("Stage: {}\n", stage.1.command));
        }
        write!(f, "{}", string)
    }
}