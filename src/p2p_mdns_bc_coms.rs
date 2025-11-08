use libp2p::gossipsub::IdentTopic;
use libp2p::gossipsub::{MessageId, PublishError};
use libp2p::identity::Keypair;
use libp2p::{
	Multiaddr, PeerId, Swarm, SwarmBuilder, gossipsub, mdns, tcp, tls, yamux,
};
use std::sync::Arc;
use std::time::Duration;
use std::{collections::HashMap, error::Error};
use strum::{EnumString, IntoEnumIterator};
use strum_macros::{Display, EnumIter};
use tokio::sync::{Mutex, OnceCell, RwLock};

#[derive(libp2p::swarm::NetworkBehaviour)]
pub struct P2PBehaviour {
	pub gossipsub: gossipsub::Behaviour,
	pub mdns: mdns::tokio::Behaviour,
}

#[derive(Debug, EnumIter, Display, EnumString)]
pub enum TopicEnum {
	#[strum(serialize = "blockchain")]
	Blockchain,
	#[strum(serialize = "transactions")]
	Transactions,
}

pub struct P2PConnection {
	pub keypair: Keypair,
	pub peer_id: PeerId,
	pub swarm: Mutex<Swarm<P2PBehaviour>>,
	pub connected_peers: RwLock<HashMap<PeerId, u32>>,
}

impl P2PConnection {
	pub async fn global() -> Arc<P2PConnection> {
		static INSTANCE: OnceCell<Arc<P2PConnection>> = OnceCell::const_new();
		INSTANCE
			.get_or_init(|| async {
				P2PConnection::new()
					.await
					.map(Arc::new)
					.expect("Failed to init P2P")
			})
			.await
			.clone()
	}

	async fn new() -> Result<Self, Box<dyn Error>> {
		// 1. Create identity
		let keypair = libp2p::identity::Keypair::generate_ed25519();
		let peer_id = PeerId::from(keypair.public());

		// 2. Create gossip behavior
		let mut gossip_sub: gossipsub::Behaviour<gossipsub::IdentityTransform> =
			gossipsub::Behaviour::new(
				gossipsub::MessageAuthenticity::Signed(keypair.clone()),
				gossipsub::Config::default(),
			)?;

		// 3. Create topic and subscribe.
		for item in TopicEnum::iter() {
			let topic = gossipsub::IdentTopic::new(item.to_string());
			gossip_sub.subscribe(&topic)?;
		}

		let mdns =
			mdns::tokio::Behaviour::new(mdns::Config::default(), peer_id)?;
		println!(
			"* MDNS discovery enabled - will automatically find local peers"
		);

		// 4. Create communication swarm.
		let mut swarm = SwarmBuilder::with_existing_identity(keypair.clone())
			.with_tokio()
			.with_tcp(
				tcp::Config::default(),
				tls::Config::new,
				yamux::Config::default,
			)?
			.with_quic()
			.with_behaviour(|_key| {
				Ok(P2PBehaviour { gossipsub: gossip_sub, mdns })
			})?
			.with_swarm_config(|cfg| {
				cfg.with_idle_connection_timeout(Duration::from_secs(60))
			})
			.build();

		swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;
		swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse()?)?;

		let connected_peers: HashMap<PeerId, u32> = HashMap::new();

		Ok(Self {
			keypair,
			peer_id,
			swarm: Mutex::new(swarm),
			connected_peers: RwLock::new(connected_peers),
		})
	}

	pub async fn dial_discovered_peers(&self, list: Vec<(PeerId, Multiaddr)>) {
		let mut swarm = self.swarm.lock().await;
		for (peer_id, addr) in list {
			println!("* Discovered peer: {} at {}", peer_id, addr);
			// Force connection to discovered peer
			if let Err(e) = swarm.dial(addr) {
				eprintln!("Failed to connect to {}: {}", peer_id, e);
			}
		}
	}

	pub async fn publish(
		&self,
		topic: IdentTopic,
		input: &[u8],
	) -> Result<MessageId, PublishError> {
		let mut swarm = self.swarm.lock().await;
		swarm
			.behaviour_mut()
			.gossipsub
			.publish(topic.clone(), input)
	}

	pub async fn add_connected_peer(&self, peer_id: &PeerId) {
		// peer number deduplication
		let mut connected_peers = self.connected_peers.write().await;
		let mut swarm = self.swarm.lock().await;
		let count = connected_peers
			.entry(*peer_id)
			.and_modify(|c| *c += 1)
			.or_insert(1);

		if *count == 1 {
			println!(
				"* New peer: {} ({} unique peers)",
				peer_id,
				connected_peers.len()
			);
		} else {
			println!(
				"* Additional connection to: {} ({} total connections)",
				peer_id, count
			);
		}
		// IMPORTANT: When we connect to a new peer, make sure gossipsub knows about it
		// This helps with topic propagation and mesh formation
		// Add this to all connections, but keep peer itself deduplicated.
		swarm
			.behaviour_mut()
			.gossipsub
			.add_explicit_peer(&peer_id);
	}

	pub async fn remove_peer(&self, list: Vec<(PeerId, Multiaddr)>) {
		let mut connected_peers = self.connected_peers.write().await;
		for (peer_id, _addr) in list {
			println!("* Peer removed: {}", peer_id);
			// connected_peers = connected_peers.saturating_sub(1);
			connected_peers.remove(&peer_id);
		}
	}

	pub async fn closed_connection(&self, peer_id: &PeerId) {
		let mut connected_peers = self.connected_peers.write().await;
		// connected_peers = connected_peers.saturating_sub(1);
		if let Some(count) = connected_peers.get_mut(&peer_id) {
			*count -= 1;
			if *count == 0 {
				connected_peers.remove(&peer_id);
				println!(
					"Peer disconnected: {} ({} peers left)",
					peer_id,
					connected_peers.len()
				);
			} else {
				println!("Connection closed: {} ({} remain)", peer_id, count);
			}
		}
	}

	pub async fn get_connected_peers_len(&self) -> usize {
		self.connected_peers.read().await.len()
	}
}
