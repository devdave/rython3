
use super::code_line::CodeLine;

#[derive(Clone)]
pub struct CodeModule<'a> {
    lines: Vec<CodeLine<'a>>,
    pos: usize,
}

impl <'a> CodeModule<'a> {

    pub fn new(input: Vec<String>) -> Self {

        let temp = input.into_iter().map(|el| CodeLine::new(el.as_str())).collect();
        Self {
            lines: temp,
            pos: 0,
        }
    }

    pub fn remaining(&self) -> usize {
        self.lines.len().saturating_sub(self.pos)
    }

    pub fn len(&self) -> usize {
        self.lines.len()
    }

}

impl <'a> Iterator for CodeModule<'a> {
    type Item = &'a CodeLine<'a>;

    fn next(&mut self) -> Option<&'a CodeLine<'a>> {
        if self.pos >= self.lines.len() {
            None
        } else {
            let retval = self.lines.get(self.pos).expect(format!("codeline@{}", self.pos).as_str()).clone();
            self.pos += 1;
            //TODO this is one more spot that bleeds memory
            return Some(&retval);
        }
    }
}

#[cfg(test)]
mod test {
    use crate::lexer::code_module::CodeModule;
    use crate::lexer::NLTransformer::{Str2Vec, String2Vec};
    use regex::Regex;

    #[test]
    fn basic() {
        let raw = Str2Vec("Hello\nWorld\nTest\nIterator\nWorks\n");

        let module = CodeModule::new(raw);
        assert_eq!(5, module.len());
        //TODO this could/is problematic that I cannot get the data back out!
        // let mut test: Vec<&str> = module.map(|el| el.get_line()).collect();
        // assert_eq!(test, data);
    }

    #[test]
    fn mockup() {
        let name_re = Regex::new(r"\A[a-zA-Z]{1}[\w\d]+").expect("regex");


    }
}