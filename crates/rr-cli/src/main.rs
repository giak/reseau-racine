use clap::{Parser, Subcommand};
use nostr::nips::nip19::{FromBech32, ToBech32};
use nostr::nips::nip59::UnwrappedGift;
use nostr::{Kind, PublicKey};
use nostr_sdk::{Filter, RelayPoolNotification};
use rr_core::cell::CellStore;
use rr_core::config::Config;
use rr_core::identity::{Identity, IdentityManager, KeySource};
use rr_core::message::MessageService;
use rr_core::transport::nostr::NostrTransport;
use rr_core::CellTransport;
use std::io::{self, Write};
use std::path::PathBuf;
use uuid::Uuid;

fn data_dir() -> PathBuf {
    std::env::var("RR_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| IdentityManager::default_data_dir())
}

fn key_source() -> KeySource {
    let from_env = KeySource::from_env();
    if !matches!(from_env, KeySource::File) {
        return from_env;
    }
    let config = Config::load();
    KeySource::from_config(&config)
}

#[derive(Parser)]
#[command(name = "rr", about = "RéseauRacine CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

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
    /// Afficher l'identité courante
    Identity,
    /// Ajouter un contact
    AddContact { npub: String, name: String },
    /// Lister les contacts
    Contacts,
    /// Envoyer un message
    Send { contact: String, message: String },
    /// Synchroniser les messages
    Sync,
    /// Exporter l'identité vers KeePassXC
    Export {
        /// Chemin vers la base KeePassXC
        #[arg(long)]
        kdbx: String,
        /// Entrée dans la base (défaut: Nostr/Identity)
        #[arg(long, default_value = "Nostr/Identity")]
        entry: String,
    },
    /// Restaurer une identité depuis une seed phrase
    Restore { phrase: String },
    /// Commandes de groupe (cellules)
    #[command(subcommand)]
    Group(GroupCommands),
    /// Exécuter les benchmarks de performance
    Bench {
        #[arg(long)]
        crypto_only: bool,
        #[arg(long)]
        transport_only: bool,
        #[arg(long, default_value = "ws://172.20.0.2:8080")]
        relay: String,
    },
}

#[derive(Subcommand)]
enum GroupCommands {
    /// Créer une cellule
    Create {
        #[arg(long)]
        label: String,
        /// Liste de npubs séparés par des virgules
        #[arg(long, value_delimiter = ',')]
        members: Vec<String>,
    },
    /// Lister les cellules
    List,
    /// Détails d'une cellule
    Info {
        /// Cell ID (UUID)
        cell_id: String,
    },
    /// Inviter un membre dans une cellule
    Invite {
        cell_id: String,
        #[arg(long)]
        member: String,
    },
    /// Envoyer un message dans une cellule
    Send {
        cell_id: String,
        #[arg(long)]
        message: String,
    },
    /// Écouter les messages d'une cellule (ou mode découverte sans argument)
    Listen { cell_id: Option<String> },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init { kdbx, entry } => cmd_init(kdbx, entry).await,
        Commands::Identity => cmd_identity().await,
        Commands::AddContact { npub, name } => cmd_add_contact(npub, name).await,
        Commands::Contacts => cmd_contacts().await,
        Commands::Send { contact, message } => cmd_send(contact, message).await,
        Commands::Sync => cmd_sync().await,
        Commands::Export { kdbx, entry } => cmd_export(kdbx, entry).await,
        Commands::Restore { phrase } => cmd_restore(phrase).await,
        Commands::Group(group_cmd) => match group_cmd {
            GroupCommands::Create { label, members } => cmd_group_create(label, members).await,
            GroupCommands::List => cmd_group_list().await,
            GroupCommands::Info { cell_id } => cmd_group_info(cell_id).await,
            GroupCommands::Invite { cell_id, member } => cmd_group_invite(cell_id, member).await,
            GroupCommands::Send { cell_id, message } => cmd_group_send(cell_id, message).await,
            GroupCommands::Listen { cell_id } => cmd_group_listen(cell_id.as_deref()).await,
        },
        Commands::Bench {
            crypto_only,
            transport_only,
            relay,
        } => cmd_bench(*crypto_only, *transport_only, relay).await,
    }
}

fn check_relay(url: &str) -> bool {
    let host = url.trim_start_matches("ws://").trim_start_matches("wss://");
    let addr = host
        .parse::<std::net::SocketAddr>()
        .unwrap_or(std::net::SocketAddr::new(
            std::net::IpAddr::V4(std::net::Ipv4Addr::new(172, 20, 0, 2)),
            8080,
        ));
    std::net::TcpStream::connect_timeout(&addr, std::time::Duration::from_secs(2)).is_ok()
}

