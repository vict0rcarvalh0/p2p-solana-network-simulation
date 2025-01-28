use libp2p::{
    core::upgrade,
    gossipsub::{
        Gossipsub, GossipsubConfig, GossipsubConfigBuilder, 
        MessageAuthenticity, TopicHash,
    },
    identity, mdns, noise,
    swarm::{NetworkBehaviour, SwarmBuilder, SwarmEvent},
    tcp, yamux, PeerId, Transport,
};
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};
use std::{
    collections::HashMap,
    error::Error,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::mpsc;

// Define the behavior for our P2P network
#[derive(NetworkBehaviour)]
struct NodeBehaviour {
    gossipsub: Gossipsub,
    mdns: mdns::async_io::Behaviour,
}

// Structure for our transaction message
#[derive(Debug, Serialize, Deserialize)]
struct TransactionMessage {
    transaction: String, // Base64 encoded transaction
    sender: String,     // PeerId of the sender
    timestamp: u64,
}

// Structure to maintain the distributed table
#[derive(Debug, Default)]
struct DistributedTable {
    peers: HashMap<String, Vec<String>>,         // PeerId -> List of transactions
    transactions: HashMap<String, String>,        // Transaction hash -> Transaction data
}

struct Node {
    peer_id: PeerId,
    distributed_table: Arc<Mutex<DistributedTable>>,
    topic: TopicHash,
}

impl Node {
    async fn new() -> Result<(Self, impl futures::Future<Output = ()>), Box<dyn Error>> {
        // Generate keypair for identity
        let id_keys = identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(id_keys.public());
        println!("Local peer id: {peer_id}");

        // Create transport
        let transport = tcp::async_io::Transport::new(tcp::Config::default())
            .upgrade(upgrade::Version::V1)
            .authenticate(noise::Config::new(&id_keys)?)
            .multiplex(yamux::Config::default())
            .boxed();

        // Create Gossipsub configuration
        let gossipsub_config = GossipsubConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(1))
            .validation_mode(libp2p::gossipsub::ValidationMode::Strict)
            .build()
            .expect("Valid config");

        // Create Gossipsub
        let mut gossipsub = Gossipsub::new(
            MessageAuthenticity::Signed(id_keys.clone()),
            gossipsub_config,
        )?;

        // Create topic
        let topic = gossipsub::Topic::new("transaction");
        gossipsub.subscribe(&topic)?;

        // Create mDNS
        let mdns = mdns::async_io::Behaviour::new(mdns::Config::default())?;

        // Create behavior
        let behaviour = NodeBehaviour {
            gossipsub,
            mdns,
        };

        // Create Swarm
        let mut swarm = SwarmBuilder::with_async_std_executor(
            transport,
            behaviour,
            peer_id,
        ).build();

        // Listen on random port
        swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

        let distributed_table = Arc::new(Mutex::new(DistributedTable::default()));
        let node = Node {
            peer_id,
            distributed_table: distributed_table.clone(),
            topic: topic.hash(),
        };

        // Create event loop
        let event_loop = async move {
            loop {
                if let SwarmEvent::Behaviour(event) = swarm.next().await.unwrap() {
                    match event {
                        NodeBehaviourEvent::Mdns(mdns::Event::Discovered(peers)) => {
                            for (peer_id, _addr) in peers {
                                println!("Discovered peer: {peer_id}");
                            }
                        }
                        NodeBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                            message,
                            ..
                        }) => {
                            if let Ok(tx_message) = serde_json::from_slice::<TransactionMessage>(&message.data) {
                                println!("Received transaction from: {}", tx_message.sender);
                                let mut table = distributed_table.lock().unwrap();
                                table.peers.entry(tx_message.sender.clone())
                                    .or_default()
                                    .push(tx_message.transaction.clone());
                                table.transactions.insert(
                                    base64::decode(&tx_message.transaction)
                                        .unwrap()
                                        .iter()
                                        .map(|b| format!("{:02x}", b))
                                        .collect(),
                                    tx_message.transaction,
                                );
                            }
                        }
                        _ => {}
                    }
                }
            }
        };

        Ok((node, event_loop))
    }

    async fn broadcast_transaction(&self, rpc_client: &RpcClient, sender: &Keypair, recipient: &Keypair) -> Result<(), Box<dyn Error>> {
        // Create a dummy Solana transaction
        let blockhash = rpc_client.get_latest_blockhash()?;
        let tx = Transaction::new_signed_with_payer(
            &[system_instruction::transfer(
                &sender.pubkey(),
                &recipient.pubkey(),
                1000,
            )],
            Some(&sender.pubkey()),
            &[sender],
            blockhash,
        );

        // Encode transaction
        let tx_data = base64::encode(tx.message_data());
        
        // Create message
        let message = TransactionMessage {
            transaction: tx_data,
            sender: self.peer_id.to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
        };

        // Broadcast message
        self.behaviour.gossipsub.publish(
            self.topic.clone(),
            serde_json::to_string(&message)?.as_bytes(),
        )?;

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Create 4 nodes
    let mut nodes = vec![];
    let mut event_loops = vec![];

    for _ in 0..4 {
        let (node, event_loop) = Node::new().await?;
        nodes.push(node);
        event_loops.push(event_loop);
    }

    // Set up Solana client
    let rpc_client = RpcClient::new("http://localhost:8899".to_string());
    
    // Create dummy keypairs for testing
    let sender = Keypair::new();
    let recipient = Keypair::new();

    // Start event loops for all nodes
    let event_loop_futures = futures::future::join_all(event_loops);
    
    // Broadcast a transaction from the first node
    nodes[0].broadcast_transaction(&rpc_client, &sender, &recipient).await?;

    // Run the network
    event_loop_futures.await;

    Ok(())
}