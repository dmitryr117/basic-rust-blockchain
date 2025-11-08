use libp2p::identity::Keypair;
use libp2p::{PeerId, Swarm, SwarmBuilder, gossipsub, mdns, tcp, tls, yamux};
use std::sync::Arc;
use std::time::Duration;
use std::{collections::HashMap, error::Error};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};
use tokio::sync::{Mutex, OnceCell, RwLock};

#[derive(libp2p::swarm::NetworkBehaviour)]
pub struct P2PBehaviour {
	pub gossipsub: gossipsub::Behaviour,
	pub mdns: mdns::tokio::Behaviour,
}

#[derive(Debug, EnumIter, Display)]
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
}
