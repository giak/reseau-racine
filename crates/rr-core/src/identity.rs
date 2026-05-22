use bip39::Mnemonic;
use nostr::nips::nip06;
use nostr::nips::nip06::FromMnemonic;
use nostr::nips::nip19::FromBech32;
use nostr::nips::nip19::ToBech32;
use nostr::{Keys, PublicKey, SecretKey};
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

pub struct Identity {
    keys: Keys,
}

impl fmt::Debug for Identity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Identity")
            .field("npub", &self.public_key_bech32())
            .finish()
    }
}

impl Identity {
    pub fn new() -> Self {
        Self {
            keys: Keys::generate(),
        }
    }

    pub fn from_secret_key(secret_key: &SecretKey) -> Self {
        Self {
            keys: Keys::new(secret_key.clone()),
        }
    }

    pub fn from_seed_phrase(mnemonic_str: &str, passphrase: &str) -> Result<Self, nip06::Error> {
        let _mnemonic = Mnemonic::from_str(mnemonic_str).map_err(nip06::Error::BIP39)?;
        let keys = Keys::from_mnemonic(mnemonic_str, Some(passphrase))?;
        Ok(Self { keys })
    }

    pub fn from_nsec(nsec: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let secret_key = SecretKey::from_bech32(nsec)?;
        Ok(Self {
            keys: Keys::new(secret_key),
        })
    }

    pub fn generate_seed_phrase() -> Result<String, Box<dyn std::error::Error>> {
        let mnemonic = Mnemonic::generate(12)?;
        Ok(mnemonic.to_string())
    }

    pub fn keys(&self) -> &Keys {
        &self.keys
    }

    pub fn public_key(&self) -> PublicKey {
        self.keys.public_key()
    }

    pub fn public_key_bech32(&self) -> String {
        self.keys
            .public_key()
            .to_bech32()
            .expect("valid public key always encodes to bech32")
    }

    pub fn secret_key_bech32(&self) -> String {
        self.keys
            .secret_key()
            .to_bech32()
            .expect("valid secret key always encodes to bech32")
    }
}

impl Default for Identity {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct IdentityManager {
    data_dir: PathBuf,
}

impl IdentityManager {
    pub fn new(data_dir: impl Into<PathBuf>) -> Self {
        Self {
            data_dir: data_dir.into(),
        }
    }

    pub fn default_data_dir() -> PathBuf {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".rr")
    }

    pub fn load_or_create(&self) -> Result<Identity, Box<dyn std::error::Error>> {
        let key_path = self.data_dir.join("keys.json");
        if key_path.exists() {
            self.load()
        } else {
            let identity = Identity::new();
            self.save(&identity)?;
            Ok(identity)
        }
    }

    pub fn load(&self) -> Result<Identity, Box<dyn std::error::Error>> {
        let key_path = self.data_dir.join("keys.json");
        let content = std::fs::read_to_string(&key_path)?;
        let data: serde_json::Value = serde_json::from_str(&content)?;
        let nsec = data["nsec"].as_str().ok_or("missing nsec field")?;
        Identity::from_nsec(nsec)
    }

