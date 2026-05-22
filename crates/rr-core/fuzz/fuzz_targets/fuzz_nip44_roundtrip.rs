#![no_main]

use libfuzzer_sys::fuzz_target;
use nostr::nips::nip44::{self, Version};
use nostr::secp256k1;

fuzz_target!(|data: &[u8]| {
    if data.len() < 32 {
        return;
    }
    let payload = &data[32..];
    if payload.len() > 65535 {
        return;
    }
    let Ok(secp_sk) = secp256k1::SecretKey::from_slice(&data[..32]) else { return };
    let secp = secp256k1::Secp256k1::new();
    let pk_secp = secp256k1::PublicKey::from_secret_key(&secp, &secp_sk);
    let (xonly, _) = pk_secp.x_only_public_key();
    let sk = nostr::SecretKey::from(secp_sk);
    let pk = nostr::PublicKey::from(xonly);
    if let Ok(ciphertext) = nip44::encrypt(&sk, &pk, payload, Version::V2) {
        if let Ok(plaintext) = nip44::decrypt(&sk, &pk, &ciphertext) {
            assert_eq!(plaintext.as_bytes(), payload);
        }
    }
});
