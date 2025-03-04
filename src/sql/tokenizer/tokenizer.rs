use super::{error::Error, token::Token};

pub struct Tokenizer {

}

impl Tokenizer {
    pub fn new() -> Self {
        Self {  }
    }

    pub fn tokenize(&self, sql: &str) -> Result<Vec<Token>, Error> {
        return Ok(vec![]);
    }
}

#[cfg(test)]
mod test {
    use super::Tokenizer;

    #[test]
    fn select_1() {
        let _ = Tokenizer::new().tokenize("SELECT 1");
    }
}