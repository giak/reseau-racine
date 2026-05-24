# EPIC 7 — KeePassXC Vault Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Distribution binaire + vault KeePassXC. L'utilisateur installe `rr` en 1 commande, reçoit un warning si clés en clair, peut basculer vers KeePassXC.

**Architecture (3 phases) :**
1. CI release (cross-compile) + `cargo publish` + avertissement dans `rr init`
2. Config file `~/.config/reseau-racine/config.toml` + `KeySource::from_config()`
3. `KeySource` enum + backends keepassxc-cli/keepass-rs + `rr init --kdbx` + `rr export`

**Tech Stack:** Rust, CI: taiki-e/upload-rust-binary-action, `keepass` crate, `rpassword`, `shellexpand`, `serde` (config)

---

## File Map

| Fichier | Changement |
|---------|-----------|
| `.github/workflows/ci.yml` | Ajouter job `release` (cross-compile + upload) |
| `Cargo.toml` (workspace) | Ajouter `keepass`, `rpassword`, `shellexpand` en workspace deps |
| `crates/rr-core/Cargo.toml` | Ajouter `keepass`, `rpassword`, `shellexpand` aux deps |
| `crates/rr-core/src/config.rs` | Nouveau — struct Config, KeystoreConfig, load/save, config_dir() |
| `crates/rr-core/src/identity.rs` | Ajouter `KeySource` enum, `from_config()`, `with_key_source()`, 2 backends, `detect_keepassxc_cli()`, `save_to_keepassxc()` |
| `crates/rr-cli/src/main.rs` | Parse `RR_KEYSTORE`, gère `--kdbx`, warning init, `rr export` |
| `docs/TRACKING.md` | Marquer EPIC 7 stories ✅ |

---

## Phase 1 — Distribution & avertissement

### Task 1: CI release (cross-compile)

- [ ] **Step 1: Ajouter job `release` à `ci.yml`**

Dans `.github/workflows/ci.yml`, après le job `build-cli` :

```yaml
  release:
    name: release
    if: startsWith(github.ref, 'refs/tags/v')
    needs: [lint, test, audit, fuzz, udeps, build-cli]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: taiki-e/upload-rust-binary-action@v2
        with:
          bin: rr
          tar: all
          zip: windows
          token: ${{ secrets.GITHUB_TOKEN }}
```

Expected : seulement déclenché sur `git tag v*`

- [ ] **Step 2: Mettre à jour `Cargo.toml` avec metadata cargo publish**

Ajouter à `crates/rr-cli/Cargo.toml` :

```toml
[package]
# ... existing ...
description = "RéseauRacine CLI — messagerie chiffrée P2P sur Nostr"
homepage = "https://github.com/giak/reseau-racine"
repository = "https://github.com/giak/reseau-racine"
```

Et dans le workspace `Cargo.toml` :

```toml
[workspace.package]
# ... existing ...
homepage = "https://github.com/giak/reseau-racine"
repository = "https://github.com/giak/reseau-racine"
```

- [ ] **Step 3: Commit**

```bash
rtk git add .github/workflows/ci.yml Cargo.toml crates/rr-cli/Cargo.toml
rtk git commit -m "feat: add release CI job + cargo publish metadata"
```

---

### Task 2: Warning clés en clair dans `rr init`

- [ ] **Step 1: Modifier `cmd_init` dans `main.rs`**

Dans `crates/rr-cli/src/main.rs`, après `manager.save(&identity)` :

```rust
async fn cmd_init() {
    let identity = Identity::new();
    let manager = rr_core::identity::IdentityManager::new(data_dir());
    if let Err(e) = manager.save(&identity) {
        eprintln!("Erreur: {}", e);
        return;
    }
    println!("✅ Identité créée : {}", identity.public_key_bech32());

    // Vérifier si l'utilisateur utilise un vault
    let keystore = std::env::var("RR_KEYSTORE").unwrap_or_default();
    if keystore.is_empty() || keystore == "file" {
        println!();
        println!("⚠️  Clé stockée en clair dans ~/.local/share/reseau-racine/keys.json");
        println!("⚠️  Pour plus de sécurité, installe KeePassXC et utilise :");
        println!("💡  export RR_KEYSTORE=keepassxc://~/vault.kdbx/Nostr/Identity");
        println!("💡  https://keepassxc.org");
    }
}
```

