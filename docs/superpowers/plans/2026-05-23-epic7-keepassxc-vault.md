# EPIC 7 — KeePassXC Vault Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Clés Nostr lues depuis KeePassXC au lieu de JSON clair. 2 backends : keepassxc-cli (bridge) + keepass-rs (crate Rust direct).

**Architecture:** `KeySource` enum dans `identity.rs` configure le backend. `IdentityManager::with_key_source()` propage la source. `load()` route vers le bon backend selon `RR_KEYSTORE`. `save()` inchangé (toujours JSON). Rétro-compatible : défaut `File`.

**Tech Stack:** Rust, `keepass` crate (KDBX reader), `rpassword`, `keepassxc-cli` (optionnel)

---

## File Map

| Fichier | Changement |
|---------|-----------|
| `Cargo.toml` (workspace) | Ajouter `keepass`, `rpassword` en workspace deps |
| `crates/rr-core/Cargo.toml` | Ajouter `keepass`, `rpassword` au `[dependencies]` |
| `crates/rr-core/src/identity.rs` | Ajouter `KeySource` enum, `with_key_source()`, 2 backends dans `load()` |
| `crates/rr-cli/src/main.rs` | Parse `RR_KEYSTORE`, passe `KeySource` à `IdentityManager` partout |
| `docs/TRACKING.md` | Marquer EPIC 7 stories ✅ |

---

### Task 1: Ajouter les dépendances

- [ ] **Step 1: Ajouter `keepass` et `rpassword` au workspace**

Modifier `Cargo.toml` (workspace root) :

```toml
[workspace.dependencies]
# ... existing ...
keepass = "0.12"
rpassword = "7"
```

- [ ] **Step 2: Ajouter les deps à rr-core**

Modifier `crates/rr-core/Cargo.toml` :

```toml
[dependencies]
# ... existing ...
keepass.workspace = true
rpassword.workspace = true
```

- [ ] **Step 3: Vérifier que ça compile**

```bash
./scripts/dev.sh cargo check --package rr-core
```

Expected : success

- [ ] **Step 4: Commit**

```bash
rtk git add Cargo.toml crates/rr-core/Cargo.toml
rtk git commit -m "deps: add keepass and rpassword for KeePassXC vault"
```

---

### Task 2: Ajouter `KeySource` enum et `with_key_source()` dans identity.rs

- [ ] **Step 1: Ajouter l'import et le enum**

Dans `crates/rr-core/src/identity.rs`, après les imports existants :

```rust
use std::process::{Command, Stdio};
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub enum KeySource {
    File,
    KeePassXc { db_path: String, entry: String },
    KeePassRs { db_path: String, entry: String },
}

impl KeySource {
    pub fn from_env() -> Self {
        match std::env::var("RR_KEYSTORE") {
            Ok(val) if val == "file" || val.is_empty() => KeySource::File,
            Ok(val) if val.starts_with("keepassxc://") => {
                let rest = val.trim_start_matches("keepassxc://");
                let (db_path, entry) = rest.split_once('/').unwrap_or((rest, ""));
                KeySource::KeePassXc {
                    db_path: db_path.to_string(),
                    entry: entry.to_string(),
                }
            }
            Ok(val) if val.starts_with("keepass-rs://") => {
                let rest = val.trim_start_matches("keepass-rs://");
                let (db_path, entry) = rest.split_once('/').unwrap_or((rest, ""));
                KeySource::KeePassRs {
                    db_path: db_path.to_string(),
                    entry: entry.to_string(),
                }
            }
            _ => KeySource::File,
        }
    }
}
```

- [ ] **Step 2: Ajouter `key_source` champ à `IdentityManager` + `with_key_source()`**

Dans `struct IdentityManager`, après `data_dir` :

```rust
pub struct IdentityManager {
    data_dir: PathBuf,
    key_source: KeySource,
}
```

Modifier `IdentityManager::new()` :

```rust
impl IdentityManager {
    pub fn new(data_dir: impl Into<PathBuf>) -> Self {
        Self {
            data_dir: data_dir.into(),
            key_source: KeySource::File,
        }
    }

    pub fn with_key_source(mut self, source: KeySource) -> Self {
        self.key_source = source;
        self
    }
}
```

- [ ] **Step 3: Ajouter test unitaire pour `KeySource::from_env()`**

