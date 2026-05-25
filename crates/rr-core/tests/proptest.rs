use nostr::{Keys, nips::nip44};
use proptest::prelude::*;

proptest! {
    #[test]
    fn decrypt_encrypt_roundtrip(msg: String) {
        let alice = Keys::generate();
        let bob = Keys::generate();

        let cipher = nip44::encrypt(alice.secret_key(), &bob.public_key(), &msg, nip44::Version::V2);
        prop_assume!(cipher.is_ok(), "nip44 may reject empty messages internally");
        let cipher = cipher.unwrap();

        let plain = nip44::decrypt(bob.secret_key(), &alice.public_key(), &cipher);
        prop_assert_eq!(plain.unwrap(), msg);
    }

    #[test]
    fn wrong_key_fails(msg: String) {
        let alice = Keys::generate();
        let bob = Keys::generate();
        let eve = Keys::generate();

        let cipher = nip44::encrypt(alice.secret_key(), &bob.public_key(), &msg, nip44::Version::V2);
        prop_assume!(cipher.is_ok());
        let cipher = cipher.unwrap();

        let result = nip44::decrypt(eve.secret_key(), &alice.public_key(), &cipher);
        prop_assert!(result.is_err(), "eve should not decrypt alice→bob message");
    }

    #[test]
    fn conversation_key_is_symmetric(alice_msg: String, bob_msg: String) {
        let alice = Keys::generate();
        let bob = Keys::generate();

        // Alice → Bob
        let a_to_b = nip44::encrypt(alice.secret_key(), &bob.public_key(), &alice_msg, nip44::Version::V2);
        prop_assume!(a_to_b.is_ok());
        // Bob → Alice uses same conversation key
        let b_to_a = nip44::encrypt(bob.secret_key(), &alice.public_key(), &bob_msg, nip44::Version::V2);
        prop_assume!(b_to_a.is_ok());

        let a_to_b = a_to_b.unwrap();
        let b_to_a = b_to_a.unwrap();

        // Alice reads Bob's message
        let alice_reads = nip44::decrypt(alice.secret_key(), &bob.public_key(), &b_to_a);
        prop_assert_eq!(alice_reads.unwrap(), bob_msg);

        // Bob reads Alice's message
        let bob_reads = nip44::decrypt(bob.secret_key(), &alice.public_key(), &a_to_b);
        prop_assert_eq!(bob_reads.unwrap(), alice_msg);
    }
}
