use futures::StreamExt;
use libp2p::{PeerId, SwarmBuilder, gossipsub, swarm::SwarmEvent, tcp, tls, yamux, mdns};
use std::error::Error;
use std::time::Duration;
use tokio::io::{self, AsyncBufReadExt};

#[derive(libp2p::swarm::NetworkBehaviour)]
struct MyBehaviour {
	gossipsub: gossipsub::Behaviour,
	mdns: mdns::tokio::Behaviour,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
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
	let topic = gossipsub::IdentTopic::new("test-topic");
	gossip_sub.subscribe(&topic)?;

	let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), peer_id)?;
    println!("* MDNS discovery enabled - will automatically find local peers");

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
			Ok( MyBehaviour {
				gossipsub: gossip_sub,
				mdns,
			})
		})?
		.with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(60)))
		.build();

	// 5. Start listening on localhost.
	swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;
	swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse()?)?;
	println!("=== Chat Node ===");
	println!("Your ID: {}", peer_id);

	// Get command line arguments to connect peers
	let args: Vec<String> = std::env::args().collect();

	// Connect to address if provided as argument
	if args.len() > 1 {
		let connect_addr = &args[1];
		println!("Attempting to connect to: {}", connect_addr);
		if let Ok(addr) = connect_addr.parse::<libp2p::Multiaddr>() {
			if let Err(e) = swarm.dial(addr) {
				eprintln!("Failed to dial: {}", e);
			}
		} else {
			eprintln!("Invalid address format: {}", connect_addr);
		}
	}

	// Print our addresses for others to connect to
	tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
	println!("Full addresses to connect to this node:");
	for addr in swarm.listeners() {
		// Changed from listen_addresses() to listeners()
		println!("  {}/p2p/{}", addr, peer_id);
	}

	println!("\nType messages to send. 'exit' to quit.");
	println!("=================\n");

	let mut stdin = io::BufReader::new(io::stdin()).lines();
	let mut connected_peers: i32 = 0;

	loop {
		tokio::select! {
            line = stdin.next_line() => {
                if let Ok(Some(input)) = line {
                    let input = input.trim();

                    if input == "exit" {
                        break;
                    }

                    if connected_peers > 0 && !input.is_empty() {
                        match swarm.behaviour_mut().gossipsub.publish(topic.clone(), input.as_bytes()) {
                            Ok(_) => println!("Message sent!"),
                            Err(e) => eprintln!("Failed to send: {}", e),
                        }
                    } else if !input.is_empty() {
                        println!("No connected peers yet. Waiting for automatic discovery...");
                    }
                }
            },
			event = swarm.select_next_some() => {
				match event {
					SwarmEvent::NewListenAddr { address, .. } => {
                        println!("* Listening on: {}", address);
                    }
                    SwarmEvent::Behaviour(MyBehaviourEvent::Gossipsub(gossipsub::Event::Message { message, .. })) => {
                        let text = String::from_utf8_lossy(&message.data);
                        println!(">> {}", text);
                    }
                    SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
						for (peer_id, addr) in list {
							println!("* Discovered peer: {} at {}", peer_id, addr);
							// Force connection to discovered peer
							if let Err(e) = swarm.dial(addr) {
								eprintln!("Failed to connect to {}: {}", peer_id, e);
							}
						}
					}
                    SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                        for (peer_id, _addr) in list {
                            println!("* Peer expired: {}", peer_id);
                            connected_peers = connected_peers.saturating_sub(1);
                        }
                    }
					SwarmEvent::ConnectionEstablished { peer_id, .. } => {
						connected_peers += 1;
						println!("* Connected to: {} ({} total peers)", peer_id, connected_peers);
						
						// IMPORTANT: When we connect to a new peer, make sure gossipsub knows about it
						// This helps with topic propagation and mesh formation
						swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
					}
					SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
						if let Some(peer) = peer_id {
							eprintln!("❌ Failed to connect to {}: {}", peer, error);
						} else {
							eprintln!("❌ Failed to connect: {}", error);
						}
					}
					SwarmEvent::Dialing { peer_id, .. } => {
						println!("🔄 Attempting to connect to: {:?}", peer_id);
					}
                    SwarmEvent::ConnectionClosed { peer_id, .. } => {
                        connected_peers = connected_peers.saturating_sub(1);
                        println!("* Disconnected from: {} ({} peers left)", peer_id, connected_peers);
                    }
                    _ => {}
				}
			}
		}
	}

	Ok(())
}
