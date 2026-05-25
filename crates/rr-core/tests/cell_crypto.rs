use nostr::Keys;
use nostr::nips::nip44;

fn encrypt(cell_sk: &nostr::SecretKey, cell_pk: &nostr::PublicKey, msg: &str) -> String {
    nip44::encrypt(cell_sk, cell_pk, msg, nip44::Version::V2).unwrap()
}

fn decrypt(cell_sk: &nostr::SecretKey, cell_pk: &nostr::PublicKey, cipher: &str) -> String {
    nip44::decrypt(cell_sk, cell_pk, cipher).unwrap()
}

#[test]
fn test_group_key_symmetric_encrypt_decrypt() {
    let cell_keys = Keys::generate();
    let cell_sk = cell_keys.secret_key();
    let cell_pk = &cell_keys.public_key();
    let msg = "Hello cellule!";

    let cipher = encrypt(cell_sk, cell_pk, msg);
    let plain = decrypt(cell_sk, cell_pk, &cipher);
    assert_eq!(plain, msg);
}

#[test]
fn test_group_key_deterministic() {
    let cell_keys = Keys::generate();
    let cell_sk = cell_keys.secret_key();
    let cell_pk = &cell_keys.public_key();
    let msg = "determinism test";

    let c1 = encrypt(cell_sk, cell_pk, msg);
    let c2 = encrypt(cell_sk, cell_pk, msg);
    assert_ne!(c1, c2);

    assert_eq!(decrypt(cell_sk, cell_pk, &c1), msg);
    assert_eq!(decrypt(cell_sk, cell_pk, &c2), msg);
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
    let cipher = encrypt(cell_sk, cell_pk, msg);

    let result = nip44::decrypt(wrong_sk, wrong_pk, &cipher);
    assert!(result.is_err());
}
