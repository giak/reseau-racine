# Spec — RéseauRacine : Architecture Multi-Transport

> **Un réseau de communication souverain, résilient, et sécurisé.**
> Priorités : coordination sécurisée → communication résiliente → autonomie totale → publication souveraine.
> Menace cible : surveillance active + pression juridique (Viginum, réquisitions, infiltration).
> Déploiement : local d'abord (ville par ville), expansion organique.
> Adhésion : hybride (ouverte pour fonctions basiques, cooptation pour cercles sensibles).

---

## §0 Résumé exécutif

RéseauRacine est un réseau de communication où chaque utilisateur possède son propre nœud. L'identité est une clé cryptographique portable qui fonctionne sur tous les transports simultanément : internet (Matrix/Nostr), Reticulum (WiFi/Ethernet/LoRa), et Meshtastic (LoRa texte seul). Le message est chiffré avec la clé du destinataire, quel que soit le transport. Le transport est un détail — l'identité et le chiffrement sont constants.

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

| Méthode | Sécurité | UX | Recommandé pour |
|---------|----------|----|----------------|
| **Fichier local chiffré** (NIP-49) | Moyenne | Simple | Nœud consommateur |
| **Hardware wallet** (Nitrokey, YubiKey) | Haute | Moyenne | Nœud relais/créateur |
| **Seed phrase** (12 mots, NIP-06) | Haute | Complexe | Backup |

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
| **Internet** | Relais Nostr + serveurs Matrix | Les pairs sont découverts via les relais publics ou privés |
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

### Limitations de sécurité NIP-44 — Honnêteté totale

| Propriété | NIP-44 V2 | Impact pour RéseauRacine |
|-----------|-----------|-------------------------|
| **Confidentialité du contenu** | ✅ Oui (ChaCha20-Poly1305 AEAD) | Le contenu est sécurisé |
| **Authentification de l'expéditeur** | ✅ Oui (signature secp256k1 dans le seal) | On peut vérifier qui a envoyé |
| **Intégrité du message** | ✅ Oui (MAC Poly1305) | Le message n'est pas modifiable |
| **Forward secrecy** | ❌ **NON** | Si une clé privée est saisie, **TOUS** les messages passés sont lisibles |
| **Post-compromise security** | ❌ **NON** | Si une clé est compromise, les messages futurs restent lisibles |
| **Déni plausible** | ❌ **NON** | On peut prouver qu'Alice a envoyé le message |
| **Protection post-quantique** | ❌ **NON** | Un ordinateur quantique pourrait déchiffrer |
| **Anonymat IP** | ❌ **NON** | Le relais voit l'IP d'Alice et de Bob |
| **Taille du message** | ⚠️ Partiel (padding) | La taille approximative est visible |

**Audit Cure53 (déc. 2023)** : NIP-44 a été audité par Cure53. Trouvailles principales : NOS-01-006 (lack of forward secrecy, Medium), NOS-01-005 (missing range checks, Medium). Aucune vulnérabilité critique.

**Le spec NIP-44 lui-même dit** : *"For high-risk situations, users should chat in specialized E2EE messaging software and limit use of nostr to exchanging contacts."*

**Pour le produit final** : il faudra ajouter une couche de **Double Ratchet** (comme Signal) au-dessus de NIP-44 pour obtenir la forward secrecy et la post-compromise security. C'est un EPIC séparé après le POC.

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
| **Forward secrecy** | Clés de session éphémères (ratchet) | Si une clé est compromise, les messages passés restent protégés |
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
| **Transport internet** | Matrix + Nostr | Email chiffré | — |
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
| **UI** | Tauri + React | Léger, cross-platform (Windows/macOS/Linux), bundle < 10 Mo |
| **Core** | Rust | Performance, sécurité mémoire, pure Rust crypto (pas de FFI C) |
| **Crypto** | Crate `nostr` v0.44.2 (secp256k1 + NIP-44 ChaCha20-Poly1305 + NIP-59) | Support natif NIP-17, crates pures Rust auditables |
| **Crypto (custom)** | `x25519-dalek` + `chacha20poly1305` + `ed25519-dalek` | Si besoin de crypto hors Nostr (Reticulum, groupes custom) |
| **Storage** | SQLite + SQLCipher | Chiffré au repos, léger |
| **P2P** | libp2p (IPFS) + WebTorrent | Mature, écosystème large |
| **Reticulum** | pythonreticulum (subprocess) ou reticulum-rs | Stack réseau off-grid |
| **Nostr** | Crate `nostr` v0.44.2 + tokio-tungstenite | Écosystème existant, async WebSocket |

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

