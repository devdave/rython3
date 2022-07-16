use regex::Regex;

pub struct ManagedLine {
    pub idx: usize,
    text: String,
    content: Vec<char>,
}

#[allow(non_snake_case)]
impl ManagedLine {

    pub fn Make(input: String) -> Self {
        Self {
            idx: 0,
            text: input,
            content: input.chars().collect(),
        }
    }

    pub fn get_idx(&mut self) -> usize {
        self.idx
    }

    pub fn get(&mut self) -> Option<char> {

        if self.content.len() > self.idx {
            let retval = self.content[self.idx] as char;
            // let retval = self.content.get(self.idx).unwrap() as char;
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
            return Some((self.idx, found.as_str()));
        }
        None
    }
}