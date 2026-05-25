use nostr::prelude::*;
use nostr_sdk::prelude::*;
use rand::RngCore;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::cell::{Cell, CellMember, CellStore, SenderKey};
use crate::sender_key;
use crate::CryptoProvider;

pub struct CellTransport {
    client: Client,
    keys: Keys,
    store: Arc<Mutex<CellStore>>,
}

impl CellTransport {
    pub fn new(client: Client, keys: Keys) -> Self {
        Self {
            client,
            keys,
            store: Arc::new(Mutex::new(CellStore::load())),
        }
    }

    pub async fn create_cell(
        &self,
        label: &str,
        member_pubkeys: &[PublicKey],
    ) -> Result<Cell, Box<dyn std::error::Error>> {
        let sender_pk = self.keys.public_key();

        // Generate own Sender Key
        let chain_key = {
            use rand::RngCore;
            let mut key = [0u8; 32];
            rand::rngs::OsRng.fill_bytes(&mut key);
            key
        };

        let sender_key = SenderKey {
            member_pubkey: sender_pk,
            chain_key_hex: hex::encode(chain_key),
            msg_count: 0,
            created_at_secs: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };

        let mut members: Vec<CellMember> = member_pubkeys
            .iter()
            .map(|pk| CellMember::new(*pk, None))
            .collect();
        members.push(CellMember::new(sender_pk, Some("me".to_string())));

        let cell = Cell {
            id: Uuid::new_v4(),
            label: label.to_string(),
            cell_key_hex: String::new(),
            sender_keys: vec![sender_key.clone()],
            members,
            created_at_secs: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };

        let cell_id_hex = cell.id.to_string();

        // Send Sender Key to each member via gift-wrap
        let payload = serde_json::json!({
            "action": "sender_key",
            "sender_pubkey": sender_pk.to_bech32()?,
            "chain_key_hex": sender_key.chain_key_hex,
            "msg_count": 0,
            "id": cell.id.to_string(),
            "label": label,
        });
        let payload_str = payload.to_string();

        for member_pk in member_pubkeys {
            self.send_cell_key(member_pk, &payload_str, &cell_id_hex)
                .await?;
        }

        let mut store = self.store.lock().await;
        store.add(cell.clone());
        store.save()?;

        Ok(cell)
    }

    pub async fn invite_member(
        &self,
        cell_id: &Uuid,
        new_member_pk: &PublicKey,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let store = self.store.lock().await;
        let cell = store
            .find(cell_id)
            .ok_or_else(|| format!("Cellule {} introuvable", cell_id))?
            .clone();
        let cell_id_hex = cell.id.to_string();
        let cell_label = cell.label.clone();
        let existing_keys = cell.sender_keys.clone();
        drop(store);

        // Generate a Sender Key for the new member
        let new_chain_key = {
            use rand::RngCore;
            let mut key = [0u8; 32];
            rand::rngs::OsRng.fill_bytes(&mut key);
            key
        };

        let new_sender_key = SenderKey {
            member_pubkey: *new_member_pk,
            chain_key_hex: hex::encode(new_chain_key),
            msg_count: 0,
            created_at_secs: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };

        // Send all existing sender keys + new member's key to the new member
        let mut all_keys = existing_keys.clone();
        all_keys.push(new_sender_key.clone());

        let payload = serde_json::json!({
            "action": "sender_key",
            "sender_keys": all_keys.iter().map(|sk| serde_json::json!({
                "member_pubkey": sk.member_pubkey.to_bech32().unwrap_or_default(),
                "chain_key_hex": &sk.chain_key_hex,
                "msg_count": sk.msg_count,
            })).collect::<Vec<_>>(),
            "id": cell_id_hex,
            "label": cell_label,
        });
        let payload_str = payload.to_string();

        self.send_cell_key(new_member_pk, &payload_str, &cell_id_hex)
            .await?;

        // Update local store
        let mut store = self.store.lock().await;
        if let Some(cell) = store.cells.iter_mut().find(|c| c.id == *cell_id) {
            cell.members.push(CellMember::new(*new_member_pk, None));
            cell.sender_keys.push(new_sender_key);
            store.save()?;
        }

        Ok(())
    }