| Risque | Probabilité | Impact | Mitigation |
|--------|------------|--------|------------|
| **Utilisation de NIP-04 (déprécié) par erreur** | Moyenne | Critique | Utiliser NIP-17 + NIP-44 + NIP-59. Crate `nostr` v0.44.2 supporte nativement. |
| **Bridge internet ↔ Reticulum non fonctionnel** | Moyenne | Élevé | Commencer avec internet seul, ajouter Reticulum en Phase 2 |
| **UX trop complexe pour le grand public** | Haute | Critique | Client Tauri simple, installation en 1 clic, documentation visuelle |
| **Adoption insuffisante** | Haute | Critique | Cibler d'abord les cellules militantes (besoin réel), pas le grand public |
| **Reticulum abandonné (1 dev)** | Moyenne | Élevé | Fork communautaire, implémentations Rust/Go en cours |
| **Infiltration Viginum** | Haute | Élevé | Web of trust + cellules de 3. ⚠️ NIP-44 n'a PAS de forward secrecy — si clé saisie, tout l'historique est lisible. Hardware wallet requis pour nœuds sensibles. |
| **Saisie de clé privée = historique lisible** | Moyenne | Critique | NIP-44 n'a pas de forward secrecy (audit Cure53 NOS-01-006). Mitigation : hardware wallet + auto-destruction TTL + Double Ratchet (EPIC futur) |
| **Coût réel > budget** | Moyenne | Moyen | Budget Phase 1 = 5 200 € (réaliste). Ne pas sur-dimensionner. |
| **Réglementation FM** | Haute | Moyen | FM = Phase 3 uniquement, avec avocat. Pas dans le MVP. |

---

## §11 Feuille de route

### Phase 1 (0-3 mois) — MVP Coordination

- [ ] Client Tauri + Rust core (identité Nostr secp256k1)
- [ ] Messagerie 1:1 E2E sur internet (NIP-17 + NIP-44 + NIP-59)
- [ ] Cellules de 3 (création, invitation, clé de groupe)
- [ ] Web of trust Niveau 0-1 (inscription ouverte + parrainage)
- [ ] 10 testeurs (cellules militantes)

**Budget** : 280 € (1 nœud relais de test)
**Critère de succès** : 3 cellules communiquent en E2E sur internet via NIP-17 (gift wrap + ChaCha20-Poly1305), avec auto-destruction (TTL NIP-40). ⚠️ Forward secrecy : NON disponible en Phase 1 (limitation NIP-44, ajoutée en Phase 2 via Double Ratchet).

### Phase 2 (3-6 mois) — Nœuds Relais

- [ ] Package nœud relais (Raspberry Pi 5 + Reticulum + IPFS + Nostr relay)
- [ ] Cache local pour articles et podcasts
- [ ] Détection automatique de dégradation internet
- [ ] Bascule automatique vers Reticulum WiFi
- [ ] 10 nœuds relais déployés

**Budget** : 1 500-2 800 € (10 relais × 150-280 €)
**Critère de succès** : quand internet est coupé, les messages texte continuent via Reticulum WiFi.

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
| **Latence messagerie** | < 1s (internet), < 5s (Reticulum WiFi), < 30s (LoRa) |
| **Disponibilité nœud relais** | > 99 % (24/7) |
| **Chiffrement** | NIP-44 V2 (ChaCha20-Poly1305 AEAD) | Audité Cure53, pas de forward secrecy (à ajouter plus tard) |
| **Stockage local** | SQLite chiffré (SQLCipher) |
| **Compatibilité** | Windows 10+, macOS 12+, Linux (Debian 12+, Ubuntu 22.04+, Arch) |
| **Mobile** | Phase 5 (après validation desktop) |

---

## §13 Glossaire / Lexique

### Cryptographie

