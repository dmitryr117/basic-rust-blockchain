use libp2p::{gossipsub};
use std::error::Error;

pub fn basic_pubsub_example() -> Result<(), Box<dyn Error>> {
	// 1. - create identity
	let keypair = libp2p::identity::Keypair::generate_ed25519();

	// 2. Create gossip behavior
	let gossipsub_config = gossipsub::Config::default();
	let mut gossipsub: gossipsub::Behaviour<gossipsub::IdentityTransform> =
		gossipsub::Behaviour::new(
			gossipsub::MessageAuthenticity::Signed(keypair),
			gossipsub_config,
		)?;

	// 3. Create topic and subscribe
	let topic = gossipsub::IdentTopic::new("test-topic");
	gossipsub.subscribe(&topic)?;

	// 4. Publish message to topic.
	let message = b"Hello p2p world.";
	gossipsub.publish(topic, message)?;

	Ok(())
}
