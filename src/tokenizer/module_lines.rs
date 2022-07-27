

use super::managed_line::ManagedLine;

#[derive(Debug, Clone)]
pub struct ModuleLines {
    idx: usize,
    name: String,
    content: Vec<ManagedLine>,
}

impl ModuleLines {

    pub fn Make(lines: Vec<String>, name: String) -> Self {

        let managed: Vec<ManagedLine> = lines.iter().enumerate().map(|(lineno, content)| ManagedLine::Make(lineno,content.to_string())).collect();
        Self {
            idx: 0,
            name: name,
            content: managed,
        }
    }

    pub fn get_lineno(&self) -> usize {
        self.idx
    }

    pub fn set_lines(&mut self, lines: Vec<String>, name: String) {
        let managed: Vec<ManagedLine> = lines.iter().enumerate().map(|(lineno, content)| ManagedLine::Make(lineno,content.to_string())).collect();
        self.name = name;
        self.content = managed;
    }

    pub fn has_lines(&self) -> bool {
        self.idx < self.content.len()
    }

    pub fn peek(&self) -> Option<&ManagedLine> {
        if self.idx < self.content.len() {
            self.content.get(self.idx)
        } else {
            None
        }

    }

    pub fn get(&mut self) -> Option<ManagedLine> {
        if self.idx < self.content.len() {
            let retval = self.content.get(self.idx).unwrap();
            let duplicate = ManagedLine::Make(retval.lineno, retval.text.to_string());
            self.idx += 1;
            return Some(duplicate);
        } else {
            return None;
        }

    }

    pub fn remaining(&self) -> usize {
        return self.len().saturating_sub(self.idx);
    }

    pub fn len(&self) -> usize {
        self.content.len()
    }


    pub fn advance_one(&mut self) {
        self.idx += 1;
    }

}

impl Iterator for ModuleLines {
    type Item = (usize, ManagedLine);

    fn next(&mut self) -> Option<Self::Item> {

        if let Some(temp) = self.get(){
            return Some((self.idx, temp));
        } else {
            return None;
        }



    }

}


#[cfg(test)]
mod test {
    use super::ModuleLines;


}
