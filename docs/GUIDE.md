# Guide Utilisateur — Réseau Racine

> Envoyer des messages chiffrés, pair à pair, sans email ni téléphone.

## Qu'est-ce que Réseau Racine ?

Un client de messagerie chiffrée qui utilise Nostr comme réseau de transport. Tes messages sont chiffrés de bout en bout (NIP-44 + NIP-17 GiftWrap), personne ne peut les lire — pas même le relais qui les transmet.

## Prérequis

- **OS :** Linux, macOS ou Windows
- **KeePassXC** (recommandé) → [keepassxc.org/download](https://keepassxc.org/download)
  - Pas obligatoire, mais tes clés seront stockées en clair sur le disque sans lui

## Installation

```bash
# Télécharger le binaire (Linux)
curl -LO https://github.com/giak/reseau-racine/releases/latest/download/rr-x86_64-unknown-linux-gnu.tar.gz
tar xzf rr-x86_64-unknown-linux-gnu.tar.gz
sudo mv rr /usr/local/bin/rr

# Vérifier
rr --version
```

**Autres plateformes :** voir la page [Releases](https://github.com/giak/reseau-racine/releases).

**Alternative (si Rust installé) :** `cargo install reseauracine`

## Premiers pas

### 1. Créer ton identité

```bash
rr init
```

Ça génère une paire de clés unique (ta signature numérique). Personne d'autre ne peut signer à ta place.

Tu vois un message comme :

```
✅ Identité créée : npub1...
⚠️  Clé stockée en clair dans ~/.local/share/reseau-racine/keys.json
💡  Pour plus de sécurité, installe KeePassXC
```

### 2. Voir ta clé publique

```bash
rr identity
```

Ta clé publique (npub) — c'est ton "adresse". Donne-la à tes contacts pour qu'ils puissent t'écrire.

### 3. Ajouter un contact

```bash
rr add-contact alice npub1...
```

Remplace `npub1...` par la clé publique d'Alice.

### 4. Voir tes contacts

```bash
rr contacts
```

## Envoyer un message

```bash
rr send alice "Salut ! Comment ça va ?"
```

Le message est :
1. Chiffré avec la clé publique d'Alice (NIP-44)
2. Emballé dans un GiftWrap (kind 1059)
3. Publié sur le relais Nostr

Si Alice est connectée, elle le reçoit en temps réel.

## Recevoir des messages

```bash
rr sync
```

Laisse tourner. Les messages arrivent en temps réel. Appuie sur `Ctrl+C` pour arrêter.

```
📨 alice: Salut ! Comment ça va ?
📨 bob: Tu viens ce soir ?
```

## Sécuriser tes clés avec KeePassXC

Par défaut, ta clé privée (nsec) est stockée dans un fichier JSON sur ton disque. N'importe quel programme peut la lire.

### Pourquoi KeePassXC ?

C'est un coffre-fort chiffré. Ta clé nsec y est protégée par un mot de passe maître. Même si ton ordinateur est volé, personne ne peut lire tes messages.

### 1. Installer KeePassXC

```bash
# Linux (Ubuntu/Debian)
sudo apt install keepassxc

# Linux (Fedora)
sudo dnf install keepassxc

# macOS
brew install --cask keepassxc

# Windows
winget install -e --id KeePassXCTeam.KeePassXC
```

Tu peux aussi télécharger l'installateur depuis [keepassxc.org/download](https://keepassxc.org/download).

### 2. Créer ta base de données

Lance KeePassXC, clique sur **Create new database** :

1. Choisis un nom (ex: "Vault") et un emplacement (ex: `~/Documents/vault.kdbx`)
2. Définis ton mot de passe maître — une phrase longue et unique que tu peux retenir
3. Termine le wizard

Tu as maintenant un fichier `.kdbx` chiffré. Garde-le précieusement, fais des backups.

### 3. Initialiser ton identité dans KeePassXC

```bash
rr init --kdbx ~/Documents/vault.kdbx --entry Nostr/Identity
```

KeePassXC te demande ton mot de passe maître. `rr` crée une identité fraîche et la stocke dans la case "Nostr/Identity".

Désormais, plus besoin de `RR_KEYSTORE` — la config est sauvegardée automatiquement. `rr send` et `rr sync` utiliseront KeePassXC sans autre intervention.

### Migrer une identité existante vers KeePassXC

```bash
# Tu as déjà fait rr init sans KeePassXC ? Pas grave :
rr export --kdbx ~/Documents/vault.kdbx --entry Nostr/Identity

# Active KeePassXC pour les prochaines commandes :
export RR_KEYSTORE=keepassxc://~/Documents/vault.kdbx/Nostr/Identity

# Ou de façon permanente : crée le fichier de config manuellement
# dans ~/.config/reseau-racine/config.toml
#
# [keystore]
# type = "keepassxc"
# db_path = "~/Documents/vault.kdbx"
# entry = "Nostr/Identity"
```

> ⚠️ Ne relance pas `rr init --kdbx` après `rr export` — ça créerait une **nouvelle** identité différente. Utilise `RR_KEYSTORE` ou la config comme ci-dessus.

## Dépannage

### `keepassxc-cli: command not found`

KeePassXC n'est pas installé ou pas dans le PATH.

**Solution :** installe KeePassXC depuis [keepassxc.org/download](https://keepassxc.org/download), ou continue sans (tes clés seront en clair sur le disque).

### `rr send` reste bloqué ou affiche une erreur

Le relais Nostr est peut-être inaccessible. Essaye un autre relais :

```bash
export RR_RELAY=wss://relay.damus.io
rr send alice "test"
```

### `RR_DATA_DIR` pour les tests

Si tu veux séparer plusieurs identités sur le même ordinateur (tests) :

```bash
export RR_DATA_DIR=/tmp/mon-test
rr init
```