| Terme | Définition | But dans RéseauRacine |
|-------|-----------|----------------------|
| **secp256k1** | Courbe elliptique utilisée par Bitcoin et Nostr pour les signatures numériques | Génère l'identité (nsec/npub), signe les messages, prouve "c'est moi" |
| **X25519** | Algorithme d'échange de clés Diffie-Hellman sur la courbe Curve25519 | Dériver une clé de chiffrement partagée entre expéditeur et destinataire (utilisé dans NIP-44) |
| **ChaCha20** | Chiffrement par flux (stream cipher) rapide et sécurisé | Chiffrer le contenu des messages dans NIP-44 V2 |
| **Poly1305** | Code d'authentification de message (MAC) | Vérifier que le message n'a pas été modifié pendant le transport (partie AEAD de NIP-44) |
| **AEAD** | Authenticated Encryption with Associated Data — chiffrement qui authentifie en même temps | Garantir confidentialité + intégrité en une seule opération (ChaCha20-Poly1305) |
| **ECDH** | Elliptic Curve Diffie-Hellman — protocole d'échange de clés | Deux parties génèrent une clé secrète partagée sans la transmettre |
| **Forward secrecy** | Propriété : si une clé est compromise, les messages PASSÉS restent protégés | ❌ **NON fourni par NIP-44**. À ajouter via Double Ratchet plus tard |
| **Post-compromise security** | Propriété : après une compromission, les messages FUTURS sont protégés | ❌ **NON fourni par NIP-44**. À ajouter via Double Ratchet plus tard |
| **NIP-44 V2** | Standard Nostr de chiffrement : secp256k1 ECDH + HKDF + ChaCha20 + HMAC-SHA256 | Chiffrement E2E des messages privés. Audité Cure53 (déc. 2023) |
| **NIP-59** | Standard Nostr de "gift wrap" — enveloppe qui cache l'expéditeur | Cacher les métadonnées (qui envoie à qui) aux relais Nostr |
| **NIP-17** | Standard Nostr de messages privés combine NIP-44 + NIP-59 | Le protocole de messagerie privée actuel (remplace NIP-04 déprécié) |
| **NIP-04** | Ancien standard de DM chiffré (AES-256-CBC) | ❌ **DÉPRÉCIÉ**. Ne pas utiliser. CVE-2026-41301 |
| **NIP-42** | Authentification des clients auprès des relais | Permettre aux relais de restreindre l'accès aux DMs aux seuls destinataires |
| **NIP-40** | Timestamp d'expiration sur les événements Nostr | Auto-destruction des messages sensibles (TTL) |
| **Rumor** | Message non signé (kind 14) dans le schéma NIP-17 | Le contenu réel du message, qui sera signé avec une clé éphémère |
| **Seal** | Enveloppe signée avec clé éphémère (kind 13) | Cache l'identité de l'expéditeur réel aux relais |
| **Gift Wrap** | Enveloppe chiffrée NIP-44 (kind 1059) | Seule chose visible par le relais : destinataire + blob chiffré |

### Réseau et Transport

| Terme | Définition | But dans RéseauRacine |
|-------|-----------|----------------------|
| **Nostr** | Protocole décentralisé de publication d'événements signés | Transport principal sur internet — relais publics, identité portable |
| **Relais Nostr** | Serveur WebSocket qui stocke et diffuse les événements Nostr | Infrastructure de transport. Ne peut pas lire les messages (chiffrés NIP-44) |
| **Reticulum** | Stack réseau cryptography-first, multi-transport, anonymat natif | Transport local/off-grid — fonctionne sans internet (WiFi, LoRa, Ethernet) |
| **Meshtastic** | Protocole mesh sur radios LoRa (<50€/nœud, texte + GPS) | Transport de dernier recours — quand tout le reste est coupé |
| **LoRa** | Radio longue portée, basse consommation, 868 MHz (bande ISM libre) | Transport physique off-grid — portée km, débit 150-500 bits/s |
| **RNode** | Module LoRa de chez Reticulum pour communiquer sur le réseau Reticulum | Interface physique entre le Pi et le réseau LoRa Reticulum |
| **Yggdrasil** | Overlay IPv6 chiffré end-to-end, peer-to-peer | Transport inter-régional alternatif — NAT traversal, pas besoin de serveurs |
| **I2P** | Réseau anonyme par routage en oignon (comme Tor mais P2P) | Anonymat renforcé optionnel sur Reticulum |
| **Mesh network** | Réseau où chaque nœud relaie les messages des autres | Résilience locale — pas de point de contrôle unique |
| **WebTorrent** | BitTorrent dans le navigateur / P2P via WebRTC | Distribution P2P des vidéos PeerTube — plus il y a de viewers, plus c'est rapide |
| **IPFS** | InterPlanetary File System — stockage distribué par contenu (CID) | Mirror des articles, vidéos, podcasts — impossible à supprimer |

### Nostr — Entités

