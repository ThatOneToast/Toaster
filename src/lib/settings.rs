
#[derive(Debug, Clone)]
pub struct Settings {
    pub threads: usize,
    pub default_row_length: usize,
}

impl Settings {
    pub fn new(threads: usize, default_row_length: usize) -> Self {
        Self {
            threads, default_row_length
        }
    }
}