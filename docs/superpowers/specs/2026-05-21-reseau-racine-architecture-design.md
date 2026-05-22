# Spec — RéseauRacine : Architecture Multi-Transport

> **Un réseau de communication souverain, résilient, et sécurisé.**
> Priorités : coordination sécurisée → communication résiliente → autonomie totale → publication souveraine.
> Menace cible : surveillance active + pression juridique (Viginum, réquisitions, infiltration).
> Déploiement : local d'abord (ville par ville), expansion organique.
> Adhésion : hybride (ouverte pour fonctions basiques, cooptation pour cercles sensibles).

---

## §0 Résumé exécutif

RéseauRacine est un réseau de communication où chaque utilisateur possède son propre nœud. L'identité est une clé cryptographique portable qui fonctionne sur tous les transports simultanément : internet fixe (DSL/fibre, box) ou mobile (4G/5G) via Nostr, Reticulum (WiFi/Ethernet/LoRa), et Meshtastic (LoRa texte seul). Le message est chiffré avec la clé du destinataire, quel que soit le transport. Le transport est un détail — l'identité et le chiffrement sont constants.

Quand un transport tombe, le message en prend un autre compatible. La dégradation est automatique et transparente.

---

## §1 Architecture en 5 couches

### Vue d'ensemble

```
┌─────────────────────────────────────────────────────────┐
│                    COUCHE 0 — IDENTITÉ                   │
│  Clé secp256k1 unique (Nostr)                           │
│  → Identité portable, indépendante du transport         │
└──────────────────────┬──────────────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────────────┐
│         COUCHE 1 — MESSAGERIE E2E (NIP-17)              │
│  NIP-44 V2 (ChaCha20-Poly1305) + NIP-59 (gift wrap)    │
│  → Le message est chiffré AVANT d'entrer dans un transport│
└──────────────────────┬──────────────────────────────────┘
                       │
         ┌─────────────┼─────────────┐
         ▼             ▼             ▼
┌────────────┐ ┌────────────┐ ┌────────────┐
│ COUCHE 2A  │ │ COUCHE 2B  │ │ COUCHE 2C  │
│ Internet   │ │ Reticulum  │ │ Meshtastic │
│ (fixe 4G/5G)│ │            │ │            │
│ Nostr      │ │ WiFi/Eth   │ │ LoRa       │
│ Matrix     │ │ LoRa       │ │ Texte seul │
│            │ │ I2P        │ │ GPS        │
└─────┬──────┘ └─────┬──────┘ └─────┬──────┘
      │              │              │
      ▼              ▼              ▼
┌─────────────────────────────────────────────────────────┐
│              COUCHE 3 — NŒUD LOCAL (3 variantes)         │
│  Consommateur (PC existant)                              │
│  Relais (Raspberry Pi 5 + LoRa)                          │
│  Créateur (Mini PC + PeerTube + Owncast)                 │
│  → Héberge l'identité, le cache, le routing             │
└──────────────────────┬──────────────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────────────┐
│              COUCHE 4 — GOUVERNANCE                      │
│  Cellules de 3 → Essaims de 10 → Collège → RIC          │
│  → Gouvernance distribuée, modération par réputation    │
└─────────────────────────────────────────────────────────┘
```

### Flux d'un message — du texte au transport

```
┌──────────────────────────────────────────────────────────────────┐
│  ALICE veut envoyer "Salut" à BOB                                │
│                                                                  │
│  ÉTAPE 1 — Le message brut                                       │
│  ┌────────────────────────────────────┐                          │
│  │ "Salut"                            │  ← Contenu en clair      │
│  └────────────────┬───────────────────┘                          │
│                   │                                              │
│  ÉTAPE 2 — Chiffrement NIP-44 V2                                │
│  ┌────────────────▼───────────────────┐                          │
│  │ ChaCha20-Poly1305                  │  ← Clé dérivée de        │
│  │ (X25519 ECDH + AEAD)              │     secp256k1 d'Alice    │
│  │                                    │     + secp256k1 de Bob   │
│  │ Résultat: blob chiffré authentifié │                          │
│  └────────────────┬───────────────────┘                          │
│                   │                                              │
│  ÉTAPE 3 — Gift Wrap NIP-59                                     │
│  ┌────────────────▼───────────────────┐                          │
│  │ Enveloppe qui cache l'expéditeur   │  ← Clé éphémère          │
│  │ Le relais ne voit que le dest.     │     ne lie pas à Alice   │
│  └────────────────┬───────────────────┘                          │
│                   │                                              │
│  ÉTAPE 4 — Publication sur relais Nostr                          │
│  ┌────────────────▼───────────────────┐                          │
│  │ WebSocket → wss://relay.damus.io   │  ← Le relais voit :      │
│  │ EVENT {kind:1059, pubkey:ephem,    │     - kind:1059          │
│  │        content:blob, tags:[p:bob]} │     - destinataire: Bob  │
│  └────────────────┬───────────────────┘     - blob chiffré       │
│                   │                        - PAS l'expéditeur     │
│  ÉTAPE 5 — Bob reçoit et unwrap                                │
│  ┌────────────────▼───────────────────┐                          │
│  │ Bob déchiffre avec sa clé privée   │  ← Seul Bob peut ouvrir  │
│  │ Vérifie signature secp256k1        │     (forward secrecy: ❌) │
│  │ Affiche "Salut"                    │     (post-compromise: ❌) │
│  └────────────────────────────────────┘                          │
└──────────────────────────────────────────────────────────────────┘
```

### Dégradation des transports

```
┌────────────────────────────────────────────────────────────────────┐
│  SCÉNARIO : Internet est coupé                                     │
│                                                                    │
│  AVANT (Mode Normal)                                               │
│  ┌──────────┐    Internet     ┌──────────┐                        │
│  │  Alice   │ ──────────────► │   Bob    │  ← Nostr + NIP-17      │
│  │  PC      │  wss://relay    │  PC      │     Vidéos, streams    │
│  └──────────┘                 └──────────┘     Tout fonctionne    │
│                                                                    │
│  APRÈS (Mode Dégradé → Reticulum WiFi)                             │
│  ┌──────────┐    WiFi mesh    ┌──────────┐                        │
│  │  Alice   │ ──────────────► │   Bob    │  ← Reticulum           │
│  │  PC      │  UDP broadcast  │  PC      │     Texte + articles   │
│  └──────────┘                 └──────────┘     Pas de vidéos      │
│                                                                    │
│  APRÈS (Mode Critique → Reticulum LoRa)                            │
│  ┌──────────┐    LoRa 868MHz  ┌──────────┐                        │
│  │  Alice   │ ──────────────► │   Bob    │  ← Reticulum + RNode   │
│  │  Pi+LoRa │  150-500 bps    │  Pi+LoRa │     Texte court seul  │
│  └──────────┘                 └──────────┘     ~140 chars/msg     │
│                                                                    │
│  APRÈS (Mode Extrême → Meshtastic)                                 │
│  ┌──────────┐    LoRa 868MHz  ┌──────────┐                        │
│  │  Alice   │ ──────────────► │   Bob    │  ← Meshtastic          │
│  │  Heltec  │  Texte + GPS    │  Heltec  │     Texte court + GPS │
│  └──────────┘                 └──────────┘     ~200 chars/msg     │
│                                                                    │
│  Le MÊME message chiffré E2E circule sur TOUS les transports.     │
│  Seul le tuyau change. Le contenu reste identique.                │
└────────────────────────────────────────────────────────────────────┘
```

**Principe central** : le message est chiffré au niveau de l'identité (Couche 1). Le transport (Couche 2) n'est qu'un tuyau. Si un tuyau se bouche, le message prend un autre tuyau. L'identité et le chiffrement ne changent jamais.

---

## §2 Les 3 types de nœuds

### Nœud Consommateur

| Élément | Détail |
|---------|--------|
| **Matériel** | PC/Mac existant de l'utilisateur |
| **Logiciel** | Client léger (navigateur ou app native) |
| **Rôle** | Consomme du contenu, participe à la distribution P2P (WebTorrent/IPFS) |
| **Identité** | Clé cryptographique stockée localement |
| **Coût** | 0 € (matériel existant) |
| **Barrière d'entrée** | Aucune — s'installe en 5 minutes |

### Nœud Relais

| Élément | Détail |
|---------|--------|
| **Matériel** | Raspberry Pi 5 (8 Go) + module LoRa RNode + SSD 256 Go |
| **Logiciel** | Reticulum + IPFS pinning + relais Nostr + cache local |
| **Rôle** | Stocke/cache du contenu, sert de relais Reticulum, héberge l'identité |
| **Identité** | Clé cryptographique sur le nœud, toujours actif |
| **Coût** | 150-280 € |
| **Barrière d'entrée** | Moyenne — nécessite configuration initiale |

### Nœud Créateur

| Élément | Détail |
|---------|--------|
| **Matériel** | Mini PC (Intel NUC / Ryzen mini) 16 Go RAM + SSD 1 To + GPU optionnel |
| **Logiciel** | PeerTube + Owncast + Reticulum + IPFS + relais Nostr |
| **Rôle** | Crée du contenu (vidéo, stream, articles), héberge son instance |
| **Identité** | Clé cryptographique + identité publique (journaliste, média) |
| **Coût** | 400-800 € |
| **Barrière d'entrée** | Haute — nécessite compétences techniques |

### Tableau récapitulatif

| Fonctionnalité | Consommateur | Relais | Créateur |
|---------------|:---:|:---:|:---:|
| Lire articles | ✅ | ✅ | ✅ |
| Regarder vidéos | ✅ | ✅ | ✅ |
| Écouter podcasts | ✅ | ✅ | ✅ |
| Messagerie E2E | ✅ | ✅ | ✅ |
| Distribution P2P (WebTorrent/IPFS) | ✅ | ✅ | ✅ |
| Cache local (voisins) | — | ✅ | ✅ |
| Relais Reticulum (WiFi/LoRa) | — | ✅ | ✅ |
| Relais Nostr | — | ✅ | ✅ |
| PeerTube (hébergement vidéo) | — | — | ✅ |
| Stream live (Owncast) | — | — | ✅ |
| Publication Nostr signée | — | — | ✅ |
| Feed RSS | — | — | ✅ |

