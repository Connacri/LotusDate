use libp2p::PeerId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub peer_id: PeerId,
    pub pseudonym: String,
    pub age: u8,
    pub interests: Vec<String>,
    pub geohash: String,
    // La photo réelle est stockée localement, non publiée
}

impl UserProfile {
    pub fn load_or_create() -> Self {
        // Charger depuis SQLCipher ou créer un nouveau
        Self {
            peer_id: PeerId::random(),
            pseudonym: "Alice".into(),
            age: 25,
            interests: vec!["music".into(), "hiking".into()],
            geohash: "u09tun".into(), // Paris ~5km
        }
    }

    pub fn public_version(&self) -> PublicProfile {
        PublicProfile {
            peer_id: self.peer_id.to_string(),
            pseudonym: self.pseudonym.clone(),
            age: self.age,
            interests: self.interests.clone(),
            geohash: self.geohash.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PublicProfile {
    pub peer_id: String,
    pub pseudonym: String,
    pub age: u8,
    pub interests: Vec<String>,
    pub geohash: String,
}