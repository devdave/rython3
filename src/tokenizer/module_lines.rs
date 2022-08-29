

use super::managed_line::ManagedLine;

#[derive(Debug, Clone)]
pub struct ModuleLines<'a> {
    idx: usize,
    name: String,
    content: Vec<ManagedLine<'a>>,
}

// impl <'a> ModuleLines<'a> {
//
//     pub fn Make(lines: Vec<String>, name: String) -> Self {
//
//         let managed: Vec<ManagedLine> = lines.iter().enumerate().map(|(lineno, content)| ManagedLine::Make(lineno,content )).collect();
//         Self {
//             idx: 0,
//             name: name,
//             content: managed,
//         }
//     }
//
//     pub fn get_lineno(&self) -> usize {
//         self.idx
//     }
//
//     pub fn set_lines(&mut self, lines: Vec<String>, name: String) {
//         let managed: Vec<ManagedLine> = lines.iter().enumerate().map(|(lineno, content)| ManagedLine::Make(lineno,content)).collect();
//         self.name = name;
//         self.content = managed;
//     }
//
//     pub fn has_lines(&self) -> bool {
//         self.idx < self.content.len()
//     }
//
//     pub fn peek(&self) -> Option<&ManagedLine> {
//         if self.idx < self.content.len() {
//             self.content.get(self.idx)
//         } else {
//             None
//         }
//
//     }
//
//     pub fn get(& 'a mut self) -> Option<ManagedLine<'a>> {
//         if self.idx < self.content.len() {
//             let retval = self.content.get(self.idx).unwrap();
//             let duplicate = ManagedLine::Make(retval.lineno, retval.text);
//             self.idx += 1;
//             return Some(duplicate);
//         } else {
//             return None;
//         }
//
//     }
//
//     pub fn remaining(&self) -> usize {
//         return self.len().saturating_sub(self.idx);
//     }
//
//     pub fn len(&self) -> usize {
//         self.content.len()
//     }
//
//
//     pub fn advance_one(&mut self) {
//         self.idx += 1;
//     }
//
// }
// //
// // impl <'a> Iterator for ModuleLines<'a> {
// //     type Item = (usize, ManagedLine<'a>);
// //
// //     fn next(&mut self) -> Option<(usize, ManagedLine<'a>)> {
// //
// //         if let Some(temp) = self.get(){
// //             return Some((self.idx, temp));
// //         } else {
// //             return None;
// //         }
// //
// //     }
// //
// // }
//
//
// #[cfg(test)]
// mod test {
//
//
//
// }
