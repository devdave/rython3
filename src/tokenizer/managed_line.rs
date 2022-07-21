
use regex::{Regex};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Clone)]
pub struct ManagedLine {
    pub lineno: usize,
    pub idx: usize,
    pub text: String,
    content: Vec<char>,
}

#[allow(non_snake_case)]
impl ManagedLine {

    pub fn Make(lineno: usize, input: String) -> Self {
        let mut hack = Self {
            lineno,
            idx: 0,
            text: input,
            content: vec!['b'],
        };
        hack.content = hack.text[..].to_string().chars().collect();

        return hack;
    }

    pub fn get_idx(&self) -> usize {
        self.idx
    }

    pub fn get(&mut self) -> Option<char> {

        if self.content.len() > self.idx {

            let retval = self.content[self.idx] as char;
            self.idx += 1;
            Some(retval)
        } else {
            None
        }
    }

    pub fn backup(&mut self) {
        self.idx -= 1;
    }

    pub fn peek(&mut self) -> Option<char> {
        if self.content.len() > self.idx {
            Some(self.content[self.idx] as char)
        } else {
            None
        }
    }

    pub fn get_pos(&self) -> usize {
        return self.idx;
    }

    pub fn len(&self) -> usize {
        return self.content.len();
    }

    pub fn remaining(&self) -> usize {
        self.len() - self.get_pos()
    }

    pub fn test_and_return(&mut self, pattern: &Regex) -> Option<(usize, &str)> {

        let remaining = &self.text[self.idx..];
        let test = pattern.find(remaining);
        if let Some(found) = test {
            let len = found.end() - found.start();
            self.idx += found.start() + len;
            return Some((self.idx.clone(), found.as_str()));
        }
        None
    }

    pub fn advance(&mut self, amount: usize) {
        self.idx += amount;
    }
}