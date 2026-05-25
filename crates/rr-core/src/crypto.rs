#[cfg(test)]
mod tests {
    use nostr::Keys;
    use nostr::nips::nip44;

    fn alice_bob() -> (Keys, Keys) {
        (Keys::generate(), Keys::generate())
    }

    fn encrypt(msg: &str, alice: &Keys, bob: &Keys) -> String {
        nip44::encrypt(alice.secret_key(), &bob.public_key(), msg, nip44::Version::V2).unwrap()
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let (alice, bob) = alice_bob();
        let msg = "Hello RéseauRacine!";
        let cipher = encrypt(msg, &alice, &bob);
        let plain = nip44::decrypt(bob.secret_key(), &alice.public_key(), &cipher).unwrap();
        assert_eq!(plain, msg);
    }

    #[test]
    fn test_wrong_key_fails() {
        let (alice, bob) = alice_bob();
        let eve = Keys::generate();
        let cipher = encrypt("secret", &alice, &bob);
        let result = nip44::decrypt(eve.secret_key(), &alice.public_key(), &cipher);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_message_rejected() {
        let (alice, bob) = alice_bob();
        let result = nip44::encrypt(alice.secret_key(), &bob.public_key(), "", nip44::Version::V2);
        assert!(result.is_err(), "NIP-44 V2 rejects empty messages");
    }

    #[test]
    fn test_large_message_roundtrip() {
        let (alice, bob) = alice_bob();
        let msg = "A".repeat(10000);
        let cipher = encrypt(&msg, &alice, &bob);
        let plain = nip44::decrypt(bob.secret_key(), &alice.public_key(), &cipher).unwrap();
        assert_eq!(plain, msg);
    }

    #[test]
    fn test_oversized_message_rejected() {
        let (alice, bob) = alice_bob();
        let msg = "A".repeat(65536);
        let result = nip44::encrypt(alice.secret_key(), &bob.public_key(), &msg, nip44::Version::V2);
        assert!(result.is_err(), "NIP-44 V2 rejects messages > 65535 bytes");
    }

    #[test]
    fn test_unicode_message() {
        let (alice, bob) = alice_bob();
        let msg = "éèêëàâäùûüôöîïç€œæ🌿🔑 ∑∏∫ ≤ ≥ ∞ 你好 👋";
        let cipher = encrypt(msg, &alice, &bob);
        let plain = nip44::decrypt(bob.secret_key(), &alice.public_key(), &cipher).unwrap();
        assert_eq!(plain, msg);
    }

    #[test]
    fn test_invalid_ciphertext_fails() {
        let (alice, bob) = alice_bob();
        let result = nip44::decrypt(bob.secret_key(), &alice.public_key(), "garbage");
        assert!(result.is_err());
    }

    #[test]
    fn test_sender_decrypts_own_message() {
        let (alice, bob) = alice_bob();
        let cipher = encrypt("self-test", &alice, &bob);
        let plain = nip44::decrypt(alice.secret_key(), &bob.public_key(), &cipher).unwrap();
        assert_eq!(plain, "self-test");
    }

    #[test]
    fn test_keys_are_unique() {
        let a = Keys::generate();
        let b = Keys::generate();
        assert_ne!(a.secret_key(), b.secret_key());
    }
}
