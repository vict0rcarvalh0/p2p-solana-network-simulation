use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde::{Serialize, Deserialize};

// Represent a transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Transaction {
    from: String,
    to: String,
    amount: f64,
    timestamp: u64,
}

// Store node state
struct NodeState {
    transactions: HashMap<String, Vec<Transaction>>,
    peers: Vec<String>,
}

async fn handle_connection(
    mut socket: TcpStream,
    tx: broadcast::Sender<Transaction>,
    state: Arc<Mutex<NodeState>>,
) {
    let mut buffer = [0u8; 1024];
    
    loop {
        match socket.read(&mut buffer).await {
            Ok(n) if n == 0 => {
                println!("Connection closed");
                break;
            }
            Ok(n) => {
                if let Ok(transaction) = serde_json::from_slice::<Transaction>(&buffer[..n]) {
                    println!("Received transaction: {:?}", transaction);
                    
                    // Store transaction
                    let mut state = state.lock().await;
                    state.transactions
                        .entry(transaction.from.clone())
                        .or_default()
                        .push(transaction.clone());
                    
                    // Broadcast to other peers
                    let _ = tx.send(transaction);
                }
            }
            Err(e) => {
                println!("Error reading from socket: {:?}", e);
                break;
            }
        }
    }
}

async fn connect_to_peer(
    addr: String,
    tx: broadcast::Sender<Transaction>,
    state: Arc<Mutex<NodeState>>,
) {
    match TcpStream::connect(&addr).await {
        Ok(socket) => {
            println!("Connected to peer: {}", addr);
            {
                let mut state = state.lock().await;
                if !state.peers.contains(&addr) {
                    state.peers.push(addr.clone());
                }
            }
            handle_connection(socket, tx, state).await;
        }
        Err(e) => println!("Failed to connect to peer {}: {:?}", addr, e),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let port = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "8000".to_string())
        .parse::<u16>()?;
    
    let state = Arc::new(Mutex::new(NodeState {
        transactions: HashMap::new(),
        peers: Vec::new(),
    }));
    
    // Channel for broadcasting transactions
    let (tx, _) = broadcast::channel(16);
    let tx_clone = tx.clone();
    
    // Listen for incoming connections
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
    println!("Node listening on port {}", port);
    
    // If a peer address is provided, connect to it
    if let Some(peer_addr) = std::env::args().nth(2) {
        let tx_peer = tx.clone();
        let state_peer = state.clone();
        tokio::spawn(async move {
            connect_to_peer(peer_addr, tx_peer, state_peer).await;
        });
    }
    
    // Accept incoming connections
    loop {
        let (socket, addr) = listener.accept().await?;
        println!("New peer connected: {:?}", addr);
        
        let tx = tx_clone.clone();
        let state = state.clone();
        
        tokio::spawn(async move {
            handle_connection(socket, tx, state).await;
        });
    }
}