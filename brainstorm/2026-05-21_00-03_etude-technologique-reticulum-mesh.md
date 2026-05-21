# Étude technologique — RéseauRacine : stack souverain

## 1. Reticulum Network — La pièce maîtresse

### Ce que c'est
**Reticulum** est un stack réseau complet basé sur la cryptographie, conçu pour construire des réseaux locaux et wide-area avec du matériel readily available. Créé par Markqvist (unsigned.io), c'est la technologie la plus alignée avec la vision de RéseauRacine.

### Caractéristiques clés
- **Pas d'adresses source** : aucun paquet ne révèle l'origine (anonymat natif)
- **Pas de contrôle central** sur l'espace d'adresses : chacun génère ses propres adresses
- **Chiffrement end-to-end par défaut** : impossible de communiquer en clair
- **Forward secrecy** : clés éphémères, chaque session est unique
- **Adresses auto-souveraines et portables** : une adresse créée peut être déplacée physiquement et reste joignable
- **Performance** : 150 bps à 500 Mbps — fonctionne même sur LoRa (très bas débit)
- **2 816 commits** sur GitHub, API stable, wire-format stable

### Interfaces supportées
- Ethernet/WiFi
- **LoRa** (via RNode) — portée kilométrique, basse consommation
- **Packet Radio** (AX.25, TNC)
- **I2P** (anonymat renforcé)
- TCP/UDP over IP
- Ports série
- Programmes externes (stdio/pipes) — extensible à volonté

### Applications existantes (écosystème Reticulum)
| Application | Usage | Plateforme |
|-------------|-------|------------|
| **NomadNet** | Réseau mesh off-grid, pages browsables, forum | Linux, macOS, Windows |
| **Sideband** | Messagerie LXMF, appels voix, télémétrie, maps offline | Android, Linux, macOS, Windows |
| **Columba** | Messagerie LXMF simple, Material Design 3 | Android |
| **MeshChatX** | Client LXMF complet, voicemail, phonebook, maps | Linux, macOS, Windows |
| **Reticulum Relay Chat** | Chat temps réel over Reticulum | Web, GUI |
| **rncp** | Transfert de fichiers | CLI |
| **rnsh** | Shell interactif distant | CLI |
| **rnx** | Exécution de commandes distantes | CLI |
| **rngit** | Serveur Git over Reticulum | CLI |