    pub async fn send_message(
        &self,
        cell_id: &Uuid,
        content: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let store = self.store.lock().await;
        let cell = store
            .find(cell_id)
            .ok_or_else(|| format!("Cellule {} introuvable", cell_id))?
            .clone();
        let cell_id_hex = cell.id.to_string();
        let members: Vec<PublicKey> = cell.members.iter().map(|m| m.pubkey).collect();
        let my_pk = self.keys.public_key();
        drop(store);

        // Sender Key path
        if let Some(sk) = cell.sender_keys.iter().find(|sk| sk.member_pubkey == my_pk) {
            let mut chain = [0u8; 32];
            hex::decode_to_slice(&sk.chain_key_hex, &mut chain)?;
            let (msg_key, next_chain) = sender_key::ratchet_forward(&chain);
            let cipher = sender_key::encrypt_with_message_key(&msg_key, content)?;
            let cipher_b64 = {
                use base64::Engine as _;
                let engine = base64::engine::general_purpose::STANDARD;
                engine.encode(&cipher)
            };

            let rumor = EventBuilder::new(Kind::TextNote, cipher_b64)
                .tag(Tag::custom(
                    TagKind::Custom("h".to_string().into()),
                    vec![cell_id_hex],
                ))
                .build(self.keys.public_key());

            for member_pk in &members {
                let wrap = EventBuilder::gift_wrap(&self.keys, member_pk, rumor.clone(), []).await?;
                self.client.send_event(&wrap).await?;
            }

            // Update chain key in store
            let mut store = self.store.lock().await;
            if let Some(cell) = store.cells.iter_mut().find(|c| c.id == *cell_id) {
                if let Some(sk) = cell.sender_keys.iter_mut().find(|sk| sk.member_pubkey == my_pk) {
                    sk.chain_key_hex = hex::encode(next_chain);
                    sk.msg_count += 1;
                }
            }
            store.save()?;
        } else {
            // Legacy EPIC 2 path (NIP-44 with cell_key_hex)
            let cell_sk = SecretKey::from_hex(&cell.cell_key_hex)?;
            let cell_pk = Keys::new(cell_sk.clone()).public_key();
            let encrypted = CryptoProvider::encrypt(&cell_sk, &cell_pk, content)?;

            let rumor = EventBuilder::new(Kind::TextNote, encrypted)
                .tag(Tag::custom(
                    TagKind::Custom("h".to_string().into()),
                    vec![cell_id_hex],
                ))
                .build(self.keys.public_key());

            for member_pk in &members {
                let wrap = EventBuilder::gift_wrap(&self.keys, member_pk, rumor.clone(), []).await?;
                self.client.send_event(&wrap).await?;
            }
        }

        Ok(())
    }

