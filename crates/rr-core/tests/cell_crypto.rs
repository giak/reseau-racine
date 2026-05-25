use nostr::Keys;
use rr_core::CryptoProvider;

#[test]
fn test_group_key_symmetric_encrypt_decrypt() {
    let cell_keys = Keys::generate();
    let cell_sk = cell_keys.secret_key();
    let cell_pk = &cell_keys.public_key();
    let msg = "Hello cellule!";

    let cipher = CryptoProvider::encrypt(cell_sk, cell_pk, msg).unwrap();
    let plain = CryptoProvider::decrypt(cell_sk, cell_pk, &cipher).unwrap();
    assert_eq!(plain, msg);
}

#[test]
fn test_group_key_deterministic() {
    let cell_keys = Keys::generate();
    let cell_sk = cell_keys.secret_key();
    let cell_pk = &cell_keys.public_key();
    let msg = "determinism test";

    let c1 = CryptoProvider::encrypt(cell_sk, cell_pk, msg).unwrap();
    let c2 = CryptoProvider::encrypt(cell_sk, cell_pk, msg).unwrap();
    assert_ne!(c1, c2);

    assert_eq!(CryptoProvider::decrypt(cell_sk, cell_pk, &c1).unwrap(), msg);
    assert_eq!(CryptoProvider::decrypt(cell_sk, cell_pk, &c2).unwrap(), msg);
}

#[test]
fn test_group_key_rejects_wrong_key() {
    let cell_keys = Keys::generate();
    let wrong_keys = Keys::generate();
    let cell_sk = cell_keys.secret_key();
    let cell_pk = &cell_keys.public_key();
    let wrong_sk = wrong_keys.secret_key();
    let wrong_pk = &wrong_keys.public_key();

    let msg = "secret group message";
    let cipher = CryptoProvider::encrypt(cell_sk, cell_pk, msg).unwrap();

    let result = CryptoProvider::decrypt(wrong_sk, wrong_pk, &cipher);
    assert!(result.is_err());
}