- [ ] **Step 2: Compiler**

```bash
./scripts/dev.sh cargo check --package rr-cli
```

Expected : success

- [ ] **Step 3: Commit**

```bash
rtk git add crates/rr-cli/src/main.rs
rtk git commit -m "feat: add plaintext key warning in rr init"
```

---

## Phase 2 — Backends KeePassXC

### Task 3: Ajouter les dépendances

- [ ] **Step 1: Ajouter au workspace**

Modifier `Cargo.toml` :

```toml
[workspace.dependencies]
# ... existing ...
keepass = "0.12"
rpassword = "7"
shellexpand = "3"
```

- [ ] **Step 2: Ajouter à rr-core**

Modifier `crates/rr-core/Cargo.toml` :

```toml
[dependencies]
# ... existing ...
keepass.workspace = true
rpassword.workspace = true
shellexpand.workspace = true
```

- [ ] **Step 3: Vérifier la compilation**

```bash
./scripts/dev.sh cargo check --package rr-core
```

Expected : success

- [ ] **Step 4: Commit**

```bash
rtk git add Cargo.toml crates/rr-core/Cargo.toml
rtk git commit -m "deps: add keepass, rpassword, shellexpand"
```

---

### Task 4: Config.toml + `KeySource::from_config()`

- [ ] **Step 1: Définir le format config**

```rust
// crates/rr-core/src/config.rs (nouveau fichier)
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    pub keystore: KeystoreConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum KeystoreConfig {
    #[serde(rename = "file")]
    File,
    #[serde(rename = "keepassxc")]
    KeePassXc {
        db_path: String,
        entry: String,
    },
    #[serde(rename = "keepass-rs")]
    KeePassRs {
        db_path: String,
        entry: String,
    },
}

impl Default for Config {
    fn default() -> Self {
        Self { keystore: KeystoreConfig::File }
    }
}
```

- [ ] **Step 2: Lire/écrire le fichier config.toml**

```rust
impl Config {
    pub fn config_dir() -> PathBuf {
        std::env::var("RR_CONFIG_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("~/.config"));
                base.join("reseau-racine")
            })
    }

    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if !path.exists() {
            return Self::default();
        }
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => return Self::default(),
        };
        toml::from_str(&content).unwrap_or_default()
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let dir = Self::config_dir();
        std::fs::create_dir_all(&dir)?;
        let content = toml::to_string_pretty(self)?;
        std::fs::write(Self::config_path(), content)?;
        Ok(())
    }
}
```

- [ ] **Step 3: Ajouter `mod config;` dans `lib.rs`**

```rust
// crates/rr-core/src/lib.rs
pub mod config;
```

- [ ] **Step 4: `KeySource::from_config()`**

```rust
// identity.rs
impl KeySource {
    pub fn from_config(config: &Config) -> Self {
        match &config.keystore {
            KeystoreConfig::File => Self::File,
            KeystoreConfig::KeePassXc { db_path, entry } =>
                Self::KeePassXc { db_path: db_path.clone(), entry: entry.clone() },
            KeystoreConfig::KeePassRs { db_path, entry } =>
                Self::KeePassRs { db_path: db_path.clone(), entry: entry.clone() },
        }
    }
}
```

Trier la priorité (dans `key_source()` dans rr-cli) : `RR_KEYSTORE` env var > `config.toml` > défaut File.

- [ ] **Step 5: Ajouter `serde` + `toml` aux dépendances**

```toml
# crates/rr-core/Cargo.toml
serde = { version = "1", features = ["derive"] }
toml = "0.8"
```

