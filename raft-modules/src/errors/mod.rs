use std::error::Error;
use std::fmt::Display;
use core::fmt;

#[derive (Debug)]
pub struct RuftError {
	text : String,
	cause: Option<Box<Error>>
}

pub type Result<T> = std::result::Result<T, Box<Error>>;

pub fn new_err<T>(text : String, cause : Option<Box<Error>>) -> Result<T>{
	Err(Box::new(RuftError{text, cause}))
}

impl Display for RuftError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}. Cause {:?}", self.text, self.cause)
	}
}

impl Error for RuftError{}

