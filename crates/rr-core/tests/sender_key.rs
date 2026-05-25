use rr_core::sender_key::{decrypt_with_message_key, encrypt_with_message_key, ratchet_forward};

#[test]
fn test_ratchet_forward_produces_unique_keys() {
    let chain_key = [0xABu8; 32];
    let (msg_key_a, chain_key_a) = ratchet_forward(&chain_key, 0);
    let (msg_key_b, chain_key_b) = ratchet_forward(&chain_key_a, 0);

    assert_ne!(msg_key_a, msg_key_b, "message keys must differ per step");
    assert_ne!(chain_key_a, chain_key_b, "chain keys must differ per step");
    assert_ne!(chain_key, chain_key_a, "chain key must change");
}

#[test]
fn test_ratchet_deterministic() {
    let chain_key = [0xABu8; 32];
    let (msg_key_1, next_1) = ratchet_forward(&chain_key, 0);
    let (msg_key_2, next_2) = ratchet_forward(&chain_key, 0);
    assert_eq!(msg_key_1, msg_key_2, "same input must produce same output");
    assert_eq!(next_1, next_2);
}

#[test]
fn test_encrypt_decrypt_roundtrip() {
    let chain_key = [0xABu8; 32];
    let (msg_key, _) = ratchet_forward(&chain_key, 0);
    let plaintext = "hello cellule test";
    let cipher = encrypt_with_message_key(&msg_key, plaintext).unwrap();
    let decrypted = decrypt_with_message_key(&msg_key, &cipher).unwrap();
    assert_eq!(decrypted, plaintext);
}

#[test]
fn test_msg_count_changes_key() {
    let chain_key = [0xABu8; 32];
    let (k1, _) = ratchet_forward(&chain_key, 0);
    let (k2, _) = ratchet_forward(&chain_key, 1);
    assert_ne!(k1, k2, "different msg_count must produce different keys");
    let (k1_again, _) = ratchet_forward(&chain_key, 0);
    assert_eq!(k1, k1_again, "same msg_count must produce same key");
}

#[test]
fn test_wrong_key_fails_to_decrypt() {
    let (msg_key_a, _) = ratchet_forward(&[0xABu8; 32], 0);
    let (msg_key_b, _) = ratchet_forward(&[0xCDu8; 32], 0);
    let cipher = encrypt_with_message_key(&msg_key_a, "secret").unwrap();
    let result = decrypt_with_message_key(&msg_key_b, &cipher);
    assert!(result.is_err(), "wrong key must fail to decrypt");
}
