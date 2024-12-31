use ring::{
    aead,
    rand::{self, SecureRandom},
};

use crate::definitions::EncryptionManager;

impl EncryptionManager {
    pub fn new() -> Self {
        const KEY_LEN: usize = 32;
        let rng = rand::SystemRandom::new();
        let mut key_bytes = [0u8; KEY_LEN];

        rng.fill(&mut key_bytes).expect("Failed to generate key");

        // let key_bytes = rand::generate(&rng, &aead::CHACHA20_POLY1305.key_len())
        //     .expect("Failed to generate key");

        Self {
            key: aead::LessSafeKey::new(
                aead::UnboundKey::new(&aead::CHACHA20_POLY1305, &mut key_bytes.as_ref())
                    .expect("Failed to create key"),
            ),
            nonce_sequence: 0,
        }
    }

    pub fn encrypt(&mut self, data: &[u8]) -> Vec<u8> {
        let nonce = aead::Nonce::assume_unique_for_key([
            0u8,
            0,
            0,
            0,
            ((self.nonce_sequence >> 32) & 0xff) as u8,
            ((self.nonce_sequence >> 24) & 0xff) as u8,
            ((self.nonce_sequence >> 16) & 0xff) as u8,
            ((self.nonce_sequence >> 8) & 0xff) as u8,
            (self.nonce_sequence & 0xff) as u8,
            0,
            0,
            0,
        ]);

        self.nonce_sequence += 1;

        let mut in_out = data.to_vec();
        self.key
            .seal_in_place_append_tag(nonce, aead::Aad::empty(), &mut in_out)
            .expect("Failed to encrypt");
        in_out
    }

    pub fn decrypt(&self, encrypted: &mut [u8]) -> Result<Vec<u8>, ()> {
        // Check if the length of the encrypted data is valid
        if encrypted.len() < aead::CHACHA20_POLY1305.tag_len() {
            return Err(()); // Return an error if the buffer is too small
        }

        let tag_pos = encrypted.len() - aead::CHACHA20_POLY1305.tag_len();
        let nonce = aead::Nonce::assume_unique_for_key([0u8; 12]);

        self.key
            .open_in_place(nonce, aead::Aad::empty(), encrypted)
            .map_err(|_| ())
            .map(|decrypted| decrypted.to_vec())
    }
}
