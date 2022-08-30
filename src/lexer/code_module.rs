use super::code_line::CodeLine;

pub struct CodeModule {
    lines: Vec<CodeLine>,
    pos: usize,
}

impl CodeModule {

    pub fn new(input: Vec<String>) -> Self {
        let mut codelines: Vec<CodeLine> = Vec::new();

        for raw_line in input {
            let codeline = CodeLine::new(raw_line);
            codelines.push(codeline);
        }

        Self {
            lines: codelines,
            pos: 0,
        }
    }

    pub fn new_str(input: &Vec<&str>) -> Self {
        let clean: Vec<String> = input.iter().map(|el| el.to_string()).collect();
        return CodeModule::new(clean);
    }

    pub fn remaining(&self) -> usize {
        self.lines.len().saturating_sub(self.pos)
    }

    pub fn len(&self) -> usize {
        self.lines.len()
    }

}

impl Iterator for CodeModule {
    type Item = CodeLine;

    fn next(&mut self) -> Option<CodeLine> {
        if self.pos >= self.lines.len() {
            None
        } else {
            let retval = self.lines.get(self.pos).expect(format!("codeline@{}", self.pos).as_str());
            self.pos += 1;
            //TODO this is one more spot that bleeds memory
            return Some(retval.clone());
        }
    }
}

#[cfg(test)]
mod test {
    use crate::lexer::code_module::CodeModule;

    #[test]
    fn basic() {
        let data = vec!["Hello\n", "World\n", "Test\n", "Iterator\n", "Works\n"];
        let module = CodeModule::new_str(&data);
        assert_eq!(5, module.len());
        let mut test: Vec<String> = Vec::new();
        for el in module {
            test.push(el.get_line());
        }

        assert_eq!(test, data);




    }
}