- [ ] **Step 6: Compiler**

```bash
./scripts/dev.sh cargo check --package rr-core
```

Expected : success

- [ ] **Step 7: Commit**

```bash
rtk git add crates/rr-core/src/config.rs crates/rr-core/src/lib.rs crates/rr-core/Cargo.toml
rtk git commit -m "feat: config.toml + KeySource::from_config()"
```

---

### Task 5: `KeySource` enum + backends dans identity.rs

- [ ] **Step 1: Ajouter les imports et le enum**

Dans `crates/rr-core/src/identity.rs` :

```rust
use std::path::Path;
use std::process::{Command, Stdio};

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
                KeySource::KeePassXc { db_path: db_path.to_string(), entry: entry.to_string() }
            }
            Ok(val) if val.starts_with("keepass-rs://") => {
                let rest = val.trim_start_matches("keepass-rs://");
                let (db_path, entry) = rest.split_once('/').unwrap_or((rest, ""));
                KeySource::KeePassRs { db_path: db_path.to_string(), entry: entry.to_string() }
            }
            _ => KeySource::File,
        }
    }

    pub fn detect_keepassxc_cli() -> bool {
        let (cmd, flag) = if cfg!(target_os = "windows") {
            ("where", "/Q")
        } else {
            ("which", "")
        };
        Command::new(cmd)
            .arg("keepassxc-cli")
            .arg(flag)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}
```

- [ ] **Step 2: Ajouter `key_source` champ et `with_key_source()`**

```rust
pub struct IdentityManager {
    data_dir: PathBuf,
    key_source: KeySource,
}

impl IdentityManager {
    pub fn new(data_dir: impl Into<PathBuf>) -> Self {
        Self { data_dir: data_dir.into(), key_source: KeySource::File }
    }

    pub fn with_key_source(mut self, source: KeySource) -> Self {
        self.key_source = source;
        self
    }
}
```

- [ ] **Step 3: Réécrire `load()` pour router selon `key_source` + ajouter `load_file()`**

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
    let expanded = shellexpand::tilde(db_path).to_string();
    let out = Command::new("keepassxc-cli")
        .args(["show", "--quiet", "-s", "-a", "Password", &expanded, entry])
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .output()?;
    if !out.status.success() {
        return Err("keepassxc-cli failed: check master password and entry path".into());
    }
    let nsec = String::from_utf8(out.stdout)?.trim().to_string();
    if nsec.is_empty() { return Err("keepassxc-cli returned empty password".into()); }
    Ok(nsec)
}

fn get_nsec_keepassrs(db_path: &str, entry: &str) -> Result<String, Box<dyn std::error::Error>> {
    let expanded = shellexpand::tilde(db_path).to_string();
    let mut file = std::fs::File::open(&expanded)?;
    let password = rpassword::prompt_password("KeePass master password: ")?;
    let key = keepass::DatabaseKey::new().with_password(&password);
    let database = keepass::Database::open(&mut file, key)?;
    for node in &database.root {
        if let keepass::db::NodeRef::Entry(e) = node {
            let title = e.get_title().unwrap_or("");
            if title == entry || entry.ends_with(&format!("/{}", title)) {
                if let Some(pwd) = e.get_password() { return Ok(pwd.to_string()); }
            }
        }
    }
    Err(format!("Entry '{}' not found in KeePass database", entry).into())
}
```

- [ ] **Step 4: Ajouter `save_to_keepassxc()`**

```rust
pub fn save_to_keepassxc(&self, identity: &Identity, db_path: &str, entry: &str) -> Result<(), Box<dyn std::error::Error>> {
    let expanded = shellexpand::tilde(db_path).to_string();
    let nsec = identity.secret_key_bech32();
    let npub = identity.public_key_bech32();

    let mut child = Command::new("keepassxc-cli")
        .args(["add", "--non-interactive", "-p", &expanded, entry])
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        writeln!(stdin, "{}\n{}", nsec, npub)?;
    }

    let status = child.wait()?;
    if !status.success() {
        return Err("keepassxc-cli add failed".into());
    }

    Ok(())
}
```

- [ ] **Step 5: Ajouter les tests**

Dans le module `#[cfg(test)]` existant de `identity.rs` :

