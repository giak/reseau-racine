use nostr::prelude::*;
use nostr_sdk::prelude::*;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::cell::{Cell, CellMember, CellStore};
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
        let cell_keys = Keys::generate();
        let cell_sk_hex = cell_keys.secret_key().to_secret_hex();
        let sender_pk = self.keys.public_key();

        let mut members: Vec<CellMember> = member_pubkeys
            .iter()
            .map(|pk| CellMember::new(*pk, None))
            .collect();
        members.push(CellMember::new(sender_pk, Some("me".to_string())));

        let cell = Cell::new(label, cell_sk_hex, Vec::new(), members);
        let cell_id_hex = cell.id.to_string();

        // Send CellKey to each member via gift-wrap
        let payload = serde_json::json!({
            "key": cell.cell_key_hex,
            "label": label,
            "id": cell.id.to_string(),
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
            .ok_or_else(|| format!("Cellule {} introuvable", cell_id))?;
        let cell_sk_hex = cell.cell_key_hex.clone();
        let cell_id_hex = cell.id.to_string();
        let cell_label = cell.label.clone();
        drop(store);

        let payload = serde_json::json!({
            "key": cell_sk_hex,
            "label": cell_label,
            "id": cell_id_hex,
        });
        let payload_str = payload.to_string();

        self.send_cell_key(new_member_pk, &payload_str, &cell_id_hex)
            .await?;

        let mut store = self.store.lock().await;
        let mut cell = store.find(cell_id).cloned().unwrap();
        cell.members.push(CellMember::new(*new_member_pk, None));
        store.update_members(cell_id, cell.members.clone());
        store.save()?;

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
