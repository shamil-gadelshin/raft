pub mod basic_replication;
pub mod leader_rsm;

#[cfg(test)]
mod tests {
    #[test]
    fn test_basic_rsm_replication() {
        crate::cases::replicated_state_machine::basic_replication::run()
    }

    #[test]
    fn test_leader_rsm() {
        crate::cases::replicated_state_machine::basic_replication::run()
    }
}
