
use std::str::Chars;

/**
Goal is to take a raw string and convert it to a Vec<String> of lines ending with "\n"


*/

struct Block2Lines {
    body: String,
    pos: usize,
}

impl Block2Lines {
    pub fn new(input: String) -> Self {
        Self {
            body: input,
            pos: 0,
        }
    }


    pub fn transform(&mut self) -> Vec<String> {

        let mut temp = "".to_string();
        let mut product: Vec<String> = Vec::new();
        let mut iter: Chars = self.body.chars();


        while let Some(sym) = iter.next() {
            if sym == '\r' {
                let mut tester = iter.clone();
                if let Some(peek) = tester.next() {
                    if peek == '\n' {
                        // \r\n - compact to \n
                        iter = tester;  // Skip ahead
                        temp.push('\n');
                        product.push(temp);
                        temp = "".to_string();
                    } else {
                        // Assume just \r on it's own
                        temp.push('\n');
                        product.push(temp);
                        temp = "".to_string();
                    }
                } else {
                    // Reached EOF
                    temp.push('\n');
                    product.push(temp);
                    temp = "".to_string();
                }
            } else if sym == '\n' {
                // Catch correct line endings and split
                temp.push(sym);
                product.push(temp);
                temp = "".to_string();

            } else {
                // Catch whatever else is not \r or \n
                temp.push(sym);
            }
        }

        if temp != "" || temp.len() > 0 {
            product.push(temp);
        }


        return product;

    }



}


pub fn NLTransformer<'a>(raw: &'a str) -> Vec<String> {

    return Block2Lines::new(raw.to_string()).transform();
}

pub fn Str2Vec(raw: &str) -> Vec<String> {
    return Block2Lines::new(raw.to_string()).transform();
}

pub fn String2Vec(raw: String) -> Vec<String> {
    return Block2Lines::new( raw).transform();
}

#[cfg(test)]
mod test {
    use crate::lexer::NLTransformer::NLTransformer;

    #[test]
    fn basic_test() {
        let data = "Hello\nWorld\nTest!\n";
        let actual = NLTransformer(data);

        assert_eq!(actual.len(), 3);

    }

    #[test]
    fn indepth_test() {
        let data = "Hello\nWorld\nTest!\n";
        let actual = NLTransformer(data);

        assert_eq!(actual.len(), 3);

        assert_eq!(actual[0], "Hello\n".to_string());
        assert_eq!(actual[1], "World\n".to_string());
        assert_eq!(actual[2], "Test!\n".to_string());
    }

    #[test]
    fn mac_test() {
        let data = "Hello\rWorld\r";
        let actual = NLTransformer(data);

        assert_eq!(actual.len(), 2);

        assert_eq!(actual[0], "Hello\n".to_string());
        assert_eq!(actual[1], "World\n".to_string());
    }
}

