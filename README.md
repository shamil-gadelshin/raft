# raft
Raft consensus implementation in Rust


This project is intended to fast prototyping of the distributed software. It is based on [Diego Ongaro's dissertation](../master/doc/raft_dissertation.pdf).
Full information about the Raft you can get at its [official site](https://raft.github.io/).

These Raft features are implemented:

- Leadership
  + election
  + status watcher 
  
- Operation log support
  + creation
  + replication
    * basic
    * forced

- Replicated state machine support

- Cluster membership changes 
  + add server (basic support)
  
- Client RPC (partial):
  + add server
  + add data
  
  
These features should be implemented to meet full Raft requirements:

- Cluster membership changes
  + add server (round-based implementation with TIMEOUT-reply to clients)
  + remove server
    * transfer leadership

- Client RPC
  + remove server
  + session support (for proper duplicate request handling)
  + read(query) RPC
  
- Log compaction


This project designed to be modular and independent of the actual operation log, replicated state machine, communication protocol, etc implementations.
Basic (example) implementations were separated in the [subproject](../master/raft-modules)

Plenty of different test configurations and cases can be found in [this subproject](../master/tests/cases).