    pub fn save(&self, identity: &Identity) -> Result<(), Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&self.data_dir)?;
        let key_path = self.data_dir.join("keys.json");
        let data = serde_json::json!({
            "npub": identity.public_key_bech32(),
            "nsec": identity.secret_key_bech32(),
        });
        let content = serde_json::to_string_pretty(&data)?;
        std::fs::write(&key_path, &content)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&key_path, std::fs::Permissions::from_mode(0o600))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_generation() {
        let identity = Identity::new();
        assert!(!identity.secret_key_bech32().is_empty());
        assert!(!identity.public_key_bech32().is_empty());
    }

    #[test]
    fn test_identity_from_nsec() {
        let original = Identity::new();
        let nsec = original.secret_key_bech32();
        let restored = Identity::from_nsec(&nsec).unwrap();
        assert_eq!(original.public_key_bech32(), restored.public_key_bech32());
    }

    #[test]
    fn test_seed_phrase_generation() {
        let phrase = Identity::generate_seed_phrase().unwrap();
        let words: Vec<&str> = phrase.split_whitespace().collect();
        assert_eq!(words.len(), 12);
    }

    #[test]
    fn test_identity_from_seed_phrase() {
        let phrase = Identity::generate_seed_phrase().unwrap();
        let identity = Identity::from_seed_phrase(&phrase, "").unwrap();
        assert!(!identity.public_key_bech32().is_empty());
    }

    #[test]
    fn test_save_and_load() {
        let dir = std::env::temp_dir().join(format!("rr-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);

        let manager = IdentityManager::new(&dir);
        let identity = Identity::new();
        manager.save(&identity).unwrap();
        assert!(dir.join("keys.json").exists());

        let loaded = manager.load().unwrap();
        assert_eq!(identity.public_key_bech32(), loaded.public_key_bech32());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_unique_keys() {
        let a = Identity::new();
        let b = Identity::new();
        assert_ne!(a.secret_key_bech32(), b.secret_key_bech32());
    }

    #[test]
    fn test_from_seed_phrase_with_passphrase() {
        let phrase = Identity::generate_seed_phrase().unwrap();
        let without = Identity::from_seed_phrase(&phrase, "").unwrap();
        let with = Identity::from_seed_phrase(&phrase, "secret").unwrap();
        assert_ne!(without.public_key_bech32(), with.public_key_bech32());
    }

    #[test]
    fn test_seed_phrase_deterministic() {
        let phrase = Identity::generate_seed_phrase().unwrap();
        let a = Identity::from_seed_phrase(&phrase, "").unwrap();
        let b = Identity::from_seed_phrase(&phrase, "").unwrap();
        assert_eq!(a.public_key_bech32(), b.public_key_bech32());
    }

    #[test]
    fn test_invalid_seed_phrase() {
        let result = Identity::from_seed_phrase("not a valid bip39 phrase", "");
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_nsec() {
        let result = Identity::from_nsec("nsec1invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_public_key_format() {
        let identity = Identity::new();
        let npub = identity.public_key_bech32();
        assert!(
            npub.starts_with("npub1"),
            "npub should start with npub1, got: {}",
            npub
        );
        assert_eq!(npub.len(), 63);
    }

    #[test]
    fn test_secret_key_format() {
        let identity = Identity::new();
        let nsec = identity.secret_key_bech32();
        assert!(
            nsec.starts_with("nsec1"),
            "nsec should start with nsec1, got: {}",
            nsec
        );
    }

    #[test]
    fn test_load_missing_file() {
        let dir = std::env::temp_dir().join(format!("rr-test-missing-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let manager = IdentityManager::new(&dir);
        let result = manager.load();
        assert!(result.is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_load_or_create_creates() {
        let dir = std::env::temp_dir().join(format!("rr-test-create-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let manager = IdentityManager::new(&dir);
        let identity = manager.load_or_create().unwrap();
        assert!(!identity.public_key_bech32().is_empty());
        assert!(dir.join("keys.json").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_load_or_create_loads_existing() {
        let dir = std::env::temp_dir().join(format!("rr-test-load-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let original = Identity::new();
        let manager = IdentityManager::new(&dir);
        manager.save(&original).unwrap();
        let loaded = manager.load_or_create().unwrap();
        assert_eq!(original.public_key_bech32(), loaded.public_key_bech32());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_debug_hides_secret_key() {
        let identity = Identity::new();
        let debug = format!("{:?}", identity);
        assert!(debug.contains("npub"));
        assert!(!debug.contains("nsec"));
    }

    #[test]
    fn test_default_identity() {
        let identity = Identity::default();
        assert!(!identity.public_key_bech32().is_empty());
    }
}
