use clap::{Parser, Subcommand};
use nostr::nips::nip19::{FromBech32, ToBech32};
use nostr::nips::nip59::UnwrappedGift;
use nostr::{Kind, PublicKey};
use nostr_sdk::{Filter, RelayPoolNotification};
use rr_core::identity::Identity;
use rr_core::message::MessageService;
use rr_core::transport::nostr::NostrTransport;
use std::io::{self, Write};
use std::path::PathBuf;

fn data_dir() -> PathBuf {
    std::env::var("RR_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| rr_core::identity::IdentityManager::default_data_dir())
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
    Init,
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
    /// Restaurer une identité depuis une seed phrase
    Restore { phrase: String },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init => cmd_init().await,
        Commands::Identity => cmd_identity().await,
        Commands::AddContact { npub, name } => cmd_add_contact(npub, name).await,
        Commands::Contacts => cmd_contacts().await,
        Commands::Send { contact, message } => cmd_send(contact, message).await,
        Commands::Sync => cmd_sync().await,
        Commands::Restore { phrase } => cmd_restore(phrase).await,
    }
}

async fn cmd_init() {
    let identity = Identity::new();
    let manager = rr_core::identity::IdentityManager::new(data_dir());
    if let Err(e) = manager.save(&identity) {
        eprintln!("Erreur: {}", e);
        return;
    }
    let phrase = match Identity::generate_seed_phrase() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Erreur génération seed phrase: {}", e);
            return;
        }
    };

    println!("✅ Identité créée");
    println!("npub: {}", identity.public_key_bech32());
    println!();

    print!("⚠️  La seed phrase suivante est votre SEULE sauvegarde. Prêt à voir ? (oui/non) : ");
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

    let keystore = std::env::var("RR_KEYSTORE").unwrap_or_default();
    if keystore.is_empty() || keystore == "file" {
        println!();
        println!("⚠️  Clé stockée en clair sur le disque.");
        println!("⚠️  Pour plus de sécurité, installe KeePassXC et utilise :");
        println!("💡  rr init --kdbx ~/vault.kdbx");
        println!("💡  https://keepassxc.org");
    }
}

async fn cmd_identity() {
    let manager = rr_core::identity::IdentityManager::new(data_dir());
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
    let manager = rr_core::identity::IdentityManager::new(data_dir());
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
    let manager = rr_core::identity::IdentityManager::new(&data_dir);
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
            let manager = rr_core::identity::IdentityManager::new(data_dir());
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
