use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Nonce,
};
use hkdf::Hkdf;
use sha2::Sha256;

/// HKDF info string for sender key ratchet
const SENDER_KEY_INFO: &[u8] = b"rr:group:sender_key:v1";

/// Ratchet forward: chain_key_n → (message_key, chain_key_{n+1})
/// Uses HKDF-SHA256 with salt = chain_key_n (no salt for simplicity)
pub fn ratchet_forward(chain_key: &[u8; 32]) -> ([u8; 32], [u8; 32]) {
    let hk = Hkdf::<Sha256>::new(None, chain_key);
    let mut okm = [0u8; 64];
    hk.expand(SENDER_KEY_INFO, &mut okm)
        .expect("HKDF expand should not fail with valid length");
    let mut message_key = [0u8; 32];
    let mut next_chain = [0u8; 32];
    message_key.copy_from_slice(&okm[..32]);
    next_chain.copy_from_slice(&okm[32..]);
    (message_key, next_chain)
}

/// Encrypt plaintext with a 32-byte message key using ChaCha20-Poly1305
pub fn encrypt_with_message_key(
    key: &[u8; 32],
    plaintext: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let cipher = ChaCha20Poly1305::new_from_slice(key)?;
    let nonce = Nonce::default(); // key is unique per message, zero nonce OK
    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_bytes())
        .map_err(|e| format!("ChaCha20 encrypt failed: {}", e))?;
    Ok(ciphertext)
}

/// Decrypt ciphertext with a 32-byte message key using ChaCha20-Poly1305
pub fn decrypt_with_message_key(
    key: &[u8; 32],
    ciphertext: &[u8],
) -> Result<String, Box<dyn std::error::Error>> {
    let cipher = ChaCha20Poly1305::new_from_slice(key)?;
    let nonce = Nonce::default();
    let plaintext = cipher
        .decrypt(&nonce, ciphertext)
        .map_err(|e| format!("ChaCha20 decrypt failed: {}", e))?;
    Ok(String::from_utf8(plaintext)?)
}
