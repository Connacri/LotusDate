use sodiumoxide::crypto::aead::xchacha20poly1305_ietf;
use zeroize::Zeroize;

pub struct EphemeralChat {
    // Utilisation simplifiée du Double Ratchet
    session_key: xchacha20poly1305_ietf::Key,
    messages: Vec<String>, // en RAM uniquement
}

impl EphemeralChat {
    pub fn new(shared_secret: &[u8]) -> Self {
        let key = xchacha20poly1305_ietf::Key::from_slice(shared_secret).unwrap();
        Self {
            session_key: key,
            messages: Vec::new(),
        }
    }

    pub fn encrypt(&self, plaintext: &str) -> Vec<u8> {
        let nonce = xchacha20poly1305_ietf::Nonce::from_slice(b"unique_nonce_12__").unwrap();
        let ciphertext = xchacha20poly1305_ietf::seal(plaintext.as_bytes(), None, &nonce, &self.session_key);
        ciphertext
    }

    pub fn decrypt(&self, ciphertext: &[u8]) -> Option<String> {
        let nonce = xchacha20poly1305_ietf::Nonce::from_slice(b"unique_nonce_12__").unwrap();
        let plain = xchacha20poly1305_ietf::open(ciphertext, None, &nonce, &self.session_key)?;
        Some(String::from_utf8(plain).ok()?)
    }

    pub fn close(&mut self) {
        self.session_key.zeroize();
        self.messages.clear();
    }
}

impl Drop for EphemeralChat {
    fn drop(&mut self) {
        self.close();
    }
}