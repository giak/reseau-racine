use nostr::PublicKey;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellMember {
    pub pubkey: PublicKey,
    pub label: Option<String>,
    pub added_at_secs: u64,
}

impl CellMember {
    pub fn new(pubkey: PublicKey, label: Option<String>) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            pubkey,
            label,
            added_at_secs: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SenderKey {
    pub member_pubkey: PublicKey,
    pub chain_key_hex: String,
    pub msg_count: u64,
    pub created_at_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cell {
    pub id: Uuid,
    pub label: String,
    /// Hex-encoded SecretKey (cell_key_hex -> SecretKey::from_hex)
    pub cell_key_hex: String,
    pub sender_keys: Vec<SenderKey>,
    pub members: Vec<CellMember>,
    pub created_at_secs: u64,
}

impl Cell {
    pub fn new(
        label: &str,
        cell_key_hex: String,
        sender_keys: Vec<SenderKey>,
        members: Vec<CellMember>,
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            id: Uuid::new_v4(),
            label: label.to_string(),
            cell_key_hex,
            sender_keys,
            members,
            created_at_secs: now,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CellStore {
    pub(crate) cells: Vec<Cell>,
}

impl CellStore {
    pub fn path() -> PathBuf {
        let base = std::env::var("RR_DATA_DIR")
            .map(PathBuf::from)
            .ok()
            .unwrap_or_else(crate::config::Config::config_dir);
        base.join("cells.json")
    }

    pub fn load() -> Self {
        let path = Self::path();

        // Clean up stale .tmp files from previous crashes
        let tmp_path = path.with_extension("tmp");
        if tmp_path.exists() {
            if let Err(e) = std::fs::remove_file(&tmp_path) {
                eprintln!("⚠️ Failed to remove stale .tmp file: {}", e);
            }
        }

        if !path.exists() {
            return Self::default();
        }

        match std::fs::read_to_string(&path) {
            Ok(content) => match serde_json::from_str(&content) {
                Ok(store) => store,
                Err(e) => {
                    eprintln!("⚠️ Failed to parse cells.json: {}. Using empty store.", e);
                    Self::default()
                }
            },
            Err(e) => {
                eprintln!("⚠️ Failed to read cells.json: {}", e);
                Self::default()
            }
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::path();
        let dir = path.parent().unwrap();
        std::fs::create_dir_all(dir)?;
        let tmp_path = path.with_extension("tmp");
        std::fs::write(&tmp_path, serde_json::to_string_pretty(self)?)?;
        std::fs::rename(&tmp_path, &path)?;
        Ok(())
    }

    pub fn find(&self, id: &Uuid) -> Option<&Cell> {
        self.cells.iter().find(|c| &c.id == id)
    }

    pub fn all(&self) -> &[Cell] {
        &self.cells
    }

    pub fn add(&mut self, cell: Cell) {
        self.cells.push(cell);
    }

    pub fn remove(&mut self, id: &Uuid) {
        self.cells.retain(|c| &c.id != id);
    }

    pub fn update_members(&mut self, id: &Uuid, members: Vec<CellMember>) -> bool {
        if let Some(cell) = self.cells.iter_mut().find(|c| &c.id == id) {
            cell.members = members;
            true
        } else {
            false
        }
    }
}