```rust
#[test]
fn test_key_source_from_env_default() {
    std::env::remove_var("RR_KEYSTORE");
    assert_eq!(KeySource::from_env(), KeySource::File);
}

#[test]
fn test_key_source_from_env_file() {
    std::env::set_var("RR_KEYSTORE", "file");
    assert_eq!(KeySource::from_env(), KeySource::File);
    std::env::remove_var("RR_KEYSTORE");
}

#[test]
fn test_key_source_from_env_keepassxc() {
    std::env::set_var("RR_KEYSTORE", "keepassxc://~/vault.kdbx/Nostr/Identity");
    assert_eq!(
        KeySource::from_env(),
        KeySource::KeePassXc { db_path: "~/vault.kdbx".into(), entry: "Nostr/Identity".into() }
    );
    std::env::remove_var("RR_KEYSTORE");
}

#[test]
fn test_key_source_from_env_invalid_fallsback() {
    std::env::set_var("RR_KEYSTORE", "garbage");
    assert_eq!(KeySource::from_env(), KeySource::File);
    std::env::remove_var("RR_KEYSTORE");
}

#[test]
fn test_detect_keepassxc_cli() {
    // Ne doit pas paniquer, retourne true ou false selon l'environnement
    let _detected = KeySource::detect_keepassxc_cli();
}
```

- [ ] **Step 6: Compiler et tester**

```bash
./scripts/dev.sh cargo test --package rr-core
```

Expected : 30+ tests pass

- [ ] **Step 7: Commit**

```bash
rtk git add crates/rr-core/src/identity.rs
rtk git commit -m "feat: KeySource enum, keepassxc-cli and keepass-rs backends, save_to_keepassxc"
```

---

### Task 6: Propager RR_KEYSTORE dans rr-cli + `rr init --kdbx`

- [ ] **Step 1: Ajouter `key_source()` helper + `--kdbx` flag à `Init`**

Dans `crates/rr-cli/src/main.rs`, avant `struct Cli` :

```rust
fn data_dir() -> PathBuf {
    std::env::var("RR_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| rr_core::identity::IdentityManager::default_data_dir())
}

fn key_source() -> rr_core::identity::KeySource {
    let from_env = rr_core::identity::KeySource::from_env();
    if !matches!(from_env, rr_core::identity::KeySource::File) {
        return from_env;
    }
    // Fallback : config.toml
    let config = rr_core::config::Config::load();
    rr_core::identity::KeySource::from_config(&config)
}
```

Modifier `Commands::Init` :

```rust
#[derive(Subcommand)]
enum Commands {
    /// Initialiser une identité
    Init {
        /// Chemin vers une base KeePassXC (.kdbx)
        #[arg(long)]
        kdbx: Option<String>,
        /// Entrée dans la base KeePassXC (défaut: Nostr/Identity)
        #[arg(long, default_value = "Nostr/Identity")]
        entry: String,
    },
    // ... le reste inchangé
}
```

- [ ] **Step 2: Détection interactive (intégrée dans `cmd_init`)**

La détection interactive est directement intégrée dans `cmd_init` (Step 3). Quand `rr init` est appelé sans `--kdbx` mais que `keepassxc-cli` est détecté dans PATH, l'utilisateur est invité à choisir avant de créer l'identité.

- [ ] **Step 3: Réécrire `cmd_init` avec support `--kdbx` + détection**