    pub async fn remove_member(
        &self,
        cell_id: &Uuid,
        target_pubkey: &PublicKey,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let store = self.store.lock().await;
        let cell = store
            .find(cell_id)
            .ok_or_else(|| format!("Cellule {} introuvable", cell_id))?
            .clone();
        let remaining: Vec<&CellMember> = cell.members.iter()
            .filter(|m| m.pubkey != *target_pubkey)
            .collect();

        if !remaining.iter().any(|m| m.pubkey == self.keys.public_key()) {
            return Err("Vous n'êtes pas membre de cette cellule".into());
        }

        let cell_id_hex = cell.id.to_string();
        drop(store);

        // Generate fresh Sender Keys for all remaining members
        let new_keys: Vec<SenderKey> = remaining
            .iter()
            .map(|m| {
                let mut chain = [0u8; 32];
                rand::rngs::OsRng.fill_bytes(&mut chain);
                SenderKey {
                    member_pubkey: m.pubkey,
                    chain_key_hex: hex::encode(chain),
                    msg_count: 0,
                    created_at_secs: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                }
            })
            .collect();

        // Distribute all new keys to each remaining member
        let all_keys_payload = serde_json::json!({
            "action": "key_rotation",
            "cell_id": cell_id_hex,
            "sender_keys": new_keys.iter().map(|sk| serde_json::json!({
                "member_pubkey": sk.member_pubkey.to_bech32().unwrap_or_default(),
                "chain_key_hex": &sk.chain_key_hex,
                "msg_count": sk.msg_count,
            })).collect::<Vec<_>>(),
            "removed_member": target_pubkey.to_bech32()?,
        });
        let payload_str = all_keys_payload.to_string();
        let cell_id_hex_clone = cell_id_hex.clone();
        for member in &remaining {
            self.send_cell_key(&member.pubkey, &payload_str, &cell_id_hex_clone)
                .await?;
        }

        // Update local store
        let mut store = self.store.lock().await;
        if let Some(cell) = store.cells.iter_mut().find(|c| c.id == *cell_id) {
            cell.members.retain(|m| m.pubkey != *target_pubkey);
            cell.sender_keys = new_keys;
            store.save()?;
        }

        println!("✅ Membre retiré : {}", target_pubkey.to_bech32()?);
        Ok(())
    }

    pub async fn rotate_key(&self, cell_id: &Uuid) -> Result<(), Box<dyn std::error::Error>> {
        let store = self.store.lock().await;
        let cell = store
            .find(cell_id)
            .ok_or_else(|| format!("Cellule {} introuvable", cell_id))?
            .clone();
        let remaining: Vec<PublicKey> = cell.members.iter().map(|m| m.pubkey).collect();
        let cell_id_hex = cell.id.to_string();
        drop(store);

        let new_keys: Vec<SenderKey> = remaining
            .iter()
            .map(|pk| {
                let mut chain = [0u8; 32];
                rand::rngs::OsRng.fill_bytes(&mut chain);
                SenderKey {
                    member_pubkey: *pk,
                    chain_key_hex: hex::encode(chain),
                    msg_count: 0,
                    created_at_secs: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                }
            })
            .collect();

        let payload = serde_json::json!({
            "action": "key_rotation",
            "cell_id": cell_id_hex,
            "sender_keys": new_keys.iter().map(|sk| serde_json::json!({
                "member_pubkey": sk.member_pubkey.to_bech32().unwrap_or_default(),
                "chain_key_hex": &sk.chain_key_hex,
                "msg_count": sk.msg_count,
            })).collect::<Vec<_>>(),
        });
        let payload_str = payload.to_string();
        let cell_id_hex_clone = cell_id_hex.clone();
        for pk in &remaining {
            self.send_cell_key(pk, &payload_str, &cell_id_hex_clone).await?;
        }

        let mut store = self.store.lock().await;
        if let Some(cell) = store.cells.iter_mut().find(|c| c.id == *cell_id) {
            cell.sender_keys = new_keys;
            store.save()?;
        }

        println!("✅ Clés de la cellule {} régénérées", cell_id);
        Ok(())
    }