async fn cmd_bench(crypto_only: bool, transport_only: bool, relay: &str) {
    let run_crypto = !transport_only;
    let run_transport = !crypto_only;

    if run_crypto {
        println!("→ Running crypto benchmarks...");
        let status = std::process::Command::new("cargo")
            .args(["bench", "--bench", "crypto"])
            .status();

        match status {
            Ok(s) if s.success() => println!("  ✅ Crypto benchmarks done"),
            Ok(s) => eprintln!("  ⚠️  Crypto benchmarks exited with code: {}", s),
            Err(e) => eprintln!("  ❌ Failed to run cargo bench: {}", e),
        }
    }

    if run_transport {
        println!("→ Checking relay at {}...", relay);
        if !check_relay(relay) {
            println!(
                "  ⚠️  Relay {} unreachable, skipping transport benchmarks",
                relay
            );
            return;
        }

        println!("→ Running transport benchmarks...");
        let status = std::process::Command::new("cargo")
            .args(["bench", "--bench", "transport"])
            .env("RR_RELAY", relay)
            .status();

        match status {
            Ok(s) if s.success() => println!("  ✅ Transport benchmarks done"),
            Ok(s) => eprintln!("  ⚠️  Transport benchmarks exited with code: {}", s),
            Err(e) => eprintln!("  ❌ Failed to run cargo bench: {}", e),
        }
    }
}

async fn cmd_init(kdbx: &Option<String>, entry: &str) {
    if kdbx.is_none() && KeySource::detect_keepassxc_cli() {
        print!("🔑 KeePassXC détecté. Utiliser pour stocker les clés ? [Y/n] ");
        io::stdout().flush().ok();
        let mut input = String::new();
        io::stdin().read_line(&mut input).ok();
        if input.trim().is_empty() || input.trim().eq_ignore_ascii_case("y") {
            print!("Chemin DB [~/vault.kdbx] : ");
            io::stdout().flush().ok();
            let mut db_path = String::new();
            io::stdin().read_line(&mut db_path).ok();
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
    let manager = IdentityManager::new(data_dir()).with_key_source(key_source());

    if let Some(db_path) = kdbx {
        cmd_init_kdbx(identity, manager, db_path, entry).await;
        return;
    }

    // Mode fichier
    let phrase = match Identity::generate_seed_phrase() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Erreur génération seed phrase: {}", e);
            return;
        }
    };

    if let Err(e) = manager.save(&identity) {
        eprintln!("Erreur: {}", e);
        return;
    }
    println!("✅ Identité créée : {}", identity.public_key_bech32());

    print!("⚠️  SEULE sauvegarde. Voir la seed phrase ? (oui/non) : ");
    io::stdout().flush().ok();
    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_ok()
        && matches!(
            input.trim().to_lowercase().as_str(),
            "oui" | "o" | "y" | "yes"
        )
    {
        println!();
        println!("SEED PHRASE (notez ces 12 mots sur papier, pas de fichier numérique) :");
        println!("{}", phrase);
        println!();
    }

    println!("Stockée dans: {:?}", data_dir().join("keys.json"));
    println!();
    println!("⚠️  Clé stockée en clair sur le disque.");
    println!("⚠️  Pour plus de sécurité, installe KeePassXC et utilise :");
    println!("💡  rr init --kdbx ~/vault.kdbx");
    println!("💡  https://keepassxc.org");
}

async fn cmd_init_kdbx(identity: Identity, manager: IdentityManager, db_path: &str, entry: &str) {
    if let Err(e) = manager.save_to_keepassxc(&identity, db_path, entry) {
        eprintln!("Erreur sauvegarde KeePassXC: {}", e);
        return;
    }
    println!(
        "✅ Identité créée et stockée dans KeePassXC ({}/{})",
        db_path, entry
    );
    println!("🔑 Pubkey: {}", identity.public_key_bech32());
    let config = Config {
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

async fn cmd_identity() {
    let manager = IdentityManager::new(data_dir()).with_key_source(key_source());
    match manager.load() {
        Ok(identity) => {
            println!("npub: {}", identity.public_key_bech32());
        }
        Err(e) => {
            eprintln!("Aucune identité trouvée (lancez `rr init`) : {}", e);
        }
    }
}

async fn cmd_add_contact(npub: &str, name: &str) {
    let path = data_dir().join("contacts.json");
    let mut contacts: Vec<serde_json::Value> = if path.exists() {
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Erreur lecture contacts.json: {}", e);
                return;
            }
        };
        match serde_json::from_str(&content) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Erreur: contacts.json corrompu: {}", e);
                return;
            }
        }
    } else {
        vec![]
    };
    contacts.push(serde_json::json!({"npub": npub, "name": name}));
    let json = serde_json::to_string_pretty(&contacts);
    match json {
        Ok(data) => {
            if let Err(e) = std::fs::write(&path, &data) {
                eprintln!("Erreur écriture contacts: {}", e);
                return;
            }
        }
        Err(e) => {
            eprintln!("Erreur sérialisation contacts: {}", e);
            return;
        }
    }
    println!("✅ Contact ajouté : {} ({})", name, npub);
}

