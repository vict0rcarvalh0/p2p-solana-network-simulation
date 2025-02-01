# P2P Network Simulation
This project implements a basic peer-to-peer (P2P) network using TCP sockets, allowing nodes to communicate and share transactions in a distributed manner. The network can support multiple nodes that can send and receive transactions, maintaining a distributed record of all transactions.

## Features
- TCP-based P2P Network: Build a network of multiple interconnected nodes
- Transaction Broadcasting: Nodes can broadcast transactions to all connected peers
- State Management: Each node maintains a record of transactions and connected peers
- Simple Architecture: Uses basic TCP sockets for easy understanding and debugging

## Prerequisites
- Rust programming language installed on your system. You can install it from rust-lang.org.

## Installation
**Clone the repository:**
```bash
git clone https://github.com/vict0rcarvalh0/p2p-solana-network-simulation.git
cd p2p-solana-network-simulation
```

**Build the project:**
```bash
cargo build
```

## Running the Project

1. Start the first node (primary node):
```bash
cargo run -- 8000
```

2. Start additional nodes in different terminals, connecting to the primary node:
```bash
# Second node on port 8001
cargo run -- 8001 127.0.0.1:8000

# Third node on port 8002
cargo run -- 8002 127.0.0.1:8000

# Fourth node on port 8003
cargo run -- 8003 127.0.0.1:8000
```

## Testing Transactions
You can send transactions using netcat or telnet:
```bash
nc 127.0.0.1 8000
```

Then send a JSON transaction:
```json
{
    "from": "node1",
    "to": "node2",
    "amount": 100.0,
    "timestamp": 1234567890
}
```

## Code Overview
The main components of the code are as follows:

- **Transaction Structure**: The `Transaction` struct represents a basic transaction with sender, receiver, amount, and timestamp.

- **Node State**: The `NodeState` struct maintains:
  - transactions: Maps sender addresses to lists of transactions
  - peers: Keeps track of connected peer addresses

- **Connection Handling**: 
  - `handle_connection`: Processes incoming TCP connections and transaction broadcasts
  - `connect_to_peer`: Establishes connections to other nodes in the network

- **Network Communication**:
  - Uses Tokio's TCP networking for async communication
  - Implements a broadcast channel for transaction distribution
  - Maintains persistent connections between peers

## How It Works
1. **Node Startup**: Each node starts by listening on a specified TCP port
2. **Peer Connection**: Nodes can connect to existing peers in the network
3. **Transaction Broadcasting**: When a node receives a transaction:
   - It stores the transaction in its local state
   - Broadcasts the transaction to all connected peers
   - Other nodes receive and store the transaction

## Project Structure
```
src/
  └── main.rs          # Main implementation file
Cargo.toml             # Project dependencies and configuration
README.md             # This file
```

## Dependencies
- `tokio`: Async runtime and networking
- `serde`: Serialization/deserialization of transactions
- `serde_json`: JSON encoding/decoding

## Contributing
Feel free to submit issues and enhancement requests!