use std::env;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Arguments {
    arguments: Vec<String>,
    dir: Option<String>,
    iterator: std::iter::Peekable<std::vec::IntoIter<String>>,
}

impl Arguments {
    pub fn new() -> Self {
        let args = env::args().collect::<Vec<String>>();
        let mut iterator = args.to_owned().into_iter().peekable();
        let path = iterator.next();

        let sself = Self {
            arguments: args,
            dir: path,
            iterator,
        };

        sself
    }

    pub fn current(&mut self) -> Option<&String> {
        self.iterator.peek()
    }

    pub fn next(&mut self) -> Option<String> {
        self.iterator.next()
    }

    pub fn peek(&mut self) -> Option<&String> {
        self.iterator.peek()
    }

    pub fn has_next(&mut self) -> bool {
        self.iterator.peek().is_some()
    }
}