async fn cmd_contacts() {
    let path = data_dir().join("contacts.json");
    if path.exists() {
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Erreur lecture contacts.json: {}", e);
                return;
            }
        };
        let contacts: Vec<serde_json::Value> = match serde_json::from_str(&content) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Erreur: contacts.json corrompu: {}", e);
                return;
            }
        };
        if contacts.is_empty() {
            println!("Aucun contact.");
            return;
        }
        for contact in &contacts {
            let name = contact["name"].as_str().unwrap_or("?");
            let npub = contact["npub"].as_str().unwrap_or("?");
            println!("  {} → {}", name, npub);
        }
    } else {
        println!("Aucun contact. Ajoutez-en avec `rr add-contact <npub> <nom>`");
    }
}

async fn cmd_send(contact: &str, message: &str) {
    // Charger l'identité
    let manager = IdentityManager::new(data_dir()).with_key_source(key_source());
    let identity = match manager.load() {
        Ok(id) => id,
        Err(e) => {
            eprintln!("Erreur: aucune identité trouvée (lancez `rr init`) : {}", e);
            return;
        }
    };

    // Résoudre le contact
    let contacts_path = data_dir().join("contacts.json");
    let contacts: Vec<serde_json::Value> = if contacts_path.exists() {
        let content = match std::fs::read_to_string(&contacts_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Erreur lecture contacts.json: {}", e);
                return;
            }
        };
        match serde_json::from_str(&content) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Erreur: contacts.json corrompu: {}", e);
                return;
            }
        }
    } else {
        vec![]
    };
    let receiver_npub = match contacts.iter().find(|c| c["name"] == contact) {
        Some(c) => match c["npub"].as_str() {
            Some(n) => n,
            None => {
                eprintln!(
                    "Erreur: contact '{}' sans npub (contacts.json corrompu)",
                    contact
                );
                return;
            }
        },
        None => {
            eprintln!(
                "Erreur: contact '{}' non trouvé. Ajoutez-le avec `rr add-contact`",
                contact
            );
            return;
        }
    };
    let receiver_pubkey = match PublicKey::from_bech32(receiver_npub) {
        Ok(pk) => pk,
        Err(e) => {
            eprintln!("Erreur: npub invalide pour '{}': {}", contact, e);
            return;
        }
    };

    // Connexion au relais
    let relay = std::env::var("RR_RELAY").unwrap_or_else(|_| "wss://relay.damus.io".to_string());
    let transport = match NostrTransport::with_keys(&relay, identity.keys().clone()).await {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Erreur connexion au relais {}: {}", relay, e);
            return;
        }
    };

    // Envoyer
    let msg_service = MessageService::new();
    match msg_service
        .send(transport.client(), receiver_pubkey, message)
        .await
    {
        Ok(event_id) => {
            println!("✅ Message envoyé à {} sur {}", contact, relay);
            println!("   Event ID: {}", event_id.to_hex());
        }
        Err(e) => {
            eprintln!("Erreur envoi message: {}", e);
        }
    }
}

