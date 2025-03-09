use std::fmt::Display;

use crate::sql::tokenizer::str_scanner::TokenLocation;

#[derive(Debug)]
pub struct ParseError {
    message: Box<str>,
    location: TokenLocation,
    raw_sql: Box<str>,
}

impl ParseError {
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

impl std::error::Error for ParseError {}

impl Display for ParseError {
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