```rust
use std::io::Write;

async fn cmd_init(kdbx: &Option<String>, entry: &str) {
    // Détection interactive (si --kdbx non fourni mais keepassxc-cli présent)
    if kdbx.is_none() && KeySource::detect_keepassxc_cli() {
        print!("🔑 KeePassXC détecté. Utiliser pour stocker les clés ? [Y/n] ");
        std::io::stdout().flush().ok();
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).ok();
        if input.trim().is_empty() || input.trim().eq_ignore_ascii_case("y") {
            print!("Chemin DB [~/vault.kdbx] : ");
            std::io::stdout().flush().ok();
            let mut db_path = String::new();
            std::io::stdin().read_line(&mut db_path).ok();
            let db_path = if db_path.trim().is_empty() {
                "~/vault.kdbx".to_string()
            } else {
                db_path.trim().to_string()
            };
            let identity = Identity::new();
            let manager = IdentityManager::new(data_dir()).with_key_source(key_source());
            return cmd_init_kdbx(identity, manager, &db_path, entry).await;
        }
    }

    let identity = Identity::new();
    let manager = rr_core::identity::IdentityManager::new(data_dir())
        .with_key_source(key_source());

    if let Some(db_path) = kdbx {
        cmd_init_kdbx(identity, manager, db_path, entry).await;
    } else {
        cmd_init_file(identity, manager).await;
    }
}

async fn cmd_init_kdbx(identity: Identity, manager: IdentityManager, db_path: &str, entry: &str) {
    // Mode KeePassXC : créer identité + sauver dans la DB
    if let Err(e) = manager.save_to_keepassxc(&identity, db_path, entry) {
        eprintln!("Erreur sauvegarde KeePassXC: {}", e);
        return;
    }
    println!("✅ Identité créée et stockée dans KeePassXC ({}/{})", db_path, entry);
    println!("🔑 Pubkey: {}", identity.public_key_bech32());
    // Sauver config.toml (pas de fallback JSON — ça irait contre le but)
    let config = rr_core::config::Config {
        keystore: rr_core::config::KeystoreConfig::KeePassXc {
            db_path: db_path.to_string(),
            entry: entry.to_string(),
        },
    };
    if let Err(e) = config.save() {
        eprintln!("⚠️  Config non sauvegardée : {}", e);
    } else {
        println!("💡 Configuration sauvegardée dans ~/.config/reseau-racine/config.toml");
    }
}

async fn cmd_init_file(identity: Identity, manager: IdentityManager) {
    // Mode fichier normal
    if let Err(e) = manager.save(&identity) {
        eprintln!("Erreur: {}", e);
        return;
    }
    println!("✅ Identité créée : {}", identity.public_key_bech32());
    let ks = std::env::var("RR_KEYSTORE").unwrap_or_default();
    if ks.is_empty() || ks == "file" {
        println!();
        println!("⚠️  Clé stockée en clair dans ~/.local/share/reseau-racine/keys.json");
        println!("⚠️  Pour plus de sécurité, installe KeePassXC et utilise :");
        println!("💡  rr init --kdbx ~/vault.kdbx");
        println!("💡  https://keepassxc.org");
    }
}
```

- [ ] **Step 3: Ajouter `--kdbx` et `--entry` aux autres commandes**

Même pattern pour `Send`, `Sync`, `Identity` si l'utilisateur veut spécifier le chemins sans RR_KEYSTORE :

```rust
#[derive(Subcommand)]
enum Commands {
    // ...
    Send {
        contact: String,
        message: String,
    },
    // ...
}
```

Pas besoin de `--kdbx` sur chaque commande — `RR_KEYSTORE` ou `config.toml` (future) suffit.

- [ ] **Step 4: Mettre à jour le match dans `main()`**

```rust
match &cli.command {
    Commands::Init { kdbx, entry } => cmd_init(kdbx, entry).await,
    Commands::Identity => cmd_identity().await,
    Commands::Send { contact, message } => cmd_send(contact, message).await,
    // ... le reste inchangé
}
```

- [ ] **Step 5: Mettre à jour `cmd_send` et `cmd_sync` pour utiliser `key_source()`**

