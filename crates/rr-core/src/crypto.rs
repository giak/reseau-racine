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

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let alice = Keys::generate();
        let bob = Keys::generate();
        let msg = "Hello RéseauRacine!";

        let cipher =
            CryptoProvider::encrypt(alice.secret_key(), &bob.public_key(), msg).unwrap();
        let plain =
            CryptoProvider::decrypt(bob.secret_key(), &alice.public_key(), &cipher).unwrap();

        assert_eq!(plain, msg);
    }

    #[test]
    fn test_wrong_key_fails() {
        let alice = Keys::generate();
        let bob = Keys::generate();
        let eve = Keys::generate();
        let msg = "secret";

        let cipher =
            CryptoProvider::encrypt(alice.secret_key(), &bob.public_key(), msg).unwrap();
        let result = CryptoProvider::decrypt(eve.secret_key(), &alice.public_key(), &cipher);

        assert!(result.is_err());
    }
}
