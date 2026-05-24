use nostr::prelude::*;
use nostr_sdk::prelude::*;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::cell::{Cell, CellMember, CellStore};

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

        let cell = Cell::new(label, cell_sk_hex, members);
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
