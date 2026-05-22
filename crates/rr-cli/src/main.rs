use clap::{Parser, Subcommand};
use rr_core::identity::Identity;
use std::io::{self, Write};

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
    let data_dir = rr_core::identity::IdentityManager::default_data_dir();
    let manager = rr_core::identity::IdentityManager::new(&data_dir);
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

    println!("Stockée dans: {:?}", data_dir.join("keys.json"));
}

async fn cmd_identity() {
    let data_dir = rr_core::identity::IdentityManager::default_data_dir();
    let manager = rr_core::identity::IdentityManager::new(&data_dir);
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
    let contacts_dir = rr_core::identity::IdentityManager::default_data_dir();
    let path = contacts_dir.join("contacts.json");
    let mut contacts: Vec<serde_json::Value> = if path.exists() {
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or_default()
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
    let contacts_dir = rr_core::identity::IdentityManager::default_data_dir();
    let path = contacts_dir.join("contacts.json");
    if path.exists() {
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        let contacts: Vec<serde_json::Value> = serde_json::from_str(&content).unwrap_or_default();
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

async fn cmd_send(_contact: &str, _message: &str) {
    println!("🔜 Envoi de message (EPIC 1 — à implémenter)");
}

async fn cmd_sync() {
    println!("🔜 Synchronisation (EPIC 1 — à implémenter)");
}

async fn cmd_restore(phrase: &str) {
    match Identity::from_seed_phrase(phrase, "") {
        Ok(identity) => {
            let data_dir = rr_core::identity::IdentityManager::default_data_dir();
            let manager = rr_core::identity::IdentityManager::new(&data_dir);
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
