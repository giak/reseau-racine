#![no_main]

use libfuzzer_sys::fuzz_target;
use nostr::nips::nip44;
use nostr::secp256k1;

fuzz_target!(|data: &[u8]| {
    if data.len() < 32 {
        return;
    }
    let Ok(secp_sk) = secp256k1::SecretKey::from_slice(&data[..32]) else { return };
    let secp = secp256k1::Secp256k1::new();
    let pk_secp = secp256k1::PublicKey::from_secret_key(&secp, &secp_sk);
    let (xonly, _) = pk_secp.x_only_public_key();
    let sk = nostr::SecretKey::from(secp_sk);
    let pk = nostr::PublicKey::from(xonly);
    let payload = &data[32..];
    let _ = nip44::decrypt(&sk, &pk, payload);
});