    pub async fn listen(&self, cell_id: Option<&Uuid>) -> Result<(), Box<dyn std::error::Error>> {
        let my_pk = self.keys.public_key();
        let client = self.client.clone();

        let (target_cell_id, cell_sk, cell_pk, cell_sender_keys) = if let Some(cid) = cell_id {
            let store = self.store.lock().await;
            let cell = store
                .find(cid)
                .ok_or_else(|| format!("Cellule {} introuvable", cid))?;
            let sk = SecretKey::from_hex(&cell.cell_key_hex).ok();
            let pk = sk.as_ref().map(|s| Keys::new(s.clone()).public_key());
            (Some(cell.id.to_string()), sk, pk, cell.sender_keys.clone())
        } else {
            (None, None, None, vec![])
        };

        let filter = Filter::new().kind(Kind::GiftWrap).pubkey(my_pk);

        client.subscribe(filter, None).await?;

        if let Some(cid) = &target_cell_id {
            println!("En écoute sur la cellule {} — Ctrl+C pour arrêter", cid);
        } else {
            println!("En écoute (mode découverte) — Ctrl+C pour arrêter");
        }

        client
            .handle_notifications(|notification| {
                let cell_sk = cell_sk.clone();
                let cell_sender_keys = cell_sender_keys.clone();
                let target_cell_id = target_cell_id.clone();
                let client = client.clone();
                let keys = self.keys.clone();
                let store_arc = self.store.clone();

                async move {
                    if let RelayPoolNotification::Event { event, .. } = notification {
                        if event.kind != Kind::GiftWrap {
                            return Ok(false);
                        }
                        let unwrapped = match client.unwrap_gift_wrap(&event).await {
                            Ok(u) => u,
                            Err(_) => return Ok(false),
                        };
                        let rumor = unwrapped.rumor;
                        let sender_pk = unwrapped.sender;

                        let h_tag_val: Option<String> = rumor
                            .tags
                            .iter()
                            .find(|t| t.kind() == TagKind::Custom("h".to_string().into()))
                            .and_then(|t| t.content())
                            .map(|s| s.to_string());

                        let h_tag = match &h_tag_val {
                            Some(v) => v.clone(),
                            None => return Ok(false),
                        };

                        // Mode 1: listening to a specific cell
                        if let Some(tid) = &target_cell_id {
                            if &h_tag != tid {
                                return Ok(false);
                            }

                            // Try Sender Key decryption first
                            if let Some(sk) = cell_sender_keys.iter().find(|sk| sk.member_pubkey == sender_pk) {
                                let mut chain = [0u8; 32];
                                if hex::decode_to_slice(&sk.chain_key_hex, &mut chain).is_ok() {
                                    let (msg_key, next_chain) = sender_key::ratchet_forward(&chain);
                                    if let Ok(cipher_bytes) = {
                                        use base64::Engine as _;
                                        let engine = base64::engine::general_purpose::STANDARD;
                                        engine.decode(&rumor.content)
                                    } {
                                        if let Ok(plaintext) = sender_key::decrypt_with_message_key(&msg_key, &cipher_bytes) {
                                            // Update chain in store
                                            let mut store = store_arc.lock().await;
                                            if let Some(cell) = store.cells.iter_mut().find(|c| c.id.to_string() == h_tag) {
                                                if let Some(sk) = cell.sender_keys.iter_mut().find(|sk| sk.member_pubkey == sender_pk) {
                                                    sk.chain_key_hex = hex::encode(next_chain);
                                                    sk.msg_count += 1;
                                                }
                                            }
                                            drop(store);

                                            if sender_pk != keys.public_key() {
                                                let snpub = sender_pk
                                                    .to_bech32()
                                                    .unwrap_or_else(|_| sender_pk.to_string());
                                                println!("[{}] {}: {}", tid, snpub, plaintext);
                                            }
                                            return Ok(false);
                                        }
                                    }
                                }
                            }

                            // Legacy fallback: NIP-44
                            if let (Some(ref sk), Some(ref pk)) = (&cell_sk, &cell_pk) {
                                if let Ok(plaintext) =
                                    CryptoProvider::decrypt(sk, pk, &rumor.content)
                                {
                                    if sender_pk != keys.public_key() {
                                        let snpub = sender_pk
                                            .to_bech32()
                                            .unwrap_or_else(|_| sender_pk.to_string());
                                        println!("[{}] {}: {}", tid, snpub, plaintext);
                                    }
                                }
                            }
                            return Ok(false);
                        }

                        // Mode 2: discovery — try key distribution first
                        let cell_id_parsed = uuid::Uuid::parse_str(&h_tag);
                        let mut store = store_arc.lock().await;

                        if let Ok(cid) = cell_id_parsed {
                            if store.find(&cid).is_none() {
                                // Unknown cell — check if this is a key distribution
                                if let Ok(payload) =
                                    serde_json::from_str::<serde_json::Value>(&rumor.content)
                                {
                                    if let (Some(key), Some(label)) = (
                                        payload.get("key").and_then(|v| v.as_str()),
                                        payload.get("label").and_then(|v| v.as_str()),
                                    ) {
                                        let new_cell = Cell::new(
                                            label,
                                            key.to_string(),
                                            Vec::new(),
                                            vec![
                                                CellMember::new(sender_pk, None),
                                                CellMember::new(
                                                    keys.public_key(),
                                                    Some("me".to_string()),
                                                ),
                                            ],
                                        );
                                        store.add(new_cell.clone());
                                        if let Err(e) = store.save() {
                                            eprintln!("Erreur sauvegarde cellule: {}", e);
                                        }
                                        println!("Nouvelle cellule: {} ({})", label, new_cell.id);
                                    }
                                }
                            } else {
                                // Known cell — decrypt and display
                                if let Some(cell) = store.find(&cid).cloned() {
                                    drop(store);

                                    // Try Sender Key first
                                    if let Some(sk) = cell.sender_keys.iter().find(|sk| sk.member_pubkey == sender_pk) {
                                        let mut chain = [0u8; 32];
                                        if hex::decode_to_slice(&sk.chain_key_hex, &mut chain).is_ok() {
                                            let (msg_key, next_chain) = sender_key::ratchet_forward(&chain);
                                            if let Ok(cipher_bytes) = {
                                                use base64::Engine as _;
                                                let engine = base64::engine::general_purpose::STANDARD;
                                                engine.decode(&rumor.content)
                                            } {
                                                if let Ok(plaintext) = sender_key::decrypt_with_message_key(&msg_key, &cipher_bytes) {
                                                    // Update chain in store
                                                    let mut store = store_arc.lock().await;
                                                    if let Some(c) = store.cells.iter_mut().find(|c| c.id == cid) {
                                                        if let Some(sk) = c.sender_keys.iter_mut().find(|sk| sk.member_pubkey == sender_pk) {
                                                            sk.chain_key_hex = hex::encode(next_chain);
                                                            sk.msg_count += 1;
                                                        }
                                                    }
                                                    store.save().ok();
                                                    drop(store);

                                                    if sender_pk != keys.public_key() {
                                                        let snpub = sender_pk
                                                            .to_bech32()
                                                            .unwrap_or_else(|_| sender_pk.to_string());
                                                        println!(
                                                            "[{}] {}: {}",
                                                            cell.label, snpub, plaintext
                                                        );
                                                    }
                                                    return Ok(false);
                                                }
                                            }
                                        }
                                    }

                                    // Legacy fallback
                                    if let Ok(sk) = SecretKey::from_hex(&cell.cell_key_hex) {
                                        let pk = Keys::new(sk.clone()).public_key();
                                        if let Ok(plaintext) =
                                            CryptoProvider::decrypt(&sk, &pk, &rumor.content)
                                        {
                                            if sender_pk != keys.public_key() {
                                                let snpub = sender_pk
                                                    .to_bech32()
                                                    .unwrap_or_else(|_| sender_pk.to_string());
                                                println!(
                                                    "[{}] {}: {}",
                                                    cell.label, snpub, plaintext
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Ok(false)
                }
            })
            .await?;

        Ok(())
    }

    async fn send_cell_key(
        &self,
        receiver_pk: &PublicKey,
        payload: &str,
        cell_id_hex: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let rumor = EventBuilder::new(Kind::TextNote, payload.to_string())
            .tag(Tag::custom(
                TagKind::Custom("h".to_string().into()),
                vec![cell_id_hex.to_string()],
            ))
            .build(self.keys.public_key());

        let wrap = EventBuilder::gift_wrap(&self.keys, receiver_pk, rumor, []).await?;
        self.client.send_event(&wrap).await?;
        Ok(())
    }
}