---

## §3 Couche Identité

### Principe

L'identité est une **paire de clés secp256k1** (clé privée + clé publique), standard Nostr.

- **Clé privée** (`nsec`) : signe les messages, prouve "c'est moi". Ne quitte jamais le nœud.
- **Clé publique** (`npub`) : vérifie les signatures, identifiant unique (hex 64 caractères). Partagée publiquement.

**Propriétés** : portable, indépendante du transport, non-supprimable, vérifiable.

### Format

- **Signature** : secp256k1 (standard Nostr) — écosystème existant, outils matures
- **Chiffrement** : X25519 dérivé de secp256k1 (NIP-44) — ChaCha20-Poly1305 AEAD

> **Note** : Nostr utilise secp256k1 pour les signatures (pas Ed25519). Pour le chiffrement E2E, la clé X25519 est dérivée de la clé secp256k1 via NIP-44. C'est le standard actuel (NIP-17 + NIP-44 + NIP-59), qui remplace NIP-04 (déprécié, AES-CBC non authentifié, CVE-2026-41301).

### Fonctionnement

```
1. L'utilisateur génère sa paire de clés secp256k1 (1 clic)
2. La clé publique (npub, hex dérivé de secp256k1) est son identité
3. Pour envoyer un message à Bob :
   a. Le message est chiffré avec NIP-44 (X25519 dérivé + ChaCha20-Poly1305)
   b. Le message chiffré est signé avec secp256k1 (clé privée d'Alice)
   c. Le paquet est enveloppé (NIP-59 gift wrap) pour cacher les métadonnées
   d. Le gift wrap est envoyé via TOUS les transports compatibles avec le type de contenu
4. Bob reçoit le message (par le premier transport qui arrive)
5. Bob unwrap le gift wrap, vérifie la signature secp256k1 et déchiffre
```

### Gestion de la clé privée

La clé privée (nsec) est le point unique de défaillance du système de confiance — celui qui possède la nsec contrôle l'identité. La stratégie de gestion repose sur 3 principes : **séparation**, **redondance**, **récupération**.

#### Méthodes de stockage

