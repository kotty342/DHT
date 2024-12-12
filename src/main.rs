use futures::prelude::*;
use libp2p::swarm::SwarmEvent;
use libp2p::{ping, Multiaddr};
use std::error::Error;
use std::time::Duration;
use tracing_subscriber::EnvFilter;

#[derive(libp2p::swarm::NetworkBehaviour)]
struct Behaviour{
	mdns:libp2p::mdns::async_io::Behaviour,
	ping:libp2p::ping::Behaviour,
}

impl Behaviour{

	pub fn new(
		mdns:libp2p::mdns::async_io::Behaviour,
		ping:libp2p::ping::Behaviour,
	)->Self{
		Self{
			mdns,
			ping,
		}
	}
}

#[async_std::main]
async fn main()->Result<(),Box<dyn Error>>{
	let mut swarm=libp2p::SwarmBuilder::with_new_identity()
		.with_async_std()
		.with_tcp(
			libp2p::tcp::Config::default(),
			libp2p::noise::Config::new,
			||libp2p::yamux::Config::default(),
		)?
		.with_behaviour(
			|keypair|{
				Behaviour::new(
					libp2p::mdns::async_io::Behaviour::new(
						libp2p::mdns::Config::default(),
						keypair.public().into(),
					).unwrap(),
					libp2p::ping::Behaviour::new(
						libp2p::ping::Config::new()
							.with_timeout(Duration::from_secs(5))
							.with_interval(Duration::from_secs(1)),
					),
				)
			}
		)?
		.with_swarm_config(|config|config.with_idle_connection_timeout(Duration::from_secs(5)))
		.build();
	println!("peer id: {}",swarm.local_peer_id().to_string());
	swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;
	loop{
		let ev=swarm
			.select_next_some()
			.await;
		println!("{:#?}",ev);
		if let libp2p::swarm::SwarmEvent::Behaviour(BehaviourEvent::Mdns(libp2p::mdns::Event::Discovered(e)))=ev{
			for peer in e{
				swarm.dial(peer.1)?;
			}
		}
	}
}
