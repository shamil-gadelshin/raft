use std::error::Error;
use std::fmt::Display;
use core::fmt;

#[derive (Debug)]
pub struct RaftError {
	text : String,
	cause: String
}

pub type Result<T> = std::result::Result<T, RaftError>;

pub fn new_err<T>(text : String, cause : String) -> Result<T>{
	Err(RaftError {text, cause})
}

impl Display for RaftError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}. Cause {:?}", self.text, self.cause)
	}
}

impl Error for RaftError {}

pub fn new_multiple_err<T>(text : String, causes :  Vec<RaftError>) -> Result<T>{
	let mut error_string = String::new();
	for err in causes{
		error_string.push_str(err.description());
		error_string.push_str(";");
	}
	Err(RaftError{text, cause: error_string})
}
