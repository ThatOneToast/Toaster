
use crate::{
    color::Color, settings::Settings, toaster::{command_builder::CommandBuilder, system_builder::SystemBuilder, system_stage::{SStage, Schedule}}
};
use toml::Value;



pub struct TomlParser<'a> {
    pub content: &'a str,
    parsed_content: Option<toml::Value>,
}

impl<'a> TomlParser<'a> {
    pub fn new(content: &'a str) -> Self {
        let value = toml::from_str::<toml::Value>(content)
            .map_err(|e| format!("Failed to parse TOML {:?}", e))
            .unwrap();
        Self {
            content,
            parsed_content: Some(value),
        }
    }
    
    pub fn get_settings(&self) -> Result<Settings, String> {
        let value = self.parsed_content.as_ref().unwrap();
        let table = value.as_table().unwrap();
        
        let settings = table.get("settings").unwrap();
        let threads = settings.get("threads").unwrap().as_integer().unwrap() as usize;
        Ok(Settings::new(threads))
    }
    
    pub fn parse(&self) -> Result<(Vec<(String, SystemBuilder)>, Vec<(String, CommandBuilder)>), String> {
        let systems = self.return_as_system()?;
        let commands = self.return_as_command()?;
        Ok((systems, commands))
    }

    fn return_as_system(&self) -> Result<Vec<(String, SystemBuilder)>, String> {
        let parsed_content = self.parsed_content.as_ref().unwrap();
        let table = parsed_content.as_table().unwrap();

        let toml_systems = match table.get("system") {
            Some(cmd) => cmd.as_table().unwrap(),
            None => return Err("No system found in TOML".to_string()),
        };

        let systems: Vec<(String, SystemBuilder)> = Vec::new();

        for (key, value) in toml_systems.iter() {
            let name = key.to_string();
            println!("Processing system: {}", name);
            let description = value
                .get("description")
                .expect("System doesn't have a description")
                .as_str()
                .unwrap()
                .to_string();
            let shell = value
                .get("shell")
                .expect("System doesn't have a shell")
                .as_str()
                .unwrap()
                .to_string();
            let stages = value
                .get("stages")
                .expect("System doesn't have stages; expected an array.")
                .as_array()
                .map(|a| a.iter().map(|s| 
                    s.as_str().unwrap().to_string())
                .collect::<Vec<String>>())
                .unwrap();
            let schedules = value
                .get("schedules")
                .expect("System doesn't have a schedule")
                .as_array()
                .map(|a| a.iter().map(|s| 
                    s.as_str().unwrap().to_string())
                .collect::<Vec<String>>())
                .unwrap();
            

            let mut system_builder = SystemBuilder::new(name.to_owned(), description, shell);
            
            for (stage, schedule) in stages.iter().zip(schedules.iter()) {
                let stage = SStage::new(stage.to_owned(), Schedule::from_str(schedule.as_str()).unwrap());
                system_builder.add_stage(stage);
            }
        }

        Ok(systems)
    }

    
    fn return_as_command(&self) -> Result<Vec<(String, CommandBuilder)>, String> {
        let parsed_content = self.parsed_content.as_ref().unwrap();
        let table = parsed_content.as_table().unwrap();

        let toml_commands = match table.get("command") {
            Some(cmd) => cmd.as_table().unwrap(),
            None => return Err("No command found in TOML".to_string()),
        };

        let mut commands: Vec<(String, CommandBuilder)> = Vec::new();

        for (key, value) in toml_commands.iter() {
            let name = key.to_string();
            println!("Processing command: {}", name);
            let description = value
                .get("description")
                .expect("Command doesn't have a description")
                .as_str()
                .unwrap()
                .to_string();
            let shell = value
                .get("shell")
                .expect("Command doesn't have a shell")
                .as_str()
                .unwrap()
                .to_string();

            let mut command_builder = CommandBuilder::new(name.to_owned(), description, shell);

            // Handle stages
            if let Some(toml_stages) = value.get("stages").and_then(Value::as_array) {
                for stage in toml_stages {
                    if let Some(stage_str) = stage.as_str() {
                        // Check for '%' at the beginning
                        if stage_str.starts_with('%') {
                            // Extract parameters
                            if let Some(params_str) = stage_str
                                .strip_prefix('%')
                                .and_then(|s| s.split_once('[').map(|(_, params)| params.trim()))
                            {
                                let (params, rest_of_command) = params_str.split_once(']').unwrap();
                                let params_vec = params.split(',').collect::<Vec<&str>>();
                                let spep = self.handle_special_parameters_cmd(
                                    &mut command_builder,
                                    &params_vec,
                                );

                                if !rest_of_command.is_empty() {
                                    command_builder.add_stage(
                                        rest_of_command.to_string(),
                                        spep.color,
                                        spep.sorted,
                                    );
                                }
                            } else {
                                eprintln!("Failed to extract parameters from: {}", stage_str);
                            }
                        } else {
                            command_builder.add_stage(stage_str.to_string(), Color::White, None);
                        }
                    } else {
                        return Err("Invalid stage format; expected string.".to_string());
                    }
                }
            } else {
                return Err("Command doesn't have stages; expected an array.".to_string());
            }

            // Check if command_builder has stages added
            if command_builder.stages.len() > 0 {
                commands.push((name, command_builder));
            } else {
                eprintln!("Command '{}' has no stages.", name);
            }
        }

        Ok(commands)
    }

    /// Handle special parameters and modify the CommandBuilder accordingly.
    fn handle_special_parameters_cmd(
        &self,
        command_builder: &mut CommandBuilder,
        params: &[&str],
    ) -> SpecialFields {
        let mut color = Color::White;
        let mut sorted = false; // Whether to sort the output
        let mut elements_per_line: usize = 1;

        for param in params {
            if param.starts_with("color:") {
                let colorr = param.strip_prefix("color:").unwrap().to_string();
                let colorrr = Color::from_str(&colorr);
                color = colorrr.unwrap_or(Color::White);
                continue;
            }

            if param.starts_with("o") {
                sorted = true;
                let flags_string = param.strip_prefix("o:").unwrap().to_string();
                let flags = flags_string.split(';').collect::<Vec<&str>>();

                for flag in flags {
                    if flag == "-s" {
                        sorted = false;
                    }

                    if flag.starts_with("l") {
                        elements_per_line =
                            flag.strip_prefix("l").unwrap().parse::<usize>().unwrap();
                        let s = if sorted { "sorting" } else { "organizing" };
                    }
                }

                continue;
            }

            match *param {
                _ => {
                    eprintln!("Unknown parameter: {}", param);
                }
            }
        }

        let sort_rules = SortRules::new(elements_per_line, sorted);
        let special_fields = SpecialFields::new(color, Some(sort_rules));

        return special_fields;
    }
}

#[derive(Debug, Clone)]
pub struct SpecialFields {
    pub color: Color,
    pub sorted: Option<SortRules>,
}

impl SpecialFields {
    pub fn new(color: Color, sort_rules: Option<SortRules>) -> Self {
        Self {
            color,
            sorted: sort_rules,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SortRules {
    pub sorting: bool,
    pub items_per_line: usize,
}

impl SortRules {
    pub fn new(i_p_l: usize, sorting: bool) -> Self {
        Self {
            sorting,
            items_per_line: i_p_l,
        }
    }
}
