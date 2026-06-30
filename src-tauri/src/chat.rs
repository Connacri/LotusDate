//! Chiffrement E2EE éphémère — XChaCha20-Poly1305 avec nonce aléatoire par message.
//!
//! Protocole simplifié (sans Double Ratchet complet) :
//! - Chaque session dérive une clé partagée depuis un secret ECDH (fourni par libp2p noise)
//! - Chaque message utilise un nonce aléatoire de 24 bytes concaténé en tête du ciphertext
//! - La session entière est effacée de la mémoire à la fermeture (zeroize)

use rand::RngCore;
use sodiumoxide::crypto::aead::xchacha20poly1305_ietf::{
    self, Key, Nonce, NONCEBYTES,
};
use zeroize::Zeroize;

/// Payload sérialisé envoyé sur le réseau via Gossipsub
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct EncryptedMessage {
    /// Nonce 24 bytes en base64
    pub nonce_b64: String,
    /// Ciphertext en base64
    pub cipher_b64: String,
    /// PeerId source (string) pour routing côté récepteur
    pub from_peer: String,
}

pub struct EphemeralChat {
    session_key: Key,
    /// Historique RAM uniquement — jamais persisté
    pub messages: Vec<PlainMessage>,
}

#[derive(Clone, Debug)]
pub struct PlainMessage {
    pub content: String,
    pub from_me: bool,
    pub timestamp_ms: u64,
}

impl EphemeralChat {
    /// `shared_secret` : 32 bytes issus de la négociation noise/ECDH
    pub fn new(shared_secret: &[u8; 32]) -> Self {
        // FIX: init sodiumoxide (idempotent)
        sodiumoxide::init().ok();
        let key = Key::from_slice(shared_secret)
            .expect("shared_secret doit faire 32 bytes");
        Self {
            session_key: key,
            messages: Vec::new(),
        }
    }

    /// Chiffre `plaintext` → EncryptedMessage avec nonce aléatoire
    pub fn encrypt(&self, plaintext: &str, from_peer: &str) -> EncryptedMessage {
        // FIX CRITIQUE : nonce ALÉATOIRE par message (pas fixe)
        let mut nonce_bytes = [0u8; NONCEBYTES];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes).unwrap();

        let ciphertext = xchacha20poly1305_ietf::seal(
            plaintext.as_bytes(),
            None,
            &nonce,
            &self.session_key,
        );

        EncryptedMessage {
            nonce_b64: base64_encode(&nonce_bytes),
            cipher_b64: base64_encode(&ciphertext),
            from_peer: from_peer.to_string(),
        }
    }

    /// Déchiffre un EncryptedMessage reçu du réseau
    pub fn decrypt(&self, msg: &EncryptedMessage) -> Option<String> {
        let nonce_bytes = base64_decode(&msg.nonce_b64)?;
        let cipher = base64_decode(&msg.cipher_b64)?;

        let nonce = Nonce::from_slice(&nonce_bytes)?;
        let plain = xchacha20poly1305_ietf::open(
            &cipher,
            None,
            &nonce,
            &self.session_key,
        ).ok()?;

        String::from_utf8(plain).ok()
    }

    pub fn push_message(&mut self, content: String, from_me: bool) {
        self.messages.push(PlainMessage {
            content,
            from_me,
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        });
    }

    /// Efface la clé et tous les messages de la mémoire
    pub fn close(&mut self) {
        self.session_key.0.zeroize();
        self.messages.clear();
    }
}

impl Drop for EphemeralChat {
    fn drop(&mut self) {
        self.close();
    }
}

// --- helpers base64 minimalistes sans dépendance supplémentaire ---
fn base64_encode(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        let chars: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        write!(out, "{}", chars[((n >> 18) & 63) as usize] as char).ok();
        write!(out, "{}", chars[((n >> 12) & 63) as usize] as char).ok();
        if chunk.len() > 1 { write!(out, "{}", chars[((n >> 6) & 63) as usize] as char).ok(); } else { out.push('='); }
        if chunk.len() > 2 { write!(out, "{}", chars[(n & 63) as usize] as char).ok(); } else { out.push('='); }
    }
    out
}

fn base64_decode(s: &str) -> Option<Vec<u8>> {
    let table: [i8; 128] = {
        let mut t = [-1i8; 128];
        let chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        for (i, &c) in chars.iter().enumerate() { t[c as usize] = i as i8; }
        t
    };
    let bytes: Vec<u8> = s.bytes().filter(|&b| b != b'=').collect();
    let mut out = Vec::new();
    for chunk in bytes.chunks(4) {
        let v: Vec<u8> = chunk.iter().map(|&b| {
            if b < 128 { table[b as usize] as u8 } else { 0 }
        }).collect();
        let n = ((v[0] as u32) << 18)
            | ((v.get(1).copied().unwrap_or(0) as u32) << 12)
            | ((v.get(2).copied().unwrap_or(0) as u32) << 6)
            | (v.get(3).copied().unwrap_or(0) as u32);
        out.push(((n >> 16) & 0xFF) as u8);
        if chunk.len() > 2 { out.push(((n >> 8) & 0xFF) as u8); }
        if chunk.len() > 3 { out.push((n & 0xFF) as u8); }
    }
    Some(out)
}