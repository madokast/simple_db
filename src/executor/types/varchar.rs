use std::{fmt::Display, marker::PhantomData};

/// Varchar 字符串，不持有所有权
pub struct Varchar<'a> {
    pointer: *const u8,
    size: u16,
    _marker: PhantomData<&'a ()>,
}

impl<'a> Varchar<'a> {
    pub fn from_str(s: &'a str) -> Self {
        Self {
            size: s.len() as u16,
            pointer: s.as_ptr(),
            _marker: PhantomData,
        }
    }

    pub fn as_str(&'a self) -> &'a str {
        unsafe {
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                self.pointer,
                self.size as usize,
            ))
        }
    }

    pub fn ref_string(s: &'a String) -> Self {
        Self::from_str(s.as_str())
    }

    pub fn to_string(&self) -> String {
        self.as_str().to_string()
    }
}

impl<'a> Display for Varchar<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
