use crate::{color::Color, tomlp::SortRules};

#[derive(Debug, Clone)]
pub struct Stage {
    /// The command to run
    pub command: String,
    /// The color to use for the output
    pub color: Color,
    /// Whether to sort the output
    pub sorted: Option<SortRules>,
}

impl Stage {
    pub fn new(command: String, color: Color, sorted: Option<SortRules>) -> Self {
        Self {
            command,
            color,
            sorted,
        }
    }
}