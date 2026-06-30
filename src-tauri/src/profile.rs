use libp2p::{identity::Keypair, PeerId};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Profil complet de l'utilisateur local (stocké sur disque, jamais diffusé entier)
#[derive(Clone, Serialize, Deserialize)]
pub struct UserProfile {
    /// Clé privée Ed25519 encodée en base64 (persistée pour garder le même PeerId)
    pub keypair_bytes: Vec<u8>,
    pub pseudonym: String,
    pub age: u8,
    pub interests: Vec<String>,
    /// Geohash à 6 chars ≈ 1.2 km de précision
    pub geohash: String,
}

impl UserProfile {
    fn profile_path(data_dir: &std::path::Path) -> PathBuf {
        std::fs::create_dir_all(data_dir).ok();
        data_dir.join("profile.json")
    }

    /// `data_dir` doit être un répertoire garanti accessible en écriture par l'app
    /// (fourni par Tauri via `app.path().app_data_dir()`). Sur Android, l'ancien
    /// `dirs_next::data_local_dir()` peut échouer silencieusement ou pointer vers
    /// un chemin non accessible en écriture par le sandbox de l'app, ce qui
    /// régénérait une nouvelle keypair (donc un nouveau PeerId) à CHAQUE lancement
    /// — cassant les matchs et la persistance entre sessions.
    pub fn load_or_create(data_dir: &std::path::Path) -> Self {
        let path = Self::profile_path(data_dir);
        if path.exists() {
            if let Ok(bytes) = std::fs::read(&path) {
                if let Ok(p) = serde_json::from_slice::<UserProfile>(&bytes) {
                    return p;
                }
            }
        }
        // Nouveau profil : génère une clé Ed25519 persistée
        let kp = Keypair::generate_ed25519();
        let encoded = kp.to_protobuf_encoding().expect("keypair encode");
        let profile = UserProfile {
            keypair_bytes: encoded,
            pseudonym: "Moi".to_string(),
            age: 25,
            interests: vec!["Musique".into(), "Voyage".into()],
            geohash: "u09tun".into(),
        };
        if let Ok(bytes) = serde_json::to_vec_pretty(&profile) {
            if let Err(e) = std::fs::write(&path, bytes) {
                tracing::warn!(
                    "Impossible de persister le profil ({:?}) : {} — un nouveau \
                     profil sera régénéré au prochain lancement.",
                    path, e
                );
            }
        }
        profile
    }

    /// Reconstruit la keypair depuis les bytes persistés
    pub fn keypair(&self) -> Keypair {
        Keypair::from_protobuf_encoding(&self.keypair_bytes)
            .expect("keypair decode — profil corrompu")
    }

    /// PeerId dérivé de la clé publique (stable entre les sessions)
    pub fn peer_id(&self) -> PeerId {
        PeerId::from_public_key(&self.keypair().public())
    }

    /// Version publique diffusée sur le réseau Kademlia
    pub fn public_version(&self) -> PublicProfile {
        PublicProfile {
            peer_id: self.peer_id().to_string(),
            pseudonym: self.pseudonym.clone(),
            age: self.age,
            interests: self.interests.clone(),
            geohash: self.geohash.clone(),
        }
    }
}

/// Ce que l'on publie sur le réseau (pas la clé privée, pas la photo)
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PublicProfile {
    pub peer_id: String,
    pub pseudonym: String,
    pub age: u8,
    pub interests: Vec<String>,
    pub geohash: String,
}