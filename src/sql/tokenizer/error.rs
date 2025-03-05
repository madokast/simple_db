use super::str_scanner::TokenLocation;


#[derive(Debug)]
pub struct Error {
    message:String,
    location:TokenLocation
}

impl Error {
    pub fn new(message:String, location:TokenLocation) -> Self {
        Self {message, location}
    }

    pub fn error_info(&self, raw_sql:&str) -> String {
        let near: String = raw_sql.chars()
            .skip(self.location.offset)
            .take(16).collect();
        format!("error {} as Ln {}, Col {} near {}", self.message, self.location.line_number, 
            self.location.column_number, near)
    }
}