### Pourquoi Reticulum est supérieur à Meshtastic pour RéseauRacine
| Critère | Meshtastic | Reticulum |
|---------|-----------|-----------|
| **Stack réseau complet** | Non (juste messagerie texte) | Oui (stack complet, applications multiples) |
| **Chiffrement** | AES-256 | Moderne, forward secrecy, anonymat natif |
| **Anonymat** | Non (adresses source visibles) | Oui (pas d'adresses source) |
| **Débit** | Très bas (texte seul) | 150 bps → 500 Mbps |
| **Applications** | Messagerie texte + GPS | Messagerie, voix, fichiers, shell, Git, chat, maps |
| **Scalabilité** | Limitée (problèmes avec grands réseaux) | Conçu pour planetary-scale |
| **Interopérabilité** | LoRa seul | LoRa, WiFi, Ethernet, I2P, Packet Radio, TCP/UDP |

### Limitations connues de Reticulum
- **Python** : la référence est en Python — pas idéal pour mobile/embedded (mais des implémentations Rust/C++/Zig/Go sont en cours)
- **UI des apps** : certaines apps ont une UI perfectible (Columba améliore ça)
- **Pas de group chats natifs** : limitation actuelle du protocole LXMF
- **Communauté petite** : projet principalement porté par un développeur (Markqvist)

---

## 2. Meshtastic — Le mesh LoRa grand public

### Ce que c'est
**Meshtastic** est un protocole open-source qui transforme des radios LoRa abordables (<50 €) en communicateurs mesh personnels. Messages texte chiffrés, GPS, télémétrie — sans internet, sans opérateur.

### État en France
- **1 849 nœuds actifs** en France (meshnetwork.fr)
- **50 000+ nœuds** dans le monde
- Communauté régionale active, carte interactive des nœuds
- Matériel : Heltec LoRa 32, RAK WisBlock, LilyGo T-Deck, Seeed Card Tracker

### Pourquoi Meshtastic est utile pour RéseauRacine
- **Backup physique** : quand internet est coupé, le mesh LoRa continue
- **Portée** : plusieurs kilomètres en milieu dégagé
- **Coût** : <50 € par nœud
- **Adoption** : communauté existante, pas besoin de construire from scratch
- **Radio Maquis** : Meshtastic = implémentation moderne du concept "Radio Maquis" de l'Asphyxie du Golem

### Limitations
- Texte seul (pas de voix, pas de fichiers volumineux)
- Problèmes de scalabilité sur grands réseaux
- Pas de stack réseau complet (juste messagerie)

**Verdict** : Meshtastic est un excellent **complément** à Reticulum, pas un remplacement. Reticulum pour le réseau complet, Meshtastic pour le backup LoRa.

---

## 3. Yggdrasil Network — L'overlay IPv6 chiffré

### Ce que c'est
**Yggdrasil** est un réseau overlay IPv6 chiffré end-to-end, auto-organisé, peer-to-peer. Chaque nœud obtient une adresse IPv6 dérivée de sa clé publique. Fonctionne sur IPv4 ou IPv6.

### Caractéristiques
- **Chiffrement end-to-end** par défaut
- **Auto-organisation** : découverte automatique des pairs, routing sans serveurs centraux
- **Self-healing** : le réseau répond rapidement aux pannes
- **Cross-platform** : Linux, macOS, Windows, iOS, Android, OpenWrt, EdgeRouter
- **5.1k stars** GitHub, relativement stable pour usage quotidien
- **NAT traversal** : fonctionne derrière CG-NAT

### Pourquoi Yggdrasil est utile pour RéseauRacine
- **Couche IP souveraine** : chaque nœud a son adresse IPv6 souveraine
- **Interopérabilité** : toute application IPv6-capable peut communiquer
- **Services existants** : sites web, IRC, DNS, Tor bridges, game servers sur le réseau Yggdrasil
- **Complément à Reticulum** : Yggdrasil pour l'overlay IP, Reticulum pour le mesh physique

### Limitations
- **Pas anonyme** : les pairs directs voient l'IP
- **Alpha** : pas audité officiellement
- **Firewall requis** : toute application écoutant sur toutes les interfaces est accessible depuis le réseau

---

## 4. Autres technologies pertinentes

### FFDN (Fédération FDN)
- **Fournisseurs d'accès Internet associatifs** depuis 1992
- Valeurs : bénévolat, solidarité, démocratie, non-lucratif, neutralité du Net
- **Aquilenet** (403 membres), **FDN** (Paris), etc.
- **Utilité pour RéseauRacine** : partenaires potentiels pour l'hébergement souverain, expertise technique, communauté engagée

### MeshFrance
- **meshfrance.org** : centralise les ressources pour les réseaux radio mesh en France
- **Utilité** : communauté technique francophone, expertise LoRa/mesh

### LibreMesh
- Firmware open-source basé sur OpenWrt pour routeurs WiFi
- **Utilité** : transformer des routeurs grand public en nœuds mesh WiFi

### Cloud souverain français
- **Celeste** : cloud privé 100% français, ISO 27001, HDS, fibre propriétaire
- **Aqua Ray** : cloud souverain certifié, conformité RGPD
- **Utilité** : hébergement de secours pour les nœuds critiques (pas pour le réseau principal — dépendance à un fournisseur)

---

## 5. Architecture technologique proposée pour RéseauRacine

### Stack en 4 couches

| Couche | Technologie | Rôle |
|--------|------------|------|
| **Physique** | LoRa (Meshtastic + RNode) + WiFi (LibreMesh) + Ethernet | Transport physique, mesh local |
| **Réseau** | Reticulum (stack principal) + Yggdrasil (overlay IPv6) | Routage chiffré, anonymat, adresses souveraines |
| **Applications** | Sideband (messagerie), NomadNet (forum/pages), PeerTube (vidéo), IPFS (stockage), Lightning (paiements) | Services utilisateurs |
| **Gouvernance** | Loomio (décisions), CryptPad (documents), Matrix (coordination) | Organisation du réseau |

### Topologie proposée

```
[Cellule locale 3 personnes]
    │
    ├── LoRa (Meshtastic) ← backup off-grid
    ├── WiFi (LibreMesh) ← mesh local
    └── Ethernet/Fibre ← connexion internet
         │
         ▼
    [Nœud Reticulum] ← stack principal
         │
         ├── Sideband ← messagerie chiffrée
         ├── NomadNet ← forum/pages
         ├── PeerTube ← vidéo
         ├── IPFS ← stockage distribué
         └── Lightning ← paiements
              │
              ▼
    [Essaim régional 10 cellules]
         │
         ├── Yggdrasil ← overlay IPv6 inter-régional
         └── I2P ← anonymat renforcé (optionnel)
```

### Coût estimé par nœud
| Composant | Coût |
|-----------|------|
| Serveur auto-hébergé (Raspberry Pi 5 ou mini PC) | 100-200 € |
| Module LoRa (RNode ou Heltec) | 30-80 € |
| Antenne LoRa | 20-50 € |
| Routeur WiFi (LibreMesh) | 50-100 € |
| Émetteur FM (Radio Maquis) | 50-150 € |
| **Total par nœud** | **250-580 €** |

---

## 6. Recommandations stratégiques

### Phase 1 (0-6 mois) : MVP Reticulum
1. **Installer Reticulum** sur 3 serveurs (cellule fondatrice)
2. **Déployer Sideband** sur Android pour la messagerie
3. **Déployer NomadNet** pour le forum/pages
4. **Connecter via LoRa** (RNode) entre les 3 nœuds
5. **Tester la résilience** : couper internet, vérifier que le mesh continue

### Phase 2 (6-18 mois) : Intégration Meshtastic + Yggdrasil
1. **Déployer Meshtastic** sur 10 nœuds LoRa (backup off-grid)
2. **Intégrer Yggdrasil** pour l'overlay IPv6 inter-régional
3. **Déployer PeerTube** sur 3 nœuds (vidéo souveraine)
4. **Intégrer IPFS** pour le stockage distribué
5. **Intégrer Lightning** pour les micro-paiements

### Phase 3 (18-36 mois) : Réseau complet
1. **200 nœuds** répartis sur 20 régions
2. **Mesh networks locaux** dans 5 villes (LibreMesh)
3. **Radio Maquis** : émetteurs FM dans 10 zones rurales
4. **Interopérabilité** avec les protocoles existants (email, RSS, ActivityPub)
5. **Audit de sécurité** indépendant

---

## 7. Risques technologiques

| Risque | Probabilité | Impact | Contre-mesure |
|--------|------------|--------|---------------|
| Reticulum abandonné (1 dev) | Moyenne | Critique | Fork communautaire, implémentations Rust/Go en parallèle |
| Meshtastic problèmes de scalabilité | Haute | Moyen | Utiliser Meshtastic uniquement comme backup, pas comme réseau principal |
| Yggdrasil non audité | Moyenne | Moyen | Ne pas y mettre de données critiques, utiliser I2P en complément |
| Matériel LoRa indisponible | Basse | Moyen | Stocker du matériel d'avance, diversifier les fournisseurs |
| Coupure ISP nationale | Moyenne | Élevé | Mesh LoRa + FM + Yggdrasil peer-to-peer = réseau continue sans internet |
| Attaque juridique sur les nœuds | Moyenne | Élevé | Hébergement multi-juridictions, constitution légale solide |

---

## 8. Conclusion

**Reticulum est la technologie centrale** de RéseauRacine. C'est le seul stack réseau complet qui offre :
- Anonymat natif (pas d'adresses source)
- Chiffrement end-to-end par défaut
- Forward secrecy
- Adresses auto-souveraines
- Support multi-transport (LoRa, WiFi, Ethernet, I2P, Packet Radio)
- Écosystème d'applications existant (Sideband, NomadNet, MeshChatX, etc.)
- Open-source, MIT license

**Meshtastic** est un excellent complément pour le backup LoRa off-grid.

**Yggdrasil** est utile pour l'overlay IPv6 inter-régional et l'interopérabilité avec les applications IPv6 existantes.

**FFDN/MeshFrance** sont des partenaires potentiels pour l'expertise et la communauté.

Le coût d'entrée est faible (250-580 € par nœud), la technologie est mature, et l'écosystème existe. Le principal risque est la dépendance à un seul développeur pour Reticulum — mais les implémentations Rust/Go/C++ en cours de développement atténuent ce risque.
