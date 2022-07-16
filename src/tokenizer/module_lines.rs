use super::managed_line::ManagedLine;

struct ModuleLines {
    idx: usize,
    name: String,
    content: Vec<ManagedLine>,
}