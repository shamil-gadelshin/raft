# Raft Modules 

This subproject of the [raft](https://github.com/shamil-gadelshin/raft) provides examples or test implementations of the Raft Modules.

## Modules

- Operation log
  + Supports operation log entries processing and calculating parameters (index, term, etc).
- Replicated state machine
  + Supports operations with replicated state machine (like 'append new entry').
- Peer request handler
  + Module responsible for communication between Raft Nodes.
- Client request handler
  + This module responsible for client communication. Module handles request like 'add new data' or 'add server'.
- Cluster
  + Defines cluster membership and quorum rules.
- Election timer
  + Calculates duration to the next elections.
- Node state saver
  + Responsible for node state persistence


## Implementations

- Operation log 
  + MemoryOperationLog
    * Memory based operation log. No persistence. For testing.
- Replicated state machine 
  + MemoryRsm
    * Memory based state machine emulation. No persistence. For testing.
- Peer request handler 
  + InProcPeerCommunicator
    * Channel-based implementation. For in-process testing.
  + NetworkPeerCommunicator
    * Grpc-based implementation. Can be used as template for real-case.
- Client request handler 
  + InProcClientCommunicator
    * Channel-based implementation. For in-process testing.
  + NetworkClientCommunicator
    * Grpc-based implementation. Can be used as template for real-case.
- Cluster
  + ClusterConfiguration
    * Calculates quorum as majority. 
- Election timer
  + RandomizedElectionTimer
    * Provides random time duration within a range. 
  + FixedElectionTimer
    * Provides fixed time duration. For 'guaranteed leadership' tests.
- Node state saver
  + MockNodeStateSaver
    * Mock implentation of node state persistence.
  