Avant le module `#[cfg(test)]` existant :

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_source_from_env_default() {
        std::env::remove_var("RR_KEYSTORE");
        assert_eq!(KeySource::from_env(), KeySource::File);
    }

    #[test]
    fn test_key_source_from_env_file() {
        std::env::set_var("RR_KEYSTORE", "file");
        assert_eq!(KeySource::from_env(), KeySource::File);
    }

    #[test]
    fn test_key_source_from_env_keepassxc() {
        std::env::set_var("RR_KEYSTORE", "keepassxc://~/vault.kdbx/Nostr/Identity");
        let parsed = KeySource::from_env();
        assert_eq!(
            parsed,
            KeySource::KeePassXc {
                db_path: "~/vault.kdbx".into(),
                entry: "Nostr/Identity".into(),
            }
        );
        std::env::remove_var("RR_KEYSTORE");
    }

    #[test]
    fn test_key_source_from_env_keepass_rs() {
        std::env::set_var("RR_KEYSTORE", "keepass-rs:///home/user/secrets.kdbx/MyEntry");
        let parsed = KeySource::from_env();
        assert_eq!(
            parsed,
            KeySource::KeePassRs {
                db_path: "/home/user/secrets.kdbx".into(),
                entry: "MyEntry".into(),
            }
        );
        std::env::remove_var("RR_KEYSTORE");
    }

    #[test]
    fn test_key_source_from_env_invalid_fallsback_to_file() {
        std::env::set_var("RR_KEYSTORE", "garbage");
        assert_eq!(KeySource::from_env(), KeySource::File);
        std::env::remove_var("RR_KEYSTORE");
    }
}
```

- [ ] **Step 4: Compiler et tester**

```bash
./scripts/dev.sh cargo test --package rr-core -- identity
```

Expected : tous les tests passent

- [ ] **Step 5: Commit**

```bash
rtk git add crates/rr-core/src/identity.rs
rtk git commit -m "feat: add KeySource enum and with_key_source() to IdentityManager"
```

---

### Task 3: Backend keepassxc-cli dans `IdentityManager::load()`

- [ ] **Step 1: Modifier `load()` pour gérer `KeySource::KeePassXc`**

Dans `crates/rr-core/src/identity.rs`, modifier `load()` :

```rust
pub fn load(&self) -> Result<Identity, Box<dyn std::error::Error>> {
    match &self.key_source {
        KeySource::File => self.load_file(),
        KeySource::KeePassXc { db_path, entry } => {
            let nsec = get_nsec_keepassxc(db_path, entry)?;
            Identity::from_nsec(&nsec)
        }
        KeySource::KeePassRs { db_path, entry } => {
            let nsec = get_nsec_keepassrs(db_path, entry)?;
            Identity::from_nsec(&nsec)
        }
    }
}

fn load_file(&self) -> Result<Identity, Box<dyn std::error::Error>> {
    let key_path = self.data_dir.join("keys.json");
    let content = std::fs::read_to_string(&key_path)?;
    let data: serde_json::Value = serde_json::from_str(&content)?;
    let nsec = data["nsec"].as_str().ok_or("missing nsec field")?;
    Identity::from_nsec(nsec)
}

