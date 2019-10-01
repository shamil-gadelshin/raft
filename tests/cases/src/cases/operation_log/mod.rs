pub mod basic_replication;
pub mod forced_replication;


#[cfg(test)]
mod tests {
	#[test]
	fn test_basic_replication() {
		crate::cases::operation_log::basic_replication::run()
	}

	#[test]
	fn test_forced_replication() {
		crate::cases::operation_log::forced_replication::run()
	}
}