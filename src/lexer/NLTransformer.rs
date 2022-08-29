use std::os::windows::fs::symlink_dir;
use std::str::Chars;

/**
Goal is to take a raw string and convert it to a Vec<String> of lines ending with "\n"


*/

struct Block2Lines {
    body: String,
    pos: usize,
}

impl Block2Lines {
    fn new(input: String) -> Self {
        Self {
            body: input,
            pos: 0,
        }
    }


    fn transform(&mut self) -> Vec<String> {
        let mut temp = String::new();
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
                        temp = String::new();
                    } else {
                        // Assume just \r on it's own
                        temp.push('\n');
                        product.push(temp);
                        temp = String::new();
                    }
                } else {
                    // Reached EOF
                    temp.push('\n');
                    product.push(temp);
                    temp = String::new();
                }
            } else if sym == '\n' {
                // Catch correct line endings and split
                temp.push(sym);
                product.push(temp);
                temp = String::new();

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


fn NLTransformer(raw: String) -> Vec<String> {

    return Block2Lines::new(raw).transform();

}

#[cfg(test)]
mod test {
    use crate::lexer::NLTransformer::NLTransformer;

    #[test]
    fn basic_test() {
        let data = "Hello\nWorld\nTest!\n".to_string();
        let actual = NLTransformer(data);

        assert_eq!(actual.len(), 3);

    }

    #[test]
    fn indepth_test() {
        let data = "Hello\nWorld\nTest!\n".to_string();
        let actual = NLTransformer(data);

        assert_eq!(actual.len(), 3);

        assert_eq!(actual[0], "Hello\n".to_string());
        assert_eq!(actual[1], "World\n".to_string());
        assert_eq!(actual[2], "Test!\n".to_string());
    }

    #[test]
    fn mac_test() {
        let data = "Hello\rWorld\r".to_string();
        let actual = NLTransformer(data);

        assert_eq!(actual.len(), 2);

        assert_eq!(actual[0], "Hello\n".to_string());
        assert_eq!(actual[1], "World\n".to_string());
    }
}

