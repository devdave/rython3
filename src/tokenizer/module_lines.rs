use std::process::id;
use super::managed_line::ManagedLine;

#[derive(Debug)]
pub struct ModuleLines<'t> {
    idx: usize,
    name: String,
    content: Vec<ManagedLine<'t>>,
}

impl <'t> ModuleLines<'t> {

    pub fn Make(lines: Vec<String>, name: String) -> Self {

        let managed: Vec<ManagedLine> = lines.iter().enumerate().map(|(lineno, content)| ManagedLine::Make(lineno,content)).collect();
        Self {
            idx: 0,
            name: name,
            content: managed,
        }
    }

    pub fn has_lines(&self) -> bool {
        self.idx <self.content.len()
    }

    pub fn peek(&mut self) -> Option<&ManagedLine> {
        if self.idx < self.content.len() {
            self.content.get(self.idx)
        } else {
            None
        }

    }

    pub fn get(&mut self) -> Option<&mut ManagedLine<'t>> {
        if self.idx < self.content.len() {
            let retval = self.content.get_mut(self.idx).unwrap();
            self.idx += 1;
            return Some(retval);
        } else {
            return None;
        }

    }

    pub fn len(&self) -> usize {
        self.content.len()
    }


}