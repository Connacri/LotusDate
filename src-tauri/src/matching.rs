//! Gestion locale des likes et matchs.
//! Les likes ENVOYÉS sont persistés sur disque pour survivre aux redémarrages.
//! Les likes REÇUS arrivent via le réseau (Gossipsub) et sont traités en RAM.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Default)]
struct PersistentLikes {
    sent: HashSet<String>,
}

/// Gère l'état des likes et détecte les matchs réciproques.
pub struct MatchState {
    /// Likes que nous avons envoyés (persistés sur disque)
    pub likes_sent: HashSet<String>,
    /// Likes reçus d'autres pairs (RAM uniquement pour la session)
    pub likes_received: HashSet<String>,
    /// Matchs confirmés (les deux côtés ont liké)
    pub matches: HashSet<String>,
}

impl MatchState {
    pub fn new() -> Self {
        let likes_sent = Self::load_sent_likes();
        Self {
            likes_sent,
            likes_received: HashSet::new(),
            matches: HashSet::new(),
        }
    }

    fn likes_path() -> PathBuf {
        let mut p = dirs_next::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."));
        p.push("proxidate-live");
        std::fs::create_dir_all(&p).ok();
        p.push("likes.json");
        p
    }

    fn load_sent_likes() -> HashSet<String> {
        let path = Self::likes_path();
        if path.exists() {
            if let Ok(bytes) = std::fs::read(&path) {
                if let Ok(data) = serde_json::from_slice::<PersistentLikes>(&bytes) {
                    return data.sent;
                }
            }
        }
        HashSet::new()
    }

    fn persist_sent_likes(&self) {
        let data = PersistentLikes {
            sent: self.likes_sent.clone(),
        };
        if let Ok(bytes) = serde_json::to_vec_pretty(&data) {
            std::fs::write(Self::likes_path(), bytes).ok();
        }
    }

    /// Enregistre un like envoyé depuis l'UI.
    /// Retourne `true` si l'autre a déjà liké → MATCH !
    pub fn register_outgoing_like(&mut self, peer_id: &str) -> bool {
        self.likes_sent.insert(peer_id.to_string());
        self.persist_sent_likes();

        let is_match = self.likes_received.contains(peer_id);
        if is_match {
            self.matches.insert(peer_id.to_string());
        }
        is_match
    }

    /// Enregistre un like reçu depuis le réseau.
    /// Retourne `true` si nous avions déjà liké cet utilisateur → MATCH !
    pub fn register_incoming_like(&mut self, from_peer: &str) -> bool {
        self.likes_received.insert(from_peer.to_string());
        let is_match = self.likes_sent.contains(from_peer);
        if is_match {
            self.matches.insert(from_peer.to_string());
        }
        is_match
    }

    pub fn is_match(&self, peer_id: &str) -> bool {
        self.matches.contains(peer_id)
    }
}