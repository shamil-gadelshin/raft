# raft
Raft consensus implementation in Rust


This project is intended to fast prototyping of the distributed software. It is based on [Diego Ongaro's dissertation](../master/doc/raft_dissertation.pdf).
Full information about the Raft you can get at its [official site](https://raft.github.io/).

## These Raft features are implemented

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
  
  
## These features should be implemented to meet full Raft requirements

- Cluster membership changes
  + add server (round-based implementation with TIMEOUT-reply to clients)
  + remove server
    * transfer leadership

- Client RPC
  + remove server
  + session support (for proper duplicate request handling)
  + read(query) RPC
  
- Log compaction


Project designed to be modular and independent of the actual implementations.

## These modules must be implemented in order to run Raft Node

- **Operation log** - supports operation log entries processing and calculating parameters (index, term, etc).
- **Replicated state machine** - supports operations with replicated state machine (like 'append new entry').
- **Peer request handler** - Module responsible for communication between Raft Nodes.
- **Client request handler** - this module responsible for client communication. Module handles request like 'add new data' or 'add server'.
- **Cluster** - defines cluster membership and quorum rules.
- **Election timer** - calculates duration to the next elections.
- **Node limits** - defines node timeouts and limits (e.g.: max request data size)
- **Node state saver** - responsible for node state persistence

## Raft Modules implementations

Basic (example) implementations were separated in the [subproject](../master/raft-modules)

## Test cases and examples
Assembled Raft Node server can be found [here](../node), with [client example](../master/tests/client)

Plenty of different test configurations and cases can be found in [this subproject](../master/tests/cases).