fn get_nsec_keepassxc(db_path: &str, entry: &str) -> Result<String, Box<dyn std::error::Error>> {
    let expanded_path = shellexpand::tilde(db_path).to_string();
    let output = Command::new("keepassxc-cli")
        .args(["show", "--quiet", "-s", "-a", "Password", &expanded_path, entry])
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .output()?;

    if !output.status.success() {
        return Err("keepassxc-cli failed: check master password and entry path".into());
    }

    let nsec = String::from_utf8(output.stdout)?
        .trim()
        .to_string();

    if nsec.is_empty() {
        return Err("keepassxc-cli returned empty password".into());
    }

    Ok(nsec)
}
```

- [ ] **Step 2: Vérifier la compilation**

```bash
./scripts/dev.sh cargo check --package rr-core
```

Expected : success

- [ ] **Step 3: Commit**

```bash
rtk git add crates/rr-core/src/identity.rs
rtk git commit -m "feat: add keepassxc-cli backend in IdentityManager::load()"
```

---

### Task 4: Backend keepass-rs dans identity.rs

- [ ] **Step 1: Ajouter le backend Rust**

Dans `crates/rr-core/src/identity.rs`, ajouter après `get_nsec_keepassxc` :

```rust
fn get_nsec_keepassrs(db_path: &str, entry: &str) -> Result<String, Box<dyn std::error::Error>> {
    let expanded_path = shellexpand::tilde(db_path).to_string();
    let mut file = std::fs::File::open(&expanded_path)?;
    let password = rpassword::prompt_password("KeePass master password: ")?;
    let key = keepass::DatabaseKey::new().with_password(&password);
    let database = keepass::Database::open(&mut file, key)?;

    for node in &database.root {
        if let keepass::db::NodeRef::Entry(e) = node {
            let title = e.get_title().unwrap_or("");
            // Allow entry matching by full path (e.g. "Nostr/Identity") or just title
            if title == entry || entry.ends_with(&format!("/{}", title)) {
                if let Some(pwd) = e.get_password() {
                    return Ok(pwd.to_string());
                }
            }
        }
    }

    Err(format!("Entry '{}' not found in KeePass database", entry).into())
}
```

- [ ] **Step 2: Ajouter `shellexpand` au workspace + rr-core**

Modifier `Cargo.toml` :

```toml
[workspace.dependencies]
shellexpand = "3"
```

Modifier `crates/rr-core/Cargo.toml` :

```toml
shellexpand.workspace = true
```

- [ ] **Step 3: Compiler**

```bash
./scripts/dev.sh cargo check --package rr-core
```

Expected : success

- [ ] **Step 4: Commit**

```bash
rtk git add Cargo.toml crates/rr-core/Cargo.toml crates/rr-core/src/identity.rs
rtk git commit -m "feat: add keepass-rs crate backend in IdentityManager::load()"
```

---

### Task 5: Parse `RR_KEYSTORE` dans rr-cli et propager aux commandes

- [ ] **Step 1: Modifier `data_dir()` pour qu'elle retourne aussi le key source**

Dans `crates/rr-cli/src/main.rs`, avant `struct Cli` :

```rust
fn data_dir() -> PathBuf {
    std::env::var("RR_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| rr_core::identity::IdentityManager::default_data_dir())
}

fn key_source() -> rr_core::identity::KeySource {
    rr_core::identity::KeySource::from_env()
}
```

- [ ] **Step 2: Remplacer `IdentityManager::new(data_dir())` par `IdentityManager::new(data_dir()).with_key_source(key_source())`** dans les 4 endroits

Premier : dans `cmd_init` (l. ~73)

```rust
async fn cmd_init() {
    let identity = Identity::new();
    let manager = rr_core::identity::IdentityManager::new(data_dir())
        .with_key_source(key_source());
    // ... reste inchangé
}
```

Deuxième : dans `cmd_identity` (l. ~109)

```rust
async fn cmd_identity() {
    let manager = rr_core::identity::IdentityManager::new(data_dir())
        .with_key_source(key_source());
    // ... reste inchangé
}
```

Troisième : dans `cmd_send` (l. ~190)

```rust
async fn cmd_send(contact: &str, message: &str) {
    let manager = rr_core::identity::IdentityManager::new(data_dir())
        .with_key_source(key_source());
    // ... reste inchangé
}
```

Quatrième : dans `cmd_restore` (l. ~363)

```rust
async fn cmd_restore(phrase: &str) {
    let manager = rr_core::identity::IdentityManager::new(data_dir())
        .with_key_source(key_source());
    // ... reste inchangé
}
```

- [ ] **Step 3: Compiler**

```bash
./scripts/dev.sh cargo check --package rr-cli
```

Expected : success

- [ ] **Step 4: Commit**

```bash
rtk git add crates/rr-cli/src/main.rs
rtk git commit -m "feat: propagate RR_KEYSTORE to IdentityManager in all CLI commands"
```

---

### Task 6: Tests et vérification

- [ ] **Step 1: Lancer toute la suite de tests**

```bash
./scripts/dev.sh cargo test --workspace --exclude rr-tauri --locked
```

Expected : 29 tests + 4 nouveaux tests = 33 pass

- [ ] **Step 2: Vérifier la rétro-compat (sans RR_KEYSTORE)**

```bash
./scripts/dev.sh env RR_DATA_DIR=/tmp/rr-test-keepass cargo run --package rr-cli -- init
./scripts/dev.sh env RR_DATA_DIR=/tmp/rr-test-keepass cargo run --package rr-cli -- identity
```

Expected : fonctionne comme avant (fichier JSON)

- [ ] **Step 3: Vérifier `RR_KEYSTORE=file` explicite**

```bash
./scripts/dev.sh env RR_KEYSTORE=file RR_DATA_DIR=/tmp/rr-test-file cargo run --package rr-cli -- init
./scripts/dev.sh env RR_KEYSTORE=file RR_DATA_DIR=/tmp/rr-test-file cargo run --package rr-cli -- identity
```

Expected : idem, inchangé

- [ ] **Step 4: Lint final**

```bash
./scripts/dev.sh cargo clippy --workspace --exclude rr-tauri -- -D warnings
```

Expected : 0 warnings

- [ ] **Step 5: Commit final**

```bash
rtk git add -A && rtk git commit -m "test: add key_source tests, verify retro-compat"
```

---

## Auto-review

- [x] Spec coverage : Toutes les stories de la spec sont couvertes (KeySource enum, 2 backends, env var parsing, retro-compat)
- [x] Placeholders : Aucun TBD/TODO — chaque étape a du code concret
- [x] Type consistency : `KeySource::from_env()` → `with_key_source()` → `load()` — types cohérents dans toutes les tâches
- [x] Gaps : Aucun — la spec ne demande pas de keyring OS, NIP-46, ou agent (YAGNI)