| Méthode | Sécurité | UX | Usage principal |
|---------|----------|----|----------------|
| **Fichier local chiffré** (NIP-49 — AES-256-GCM, clé dérivée d'un mot de passe par argon2) | Moyenne | Simple | Nœud consommateur. La clé est déchiffrée en mémoire au déverrouillage de l'app. |
| **NIP-46 Nostr Connect** (bunker) — la clé reste sur un appareil séparé (téléphone dédié, YubiKey avec OpenPGP, ou nsecBunker auto-hébergé). L'app signe via IPC/RPC sans jamais voir la nsec. | Haute | Moyenne | Nœud consommateur sensible, journaliste. La nsec ne réside pas sur la machine de travail. |
| **Hardware wallet** (Nitrokey 3, YubiKey 5 avec OpenPGP Card, ou SeedSigner pour seed offline). Pas de support natif secp256k1 sur tous les wallets — nécessite passage par OpenPGP→secp256k1 via middleware (signify, ketrew) ou export seed→dérivation NIP-06. | Haute | Complexe | Nœud relais/créateur. La clé ne quitte jamais le hardware sauf pour signer. |
| **Seed phrase** (12 ou 24 mots, NIP-06 — BIP-39 standard, compatible wallets Bitcoin) | Très haute | Complexe | Backup. La seed permet de reconstruire l'identité sur tout client Nostr. |

**NIP-46 (Nostr Connect)** est le mécanisme le plus robuste pour les utilisateurs à risque : l'app RéseauRacine tourne sur le PC de travail, le bunker (contenant la nsec) tourne sur un télécheap Android dédié ou un Raspberry Pi isolé, et communique via NIP-46 sur le même réseau local ou un relais dédié. L'app ne peut pas fuiter la clé même compromise.

NDK (Nostr Development Kit) supporte `NDKNip46Signer` — l'intégration Rust est faisable via la crate `nostr-sdk` qui implémente NIP-46 côté signer et côté client.

#### Sauvegarde et récupération

| Mécanisme | Détail | Niveau de risque |
|-----------|--------|------------------|
| **Seed phrase NIP-06** | 12 ou 24 mots BIP-39, stockée physiquement (acier inoxydable Cryptosteel, gravure). 3 copies : domicile, coffre, personne de confiance. | Perte physique, vol |
| **Multi-sig / Shamir** | Split BIP-39 en 3 parts (Shamir SLIP-39, 2-of-3) : une chez un notaire, une chez un contact de confiance, une en local. Aucune part unique ne suffit. | Complexité, dépendance aux tiers |
| **Social recovery** | 5 gardiens choisis dans l'essaim (contacts vérifiés Niveau 2+). L'utilisateur désigne ses gardiens via un événement Nostr kind 31337 (NIP-62 draft). La récupération active les gardiens qui signent une demande collective. L'app reconstruit l'accès après 3/5 signatures. Pas de stockage de la clé chez les gardiens — juste un quorum de signatures. | Attaque sociale ciblée |

**Recommandation par profil** :
| Profil | Stockage primaire | Backup |
|--------|------------------|--------|
| **Consommateur standard** | NIP-49 (fichier + mot de passe) | Seed phrase papier |
| **Journaliste / source** | NIP-46 bunker (téléphone dédié) | Seed phrase acier + 2 gardiens |
| **Nœud relais** | Hardware wallet (Nitrokey) | SLIP-39 2-of-3 |
| **Nœud créateur** | Hardware wallet + NIP-46 (redondance) | SLIP-39 2-of-3 + 3 gardiens |

### Réputation liée à l'identité

- **Signature vérifiable** : chaque message est signé avec secp256k1 → impossible de falsifier l'auteur
- **Historique portable** : l'historique suit la clé, pas le transport
- **Réputation distribuée** : score basé sur les interactions vérifiables
- **Web of trust** : "Je fais confiance à Alice" → si Alice fait confiance à Bob, je peux faire confiance à Bob

---

## §4 Routage multi-transport

### Principe

Le routeur utilise TOUS les transports disponibles simultanément, avec une logique de priorité et de fallback automatique.

### Logique de décision

| Critère | Poids | Détail |
|---------|-------|--------|
| **Disponibilité** | Prioritaire | Le transport est-il actif ? |
| **Type de contenu** | Élevé | Texte court → tous. Vidéo → internet seul. |
| **Urgence** | Élevé | Message critique → envoyer sur TOUS les transports |
| **Bande passante** | Moyen | Le transport a-t-il assez de débit ? |
| **Latence** | Moyen | Le transport est-il rapide ? |

### Matrice de routage

| Contenu | Mode Normal | Mode Dégradé | Mode Critique | Mode Extrême |
|---------|------------|-------------|--------------|-------------|
| **Message urgent** | Internet + Reticulum + Meshtastic | Reticulum + Meshtastic | Reticulum LoRa | Meshtastic |
| **Message normal** | Internet | Reticulum WiFi | Reticulum LoRa | — |
| **Article** | Internet (Nostr + IPFS) | Reticulum WiFi | Reticulum LoRa (texte) | — |
| **Vidéo** | Internet (PeerTube + WebTorrent) | Cache local WiFi | — | — |
| **Stream live** | Internet (Owncast + WebTorrent) | — | — | — |
| **Podcast** | Internet (RSS + IPFS) | Cache local WiFi | — | — |
| **Fichier** | Internet (IPFS) | Reticulum WiFi | — | — |

### Détection automatique de dégradation

Le routeur surveille chaque transport en continu (ping toutes les 30s). Temps de détection : 30-60 secondes.

### Découverte de pairs (Peer Discovery)

| Transport | Mécanisme | Détail |
|-----------|-----------|--------|
| **Internet** (fixe ou mobile) | Relais Nostr + serveurs Matrix | TCP/IP sur DSL/fibre (box) ou 4G/5G (mobile). Mêmes protocoles (TLS, WebSocket), mêmes menaces. |
| **Reticulum WiFi/Ethernet** | Announce packets (broadcast UDP) | Découverte automatique sur le réseau local |
| **Reticulum LoRa** | Beacon périodique sur fréquence partagée | Chaque nœud diffuse son existence toutes les 60s |
| **Meshtastic** | NodeDB + position GPS partagée | Découverte automatique via le protocole Meshtastic |

**Bootstrap initial** : un nouvel utilisateur configure manuellement l'adresse d'au moins 1 nœud relais connu (QR code, lien, saisie manuelle). Après ça, la découverte est automatique.

```
Internet actif ? → OUI = Mode Normal
    │
    └─ NON → Reticulum WiFi actif ? → OUI = Mode Dégradé
                 │
                 └─ NON → Reticulum LoRa actif ? → OUI = Mode Critique
                              │
                              └─ NON → Meshtastic actif ? → OUI = Mode Extrême
                                           │
                                           └─ NON → Mode Hors-ligne (cache local seul)
```

### Gestion des doublons

Chaque message a un ID unique (hash du contenu + timestamp + signature). Le client deduplique automatiquement — il garde la version la plus complète (ex: internet avec fichier > LoRa texte seul). Si la première réception est une version dégradée, elle est remplacée quand la version complète arrive.

### Synchronisation post-dégradation

Quand internet revient :
1. Le nœud détecte qu'internet est de nouveau actif
2. Il synchronise tous les messages accumulés en mode dégradé/critique
3. Les messages sont publiés sur Nostr/Matrix avec leur timestamp de création original (champ `created_at` dans le header, pas le timestamp de publication). En cas de conflit (message modifié pendant la coupure), la version avec le `created_at` le plus récent prevaut.
4. Les fichiers lourds (vidéos, podcasts) sont uploadés en arrière-plan
5. **Gestion de conflits** : si un message a été modifié ou supprimé pendant la coupure, la version avec le timestamp le plus récent prevaut. Les suppressions sont répliquées (tombstone avec signature).

### Considérations sur les transports

#### Nostr sur WebSocket

Le protocole Nostr (NIP-01) utilise WebSocket comme transport unique. C'est performant pour le volume cible d'un nœud RéseauRacine (10-100 événements/jour) — l'overhead WebSocket se limite à 2-6 octets par frame après le handshake initial (TLS + WS upgrade).

**Points d'attention :**

| Contexte | WebSocket | Mitigation RéseauRacine |
|----------|-----------|------------------------|
| **Desktop (fixe actif)** | ✅ Aucun problème | Connexion persistante à 1+ relay |
| **Desktop veille/reveil** | ⚠️ Déconnexion, reconnexion | `REQ since:` récupère les messages manqués au réveil |
| **Mobile 4G/5G actif** | ✅ Latence <1s | Idem desktop |
| **Mobile en background** | ❌ OS tue la socket | Pas de solution universelle. Sync différée au retour au premier plan via `since:` |
| **Handoff antenne/roaming** | ⚠️ Reconnexion TLS (1-3 RTT) | Le modèle "pull" Nostr rend la coupure transparente |
| **Réseau intermittent** | ⚠️ Keepalive, reconnexions | `tokio-tungstenite` gère la reconnexion avec backoff exponentiel |

**Le modèle "pull" de Nostr est un avantage ici** : contrairement à Signal où la connexion permanente est nécessaire pour recevoir les messages, Nostr stocke les événements sur les relays. Une simple requête `["REQ", "sub", {"since": <dernier_sync>}]` récupère tout le trafic manqué. Pas de file d'attente côté client, pas de push notification nécessaire.

#### Transports alternatifs (non Nostr)

Le protocole Nostr est verrouillé sur WebSocket (NIP-01). Des propositions alternatives existent mais sont immatures en mai 2026 :

| Proposition | Principe | Maturité | Pertinence pour RR |
|------------|----------|----------|--------------------|
| **NIP-200 (NoH)** | HTTP GET/POST au lieu de WS | Proxy `nhttp` existe (Rust). Draft 2024. | Faible — pas de push natif, peu de relais supportent |
| **NIP-95 WebRTC P2P** | Relais seed + Super Peers en WebRTC | Prototype Nexus Relay, 1er transfert P2P réussi avril 2026 | Moyenne — réduit charge relay, mais expose IP |
| **WebTransport (QUIC)** | HTTP/3 + QUIC au lieu de TCP+WS | NIP-100 fermé (2023). `webtransport-go` v0.10.0 existe (Go, Rust en retard) | Future — 0-RTT, handle IP change, mais immature |
| **Iroh HTTP** | QUIC P2P adressé par Ed25519, trouée NAT | Pre-v1.0. Plugin Tauri v2 disponible | Expérimental — concept intéressant (pas de TLS, pas de DNS, pas de relay), mais pas Nostr |

**Décision** : WebSocket est le transport Nostr standard, suffisant pour tous les modes avec connectivité internet. Le vrai gap n'est pas WebSocket vs X — c'est **internet vs pas d'internet**, résolu par Reticulum en Phase 2+. Si un transport Nostr alternatif émerge et devient mature (notamment QUIC/WebTransport), il pourra remplacer WS côté client sans changer l'architecture — le crate `nostr` abstrait le transport.

### Résilience des relais Nostr

#### NIP-66 — Découverte dynamique de relais

NIP-66 (standard Nostr, mai 2025) permet la découverte et l'évaluation de relais via des événements kind 30166 publiés par des *monitors* indépendants (Nostr.watch, Relay.Exchange, etc.). Chaque monitor publie périodiquement des métriques cross-validées.

**Fonctionnement** :

| Étape | Acteur | Action |
|-------|--------|--------|
| 1 | Monitors | Scrutent les relais toutes les 6h (uptime, latence, NIPs supportés, politique) |
| 2 | Monitors | Publient kind 30166 : URL, uptime 24h/7j/30j, latence médiane, NIPs, coût |
| 3 | Clients RR | S'abonnent aux événements kind 30166 pour découvrir les meilleurs relais |
| 4 | Clients RR | Cross-valident entre plusieurs monitors (pas de point de confiance unique) |

**Intégration dans la roadmap** :
- **Phase 0** : liste statique de 3+ relais publics fiables dans la configuration
- **Phase 1+** : souscription NIP-66 en fond, suggestion de relais supplémentaires à l'utilisateur
- **Phase 2+** : sélection automatique basée sur les métriques NIP-66 (latence, uptime, NIPs supportés)

#### Stratégie de redondance — minimum 3 relais

Un nœud RéseauRacine maintient **3 connexions simultanées minimum** à des relais indépendants :

| Scénario | Comportement |
|----------|-------------|
| **Normal** | Publie et s'abonne sur les 3 relais en parallèle |
| **1 relais tombe** | Les 2 autres continuent → pas de perte de message |
| **Tous les relais tombent** | Queue persistante SQLite → re-publication à la reconnexion |
| **Reconnexion** | Immédiate → backoff exponentiel (base 2s, max 60s). Après 5 échecs, bascule vers relais de backup |

**Relais local prioritaire** : chaque nœud relais RR héberge son propre relais Nostr (nostr-rs-relay). Les messages intra-cellule passent d'abord par le relais local avant d'atteindre les relais publics — latence réduite, résilience même si les relais publics sont inaccessibles, bande passante externe économisée.

#### Trust scoring — sélection des relais

Chaque relais reçoit un score pondéré :

| Métrique | Source | Poids |
|----------|--------|-------|
| **Uptime 24h** | Monitors NIP-66 | 30 % |
| **Latence WebSocket** | Mesure locale (ping direct) | 25 % |
| **NIPs supportés** | NIP-66 + requête NIP-11 directe | 20 % |
| **Coût d'accès** | Gratuit > inscription > payant | 15 % |
| **Réciprocité RR** | Relais hébergé par un nœud du même essaim | 10 % |

- Score < 0.3 → relais retiré de la liste active
- Relais d'un nœud RR du même essaim → prioritaire dans le routage local
- La liste est mise à jour toutes les 24h ou sur changement de connectivité

#### Configuration par défaut (Phase 0)

```toml
[relays]
default = [
  "wss://relay.damus.io",
  "wss://nos.lol",
  "wss://relay.snort.social",
]
min_relays = 3
max_relays = 10

[relays.discovery]
nip66_enabled = true
score_threshold = 0.3
```

---

## §5 Messagerie E2E (Coordination sécurisée — Priorité 1)

### Structure du message (NIP-17)

Le message passe par 3 couches (NIP-17) :

```
┌─────────────────────────────────────────────────────────────┐
│  COUCHE 1 — RUMOR (kind 14 — PrivateDirectMessage)          │
│  Le message réel, en clair (sera chiffré + signé)           │
│  { "kind": 14, "content": "...", "tags": [["p", "npub"]] } │
└──────────────────────────┬──────────────────────────────────┘
                           │ signé avec clé éphémère secp256k1
┌──────────────────────────▼──────────────────────────────────┐
│  COUCHE 2 — SEAL (kind 13)                                  │
│  Le rumor signé avec une clé secp256k1 ÉPHÉMÈRE             │
│  Cache l'identité de l'expéditeur aux relais                │
│  { "kind": 13, "pubkey": "<ephem>", "content": "<rumor>" } │
└──────────────────────────┬──────────────────────────────────┘
                           │ chiffré NIP-44 V2 (ChaCha20-Poly1305)
┌──────────────────────────▼──────────────────────────────────┐
│  COUCHE 3 — GIFT WRAP (kind 1059)                           │
│  Le seal chiffré avec la clé publique du destinataire       │
│  Seule métadonnée visible : le destinataire                 │
│  { "kind": 1059, "content": "<chiffré>", "tags": [["p"]] } │
└─────────────────────────────────────────────────────────────┘
```

> **Pourquoi 3 couches** : NIP-04 (kind 4, déprécié) exposait les métadonnées (qui envoie à qui). NIP-17 résout ça avec le gift wrap (NIP-59) qui cache l'expéditeur, et NIP-44 qui remplace AES-CBC par ChaCha20-Poly1305 AEAD.

> **Sécurité et limites** : voir analyse complète en §13.1 (audits), §13.2 (stockage), §13.4 (métadonnées), §13.5 (post-quantique), §13.7 (Double Ratchet). Résumé Phase 0 : NIP-44 (Cure53) — confidentialité ✅, forward secrecy ❌, post-compromise ❌, déni plausible ❌, PQ ❌.

### Types de conversations

| Type | Description | Chiffrement | Usage |
|------|-------------|-------------|-------|
| **1:1** | Conversation privée entre 2 personnes | NIP-44 (X25519 + ChaCha20-Poly1305) | Communication sensible, sources |
| **Cellule (3)** | Groupe fermé de 3 personnes | NIP-44 + clé de groupe X25519 | Coordination d'action directe |
| **Essaim (10)** | Groupe de 10 cellules | NIP-44 + clé de groupe X25519 | Coordination régionale |
| **Broadcast** | Message public signé | Signature secp256k1 seule | Publication, annonces |
| **Canal** | Canal thématique ouvert | Signature secp256k1 | Discussion communautaire |

### Fonctionnalités de sécurité

| Fonctionnalité | Comment | Pourquoi |
|---------------|---------|----------|
| **Forward secrecy** | ✅ Intégrée (nostr-double-ratchet, crate Rust) | Si une clé est compromise, les messages passés restent protégés. Post-compromise recovery aussi. |
| **Suppression sécurisée** | Les messages supprimés sont écrasés (overwrite 3 passes) | Pas de preuve récupérable par saisie de matériel |
| **Auto-destruction** | TTL configurable (1h, 24h, 7j, 30j) | Les messages sensibles expirent automatiquement |
| **Pas de métadonnées** | Pas de "vu à", pas de "en ligne", pas de typing indicator | Réduit la surface d'attaque informationnelle |
| **Vérification d'identité** | Comparaison de fingerprint (QR code ou 12 mots) | Prévention des attaques MITM |
| **Alerte de compromission** | Notification si la signature d'un contact change | Détection d'infiltration |

### Protection contre l'infiltration

| Menace | Contre-mesure |
|--------|--------------|
| **Infiltration Viginum** | Web of trust : chaque membre d'une cellule doit être vérifié en personne ou par parrainage croisé |
| **Faux compte** | Signature secp256k1 obligatoire + vérification de fingerprint |
| **Capture de nœud** | Forward secrecy + auto-destruction + NIP-44 AEAD = les messages capturés sont illisibles et non falsifiables |
| **Analyse de trafic** | Reticulum n'a pas d'adresses source + padding des messages (uniquement sur transport Reticulum). Sur internet (Matrix/Nostr), les métadonnées sont visibles par les serveurs — le contenu reste chiffré E2E mais pas l'anonymat. |
| **Social engineering** | Cellules de 3 : un infiltré ne connaît que 2 personnes, pas l'essaim |
| **Spam** | Rate limiting : Niveau 0 = 10 messages/min, Niveau 1 = 50 messages/min, Niveau 2+ = illimité. Les messages en excès sont rejetés localement. |

### Stockage local

| Élément | Stockage | Chiffré ? |
|---------|----------|-----------|
| Messages actifs | SQLite local | Oui (SQLCipher) |
| Messages expirés | Supprimés (overwrite) | — |
| Clés privées (secp256k1) | Fichier chiffré (NIP-49) ou hardware wallet | Oui (AES-256-GCM) |
| Cache contenu | IPFS local | Non (contenu public) |
| Contacts | SQLite local | Oui |

### Protocole de groupe

1. **Création** : le créateur génère une clé de groupe X25519 (dérivée de secp256k1)
2. **Invitation** : chaque membre reçoit la clé de groupe chiffrée avec NIP-44 (sa clé publique individuelle)
3. **Communication** : les messages de groupe sont chiffrés avec NIP-44 (clé de groupe X25519 + ChaCha20-Poly1305)
4. **Départ** : si un membre quitte, la clé de groupe est regenerée et redistribuée (post-compromise security)
5. **Exclusion** : un membre exclu ne peut plus déchiffrer les nouveaux messages

---

## §6 Publication de contenu

### Articles

| Élément | Détail |
|---------|--------|
| **Format** | Markdown + signature Nostr (nip-01) |
| **Distribution** | Nostr relays + IPFS (CID) |
| **Vérification** | Signature secp256k1 = l'auteur est vérifiable cryptographiquement |
| **Résilience** | Disponible sur TOUS les modes (même LoRa pour les articles courts) |
| **Censure** | Impossible à supprimer (IPFS + Nostr distribué) |

### Vidéos

| Élément | Détail |
|---------|--------|
| **Encodage** | Pré-transcodage local (avant upload) — H.264/H.265, multiples qualités |
| **Hébergement** | PeerTube local sur le nœud créateur |
| **Distribution** | WebTorrent (P2P entre viewers) + IPFS (mirror) |
| **Fédération** | ActivityPub (les instances PeerTube se synchronisent) |
| **Résilience** | Mode normal uniquement. Mode dégradé = cache local WiFi |

**Note** : un Raspberry Pi 5 ne peut pas transcoder en temps réel. L'auteur transcode sur son PC **avant** d'uploader. Le Pi ne sert que les fichiers déjà encodés.

### Streams live

| Élément | Détail |
|---------|--------|
| **Technologie** | Owncast (self-hosted, RTMP in → HLS out) |
| **Distribution** | WebTorrent P2P (les viewers partagent les segments) |
| **Matériel** | Mini PC (pas un Pi) — 8 Go RAM minimum |
| **Résilience** | Mode normal uniquement. Pas de fallback. |

**Limitation** : un stream live nécessite internet. Si internet tombe, le stream s'arrête. Le contenu peut être enregistré et redistribué en VOD après coup.

### Podcasts audio

| Élément | Détail |
|---------|--------|
| **Format** | MP3/Opus + feed RSS |
| **Distribution** | RSS + IPFS |
| **Résilience** | Mode normal + mode dégradé (cache local WiFi) |
| **Taille** | 30-100 Mo par épisode — transférable via Reticulum WiFi en quelques minutes |

### Tableau récapitulatif par mode

| Contenu | Normal (Internet) | Dégradé (Reticulum WiFi) | Critique (LoRa) | Extrême (Meshtastic) |
|---------|------------------|-------------------------|-----------------|---------------------|
| Articles | ✅ Nostr + IPFS | ✅ Cache local | ✅ Texte court | ✅ Texte court |
| Vidéos | ✅ PeerTube + WebTorrent | ⚠️ Cache local (pré-chargé) | ❌ | ❌ |
| Streams | ✅ Owncast + WebTorrent | ❌ | ❌ | ❌ |
| Podcasts | ✅ RSS + IPFS | ⚠️ Cache local | ❌ | ❌ |
| Messagerie | ✅ E2E multi-transport | ✅ E2E Reticulum | ✅ Texte seul | ✅ Texte seul |

---

## §7 Gouvernance et modération

### Structure

| Niveau | Structure | Rôle | Taille |
|--------|-----------|------|--------|
| **Cellule** | 3 personnes | Coordination locale, prise de décision rapide | 3 |
| **Essaim** | 10 cellules | Coordination régionale, arbitrage | 30 |
| **Collège** | 7-15 membres élus | Gouvernance globale, modification de la Charte | 7-15 |
| **Réseau** | Tous les nœuds | Vote sur les modifications majeures (RIC distribué) | Illimité |

### Mécanisme de décision

| Type de décision | Qui décide | Quorum | Majorité |
|-----------------|-----------|--------|----------|
| **Modification mineure** (documentation, paramètres) | Collège | 50 % | 66 % |
| **Modification majeure** (protocole, Charte) | RIC distribué | 30 % des nœuds actifs | 66 % |
| **Exclusion d'un nœud** | Essaim + appel au Collège | — | Unanimité essaim |
| **Budget** | Collège + transparence publique | — | 66 % |

### Modération distribuée

| Mécanisme | Comment |
|-----------|---------|
| **Signature obligatoire** | Chaque message est signé → pas d'anonymat total pour les messages publics |
| **Signalement** | Tout utilisateur peut signaler un contenu. Vérifié par 3 pairs aléatoires (cellule) |
| **Décision** | Si 2/3 confirment, le contenu est marqué (pas supprimé) — réduit en visibilité |
| **Appel** | Jugé par un essaim de 10 pairs |
| **Transparence** | Toutes les décisions de modération sont publiques (sauf données personnelles) |

### Web of Trust pour l'adhésion

```
Niveau 0 — Inscrit ouvert
  │
  ├─ Peut lire les contenus publics
  ├─ Peut envoyer des messages 1:1
  ├─ Peut publier des messages broadcast (signés, publics)
  ├─ Ne peut PAS rejoindre de cellules
  ├─ Ne peut PAS créer de canaux
  └─ Identité: clé cryptographique seule
  │
  ▼
Niveau 1 — Vérifié (1 parrain)
  │
  ├─ Peut rejoindre des cellules (sur invitation)
  ├─ Peut participer aux canaux
  ├─ Peut créer des canaux
  └─ Identité: parrainé par un membre Niveau 2+
  │
  ▼
Niveau 2 — Fiable (2 parrains croisés)
  │
  ├─ Peut créer des cellules
  ├─ Peut héberger un nœud relais
  └─ Identité: vérifié par 2 membres indépendants
  │
  ▼
Niveau 3 — Collège (élection)
  │
  ├─ Peut participer à la gouvernance
  ├─ Peut modifier la Charte (via RIC)
  └─ Identité: élu par le réseau
```

### Protection contre la capture

| Risque | Contre-mesure |
|--------|--------------|
| **Capture financière** | Plafond 5 % par contributeur au budget |
| **Capture par infiltration** | Cellules de 3 : un infiltré ne connaît que 2 personnes |
| **Capture par fork** | Forks autorisés (structure gazeuse) |
| **Capture juridique** | Multi-juridictions (France + Suisse + Islande) |
| **Capture technique** | Code open-source, auditabilité totale |

---

## §8 Analyse de faisabilité

### Faisabilité par couche

| Couche | Faisabilité | Risque | Détail |
|--------|------------|--------|--------|
| **Identité (Nostr secp256k1)** | ✅ Haute | Faible | Nostr existe, secp256k1 standard. Client à construire. |
| **Messagerie E2E** | ✅ Haute | Faible | NIP-17 + NIP-44 + NIP-59 = standard actuel. Crate `nostr` v0.44.2 supporte nativement. |
| **Routage multi-transport** | ⚠️ Moyenne | Moyen | Reticulum gère le multi-transport nativement. Bridge internet à construire. |
| **Nœud Consommateur** | ✅ Haute | Faible | App légère + WebTorrent + IPFS desktop. Existe déjà. |
| **Nœud Relais** | ✅ Haute | Faible | Reticulum + IPFS + Nostr relay sur Pi 5. Faisable. |
| **Nœud Créateur** | ⚠️ Moyenne | Moyen | PeerTube + Owncast sur mini PC. Faisable mais configuration nécessaire. |
| **Publication vidéo P2P** | ✅ Haute | Faible | PeerTube utilise déjà WebTorrent. |
| **Stream live P2P** | ⚠️ Moyenne | Moyen | Owncast + WebTorrent = faisable mais latence 10-30s. |
| **Dégradation auto** | ⚠️ Moyenne | Moyen | Détection facile. Bascule transparente = plus complexe. |
| **Gouvernance distribuée** | ✅ Haute | Faible | Loomio/CryptPad existent. RIC distribué = vote sur Nostr. |

### Alternatives et remplacements dégradés

| Composant | Primaire | Alternative | Remplacement dégradé |
|-----------|----------|-------------|---------------------|
| **Identité** | Nostr (nsec/npub) | PGP | Seed phrase 12 mots (backup) |
| **Messagerie 1:1** | NIP-17 (NIP-44 + NIP-59) | Signal | Reticulum Sideband |
| **Messagerie groupe** | NIP-44 + clé de groupe X25519 | Matrix | Reticulum Sideband |
| **Articles** | Nostr + IPFS | WriteFreely | Reticulum NomadNet |
| **Vidéos** | PeerTube + WebTorrent | IPFS seul | Cache local WiFi |
| **Streams** | Owncast + WebTorrent | PeerTube live | Enregistré → VOD |
| **Podcasts** | RSS + IPFS | RSS seul | Cache local |
| **Transport internet** (fixe + 4G/5G) | Matrix + Nostr | Email chiffré | — |
| **Transport local** | Reticulum | Yggdrasil | WiFi ad-hoc |
| **Transport off-grid** | Reticulum LoRa | Meshtastic | — |
| **Transport extrême** | Meshtastic | Packet radio (AX.25) | FM (illégal) |

### Coûts réels

| Type | Matériel | Coût | Coût annuel |
|------|----------|------|------------|
| **Consommateur** | PC existant | 0 € | 0 € |
| **Relais** | Pi 5 + LoRa + SSD | 150-280 € | 50-150 € (électricité) |
| **Créateur** | Mini PC 16 Go + SSD 1 To | 400-800 € | 100-300 € (électricité + bande passante) |

**Budget Phase 1 (10 relais + 3 créateurs + 100 consommateurs)** :
- Matériel : 10 × 150-280 € + 3 × 400-800 € = **2 700-5 200 €**
- Annuel : 10 × 50-150 € + 3 × 100-300 € = **800-2 400 €/an**

---

## §9 Spécifications techniques du client

### Architecture du client

```
┌─────────────────────────────────────────────────────┐
│                    CLIENT UNIFIÉ                     │
│                                                     │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐ │
│  │   UI Layer  │  │  Core Logic │  │  Transport  │ │
│  │  (React/    │  │  (Rust)     │  │  Adapters   │ │
│  │   Tauri)    │  │             │  │             │ │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘ │
│         │                │                │         │
│  ┌──────▼────────────────▼────────────────▼──────┐ │
│  │              Identity Manager                  │ │
│  │  - Key generation/storage                      │ │
│  │  - Sign/verify                                 │ │
│  │  - Encrypt/decrypt                             │ │
│  └──────────────────────┬────────────────────────┘ │
│                         │                           │
│  ┌──────────────────────▼────────────────────────┐ │
│  │              Message Router                    │ │
│  │  - Multi-transport dispatch                    │ │
│  │  - Deduplication                               │ │
│  │  - Priority/urgency handling                   │ │
│  │  - Degradation detection                       │ │
│  └───────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────┘
```

### Technologies proposées

| Composant | Technologie | Pourquoi |
|-----------|------------|----------|
| **UI** | Tauri v2 + React + TypeScript | Léger (600 Ko-15 Mo), cross-platform, WebView native du système |
| **État & cache** | TanStack Query (React) | Cache async des appels Tauri invoke(), stale-while-revalidate, mutations, loading states. Évite de réécrire la gestion des données Rust→UI |
| **Core** | Rust (workspace Cargo, resolver = "2") | Performance, sécurité mémoire, pure Rust crypto (pas de FFI C) |
| **Crypto** | Crate `nostr` v0.44.2 (secp256k1 + NIP-44 ChaCha20-Poly1305 + NIP-59) | Support natif NIP-17, crates pures Rust auditables |
| **Crypto (custom)** | `x25519-dalek` + `chacha20poly1305` | Si besoin de crypto hors Nostr (Reticulum, groupes custom) |
| **Double Ratchet** | Crate `nostr-double-ratchet` (Phase 1+ conditionnel) | Forward secrecy + post-compromise. Non intégré en Phase 0 (crate non auditée, API instable, NIP en draft). Migration transparente depuis NIP-44. |
| **Storage** | SQLx + SQLite | Chiffré au repos, léger, compile-time query validation |
| **P2P** | libp2p (IPFS) + WebTorrent | Mature, écosystème large |
| **Reticulum** | pythonreticulum (subprocess) ou reticulum-rs | Stack réseau off-grid |
| **Nostr** | Crate `nostr` v0.44.2 + tokio-tungstenite | Écosystème existant, async WebSocket |

### Architecture Tauri v2 — Standard 2026

```
crates/rr-tauri/
├── Cargo.toml              # [lib] name = "rr_tauri_lib"
├── tauri.conf.json         # Config Tauri v2 (identifier, bundle, updater)
├── build.rs                # fn main() { tauri_build::build() }
├── capabilities/
│   └── default.json        # Permissions: quelles commandes le frontend peut appeler
├── icons/                  # Générées par `tauri icon`
└── src/
    ├── main.rs             # Minimal: fn main() { rr_tauri_lib::run() }
    ├── lib.rs              # Entry point principal + mobile_entry_point
    ├── commands.rs         # #[tauri::command] functions
    ├── state.rs            # State management (Mutex/RwLock)
    └── error.rs            # Custom error types
```

> **Note** : Tauri v2 utilise `lib.rs` comme entry point principal. `main.rs` appelle juste `app_lib::run()`. Les commandes sont définies dans `commands.rs` et exposées via `#[tauri::command]`. Les permissions sont dans `capabilities/default.json` (remplace l'ancien allowlist).

### Interfaces

| Interface | Description |
|-----------|-------------|
| `IdentityManager` | Génère, stocke, utilise les clés. Signe, vérifie, chiffre, déchiffre. |
| `MessageRouter` | Reçoit un message chiffré, choisit le(s) transport(s), gère la déduplication. |
| `TransportAdapter` | Interface commune pour chaque transport (send, receive, status, bandwidth). |
| `ContentCache` | Gère le cache local (articles, vidéos, podcasts). TTL, eviction, pinning. |
| `GroupManager` | Crée, gère les clés de groupe. Invitation, exclusion, rotation de clé. |

---

## §10 Risques et mitigations

Vue synthétique — chaque risque est détaillé dans la section référencée.

| Risque | Probabilité | Impact | Réf. |
|--------|------------|--------|------|
| **Infiltration Viginum** | Haute | Élevé | §3 WoT + §7 cellules + §13.8 |
| **Saisie de clé privée (pas de FS en Phase 0)** | Faible | Critique | §13.2 stockage + §13.7 DR conditionnel |
| **Dépendance `nostr-double-ratchet` (Phase 1+)** | Haute | Critique | §13.7 — audit, bus factor, fallback |
| **UX trop complexe** | Haute | Critique | §12 — Tauri, 1 clic |
| **Adoption insuffisante** | Haute | Critique | §11 — cellules militantes d'abord |
| **Bridge internet ↔ Reticulum** | Moyenne | Élevé | §11 Phase 2+ |
| **Reticulum abandonné** | Moyenne | Élevé | Leviculum (Rust, AGPL) disponible |
| **Réglementation financement** | Haute | Moyen | §11 Phase 3+ |
| **Coût > budget** | Moyenne | Moyen | §11 budget détaillé |
| **Complexité inter-couches (Phase 2+)** | Moyenne | Élevé | §4 backfill + §13.7 |

---

## §11 Feuille de route

### Phase 0 (0-1 semaine) — EPIC 0 : Infrastructure & Setup

- [ ] Structure Cargo workspace (rr-core, rr-cli, rr-tauri)
- [ ] DevContainer + Docker Compose (nostr-relay, IPFS)
- [ ] CI/CD GitHub Actions (lint + test + build)
- [ ] rr-core: crypto (NIP-44, secp256k1) + identity + message (NIP-17)
- [ ] rr-core: transport Nostr (WebSocket)
- [ ] rr-cli: CLI complet (init, add-contact, send, sync)
- [ ] Tests + docs + README

**Budget** : 0 € (développement)
**Critère de succès** : `git clone` + `cargo build` + `cargo test` fonctionne, CI green

### Phase 1 (1-2 semaines) — EPIC 1 : POC "Premier Message Chiffré"

- [ ] Client Tauri + Rust core (identité Nostr secp256k1)
- [ ] Messagerie 1:1 E2E sur internet (NIP-17 + NIP-44 + NIP-59)
- [ ] Cellules de 3 (création, invitation, clé de groupe)
- [ ] Web of trust Niveau 0-1 (inscription ouverte + parrainage)
- [ ] 10 testeurs (cellules militantes)
- [ ] (Optionnel) **Double Ratchet** : intégration conditionnelle — si audit passé ou merge dans `nostr` officiel, ajouter forward secrecy + post-compromise + sender-keys

**Budget** : 280 € (1 nœud relais de test)
**Critère de succès** : 3 cellules communiquent en E2E sur internet via NIP-17 + NIP-44 + NIP-59 (ChaCha20-Poly1305, gift wrap), avec auto-destruction (TTL NIP-40). Sans forward secrecy en Phase 1 — l'architecture permet une migration transparente vers Double Ratchet en Phase 2+ sans changer le transport ni le format événement.

### Phase 2 (3-6 mois) — Nœuds Relais

- [ ] Package nœud relais (Raspberry Pi 5 + Reticulum + IPFS + Nostr relay)
- [ ] Cache local pour articles et podcasts
- [ ] Détection automatique de dégradation internet (fixe + 4G/5G)
- [ ] Bascule automatique vers Reticulum WiFi
- [ ] 10 nœuds relais déployés

**Budget** : 1 500-2 800 € (10 relais × 150-280 €)
**Critère de succès** : quand internet (fixe ET 4G/5G) est coupé, les messages texte continuent via Reticulum WiFi.

### Phase 3 (6-12 mois) — Publication

- [ ] Nœud créateur (PeerTube + Owncast sur mini PC)
- [ ] Pré-transcodage vidéo + distribution WebTorrent
- [ ] Feed RSS + IPFS pour podcasts
- [ ] Articles Nostr + IPFS avec signature vérifiable
- [ ] 3 nœuds créateurs déployés

**Budget** : 1 200-2 400 € (3 créateurs × 400-800 €)
**Critère de succès** : 1 créateur publie une vidéo + un article + un podcast, distribués P2P.

### Phase 4 (12-18 mois) — Résilience off-grid

- [ ] Reticulum LoRa (RNode) sur les nœuds relais
- [ ] Meshtastic en fallback extrême
- [ ] Synchronisation post-dégradation automatique
- [ ] 100 consommateurs + 10 relais + 3 créateurs

**Budget** : 800-2 400 €/an (électricité + maintenance)
**Critère de succès** : test de résilience — couper internet, les messages texte continuent via LoRa.

---

## §12 Spécifications non-fonctionnelles

| Exigence | Valeur |
|----------|--------|
| **Taille du client** | < 15 Mo (Tauri) |
| **Mémoire RAM** | < 200 Mo (consommateur), < 1 Go (relais), < 4 Go (créateur) |
| **CPU** | < 5 % idle (consommateur), < 20 % idle (relais) |
| **Démarrage** | < 3 secondes |
| **Latence messagerie** | < 1s (internet fixe), < 2s (4G/5G), < 5s (Reticulum WiFi), < 30s (LoRa) |
| **Disponibilité nœud relais** | > 99 % (24/7) |
| **Chiffrement** | NIP-44 V2 (ChaCha20-Poly1305 AEAD) | Audité Cure53. Forward secrecy : ❌ Phase 0 (NIP-44 seul). Phase 1+ conditionnel (Double Ratchet). |
| **Stockage local** | SQLite chiffré (SQLCipher) |
| **Compatibilité** | Windows 10+, macOS 12+, Linux (Debian 12+, Ubuntu 22.04+, Arch) |
| **Mobile** | Phase 5 (après validation desktop) |

---

## §12b Stratégie de packaging — Docker vs Natif

### Principe fondamental

| Type d'utilisateur | Packaging | Pourquoi |
|---|---|---|
| **Nœud Consommateur** (PC de l'utilisateur) | **Tauri natif** (.exe/.dmg/.deb) | Double-clic, < 15 Mo, pas de Docker à installer |
| **Nœud Relais** (Pi 5) | **Docker Compose** | Infrastructure = Docker, c'est le standard. `docker compose up -d` |
| **Nœud Créateur** (mini PC) | **Docker Compose** | PeerTube + Owncast + IPFS = containers |
| **Développeur** | **DevContainer** (optionnel) | Environnement reproductible via VS Code |
| **POC / CLI** | **Binaire Rust** (`cargo install`) | Pas besoin de container pour du CLI |

### Pourquoi PAS Docker pour les consommateurs

Un novice Windows veut : télécharger un `.exe` → double-cliquer → ça marche.

Avec Docker, il doit : Windows Pro → WSL2 → Docker Desktop (2 Go) → redémarrer → comprendre volumes/ports/networks → `docker compose up -d` → debugger le port 8080 occupé → **90% abandonnent à l'étape 3**.

### Architecture de packaging

```
┌─────────────────────────────────────────────────────────────────┐
│                    UTILISATEUR FINAL (Consommateur)              │
│                                                                 │
│  Windows:  reseau-racine-setup.exe        (Tauri + WebView2)   │
│  macOS:    reseau-racine.dmg              (Tauri + WebKit)     │
│  Linux:    reseau-racine.deb / .AppImage  (Tauri + WebKitGTK)  │
│                                                                 │
│  Installation: double-clic, < 15 Mo, < 30s                     │
│  Pas de Docker, pas de Node.js, pas de Rust                    │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                    NŒUD RELAIS (Pi 5)                           │
│                                                                 │
│  curl -sL https://... | bash                                    │
│                                                                 │
│  Installe: Docker + Docker Compose                              │
│  Lance: docker compose up -d                                    │
│  Services: nostr-relay + IPFS + Reticulum + cache               │
│                                                                 │
│  Tout est containerisé, reproductible, mise à jour = pull       │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                    NŒUD CRÉATEUR (mini PC)                      │
│                                                                 │
│  docker compose up -d                                           │
│                                                                 │
│  Services: PeerTube + Owncast + IPFS + nostr-relay              │
│  Configuration: fichiers .env + volumes                        │
│  Monitoring: logs Docker + health checks                        │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                    DÉVELOPPEUR                                   │
│                                                                 │
│  Option A: DevContainer (VS Code)                               │
│    → Ouvrir le repo → "Reopen in Container" → tout est prêt     │
│                                                                 │
│  Option B: Setup local                                          │
│    → curl -sL https://... | bash → installe Rust + deps         │
│    → cargo build → cargo test → cargo run                       │
│                                                                 │
│  CI/CD: GitHub Actions (lint + test + build sur push/PR)        │
└─────────────────────────────────────────────────────────────────┘
```

### Structure du repository (EPIC 0)

```
reseau-racine/
├── Cargo.toml                    # Workspace root (resolver = "2")
├── rust-toolchain.toml           # Rust version pinned
├── .github/workflows/            # CI/CD: ci.yml, release.yml
├── .devcontainer/                # DevContainer: devcontainer.json, Dockerfile, compose.yaml
├── docker/                       # Docker configs pour relais/créateur
│   ├── nostr-relay/
│   └── ipfs/
├── crates/
│   ├── rr-core/                  # Bibliothèque core (crypto, identité, messages)
│   ├── rr-cli/                   # CLI pour le POC (binaire)
│   └── rr-tauri/                 # App Tauri (binaire desktop)
├── ui/                           # Frontend Tauri (React + TypeScript)
├── docs/superpowers/specs/       # Specs et EPICs
├── tests/                        # Tests E2E + integration
└── scripts/                      # Setup, test-e2e, release
```

Voir [EPIC 0 — Infrastructure & Setup](2026-05-21-epic-0-infrastructure-setup.md) pour le détail complet.

---

---

## §13 Sécurité et Robustesse — Audit & Recommandations

### 13.1 Audits existants et historique des vulnérabilités

| Composant | Audit | Trouvailles | Résolution |
|-----------|-------|-------------|------------|
| **NIP-44 V2** | Cure53 (déc. 2023) | NOS-01-006 : pas de forward secrecy (Medium). NOS-01-005 : range checks manquants (Medium). Aucune vulnérabilité critique. | NOS-01-006 : accepté (Phase 0 sans DR). Résolu en Phase 1+ conditionnel via Double Ratchet (§13.7). NOS-01-005 : fixé dans la V2. |
| **NIP-04** (ancêtre) | CVE-2026-41301 | AES-256-CBC non authentifié → oracle padding + falsification de message | **Remplacé** par NIP-17 + NIP-44 + NIP-59. Interdit dans RR. |
| **RNS 1.1.9** (Reticulum) | Rapport de sécurité interne (avril 2026, @ratspeak) | BZ2 decompression bomb OOM — un pair malveillant peut saturer la RAM du nœud | Fixé dans RNS 1.1.9. RR utilise leviculum (Rust, AGPL-3.0) qui n'a pas ce bug. |
| **tauri-plugin-sqlite** | Audit de sécurité (fév. 2026) | Path traversal dans les requêtes, transaction timeout, channel bounds, DB size limits | Fixé avant release. RR utilise le plugin audité. |
| **nostr-double-ratchet** v0.0.141 | Pas d'audit formel (mai 2026, 1 373 téléchargements) | Crate jeune, écosystème small mais actif (mai 2026) | Auditer ou reproduire les tests du spec Double Ratchet avant production. |

### 13.2 Chiffrement au repos — Stockage local

| Élément | Solution actuelle | Analyse |
|---------|------------------|---------|
| **Messages actifs** | SQLite + SQLCipher | SQLCipher utilise AES-256-CBC + HMAC-SHA256 avec une clé dérivée par PBKDF2. Overhead ~5-15 % sur les performances. |
| **Alternative** | SQLite + chiffrement au niveau application (ChaCha20-Poly1305, clé dérivée du mot de passe via argon2id) | Pas d'overhead SQL, pas de dépendance SQLCipher, mais implémentation custom à maintenir. |
| **Recommandation** | **SQLCipher** | Audité, standard, pas de risque d'implémentation. L'overhead est négligeable pour le volume RR (~10-100 événements/jour). |
| **Clés privées** | NIP-49 (AES-256-GCM + argon2) | Standard Nostr. La clé de chiffrement est dérivée du mot de passe utilisateur via argon2id (résistant GPU/ASIC). |

**Danger spécifique RR** : sur un nœud relais (Pi 5, 24/7), la clé de déchiffrement SQLCipher est en mémoire tant que le service tourne. Mitigation :
- Chiffrement au repos protège contre la saisie physique à froid (vol du Pi éteint)
- Ne protège PAS contre la capture à chaud (saisie du Pi allumé) → compléter par auto-destruction TTL des messages (§5), hardware wallet pour la nsec
- Option RAM encryption (dm-crypt + tmpfs) pour les nœuds relais sensibles

### 13.3 Anti-DoS et spam

Le protocole Nostr est intrinsèquement vulnérable au spam : tout relais public accepte des événements de quiconque. RR implémente une défense multi-couche :

#### Rate limiting par niveau d'accès

| Niveau WoT | Messages/min | Événements/jour | Taille max/événement |
|------------|-------------|----------------|---------------------|
| Niveau 0 (ouvert) | 10 | 1 000 | 8 Ko |
| Niveau 1 (vérifié) | 50 | 10 000 | 64 Ko |
| Niveau 2 (fiable) | Illimité contextuel | 100 000 | 512 Ko |
| Niveau 3 (collège) | Illimité | Illimité | 1 Mo |

Les événements en excès sont rejetés localement (pas publiés sur les relais) et l'expéditeur est blacklisté 5 minutes.

#### Filtrage côté relais (nostr-rs-relay)

Le relais local RR est configuré avec :
- **Limite de connexion** : max 50 connexions simultanées (suffisant pour un essaim)
- **Rate limiting IP** : 50 req/min par IP
- **Validation NIP-42** : seuls les clients authentifiés (NIP-42) peuvent publier sur les canaux privés
- **Liste blanche** : les événements de type kind 30166 (NIP-66 monitors) sont priorisés
- **Blacklist** : les relais avec score de confiance < 0.3 (§4) ne sont pas utilisés comme source d'événements

#### Modération NIP-29 pour les groupes

NIP-29 permet aux modérateurs de groupe de :
- Supprimer des messages (kind 9000-9030 : événements de modération)
- Bannir des membres (tag `["-"]` sur le membre)
- Verrouiller un canal (kind 39000 : seule la modération peut publier)

RR étend ce mécanisme : dans une cellule/essaim, le signalement d'un contenu par 2 membres déclenche une réduction de visibilité automatique. La modération est distribuée — pas de pouvoir unilatéral.

#### Coût comme barrière anti-spam

Les relais publics les plus fiables (Damus, Snort, Nostr.watch) imposent un **coût de publication** (payant ou inscription). C'est une barrière efficace contre le spam de masse. RR recommande d'utiliser au moins 1 relais nécessitant une inscription NIP-42 dans la rotation.

### 13.4 Métadonnées et analyse de timing

#### Surface d'attaque métadonnées

| Métadonnée | Visible par | Protection actuelle |
|------------|------------|-------------------|
| **IP de l'expéditeur** | Relais Nostr (TCP/TLS). Reticulum masque l'IP en mode mesh natif. | NIP-59 ne masque pas l'IP. Mitigation : Tor/I2P pour le transport internet, ou Reticulum comme transport par défaut. |
| **IP du destinataire** | Relais Nostr (si Bob s'abonne en direct) | Idem. Bob peut utiliser un relais distant ou Tor. |
| **Destinataire** (pubkey) | Relais Nostr (tag `["p"]` dans le gift wrap) | Visible par le relais. C'est le seul lien entre Alice et Bob. |
| **Taille du message** | Relais Nostr | Padding NIP-59 : limité. Les messages font des tailles distinctes selon le contenu. |
| **Timestamp** | Relais Nostr (champ `created_at`) | Requis par le protocole. Visible. |
| **Fréquence** | Relais Nostr (analyse de flux) | Si Alice et Bob échangent régulièrement, le rythme est visible. |
| **Type d'événement** | Relais Nostr (kind 1059 pour gift wrap) | Uniforme — tous les messages privés sont kind 1059. |

#### Limitations du padding NIP-59

Le gift wrap NIP-59 ajoute un padding ChaCha20-Poly1305 (16 octets de MAC) mais ne standardise pas le padding de taille de message. En pratique :
- Un message "oui" (3 octets) → blob chiffré de ~100 octets (overhead enveloppes + gift wrap)
- Un message de 1 000 octets → blob chiffré de ~1 100 octets
- La taille approximative du message original est **déductible** par un adversaire qui observe les relais

**Mitigation RR** : padding au bloc supérieur (256, 512, 1024, 2048, 4096 octets) — les messages sont arrondis à la taille standardisée la plus proche. Overhead négligeable pour le volume cible. À activer par défaut pour les messages des cellules.

#### Timing side-channel

Le temps de traitement cryptographique (déchiffrement, vérification de signature) est indépendant du contenu pour ChaCha20-Poly1305 et secp256k1 — pas de variable-time operations identifiées. Les implémentations Rust (dalek, chacha20poly1305) sont constant-time.

Le **timing réseau** (intervalle entre publication et subscription) peut révéler :
- Qui est en ligne (réponse rapide = actif)
- Qui lit ses messages (ACK implicite)

**Mitigation RR** : pas de confirmations de lecture, pas d'ACK, pas d'indicateur "en ligne" ou "vu à". Les accusés de réception sont optionnels et chiffrés (rien n'est leaké au transport).

#### Analyse de corrélation

Un adversaire qui observe 3+ relais peut corréler :
- Publication d'un gift wrap kind 1059 par la pubkey éphémère d'Alice
- Subscription de Bob aux événements kind 1059 taggés avec sa pubkey
- → **L'expéditeur n'est pas visible** (clé éphémère), mais le timing et la taille créent une corrélation statistique

**Mitigation RR** :
- Délai aléatoire avant publication des messages (0-10s, jitter gaussien)
- Messages factices périodiques (cover traffic) pour les nœuds relais actifs 24/7
- En phase 2+ : routage via Reticulum cache l'IP et brise la corrélation temporelle

### 13.5 Post-quantum readiness

| Composant | PQ-safe ? | Détail |
|-----------|-----------|--------|
| **ChaCha20-Poly1305** | ✅ Oui | Chiffrement par flux — pas vulnérable à Shor. |
| **HMAC-SHA256 / HKDF** | ✅ Oui | Fonctions de hachage — vulnérables à Grover (réduit de 256 à 128 bits, acceptable). |
| **X25519 ECDH** | ❌ **Non** | Shor casse l'ECDH. Utilisé dans NIP-44 et Double Ratchet. |
| **secp256k1 (signatures)** | ❌ **Non** | Shor casse secp256k1. |
| **Courbes de Reticulum** | ⚠️ Peut-être | Reticulum utilise ECDH Curve25519 (même classe que X25519). |

**Progression** : une proposition hybride ML-KEM (paulmillr, NIP-44 issue #1971, mai 2026) ajouterait un échange de clé post-quantique (FIPS 203 ML-KEM) en parallèle de X25519. Le chiffrement serait X25519 + ML-KEM, protégé contre Shor et contre une éventuelle faiblesse de ML-KEM.

**Stratégie RR** :
- Phase 0-2 : X25519 seul (standards actuels, matures). La menace quantique est spéculative à horizon 5-10 ans.
- Phase 3+ : migration vers hybrid X25519 + ML-KEM dès que NIP-44 intègre la proposition (suivre issue #1971). Les sessions Double Ratchet seront migrées via renégociation.
- **Aucune donnée classifiée** transportée sur RR — le risque PQ est acceptable dans le modèle de menace actuel.

### 13.6 Surface d'attaque — gouvernance

| Vecteur | Scénario | Mitigation |
|---------|----------|------------|
| **Infiltration des validateurs** | Viginum infiltre 2/3 des membres d'une cellule → contrôle les décisions de modération | Web of trust + vérification en personne obligatoire pour Niveau 2+. Taille cellule limitée à 3 (un infiltré ne contrôle pas). |
| **Sybil attack** | Création massive de fausses identités → prise de contrôle du RIC | Quorum 30 % + majorité 66 % + seuil d'ancienneté (30 jours minimum pour voter). |
| **Capture du budget** | Proposition budgétaire massive → détournement | Plafond 5 % par contributeur. Transparence publique obligatoire. Audit des dépenses par le Collège. |
| **Attaque juridique** | Saisie des serveurs / injonction de fermeture | Multi-juridictions (France + Suisse + Islande). Pas de point de contrôle unique. Structure légale : SCOP (France) + association (Suisse). |
| **Fork malveillant** | Capture du repo GitHub → code compromis | Mirror Git auto-hébergé + signatures commit + CI auditée. Le repo principal est en AGPL-3.0 — n'importe qui peut fork. |
| **Déni de service social** | Flood de signalements → submerge la modération | Quorum de 3 pairs aléatoires pour le premier niveau. Si > 100 signalements/jour pour un même utilisateur, escalation automatique vers l'essaim. |

### 13.7 Dépendance critique — `nostr-double-ratchet`

#### État des lieux

| Critère | Constat |
|---------|---------|
| **Auteur** | mmalmi (même personne que le NIP draft #1813) |
| **Versions** | 14 publiées entre le 2 et le 7 mai 2026 (0.0.132 → 0.0.141) |
| **Téléchargements** | 1 373 total (1 132 dans les 90 derniers jours) |
| **Audit** | Aucun |
| **Standardisation** | NIP #1813 ouvert depuis février 2025, en draft, non mergé |
| **Intégration tierce** | Bitchat (PR #1107, 9 514 ajouts, avril 2026), chat.iris.to |
| **Bus factor** | 1 |
| **Dépendances** | `nostr ^0.44.2`, `hkdf ^0.12`, `sha2 ^0.10`, `rand ^0.8`, `serde` — correctes |
| **Tests interop** | TypeScript/Rust (cross-language) |

#### Décision : Phase 1+ conditionnelle, pas Phase 0

Le Double Ratchet est **exclu de la Phase 0 et Phase 1** pour privilégier un socle audité et stable (NIP-44 Cure53). L'architecture maintient la possibilité de l'ajouter plus tard sans rupture : NIP-44 et DR utilisent le même format de transport (NIP-17/NIP-59), les mêmes relais, la même identité. La migration se fait en ajoutant une couche de chiffrement, pas en remplaçant l'existante.

| Mesure | Application | Priorité |
|--------|-------------|----------|
| **Décision fondamentale** | Phase 0-1 : NIP-44 seul. Phase 1+ : DR si conditions remplies. | Maintenant |
| **Conditions d'intégration DR** | (1) Audit de sécurité passé, OU (2) merge dans le crate `nostr` officiel (rust-nostr), OU (3) le NIP #1813 est standardisé. | Phase 1+ |
| **Abstraction trait** | `rr-core::crypto` expose un trait `EncryptionProvider` avec deux implémentations : `Nip44Provider` (Phase 0) et `DoubleRatchetProvider` (Phase 1+). Le message router ne change pas. | Phase 0 |
| **Veille rust-nostr** | Surveiller PR rust-nostr/nostr (#797, fermé, #804, #1107). Si yukibtc ou mmalmi merge le DR dans le crate officiel, réévaluer. | Continu |
| **Audit communautaire** | Financer un audit de la crate si l'adoption dépasse 100 nœuds avant standardisation. Coût estimé : 15 000-30 000 € (Cure53). | Phase 2+ |

### 13.8 Risques opérationnels et humains

Le maillon le plus faible n'est pas technique — il est humain. Les cas documentés en France (2025-2026) montrent une capacité de surveillance active des journalistes et militants par la DGSI : filature, géolocalisation permanente, perquisition, garde à vue. Viginum étend cette surveillance aux manipulations informationnelles. Les cellules RR doivent intégrer ces risques dans leur fonctionnement.

#### Protocole de rencontre pour vérification en personne

**Contexte** : rencontrer physiquement un contact pour vérifier sa clé (fingerprint) crée une corrélation exploitable.

| Mesure | Application |
|--------|-------------|
| **Absence de téléphone** | Aucun appareil connecté pendant la rencontre. Pas de téléphone, pas de smartwatch, pas de laptop. Stockés dans une faraday pouch ou laissés ailleurs. |
| **Transport** | Transport en commun uniquement (pas de véhicule personnel, pas de VTC lié à un compte). Pas de ticket de transport dématérialisé lié à l'identité. |
| **Lieu** | Espace public avec couverture vidéo faible, angle mort identifié à l'avance. Pas de lieu régulier. |
| **Signal discret** | Mot de code convenu à l'avance via un canal différent (Signal, rendez-vous précédent). Si le code n'est pas donné, la rencontre est annulée. |
| **Durée** | 5 minutes maximum. Pas de discussion sensible — juste échange de fingerprints (QR code statique, pas d'appareil). |

#### Compartimentation des identités

Chaque utilisateur RR maintient **au minimum 3 identités séparées** :

| Identité | Usage | Clé | Stockage |
|----------|-------|-----|----------|
| **Personnelle** | Famille, amis, vie quotidienne | Sans lien avec RR | Téléphone perso |
| **Professionnelle** | Travail, Signal, email pro | Sans lien avec RR | Téléphone travail |
| **RéseauRacine** | Cellules, essaim, sources | Seed RR dédiée | Hardware wallet ou téléphone dédié |

**Règle stricte** : aucune corrélation entre les identités. Pas de même email, pas de même téléphone, pas de même navigateur, pas de même compte Google/Apple.

#### Gestion de la contrainte

Si un membre de cellule est arrêté ou perquisitionné :

| Scénario | Réaction |
|----------|----------|
| **Garde à vue + pression pour déverrouiller** | Forward secrecy protège les messages passés. L'auto-destruction TTL protège les messages récents. La nsec sur hardware wallet ne peut pas être extraite sans PIN. |
| **Compromission avérée** | Les autres membres de la cellule la retirent des clés de groupe (sender-key rotation). La cellule change de clé de groupe. L'essaim est notifié discrètement. |
| **Canary** | Chaque membre publie un événement Nostr signé toutes les 24h (kind spécial, contenu codé : "tout va bien", "sous pression", "brûlé"). Si le canary n'est pas publié ou change de code, la cellule déclenche le protocole d'urgence. |
| **Pas de canary disponible** | Si un membre ne répond pas pendant 72h, les autres membres considèrent qu'il peut être compromis et initient la rotation des clés de groupe. |

#### Stress test opérationnel

Avant déploiement, chaque cellule devrait simuler un scénario de compromission :
1. Un membre "est arrêté" — la cellule détecte l'absence de canary
2. Rotation des clés de groupe
3. Vérification que les messages post-rotation sont inaccessibles au membre exclu
4. Réintégration via une nouvelle invitation (si le membre est libéré et déclaré sûr)

### 13.9 Exposition juridique — statut d'hébergeur

#### Applicabilité LCEN / DSA à un relais Nostr

Un nœud relais RR qui héberge un relais Nostr (nostr-rs-relay) est techniquement un **hébergeur** au sens de la LCEN (Loi pour la Confiance dans l'Économie Numérique, 2004) et du DSA (Digital Services Act, UE, applicable depuis février 2024).

| Obligation | Texte | Applicable ? | Réalité pour RR |
|------------|-------|-------------|-----------------|
| Retrait des contenus illicites sous 24-48h | LCEN art. 6 I 5, DSA art. 16 | Oui | **Impossible** : le contenu est chiffré E2E (NIP-44). Le relayeur n'a pas "connaissance effective" du contenu illicite — condition légale pour engager sa responsabilité. |
| Signalement des contenus pédopornographiques | LCEN art. 6-1, obligation stricte | Oui | Même limitation : contenu illisible. La loi ne fait pas d'exception technique — risque théorique mais défense solide. |
| Conservation des logs IP (1 an) | LCEN art. 6 II | Oui | Nostr ne stocke pas les IP des utilisateurs (connexions WebSocket éphémères). Si on ne collecte pas, on ne conserve pas. |
| Transmission sur réquisition judiciaire | LCEN art. 6 II | Oui | Pénalité : €3 750, jusqu'à €15 000 en récidive. Si on n'a pas les logs, techniquement impossible de les transmettre. |
| Surveillance générale des contenus | DSA art. 8 | **Interdite** par le DSA | Aucune obligation de surveillance. Le DSA interdit explicitement l'obligation générale de surveillance. |

#### Protection par le chiffrement

Le point juridique clé : la responsabilité d'un hébergeur suppose une "connaissance effective" du contenu illicite (LCEN art. 6 I 5, DSA art. 16). **Le chiffrement E2E rend cette connaissance impossible**. Un juge ne peut pas vous demander de lire ce que vous ne pouvez pas lire.

**Risque résiduel** : les événements non chiffrés (kind 1, kind 30166, metadata) restent en clair. Si un contenu manifestement illicite (apologie du terrorisme, incitation à la haine raciale, pédopornographie) est publié en clair sur le relais, l'obligation de retrait s'applique. Mitigation : le relais RR n'accepte que les événements chiffrés (kind 1059, 13, 14) et les événements de ses pairs de confiance (authentification NIP-42).

#### Sanctions applicables

| Infraction | Sanction |
|------------|----------|
| Non-retrait de contenu illicite notifié | 1 an prison + €250 000 amende |
| Non-transmission de données sur réquisition | €3 750 (€15 000 récidive) |
| Entrave à la lutte contre le terrorisme/pédopornographie (SREN 2024, ARCOM) | Jusqu'à €250 000 ou % du CA |
| Article 323-3-2 CP (non-respect LCEN/DSA + laisser des contenus manifestement illicites) | 7 ans prison + €500 000. **Pas encore de condamnation** à ce jour. |

#### Recommandations

| Recommandation | Détail |
|----------------|--------|
| **Ne pas conserver les logs IP** | Pas de registre = pas de réquisition possible sur ce point. Les connexions WebSocket sont éphémères par conception. |
| **Héberger le relais hors de France** | Islande, Norvège, Suisse = hors UE (ni LCEN, ni DSA, ni SREN). Un relais RR peut être hébergé chez un pair islandais. |
| **Structure légale** | SCOP (France) pour les actifs français + association en Suisse ou Islande pour les relais. Pas de point de saisie unique. |
| **Transparence E2E** | Documenter publiquement que le relais ne peut pas déchiffrer le contenu. C'est une défense juridique. |
| **Pas de contenu en clair** | Configurer nostr-rs-relay pour n'accepter que les événements chiffrés (kind 1059) et les événements des pairs authentifiés NIP-42. |
| **Avocat spécialisé** | Identifier un avocat spécialisé libertés numériques (Anita, Splunk) AVANT d'avoir besoin d'un avocat. Budget : 500-1 500 € pour une consultation. |

---

## §14 Glossaire / Lexique

Seuls les termes spécifiques à RR ou non évidents dans le corps du document.

| Terme | Définition / Pertinence |
|-------|------------------------|
| **Canary** | Signal périodique signé (§13.8) : si un membre ne publie pas pendant 72h, la cellule initie la rotation des clés. |
| **Cellule / Essaim / Collège / RIC** | Unités de gouvernance (§7) : cellule de 3, essaim de 10 cellules, collège de 7-15 membres, RIC distribué (quorum 30 %, majorité 66 %). |
| **Charte des Racines** | Constitution du réseau — document fondateur définissant principes, règles de gouvernance, mécanismes de modification. |
| **Déni plausible** | Phase 0 : ❌ (NIP-44). Phase 1+ DR : ⚠️ partiel — contenu non signé → déniable, existence de la session prouvable. |
| **Forward secrecy / Post-compromise** | Phase 0 : ❌. Phase 1+ DR : ✅ conditionnel. Propriétés : clé compromise → messages passés (FS) ou futurs (PCS) restent protégés. |
| **Gift Wrap / Seal / Rumor** | NIP-17 (§5) : rumor (kind 14, non signé) → seal (kind 13, signé clé éphémère) → gift wrap (kind 1059, chiffré NIP-44). |
| **La Source** | Portail d'information souverain — agrégateur RSS/Nostr/IPFS, sans tracking ni algorithme. |
| **Modèle de menace** | Surveillance active + pression juridique (Viginum, réquisitions, infiltration). |
| **NomadNet** | Forum/pages browsables sur Reticulum — publication et consultation hors internet. |
| **PeerTube / Owncast** | Plateforme vidéo fédérée (ActivityPub, WebTorrent) et serveur de streaming live. |
| **Polydépendance** | Principe de conception : dépendre de ses pairs, de la matière, du temps — pas des algorithmes. |
| **RNode** | Module LoRa Reticulum — interface physique entre le Pi et le réseau LoRa. |
| **SCOP** | Société Coopérative et Participative — gouvernance démocratique (1 personne = 1 voix). |
| **Sideband** | App de messagerie sur Reticulum — messagerie E2E hors internet. |
| **6 règles du ré-enracinement** | Ne jamais devenir leader, ne rien demander au système, ne pas se faire connaître, ne rien promettre, ne haïr personne, reconnaître les siens sans s'organiser. |
| **Structure gazeuse** | Organisation sans centre, forks autorisés, protocoles mutables. |
| **Surface d'attaque** | Points de compromission. Réduire = moins de fonctionnalités = moins de risques. |
| **Viginum** | Service français (SGDSN, 2021) de lutte contre les manipulations informationnelles. Menace : infiltration, réquisition, surveillance. |
| **Web of Trust** | Confiance par parrainage croisé — vérification en personne, pas d'autorité centrale. |
| **Yggdrasil / I2P** | Overlay IPv6 chiffré P2P (Yggdrasil) ou routage oignon anonyme (I2P) — transports alternatifs Reticulum. |
