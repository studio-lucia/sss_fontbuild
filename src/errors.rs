use std::fmt;
use std::error::Error;

#[derive(Debug)]
pub struct FontCreationError {
    message: String,
}

impl FontCreationError {
    pub fn new(message: String) -> FontCreationError {
        return FontCreationError {
            message: message,
        }
    }
}

impl Error for FontCreationError {
    fn description(&self) -> &str {
        return self.message.as_str();
    }
}

impl fmt::Display for FontCreationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}
