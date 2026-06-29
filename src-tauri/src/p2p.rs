use crate::battery::BatteryMonitor;
use crate::profile::PublicProfile;
use libp2p::{
    autonat, kad, noise, relay,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, PeerId, Swarm, SwarmBuilder, Transport,
};
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(NetworkBehaviour)]
pub struct ProxiBehaviour {
    pub kademlia: kad::Behaviour<kad::store::MemoryStore>,
    pub autonat: autonat::Behaviour,
    pub relay: relay::Behaviour,
}

pub struct P2pNetwork {
    swarm: Swarm<ProxiBehaviour>,
    battery_monitor: Arc<Mutex<BatteryMonitor>>,
}

impl P2pNetwork {
    pub async fn new(peer_id: PeerId) -> Result<Self, Box<dyn Error>> {
        let id_keys = libp2p::identity::Keypair::generate_ed25519();

        let noise_keys = noise::Keypair::<noise::X25519Spec>::new()
            .into_authentic(&id_keys)
            .unwrap();

        let transport = tcp::tokio::Transport::default()
            .upgrade(libp2p::core::upgrade::Version::V1)
            .authenticate(noise::Config::new(noise_keys).unwrap())
            .multiplex(yamux::Config::default())
            .boxed();

        let behaviour = ProxiBehaviour {
            kademlia: kad::Behaviour::new(
                peer_id,
                kad::store::MemoryStore::new(peer_id),
            ),
            autonat: autonat::Behaviour::new(peer_id, Default::default()),
            relay: relay::Behaviour::new(peer_id, Default::default()),
        };

        let swarm = SwarmBuilder::with_tokio_executor(transport, behaviour, peer_id).build();

        Ok(Self {
            swarm,
            battery_monitor: Arc::new(Mutex::new(BatteryMonitor::new())),
        })
    }

    pub async fn discover_profiles(&self) -> Result<Vec<PublicProfile>, Box<dyn Error>> {
        // Mock data for better initial UX
        let mock_profiles = vec![
            PublicProfile {
                peer_id: "12D3KooW..." .to_string(),
                pseudonym: "Léo".to_string(),
                age: 28,
                interests: vec!["Guitare".to_string(), "Code".to_string(), "Pizza".to_string()],
                geohash: "u09tun".to_string(),
            },
            PublicProfile {
                peer_id: "12D3KooX..." .to_string(),
                pseudonym: "Chloé".to_string(),
                age: 24,
                interests: vec!["Randonnée".to_string(), "Photographie".to_string()],
                geohash: "u09tvm".to_string(),
            },
        ];

        Ok(mock_profiles)
    }

    pub async fn send_like(&mut self, _peer_id: &str) -> Result<bool, Box<dyn Error>> {
        // Envoi d'un message "like" (mock match for demo)
        Ok(true)
    }

    pub async fn open_chat(&mut self, _peer_id: &str) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    pub async fn send_chat_message(&mut self, _peer_id: &str, _msg: &str) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    pub async fn close_chat(&mut self, _peer_id: &str) {
    }
}
