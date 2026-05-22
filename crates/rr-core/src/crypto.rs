use nostr::nips::nip44;
use nostr::{Keys, PublicKey, SecretKey};

#[derive(Debug, Clone)]
pub struct CryptoProvider;

impl CryptoProvider {
    pub fn encrypt(
        secret_key: &SecretKey,
        public_key: &PublicKey,
        content: &str,
    ) -> Result<String, nip44::Error> {
        nip44::encrypt(secret_key, public_key, content, nip44::Version::V2)
    }

    pub fn decrypt(
        secret_key: &SecretKey,
        public_key: &PublicKey,
        payload: &str,
    ) -> Result<String, nip44::Error> {
        nip44::decrypt(secret_key, public_key, payload)
    }

    pub fn generate_keys() -> Keys {
        Keys::generate()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn alice_bob() -> (Keys, Keys) {
        (Keys::generate(), Keys::generate())
    }

    fn encrypt(msg: &str, alice: &Keys, bob: &Keys) -> String {
        CryptoProvider::encrypt(alice.secret_key(), &bob.public_key(), msg).unwrap()
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let (alice, bob) = alice_bob();
        let msg = "Hello RéseauRacine!";
        let cipher = encrypt(msg, &alice, &bob);
        let plain =
            CryptoProvider::decrypt(bob.secret_key(), &alice.public_key(), &cipher).unwrap();
        assert_eq!(plain, msg);
    }

    #[test]
    fn test_wrong_key_fails() {
        let (alice, bob) = alice_bob();
        let eve = Keys::generate();
        let cipher = encrypt("secret", &alice, &bob);
        let result = CryptoProvider::decrypt(eve.secret_key(), &alice.public_key(), &cipher);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_message_rejected() {
        let (alice, bob) = alice_bob();
        let result = CryptoProvider::encrypt(alice.secret_key(), &bob.public_key(), "");
        assert!(result.is_err(), "NIP-44 V2 rejects empty messages");
    }

    #[test]
    fn test_large_message_roundtrip() {
        let (alice, bob) = alice_bob();
        let msg = "A".repeat(10000);
        let cipher = encrypt(&msg, &alice, &bob);
        let plain =
            CryptoProvider::decrypt(bob.secret_key(), &alice.public_key(), &cipher).unwrap();
        assert_eq!(plain, msg);
    }

    #[test]
    fn test_oversized_message_rejected() {
        let (alice, bob) = alice_bob();
        let msg = "A".repeat(65536);
        let result = CryptoProvider::encrypt(alice.secret_key(), &bob.public_key(), &msg);
        assert!(result.is_err(), "NIP-44 V2 rejects messages > 65535 bytes");
    }

    #[test]
    fn test_unicode_message() {
        let (alice, bob) = alice_bob();
        let msg = "éèêëàâäùûüôöîïç€œæ🌿🔑 ∑∏∫ ≤ ≥ ∞ 你好 👋";
        let cipher = encrypt(msg, &alice, &bob);
        let plain =
            CryptoProvider::decrypt(bob.secret_key(), &alice.public_key(), &cipher).unwrap();
        assert_eq!(plain, msg);
    }

    #[test]
    fn test_invalid_ciphertext_fails() {
        let (alice, bob) = alice_bob();
        let result = CryptoProvider::decrypt(bob.secret_key(), &alice.public_key(), "garbage");
        assert!(result.is_err());
    }

    #[test]
    fn test_sender_decrypts_own_message() {
        let (alice, bob) = alice_bob();
        let cipher = encrypt("self-test", &alice, &bob);
        let plain =
            CryptoProvider::decrypt(alice.secret_key(), &bob.public_key(), &cipher).unwrap();
        assert_eq!(plain, "self-test");
    }

    #[test]
    fn test_keys_are_unique() {
        let a = CryptoProvider::generate_keys();
        let b = CryptoProvider::generate_keys();
        assert_ne!(a.secret_key(), b.secret_key());
    }
}