async fn cmd_sync() {
    let data_dir = data_dir();

    // Charger l'identité
    let manager = IdentityManager::new(&data_dir).with_key_source(key_source());
    let identity = match manager.load() {
        Ok(id) => id,
        Err(e) => {
            eprintln!("Erreur: aucune identité trouvée (lancez `rr init`) : {}", e);
            return;
        }
    };

    // Connexion au relais
    let relay = std::env::var("RR_RELAY").unwrap_or_else(|_| "wss://relay.damus.io".to_string());
    let transport = match NostrTransport::with_keys(&relay, identity.keys().clone()).await {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Erreur connexion au relais {}: {}", relay, e);
            return;
        }
    };

    println!("🔄 Connecté à {}, synchronisation...", relay);
    let client = transport.client().clone();

    // Charger les contacts pour résoudre npub → nom
    let contacts_path = data_dir.join("contacts.json");
    let contacts: Vec<serde_json::Value> = if contacts_path.exists() {
        let content = match std::fs::read_to_string(&contacts_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Erreur lecture contacts.json: {}", e);
                return;
            }
        };
        match serde_json::from_str(&content) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Erreur: contacts.json corrompu: {}", e);
                return;
            }
        }
    } else {
        vec![]
    };

    // S'abonner aux GiftWrap pour notre pubkey
    let subscription = Filter::new()
        .kind(Kind::GiftWrap)
        .pubkey(identity.public_key());

    if let Err(e) = client.subscribe(subscription, None).await {
        eprintln!("Erreur abonnement: {}", e);
        return;
    }

    println!("Appuyez sur Ctrl+C pour arrêter.");

    if let Err(e) = client
        .handle_notifications(|notification| async {
            if let RelayPoolNotification::Event { event, .. } = notification {
                if event.kind == Kind::GiftWrap {
                    match MessageService::new().receive(&client, &event).await {
                        Ok(UnwrappedGift { rumor, sender }) => {
                            if rumor.kind == Kind::PrivateDirectMessage {
                                let sender_npub =
                                    sender.to_bech32().unwrap_or_else(|_| sender.to_string());
                                let sender_name = contacts
                                    .iter()
                                    .find(|c| c["npub"] == sender_npub)
                                    .and_then(|c| c["name"].as_str())
                                    .unwrap_or(&sender_npub);
                                println!("📨 {}: {}", sender_name, rumor.content);
                            }
                        }
                        Err(e) => eprintln!("⚠️  Erreur déchiffrement: {}", e),
                    }
                }
            }
            Ok(false)
        })
        .await
    {
        eprintln!("Erreur notification loop: {}", e);
    }
}

async fn cmd_restore(phrase: &str) {
    match Identity::from_seed_phrase(phrase, "") {
        Ok(identity) => {
            let manager = IdentityManager::new(data_dir()).with_key_source(key_source());
            if let Err(e) = manager.save(&identity) {
                eprintln!("Erreur sauvegarde: {}", e);
                return;
            }
            println!("✅ Identité restaurée : {}", identity.public_key_bech32());
        }
        Err(e) => {
            eprintln!("Erreur: seed phrase invalide : {}", e);
        }
    }
}

async fn cmd_export(kdbx: &str, entry: &str) {
    let manager = IdentityManager::new(data_dir()).with_key_source(key_source());

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
    println!(
        "💡 Utilisez: RR_KEYSTORE=keepassxc://{}/{} pour activer",
        kdbx, entry
    );
}

async fn cmd_group_create(label: &str, members_npub: &[String]) {
    let manager = IdentityManager::new(data_dir()).with_key_source(key_source());
    let identity = match manager.load() {
        Ok(id) => id,
        Err(e) => {
            eprintln!("Erreur: aucune identité trouvée (lancez `rr init`) : {}", e);
            return;
        }
    };

    let mut member_pubkeys = Vec::new();
    for npub in members_npub {
        match PublicKey::from_bech32(npub) {
            Ok(pk) => member_pubkeys.push(pk),
            Err(e) => {
                eprintln!("Erreur: npub invalide '{}': {}", npub, e);
                return;
            }
        }
    }

    if member_pubkeys.is_empty() {
        eprintln!("Erreur: au moins un membre requis");
        return;
    }

    let relay = std::env::var("RR_RELAY").unwrap_or_else(|_| "wss://relay.damus.io".to_string());
    let transport = match NostrTransport::with_keys(&relay, identity.keys().clone()).await {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Erreur connexion relais: {}", e);
            return;
        }
    };

    let cell_transport = CellTransport::new(transport.client().clone(), identity.keys().clone());

    match cell_transport.create_cell(label, &member_pubkeys).await {
        Ok(cell) => {
            println!("✅ Cellule créée : {}", cell.id);
            println!("   Label: {}", cell.label);
            println!("   Membres: {}", cell.members.len());
        }
        Err(e) => eprintln!("Erreur création cellule: {}", e),
    }
}

async fn cmd_group_list() {
    let store = CellStore::load();
    let cells = store.all();
    if cells.is_empty() {
        println!("Aucune cellule. Créez-en une avec `rr group create`");
        return;
    }
    for cell in cells {
        println!(
            "  {} — {} ({} membres)",
            cell.id,
            cell.label,
            cell.members.len()
        );
    }
}

