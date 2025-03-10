use std::{iter::Peekable, str::Chars};

#[derive(Debug, Clone, Copy)]
pub struct TokenLocation {
    pub line_number: usize,
    pub column_number: usize,
    pub offset: usize,
    previous: char, // \n \r\n \r
}

impl TokenLocation {
    pub fn new() -> Self {
        Self {
            line_number: 1,
            column_number: 1,
            offset: 0,
            previous: '\0',
        }
    }

    pub fn uninit() -> Self {
        Self {
            line_number: 0,
            column_number: 0,
            offset: 0,
            previous: char::default(),
        }
    }

    pub fn next_char(&mut self, c: char) {
        match c {
            '\r' => {
                // new line
                self.column_number = 1;
                self.line_number += 1;
            }
            '\n' => {
                match self.previous {
                    '\r' => {
                        // \r\n
                        debug_assert!(self.column_number == 1)
                    }
                    _ => {
                        // new line
                        self.column_number = 1;
                        self.line_number += 1;
                    }
                }
            }
            _ => {
                self.column_number += 1;
            }
        }
        self.offset += 1;
        self.previous = c;
    }
}

pub struct Scanner<'a> {
    peekable: Peekable<Chars<'a>>,
    location: TokenLocation,
}

impl<'a> Scanner<'a> {
    pub fn new(text: &'a str) -> Scanner<'a> {
        Self {
            peekable: text.chars().peekable(),
            location: TokenLocation::new(),
        }
    }

    pub fn location(&self) -> TokenLocation {
        self.location.clone()
    }

    /// peek the next but not consume
    pub fn peek(&mut self) -> Option<char> {
        self.peekable.peek().map(|c| *c)
    }

    /// next advances the iter
    pub fn next(&mut self) {
        if let Some(c) = self.peekable.next() {
            self.location.next_char(c);
        }
    }
}

#[cfg(test)]
mod test {
    use super::Scanner;

    #[test]
    fn one_word() {
        let text = "hello";
        let mut s = Scanner::new(text);
        while let Some(c) = s.peek() {
            // println!("{}, {:?}", c, s.location());
            assert_eq!(c, text.chars().nth(s.location.offset).unwrap());
            s.next();
        }
        assert_eq!(s.location.column_number, text.chars().count() + 1);
        assert_eq!(s.location.line_number, 1);
        assert_eq!(s.location.offset, text.chars().count());
    }

    #[test]
    fn one_utf8word() {
        let text = "helle 你好 123";
        let mut s = Scanner::new(text);
        while let Some(c) = s.peek() {
            // println!("{}, {:?}", c, s.location());
            assert_eq!(c, text.chars().nth(s.location.offset).unwrap());
            s.next();
        }
        assert_eq!(s.location.column_number, text.chars().count() + 1);
        assert_eq!(s.location.line_number, 1);
        assert_eq!(s.location.offset, text.chars().count());
    }

    #[test]
    fn one_newline() {
        let text = "helle\r 你\n好 1\r\n23";
        let mut s = Scanner::new(text);
        while let Some(c) = s.peek() {
            // println!("{}, {:?}", c, s.location());
            assert_eq!(c, text.chars().nth(s.location.offset).unwrap());
            s.next();
        }
        assert_eq!(s.location.column_number, 3);
        assert_eq!(s.location.line_number, 4);
        assert_eq!(s.location.offset, text.chars().count());
    }

    #[test]
    fn one_newline2() {
        let text = "\n\n123\n\n123";
        let mut s = Scanner::new(text);
        while let Some(c) = s.peek() {
            // println!("{}, {:?}", c, s.location());
            assert_eq!(c, text.chars().nth(s.location.offset).unwrap());
            s.next();
        }
        assert_eq!(s.location.column_number, 4);
        assert_eq!(s.location.line_number, 5);
        assert_eq!(s.location.offset, text.chars().count());
    }

    #[test]
    fn one_newline3() {
        let text = "\r\n\r\n123\r\n\r\n123";
        let mut s = Scanner::new(text);
        while let Some(c) = s.peek() {
            // println!("{}, {:?}", c, s.location());
            assert_eq!(c, text.chars().nth(s.location.offset).unwrap());
            s.next();
        }
        assert_eq!(s.location.column_number, 4);
        assert_eq!(s.location.line_number, 5);
        assert_eq!(s.location.offset, text.chars().count());
    }
}
