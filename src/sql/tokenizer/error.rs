use std::fmt::Display;

use super::str_scanner::TokenLocation;

#[derive(Debug)]
pub struct TokenizeError {
    message: Box<str>,
    location: TokenLocation,
    raw_sql: Box<str>,
}

impl TokenizeError {
    pub fn new<Msg: Into<Box<str>>, SQL: Into<Box<str>>>(
        message: Msg,
        location: TokenLocation,
        raw_sql: SQL,
    ) -> Self {
        Self {
            message: message.into(),
            location: location,
            raw_sql: raw_sql.into(),
        }
    }
}

impl std::error::Error for TokenizeError {}

impl Display for TokenizeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const SKIP_BACKWARD: usize = 16;
        let skip: usize = {
            if self.location.offset > SKIP_BACKWARD {
                self.location.offset - SKIP_BACKWARD
            } else {
                0
            }
        };
        let near: String = self
            .raw_sql
            .chars()
            .skip(skip)
            .take(SKIP_BACKWARD * 2)
            .collect();
        f.write_fmt(format_args!(
            "error {} as Ln {}, Col {} near \"{}\"",
            self.message, self.location.line_number, self.location.column_number, near
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_error() {
        let mut loc = TokenLocation::new();
        "SELECT 1, ".chars().for_each(|c| loc.next_char(c));

        let error: TokenizeError =
            TokenizeError::new("unknown char @", loc, "SELECT 1, @a FROM stu WHERE a > 1;");
        println!("{}", error);

        assert_eq!(
            format!("{}", error),
            "error unknown char @ as Ln 1, Col 11 near \"SELECT 1, @a FROM stu WHERE a > \""
        );
    }
}
