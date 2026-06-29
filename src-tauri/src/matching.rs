use std::collections::HashSet;

/// Gère l'état local des likes et détecte les matchs réciproques.
pub struct MatchState {
    likes_sent: HashSet<String>,     // peer_ids que nous avons likés
    likes_received: HashSet<String>, // peer_ids qui nous ont likés
}

impl MatchState {
    pub fn new() -> Self {
        Self {
            likes_sent: HashSet::new(),
            likes_received: HashSet::new(),
        }
    }

    /// Enregistre un like reçu (depuis l'extérieur) et vérifie s'il y a match.
    pub fn receive_like(&mut self, from: &str) -> bool {
        self.likes_received.insert(from.to_string());
        self.likes_sent.contains(from)
    }

    /// Enregistre un like envoyé (depuis l'UI) et retourne true s'il y a match immédiat.
    pub fn send_like(&mut self, to: &str) -> bool {
        self.likes_sent.insert(to.to_string());
        self.likes_received.contains(to)
    }

    /// Pour la commande `send_like` : on enregistre le like envoyé et on vérifie le match.
    pub fn register_like(&mut self, peer_id: &str) -> bool {
        self.send_like(peer_id)
    }
}