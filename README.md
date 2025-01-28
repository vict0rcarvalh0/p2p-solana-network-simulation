# P2P Solana Network Simulation

This project implements a peer-to-peer (P2P) network using the libp2p library, allowing nodes to send and receive transactions in a decentralized manner. The network consists of at least four nodes that subscribe to a "transaction" topic, enabling them to broadcast and receive dummy transactions. Additionally, the system maintains a distributed table that records peers and their associated transactions.

## Features
- P2P Network: Build a network of at least four nodes.
- Topic Subscription: Nodes subscribe to a common topic ("transaction") for transaction broadcasting.
- Dummy Transactions: Create and send dummy transactions between nodes.
- Distributed Table: Maintain a record of peers and their transactions.

## Prerequisites

- Rust programming language installed on your system. You can install it from rust-lang.org.
- A running Solana RPC node (e.g., local Solana test validator).

## Installation
**Clone the repository:**

```bash
git clone <repository-url>
cd <repository-directory>
```

**Build the project:**

```bash
cargo build
```

**Running the Project**

Start your Solana test validator if you haven't already:
```bash
solana-test-validator
```

Run the P2P network:
```bash
cargo run
```

This will create four nodes that connect to each other, subscribe to the "transaction" topic, and begin broadcasting dummy transactions.

## Code Overview
The main components of the code are as follows:

- Node Behavior: The NodeBehaviour struct defines how each node interacts with the network, utilizing Gossipsub for message passing and mDNS for peer discovery.
- Transaction Message Structure: The TransactionMessage struct represents the structure of a transaction message, including the transaction data (encoded in Base64), sender's PeerId, and timestamp.
- Distributed Table: The DistributedTable struct maintains two HashMaps:
    - peers: Maps PeerIds to lists of transactions.
    - transactions: Maps transaction hashes to transaction data.
- Node Initialization: The Node struct initializes each node with a unique PeerId, transport configuration, Gossipsub instance, and a distributed table. It also handles event loops for processing incoming messages.
- Transaction Broadcasting: The broadcast_transaction method creates a dummy Solana transaction and broadcasts it to all subscribed peers.

## How It Works
- Node Creation: The application creates four nodes that join the P2P network.
- Subscription: Each node subscribes to the "transaction" topic using Gossipsub.
- Transaction Generation: The first node generates a dummy transaction using Solana's system instructions.
- Broadcasting: The generated transaction is broadcasted across the network.
- Receiving Transactions: Other nodes receive the transaction messages and update their distributed tables accordingly.