| Terme | Définition | Exemple |
|-------|-----------|---------|
| **nsec** | Clé privée Nostr (format bech32) | `nsec1...` — NE JAMAIS partager |
| **npub** | Clé publique Nostr (format bech32) | `npub1...` — identité publique |
| **kind** | Type d'événement Nostr (nombre entier) | kind 1 = note, kind 4 = DM (déprécié), kind 14 = private msg, kind 1059 = gift wrap |
| **Event** | Objet JSON signé publié sur les relais Nostr | `{id, pubkey, created_at, kind, tags, content, sig}` |
| **Tag** | Métadonnée structurée dans un event | `["p", "npub_dest"]` = destinataire, `["e", "event_id"]` = réponse |

### Infrastructure et Nœuds

| Terme | Définition | But dans RéseauRacine |
|-------|-----------|----------------------|
| **Nœud Consommateur** | PC/Mac existant de l'utilisateur | Consomme du contenu, participe à la distribution P2P. Coût : 0 € |
| **Nœud Relais** | Raspberry Pi 5 + LoRa RNode + SSD | Stocke/cache du contenu, sert de relais Reticulum, héberge l'identité 24/7. Coût : 150-280 € |
| **Nœud Créateur** | Mini PC 16 Go + SSD 1 To + GPU optionnel | PeerTube + Owncast + publication de contenu. Coût : 400-800 € |
| **Cellule** | Groupe fermé de 3 personnes | Unité de base de la gouvernance et de la coordination |
| **Essaim** | Coordination de 10 cellules (30 personnes) | Niveau régional de gouvernance |
| **Collège** | 7-15 membres élus pour 18 mois non renouvelables | Gouvernance globale du réseau |
| **RIC distribué** | Mécanisme de vote sur le réseau pour modifier la Charte | Quorum 30 %, majorité 66 % |

### Applications et Services

| Terme | Définition | But dans RéseauRacine |
|-------|-----------|----------------------|
| **PeerTube** | Plateforme vidéo fédérée (ActivityPub), P2P via WebTorrent | Hébergement et distribution de vidéos souveraines |
| **Owncast** | Serveur de streaming live self-hosted | Streams live avec distribution P2P WebTorrent |
| **Sideband** | Application de messagerie sur le réseau Reticulum | Messagerie E2E quand internet est coupé |
| **NomadNet** | Forum/pages browsables sur Reticulum | Publication et consultation hors internet |
| **La Source** | Portail d'information souverain (agrégateur RSS/Nostr/IPFS) | Alternative à Google News — sans tracking, sans algorithme |
| **SCOP** | Société Coopérative et Participative (structure légale française) | Gouvernance démocratique du réseau — 1 personne = 1 voix |

### Concepts de sécurité

| Terme | Définition | Pertinence |
|-------|-----------|-----------|
| **E2E (End-to-End)** | Chiffrement de l'expéditeur au destinataire seul — aucun intermédiaire ne peut lire | Fondamental — le contenu est chiffré AVANT d'entrer dans un transport |
| **Métadonnées** | Données sur le message (qui, quand, à qui) — pas le contenu | Le plus grand risque : NIP-44 protège le contenu mais NIP-59 est nécessaire pour cacher les métadonnées |
| **Surface d'attaque** | Ensemble des points par lesquels un adversaire peut compromettre le système | Réduire = moins de fonctionnalités = moins de risques |
| **Menace** | Adversaire spécifique avec des capacités spécifiques | Notre menace : surveillance active + pression juridique (Viginum, réquisitions, infiltration) |
| **Modèle de menace** | Description formelle de ce qu'on protège et contre qui | Détermine les choix crypto : NIP-44 seul ≠ suffisant pour haute sécurité |
| **Viginum** | Service français de lutte contre la manipulation de l'information | Peut infiltrer, réquisitionner, surveiller — d'où cellules de 3 + web of trust |
| **Web of Trust** | Système de confiance par parrainage croisé plutôt que par autorité centrale | Remplace la vérification d'identité par l'État — chaque membre vérifie en personne |

### Gouvernance

| Terme | Définition | But |
|-------|-----------|-----|
| **Charte des Racines** | Constitution du réseau — document fondateur | Définit les principes, règles de gouvernance, mécanismes de modification |
| **Structure gazeuse** | Organisation sans centre, forks autorisés, protocoles mutables | Si le réseau est capturé, on fork — pas de point de contrôle unique |
| **Polydépendance** | Dépendre de ses pairs, de la matière, du temps — pas des algorithmes | Principe de conception — pas d'autonomie totale (fantasme) |
| **6 règles du ré-enracinement** | Ne jamais devenir leader, ne rien demander au système, ne pas se faire connaître, ne rien promettre, ne haïr personne, reconnaître les siens sans s'organiser | Principes de gouvernance du réseau |