async fn cmd_group_info(cell_id_str: &str) {
    let cell_id = match Uuid::parse_str(cell_id_str) {
        Ok(id) => id,
        Err(e) => {
            eprintln!("Erreur: UUID invalide '{}': {}", cell_id_str, e);
            return;
        }
    };
    let store = CellStore::load();
    match store.find(&cell_id) {
        Some(cell) => {
            println!("  ID: {}", cell.id);
            println!("  Label: {}", cell.label);
            println!("  Membres:");
            for member in &cell.members {
                let label = member.label.as_deref().unwrap_or("?");
                let npub = member
                    .pubkey
                    .to_bech32()
                    .unwrap_or_else(|_| member.pubkey.to_string());
                println!("    * {} ({})", label, npub);
            }
            println!("  Créée le: {}", cell.created_at_secs);
        }
        None => eprintln!("Cellule '{}' introuvable", cell_id_str),
    }
}

async fn cmd_group_invite(cell_id_str: &str, member_npub: &str) {
    let cell_id = match Uuid::parse_str(cell_id_str) {
        Ok(id) => id,
        Err(e) => {
            eprintln!("Erreur: UUID invalide '{}': {}", cell_id_str, e);
            return;
        }
    };
    let member_pk = match PublicKey::from_bech32(member_npub) {
        Ok(pk) => pk,
        Err(e) => {
            eprintln!("Erreur: npub invalide '{}': {}", member_npub, e);
            return;
        }
    };

    let manager = IdentityManager::new(data_dir()).with_key_source(key_source());
    let identity = match manager.load() {
        Ok(id) => id,
        Err(e) => {
            eprintln!("Erreur: identité non trouvée: {}", e);
            return;
        }
    };

    let relay = std::env::var("RR_RELAY").unwrap_or_else(|_| "wss://relay.damus.io".to_string());
    let transport = match NostrTransport::with_keys(&relay, identity.keys().clone()).await {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Erreur connexion relais: {}", e);
            return;
        }
    };

    let cell_transport = CellTransport::new(transport.client().clone(), identity.keys().clone());

    match cell_transport.invite_member(&cell_id, &member_pk).await {
        Ok(()) => println!("✅ Membre invité dans la cellule {}", cell_id),
        Err(e) => eprintln!("Erreur invitation: {}", e),
    }
}

async fn cmd_group_send(cell_id_str: &str, message: &str) {
    let cell_id = match Uuid::parse_str(cell_id_str) {
        Ok(id) => id,
        Err(e) => {
            eprintln!("Erreur: UUID invalide '{}': {}", cell_id_str, e);
            return;
        }
    };

    let manager = IdentityManager::new(data_dir()).with_key_source(key_source());
    let identity = match manager.load() {
        Ok(id) => id,
        Err(e) => {
            eprintln!("Erreur: identité non trouvée: {}", e);
            return;
        }
    };

    let relay = std::env::var("RR_RELAY").unwrap_or_else(|_| "wss://relay.damus.io".to_string());
    let transport = match NostrTransport::with_keys(&relay, identity.keys().clone()).await {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Erreur connexion relais: {}", e);
            return;
        }
    };

    let cell_transport = CellTransport::new(transport.client().clone(), identity.keys().clone());

    match cell_transport.send_message(&cell_id, message).await {
        Ok(()) => println!("✅ Message envoyé dans la cellule {}", cell_id),
        Err(e) => eprintln!("Erreur envoi: {}", e),
    }
}

async fn cmd_group_listen(cell_id_str: Option<&str>) {
    let cell_id = match cell_id_str {
        Some(s) => match Uuid::parse_str(s) {
            Ok(id) => Some(id),
            Err(e) => {
                eprintln!("Erreur: UUID invalide '{}': {}", s, e);
                return;
            }
        },
        None => None,
    };

    let manager = IdentityManager::new(data_dir()).with_key_source(key_source());
    let identity = match manager.load() {
        Ok(id) => id,
        Err(e) => {
            eprintln!("Erreur: identité non trouvée: {}", e);
            return;
        }
    };

    let relay = std::env::var("RR_RELAY").unwrap_or_else(|_| "wss://relay.damus.io".to_string());
    let transport = match NostrTransport::with_keys(&relay, identity.keys().clone()).await {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Erreur connexion relais: {}", e);
            return;
        }
    };

    let cell_transport = CellTransport::new(transport.client().clone(), identity.keys().clone());

    if let Err(e) = cell_transport.listen(cell_id.as_ref()).await {
        eprintln!("Erreur écoute: {}", e);
    }
}
