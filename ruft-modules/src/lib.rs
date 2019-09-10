#[macro_use] extern crate log;
extern crate ruft;

mod memory_log;
mod memory_fsm;

pub use memory_fsm::MemoryFsm;
pub use memory_log::MemoryLogStorage;
