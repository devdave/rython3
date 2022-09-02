
use regex::{Regex};
use unicode_segmentation::UnicodeSegmentation;
// use unicode_segmentation::UnicodeSegmentation;


#[derive(Debug, Clone)]
pub struct ManagedLine<'a>  {
    pub lineno: usize,
    pub idx: usize,
    pub text: &'a str,
    content: Vec<char>,
}

#[allow(non_snake_case)]
impl<'a> ManagedLine<'a>  {

    pub fn Make(lineno: usize, input: &'a str) -> Self {
        let mut hack = Self {
            lineno,
            idx: 0,
            text: input,
            content: vec!['b'],
        };

        hack.content = hack.text.graphemes(true).as_str().chars().collect();

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

        //Honestly... no idea if this worked as I don't test for unicode right now.
        let remaining = &self.text.graphemes(true).as_str()[self.idx..];
        let test = pattern.find(remaining);
        if let Some(found) = test {
            // let len = found.end() - found.start();
            self.idx += found.as_str().len();
            return Some((self.idx.clone(), found.as_str()));
        }
        None
    }

    pub fn advance(&mut self, amount: usize) {
        self.idx += amount;
    }

    pub fn return_all(&mut self) -> &str {
        let remaining = &self.text[self.idx..];
        self.idx = self.len()+1;
        return remaining;
    }
}

#[cfg(test)]
mod test {

    use super::ManagedLine;
    use crate::tokenizer::operators::OPERATOR_RE;

        #[test]
    fn managed_line_works() {
        let mut line = ManagedLine::Make(1,"abc");
        assert_eq!(line.peek().unwrap(), 'a');
    }

    #[test]
    fn managed_line_gets_to_end() {
        let mut line = ManagedLine::Make(1,"abc");
        assert_eq!(line.get().unwrap(), 'a');
        assert_eq!(line.get().unwrap(), 'b');
        assert_eq!(line.get().unwrap(), 'c');
        assert_eq!(line.get(), None);
    }

    #[test]
    fn managed_line_goes_back() {
        let mut line = ManagedLine::Make(1,"abc");
        assert_eq!(line.get().unwrap(), 'a');
        assert_eq!(line.get().unwrap(), 'b');
        assert_eq!(line.get().unwrap(), 'c');
        assert_eq!(line.get(), None);
        line.backup();
        assert_eq!(line.peek().unwrap(), 'c');
        assert_eq!(line.get().unwrap(), 'c');
        assert_eq!(line.get(), None);
        assert_eq!(line.get(), None);
    }

    #[test]
    fn managed_line_swallows_operators() {

        let mut sane = ManagedLine::Make(1,"()[]");
        //Somewhat problematic here is the regex is still Lazy<Regex> at this point
        // so for now primestart it
        let (_current_idx, retval1) = sane.test_and_return(&OPERATOR_RE.to_owned()).unwrap();

        assert_eq!(retval1.len(), 1);
        assert_eq!(retval1, "(");

        let (_current_idx, retval2) = sane.test_and_return(&OPERATOR_RE.to_owned()).unwrap();
        assert_eq!(retval2, ")");

        let (_current_idx, retval3) = sane.test_and_return(&OPERATOR_RE.to_owned()).unwrap();
        assert_eq!(retval3, "[");

        let (_current_idx, retval4) = sane.test_and_return(&OPERATOR_RE.to_owned()).unwrap();
        assert_eq!(retval4, "]");


    }
}