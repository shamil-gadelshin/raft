use std::error::Error;
use std::fmt::Display;
use core::fmt;

#[derive (Debug)]
pub struct RaftError {
	text : String,
	cause: String
}

pub(crate) type Result<T> = std::result::Result<T, RaftError>;

pub fn new_err<T>(text : String, cause : String) -> Result<T>{
	Err(RaftError {text, cause})
}

impl Display for RaftError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let cause_word = {
			if !self.cause.is_empty() {
				" Cause: ".to_string()
			} else {
				String::new()
			}
		};
		write!(f, "{}.{}{}", self.text, cause_word, self.cause)
	}
}

impl Error for RaftError {}

pub(crate) fn new_multiple_err<T>(text : String, causes :  Vec<RaftError>) -> Result<T>{
	let mut error_string = String::new();

	if !causes.is_empty() {
		error_string.push_str("Errors: ");
	}

	let mut error_index = 1;
	for err in causes{
		error_string.push_str(&format!("{}) {}",error_index, err));
		error_index += 1;
	}
	Err(RaftError{text, cause: error_string})
}
