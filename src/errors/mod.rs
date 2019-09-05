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

#[derive (Debug)]
pub struct RuftMultipleError {
	text : String,
	causes: Vec<Box<Error>>
}

pub fn new_multiple_err<T>(text : String, causes :  Vec<Box<Error>>) -> Result<T>{
	Err(Box::new(RuftMultipleError{text, causes}))
}

impl Display for RuftMultipleError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let mut error_string = String::new();
		for err in &self.causes{
			error_string.push_str(err.description());
		}
		write!(f, "{}. Cause {}", self.text, error_string)
	}
}

impl Error for RuftMultipleError{}