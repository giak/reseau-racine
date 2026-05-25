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
        let sk = cell
            .sender_keys
            .iter()
            .find(|sk| sk.member_pubkey == my_pk)
            .ok_or("Aucune clé d'envoi")?;
        let mut chain = [0u8; 32];
        hex::decode_to_slice(&sk.chain_key_hex, &mut chain)?;
        let (msg_key, next_chain) = sender_key::ratchet_forward(&chain, sk.msg_count);
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

        // UPDATE STORE BEFORE NETWORK — atomicity guarantee
        let mut store = self.store.lock().await;
        if let Some(cell) = store.cells.iter_mut().find(|c| c.id == *cell_id) {
            if let Some(sk) = cell
                .sender_keys
                .iter_mut()
                .find(|sk| sk.member_pubkey == my_pk)
            {
                sk.chain_key_hex = hex::encode(next_chain);
                sk.msg_count += 1;
            }
        }
        store.save()?;
        drop(store);

        // Now send (crash here is safe — msg_count already consumed)
        for member_pk in &members {
            let wrap =
                EventBuilder::gift_wrap(&self.keys, member_pk, rumor.clone(), []).await?;
            self.client.send_event(&wrap).await?;
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
        let remaining: Vec<&CellMember> = cell
            .members
            .iter()
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
            self.send_cell_key(pk, &payload_str, &cell_id_hex_clone)
                .await?;
        }

        let mut store = self.store.lock().await;
        if let Some(cell) = store.cells.iter_mut().find(|c| c.id == *cell_id) {
            cell.sender_keys = new_keys;
            store.save()?;
        }

        println!("✅ Clés de la cellule {} régénérées", cell_id);
        Ok(())
    }

    async fn handle_key_rotation(
        store: &Arc<tokio::sync::Mutex<CellStore>>,
        payload: &serde_json::Value,
        cid: &Uuid,
        sender_pk: &PublicKey,
    ) {
        if payload.get("action").and_then(|v| v.as_str()) != Some("key_rotation") {
            return;
        }
        let keys_val = match payload.get("sender_keys").and_then(|v| v.as_array()) {
            Some(k) => k,
            None => return,
        };
        let new_keys: Vec<SenderKey> = keys_val
            .iter()
            .filter_map(|sk_val| {
                let pk_str = sk_val.get("member_pubkey")?.as_str()?;
                let pk = PublicKey::from_bech32(pk_str).ok()?;
                let chain = sk_val.get("chain_key_hex")?.as_str()?;
                Some(SenderKey {
                    member_pubkey: pk,
                    chain_key_hex: chain.to_string(),
                    msg_count: sk_val
                        .get("msg_count")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0) as u64,
                    created_at_secs: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                })
            })
            .collect();
        if new_keys.is_empty() {
            return;
        }

        let mut store = store.lock().await;
        let is_member = store
            .find(cid)
            .map(|cell| cell.members.iter().any(|m| m.pubkey == *sender_pk))
            .unwrap_or(false);
        if !is_member {
            eprintln!(
                "⚠️ Key rotation rejected: sender {} is not a member of cell {}",
                sender_pk, cid
            );
            return;
        }

        if let Some(cell) = store.cells.iter_mut().find(|c| c.id == *cid) {
            for nk in new_keys {
                if let Some(existing) = cell
                    .sender_keys
                    .iter_mut()
                    .find(|sk| sk.member_pubkey == nk.member_pubkey)
                {
                    existing.chain_key_hex = nk.chain_key_hex;
                    existing.msg_count = nk.msg_count;
                    existing.created_at_secs = nk.created_at_secs;
                } else {
                    cell.sender_keys.push(nk);
                }
            }
            let _ = store.save();
            println!("🔑 Clés de cellule mises à jour (key_rotation)");
        }
    }

    pub async fn listen(&self, cell_id: Option<&Uuid>) -> Result<(), Box<dyn std::error::Error>> {
        let my_pk = self.keys.public_key();
        let client = self.client.clone();

        let (target_cell_id, cell_sk, cell_pk) = if let Some(cid) = cell_id {
            let store = self.store.lock().await;
            let cell = store
                .find(cid)
                .ok_or_else(|| format!("Cellule {} introuvable", cid))?;
            let sk = SecretKey::from_hex(&cell.cell_key_hex).ok();
            let pk = sk.as_ref().map(|s| Keys::new(s.clone()).public_key());
            (Some(cell.id.to_string()), sk, pk)
        } else {
            (None, None, None)
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

                            // Lock store: read state, derive, update — all atomically
                            let mut store = store_arc.lock().await;
                            if let Some(cell) = store.cells.iter_mut().find(|c| c.id.to_string() == h_tag) {
                                if let Some(sk) = cell
                                    .sender_keys
                                    .iter_mut()
                                    .find(|sk| sk.member_pubkey == sender_pk)
                                {
                                    let mut chain = [0u8; 32];
                                    let msg_count = sk.msg_count;
                                    let chain_hex = sk.chain_key_hex.clone();
                                    if hex::decode_to_slice(&chain_hex, &mut chain).is_ok() {
                                        let (msg_key, next_chain) =
                                            sender_key::ratchet_forward(&chain, msg_count);
                                        if let Ok(cipher_bytes) = {
                                            use base64::Engine as _;
                                            let engine = base64::engine::general_purpose::STANDARD;
                                            engine.decode(&rumor.content)
                                        } {
                                            if let Ok(plaintext) =
                                                sender_key::decrypt_with_message_key(
                                                    &msg_key,
                                                    &cipher_bytes,
                                                )
                                            {
                                                sk.chain_key_hex = hex::encode(next_chain);
                                                sk.msg_count += 1;
                                                store.save().ok();
                                                drop(store);

                                                if sender_pk != keys.public_key() {
                                                    let snpub = sender_pk
                                                        .to_bech32()
                                                        .unwrap_or_else(|_| sender_pk.to_string());
                                                    println!(
                                                        "[{}] {}: {}",
                                                        tid, snpub, plaintext
                                                    );
                                                }
                                                return Ok(false);
                                            }
                                        }
                                    }
                                }
                            }
                            drop(store);

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
                            // Check for key rotation payload first
                            if let Ok(payload) =
                                serde_json::from_str::<serde_json::Value>(&rumor.content)
                            {
                                if payload.get("action").and_then(|v| v.as_str())
                                    == Some("key_rotation")
                                {
                                    drop(store);
                                    Self::handle_key_rotation(&store_arc, &payload, &cid, &sender_pk).await;
                                    return Ok(false);
                                }
                            }

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
                                // Clone cell info for legacy fallback before dropping store
                                let cell_info = store.find(&cid).map(|c| {
                                    (c.cell_key_hex.clone(), c.label.clone())
                                });

                                // Read and update atomically inside store lock
                                let state = store.cells.iter().position(|c| c.id == cid).and_then(|idx| {
                                    store.cells[idx].sender_keys.iter()
                                        .find(|sk| sk.member_pubkey == sender_pk)
                                        .map(|sk| (idx, sk.msg_count, sk.chain_key_hex.clone()))
                                });

                                if let Some((idx, msg_count, chain_hex)) = state {
                                    let mut chain = [0u8; 32];
                                    if hex::decode_to_slice(&chain_hex, &mut chain).is_ok() {
                                        let (msg_key, next_chain) =
                                            sender_key::ratchet_forward(&chain, msg_count);
                                        if let Ok(cipher_bytes) = {
                                            use base64::Engine as _;
                                            let engine =
                                                base64::engine::general_purpose::STANDARD;
                                            engine.decode(&rumor.content)
                                        } {
                                            if let Ok(plaintext) =
                                                sender_key::decrypt_with_message_key(
                                                    &msg_key,
                                                    &cipher_bytes,
                                                )
                                            {
                                                if let Some(sk) = store.cells[idx]
                                                    .sender_keys
                                                    .iter_mut()
                                                    .find(|sk| {
                                                        sk.member_pubkey == sender_pk
                                                    })
                                                {
                                                    sk.chain_key_hex =
                                                        hex::encode(next_chain);
                                                    sk.msg_count += 1;
                                                }
                                                store.save().ok();
                                                drop(store);

                                                if sender_pk != keys.public_key() {
                                                    let snpub = sender_pk
                                                        .to_bech32()
                                                        .unwrap_or_else(|_| {
                                                            sender_pk.to_string()
                                                        });
                                                    let cell_label = cell_info.as_ref()
                                                        .map(|(_, l)| l.as_str())
                                                        .unwrap_or("");
                                                    println!(
                                                        "[{}] {}: {}",
                                                        cell_label, snpub, plaintext
                                                    );
                                                }
                                                return Ok(false);
                                            }
                                        }
                                    }
                                }
                                drop(store);

                                // Legacy fallback
                                if let Some((cell_key_hex, cell_label)) = cell_info {
                                    if let Ok(sk) = SecretKey::from_hex(&cell_key_hex) {
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
                                                    cell_label, snpub, plaintext
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
