

pub enum StringType {
    NONE,
    AposTriple,
    QuoteTriple,
    APOS,
    QUOTE,
}


pub struct LexerState {
    pub string_continues: bool,
    pub string_type: StringType,
    pub string_body: String,
}

impl LexerState {

    pub fn new() -> Self {
        Self {
            string_continues: false,
            string_type: StringType::NONE,
            string_body: "".to_string(),
        }
    }

}