```rust
async fn cmd_send(contact: &str, message: &str) {
    let manager = rr_core::identity::IdentityManager::new(data_dir())
        .with_key_source(key_source());
    // ... reste inchangé
}
```

Même changement dans `cmd_identity` et `cmd_sync` (si load est appelé).

- [ ] **Step 6: Compiler**

```bash
./scripts/dev.sh cargo check --package rr-cli
```

Expected : success

- [ ] **Step 7: Commit**

```bash
rtk git add crates/rr-cli/src/main.rs
rtk git commit -m "feat: rr init --kdbx, RR_KEYSTORE propagation to all commands"
```

---

### Task 7: `rr export` — migrer identité existante vers KeePassXC

- [ ] **Step 1: Ajouter la sous-commande `Export`**

```rust
#[derive(Subcommand)]
enum Commands {
    // ... existing ...
    /// Exporter l'identité vers KeePassXC
    Export {
        /// Chemin vers la base KeePassXC
        #[arg(long)]
        kdbx: String,
        /// Entrée dans la base (défaut: Nostr/Identity)
        #[arg(long, default_value = "Nostr/Identity")]
        entry: String,
    },
}
```

- [ ] **Step 2: Ajouter dans le match**

```rust
Commands::Export { kdbx, entry } => cmd_export(kdbx, entry).await,
```

- [ ] **Step 3: Implémenter `cmd_export`**

```rust
async fn cmd_export(kdbx: &str, entry: &str) {
    let manager = rr_core::identity::IdentityManager::new(data_dir())
        .with_key_source(key_source());

    let identity = match manager.load() {
        Ok(id) => id,
        Err(e) => {
            eprintln!("Erreur: identité non trouvée ({})", e);
            eprintln!("Exécutez d'abord 'rr init'");
            return;
        }
    };

    if let Err(e) = manager.save_to_keepassxc(&identity, kdbx, entry) {
        eprintln!("Erreur export KeePassXC: {}", e);
        return;
    }

    println!("✅ Identité exportée vers KeePassXC ({})", kdbx);
    println!("🔑 Entrée: {}", entry);
    println!("🔑 Pubkey: {}", identity.public_key_bech32());
    println!("💡 Utilisez: RR_KEYSTORE=keepassxc://{}/{} pour activer", kdbx, entry);
}
```

- [ ] **Step 4: Compiler**

```bash
./scripts/dev.sh cargo check --package rr-cli
```

Expected : success

- [ ] **Step 5: Commit**

```bash
rtk git add crates/rr-cli/src/main.rs
rtk git commit -m "feat: add rr export command for KeePassXC migration"
```

---

### Task 8: Tests et vérification finale

- [ ] **Step 1: Tous les tests**

```bash
./scripts/dev.sh cargo test --workspace --exclude rr-tauri --locked
```

Expected : 33+ pass (29 existants + 4 nouveaux KeySource)

- [ ] **Step 2: Rétro-compat test**

```bash
./scripts/dev.sh env RR_DATA_DIR=/tmp/rr-oldtest cargo run --package rr-cli -- init
./scripts/dev.sh env RR_DATA_DIR=/tmp/rr-oldtest cargo run --package rr-cli -- identity
```

Expected : exactement comme avant (warning en plus)

- [ ] **Step 3: Lint**

```bash
./scripts/dev.sh cargo clippy --workspace --exclude rr-tauri -- -D warnings
```

Expected : 0 warnings

- [ ] **Step 4: Commit final**

```bash
rtk git add -A && rtk git commit -m "test: full test suite + retro-compat verification"
```

---

## Auto-review

- [x] Spec coverage : distribution (CI release), warning init, Config + from_config, KeySource enum, 2 backends, rr init --kdbx, rr export, tests — tout couvert
- [x] Placeholders : aucun TBD/TODO — code concret partout
- [x] Type consistency : env > config.toml > File — priorité claire
- [x] Gaps corrigés : config.toml déplacé en Phase 2 (spec required), cross-platform detect (which/where), --kdbx sauve config au lieu de JSON fallback
