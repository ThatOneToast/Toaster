
#[derive(Debug, Clone)]
pub struct Settings {
    pub threads: usize,
}

impl Settings {
    pub fn new(threads: usize) -> Self {
        Self {
            threads,
        }
    }
}