# Les Couches de l'Oignon

## Histoire d'un message chiffré de bout en bout

> **Pourquoi ce document ?** Ce projet est techniquement complexe : cryptographie, protocoles réseau, concurrence, sécurité.
> Ce récit raconte chaque étape de sa construction sans jargon technique, avec des métaphores et des analogies.
> Pour comprendre **pourquoi** on a fait ces choix, **comment** ça marche, et **ce que chaque couche apporte**.

---

## L'oignon : la métaphore

Imagine un oignon. Chaque couche que tu enlèves en révèle une autre. Dans notre projet, c'est l'inverse : **chaque couche qu'on ajoute protège le message un peu plus**.

Sans une couche, les autres existent encore. Mais le système est moins sûr, moins résistant.

Ce récit suit l'ordre dans lequel on a construit ces couches. Dans la vraie vie, on n'a pas commencé par la première puis la deuxième : on a découvert au fur et à mesure qu'il fallait ajouter des couches.

---

## Couche 0 — L'Atelier du Forgeron (EPIC 0)

> **Avant de forger une épée, il faut un atelier.**

On n'a pas commencé par coder du chiffrement. On a d'abord construit l'environnement de travail :

- **Le langage (Rust)** : un langage qui détecte les erreurs de mémoire au moment de la compilation, pas au moment de l'exécution. Comme un architecte qui vérifie les plans avant de couler le béton, plutôt que de réparer les fissures après construction.

- **Les outils de construction (CI/CD)** : une chaîne automatique qui, à chaque modification, vérifie que tout compile, que les tests passent, qu'il n'y a pas de failles de sécurité. Comme un contremaître qui inspecte chaque brique avant qu'elle soit posée.

- **L'atelier portable (Docker)** : tout le développement se fait dans un conteneur. Pas besoin d'installer Rust, Python, ou quoi que ce soit sur la machine. Un seul logiciel à installer : Docker. Le conteneur contient tout le nécessaire.

- **Les briques de base (cryptographie)** : les algorithmes de chiffrement, les générateurs de clés, les fonctions de hachage. Ce sont les matériaux de base qu'on va utiliser partout.

**Pourquoi Rust ?** La mémoire est le cimetière de 70% des failles de sécurité. Rust empêche ces erreurs à la compilation. Pour un logiciel de chiffrement, c'est non-négociable.

**Pourquoi Docker ?** Pour qu'un nouveau contributeur puisse compiler et tester le projet en 2 commandes, sans configuration. Une seule contrainte : avoir Docker.

---

## Couche 1 — L'Enveloppe (EPIC 1)

> **Le premier geste : envoyer une lettre confidentielle.**

Tu écris un message à quelqu'un. Comment faire pour que personne d'autre ne le lise ?

**La solution évidente :** tu chiffres le message avec la clé publique du destinataire. Seul lui peut le déchiffrer avec sa clé privée. C'est le principe de base — un algorithme appelé NIP-44.

Mais il y a un problème : **le facteur sait à qui tu écris**. Le message est chiffré, mais n'importe qui peut voir que Alice envoie un message à Bob. Dans certains pays, le simple fait de communiquer avec quelqu'un peut être dangereux.

**La solution Réseau Racine : le GiftWrap.**

Imagine que tu mets ta lettre dans une enveloppe, que tu scelles avec la cire du destinataire. Même le facteur ne peut pas voir à qui elle est destinée — il voit juste une enveloppe scellée. Seul le destinataire peut l'ouvrir... et découvre qu'elle contient une deuxième enveloppe, la vraie lettre.

Techniquement : c'est le NIP-17. On crée une rumeur (le message), on la met dans un sceau (sealed seal), puis on l'emballe dans un GiftWrap par destinataire. Le relais Nostr voit juste un événement chiffré, sans savoir qui parle à qui.

**Ce qui a guidé nos choix :**

-
  **Pourquoi Nostr ?** Nostr est un protocole décentralisé, sans serveur central. Chaque relais est interchangeable. Pas de risque qu'une entreprise unique contrôle vos communications.

-
  **Pourquoi ne pas utiliser Signal/WhatsApp ?** Ces applications sont centralisées et nécessitent un numéro de téléphone. Nostr est ouvert, pseudonyme, et décentralisé.

---

## Couche 2 — Le Club Secret (EPIC 2)

> **Maintenant on est plusieurs. Comment parler à tout un groupe sans répéter le message pour chaque personne ?**

La solution naïve : tu prends le message, tu le chiffres une fois par membre, et tu envoies chaque copie. Ça marche, mais c'est lourd.

**La solution : une clé partagée.** C'est comme une poignée de main secrète que tous les membres du club connaissent. Tu écris ton message une fois, tu le chiffres avec cette clé commune, et tu l'envoies à tout le monde en une fois.

Chaque groupe — on l'appelle une **cellule** — a sa propre clé. Créer une cellule, c'est comme fonder un club : tu définis qui en fait partie, tu distribues la poignée de main secrète, et la conversation peut commencer.

**Les problèmes qu'on a dû résoudre :**

- **Inviter quelqu'un** : comment donner la poignée de main à un nouveau membre sans que les autres puissent l'intercepter ? On utilise le chiffrement de la Couche 1 (message individuel).

- **Exclure quelqu'un** : si un membre quitte le club, il faut changer la poignée de main. On régénère une nouvelle clé, on la distribue aux membres restants. L'ancien membre ne peut plus lire les nouveaux messages. C'est une **rotation de clés**.

- **Découvrir des groupes** : tu peux lancer le mode écoute sans savoir quels groupes existent. Le système détecte automatiquement les nouveaux groupes qui te parlent et les crée. C'est le **mode découverte**.

**Zone technique : Sender Keys.** C'est un mécanisme de Signal (l'application de messagerie) adapté à nos besoins. Chaque membre du groupe a sa propre "chaîne de clés" qui se transforme à chaque message. On verra ça plus en détail dans la Couche 3.

---

## Couche 3 — La Serrure qui Danse (EPIC 5)

> **Et si la serrure se transformait après chaque utilisation ?**

C'est la question fondamentale du **secret parfait** (forward secrecy).

**Le problème :** avec la clé partagée de la Couche 2, si quelqu'un vole la clé aujourd'hui, il peut déchiffrer **tous** les messages passés et futurs. C'est comme si tu avais une seule serrure pour toute la vie du club.

**La solution : changer la serrure après chaque message.**

Imagine une serrure dont le pêne (la pièce qui coulisse) se transforme chaque fois que tu tournes la clé. La position d'aujourd'hui ne peut pas être déduite de la position d'hier. Et la position de demain ne peut pas être prédite à partir de celle d'aujourd'hui.

Techniquement : c'est un **ratchet** (cliquet). Tu ne peux aller que vers l'avant, jamais revenir en arrière. Chaque message produit :
- Une **clé de message** (pour déchiffrer ce message précis)
- Une **clé de chaîne** (pour le prochain message)

Si on vole la clé de chaîne du message n°5, on peut déchiffrer le message n°6 (le suivant), mais pas le message n°4 (le précédent). C'est le **forward secrecy**.

**Comment on fabrique ces clés ?** Avec une fonction mathématique appelée HKDF-SHA256. C'est immangeable en cuisine, mais en clair : c'est une machine qui prend un ingrédient (la clé actuelle), le mixe avec un algorithme, et produit deux nouveaux ingrédients (la clé de message et la prochaine clé de chaîne). Déterministe : mêmes entrées = mêmes sorties.

**Pourquoi Sender Keys plutôt que MLS ?** MLS (Messaging Layer Security) est le protocole standard pour les groupes chiffrés. Mais il est très complexe — conçu pour des centaines de participants. Nos groupes font 3 à 5 personnes. Sender Keys, c'est plus simple et plus adapté : chaque membre a sa propre chaîne, la rotation est triviale.

---

## Couche 4 — Le Coffre-Fort (EPIC 7)

> **Où ranges-tu tes clés quand tu ne les utilises pas ?**

**Le problème :** par défaut, ta clé privée est stockée dans un fichier JSON sur ton disque. C'est comme laisser tes clés de maison sous le paillasson. N'importe quel programme malveillant peut les lire.

**La solution : un coffre-fort avec mot de passe.**

KeePassXC est un gestionnaire de mots de passe open-source. Il stocke tes secrets dans une base chiffrée, protégée par un mot de passe maître. Même si ton ordinateur est volé, personne ne peut ouvrir le coffre sans le mot de passe.

**Comment ça marche concrètement :**

1. Tu crées ou tu ouvres une base KeePassXC (un fichier `.kdbx`)
2. Tu donnes le chemin à Réseau Racine (`rr init --kdbx ~/vault.kdbx`)
3. À chaque fois que le logiciel a besoin de ta clé privée, il demande le mot de passe à KeePassXC
4. KeePassXC déverrouille le coffre, donne la clé, et le coffre se referme

**Et si KeePassXC n'est pas installé ?** On a prévu une alternative : le logiciel peut ouvrir le fichier `.kdbx` directement en Rust, sans avoir besoin de KeePassXC installé. Moins sécurisé que KeePassXC (pas d'interface de déverrouillage), mais plus pratique.

**Choix de conception :** on n'a pas réinventé la roue. KeePassXC existe, il est mature, audité, open-source. Pourquoi coder un coffre-fort quand un bon existe déjà ?

---

## Couche 5 — Le Chronomètre (EPIC 8)

> **Combien de temps ça prend, chaque geste ?**

On a construit pas mal de couches. Mais on ne savait pas si le logiciel était rapide ou lent. Plutôt que de deviner, on a mesuré.

**Les benchmarks :** on a chronométré chaque opération :

- Chiffrer un message de 16 caractères, 1 000 caractères, 10 000 caractères
- Signer un événement
- Emballer un GiftWrap
- Envoyer un message à un relais
- Synchroniser les messages reçus

**Pourquoi le faire en CI (intégration continue) ?** Pour détecter les régressions. Imagine : tu ajoutes une fonctionnalité, et sans le savoir, tu ralentis le chiffrement de 50%. Si le benchmark est automatisé, l'ordinateur te le dit immédiatement. Si tu ne mesures pas, tu le découvres six mois plus tard en production.

**Les outils :** on utilise Criterion, une bibliothèque de benchmark pour Rust. Très précis, avec analyse statistique, détection de régressions intégrée.

---

## Couche 6 — Le Test de Foule (EPIC 9)

> **Ça tient si tout le monde parle en même temps ?**

Un logiciel de messagerie, c'est fait pour être utilisé par plusieurs personnes simultanément. Mais comment savoir si le système tient la charge ?

On a créé un simulateur : `rr-stress`.

**Comment ça marche :**

1. On crée 5 identités virtuelles
2. Chaque identité envoie 3 messages à des destinataires aléatoires
3. On chronomètre : combien de messages réussissent, combien échouent, combien de temps ça prend
4. On calcule des statistiques : latence moyenne, latence maximale, débit

**Résultat typique :** 15 messages en 0,32 secondes. Tout réussi. Latence : 1,8ms en médiane, 7,6ms au pire.

C'est un test modeste (5 utilisateurs), mais il valide que le mécanisme de base fonctionne sous charge. On pourra l'augmenter plus tard.

**Pourquoi ne pas avoir testé directement avec 50 utilisateurs ?** Parce que le goulot d'étranglement, c'est le relais Nostr local. Au-delà d'un certain nombre de connexions simultanées, le relais sature. C'est un problème connu, qu'on adressera plus tard (Couche future : le nœud relais embarqué).

---

## Couche 7 — Les Rustines (SEC-1)

> **En construisant, on a découvert des fissures. On les a bouchées.**

Aucun logiciel n'est parfait du premier coup. En ajoutant les couches précédentes, on a identifié trois failles de sécurité. Cette couche, c'est les correctifs.

### Rustine n°1 : la serrure qui danse pouvait se bloquer

**Le problème :** dans la Couche 3 (Serrure qui danse), on utilisait une fonction mathématique qui prenait la clé actuelle et produisait la clé suivante. Si le même "numéro de message" était utilisé deux fois avec la même clé, la serrure produisait exactement la même position. Comme si tu tournais la clé et qu'elle s'arrêtait au même endroit.

**Scénario accidentel :** l'ordinateur plante après avoir sauvegardé la nouvelle clé mais avant d'envoyer le message. Au redémarrage, on réessaie avec la même clé... et on produit exactement la même clé de message. Deux messages différents avec la même clé de chiffrement — c'est une faille grave.

**La rustine :** on ajoute le numéro du message dans le calcul de la nouvelle clé. Même si on réutilise la même clé de départ, le numéro de message (qui a augmenté) garantit une clé de message différente.

Métaphore : avant, la serrure regardait juste la position actuelle du pêne. Maintenant, elle regarde aussi combien de fois elle a été tournée. Deux tours avec la même position de départ ne donnent pas la même serrure.

### Rustine n°2 : n'importe qui pouvait annoncer un changement de serrure

**Le problème :** quand on exclut un membre (rotation de clés), on envoie un message spécial à tous les membres restants : "changez votre serrure". Mais n'importe qui pouvait envoyer ce message, même sans être membre du club.

**Scénario d'attaque :** un attaquant envoie un faux message de rotation. Les membres changent leur serrure, mais l'attaquant a fourni sa propre serrure. Maintenant il peut lire tous les messages.

**La rustine :** on vérifie que l'expéditeur du message de rotation est bien membre du club, et on fait cette vérification sous verrouillage (pour éviter qu'il ne soit retiré entre la vérification et la mise à jour).

Métaphore : avant d'accepter qu'un nouveau verrou soit installé, on vérifie que la personne qui demande le changement a bien le droit d'être dans la pièce. Et cette vérification est faite d'un bloc, sans interruption possible.

### Rustine n°3 : si l'ordinateur plantait, on perdait le carnet d'adresses

**Le problème :** le fichier contenant la liste des groupes et des clés était écrit directement. Si l'ordinateur plantait au milieu de l'écriture, le fichier pouvait être corrompu — ou pire, vidé mais pas encore rempli.

**La rustine :** on écrit d'abord dans un fichier temporaire, puis on renomme le fichier temporaire en fichier définitif. Si l'ordinateur plante pendant l'écriture, seul le fichier temporaire est perdu. L'original reste intact.

Métaphore : avant de remplacer ton carnet d'adresses, tu écris d'abord les nouvelles adresses sur un post-it. Une fois que tout est correctement noté, tu remplaces le carnet par le post-it. Si le stylo tombe en panne pendant que tu écris le post-it, le carnet original est toujours là.

---

## La Suite — Les Prochaines Couches

> **Un oignon n'a jamais fini de pousser.**

Plusieurs couches sont en projet ou en réflexion :

### Reticulum (EPIC 3) — Le réseau maillé
Aujourd'hui, les messages passent par Internet (via les relais Nostr). Demain, ils pourront passer par un réseau maillé WiFi, sans Internet. Chaque téléphone devient un relais. Communication possible même en zone blanche.

### Client Tauri (EPIC 4) — L'interface graphique
Aujourd'hui, tout se fait en ligne de commande. Demain, une vraie application de bureau avec fenêtres, notifications, contacts visuels.

### Nœud relais (EPIC 6) — Le serveur personnel
Un petit boîtier (Raspberry Pi) qui fait office de relais Nostr personnel. Tu contrôles ton propre serveur de messages, hébergé chez toi.

### Nettoyage (CLEAN-1) — Faire le ménage
Du code mort, des fonctions inutilisées, des chemins de code obsolètes qui traînent. Rien de cassé, mais c'est plus propre sans.

### Erreurs élégantes (ERR-1) — Des messages d'erreur qui veulent dire quelque chose
Quand quelque chose plante, le logiciel affiche parfois des messages obscurs. On les remplace par des messages clairs et utiles.

### Refactoring (REFACTOR-1) — Moins de duplication
Le code qui écoute les messages est dupliqué en deux endroits presque identiques. On le fusionne en un seul. Plus facile à maintenir, moins de bugs.

### Tests (TEST-1) — Attraper les bugs avant qu'ils arrivent
Plus de tests automatiques pour les fonctions critiques. Chaque bug qu'on a corrigé dans SEC-1 est documenté par un test qui vérifie qu'il ne reviendra pas.

---

## Conclusion

Un message Réseau Racine, c'est :

1. Un texte clair, écrit par toi
2. **Chiffré** avec une clé unique par message (Couche 3)
3. **Emballé** dans une enveloppe anonyme (Couche 1)
4. **Distribué** au groupe (Couche 2)
5. **Signé** avec ta clé privée, stockée dans un coffre (Couche 4)
6. **Transmis** via des relais, validé par des tests de charge (Couches 5-6)
7. **Protégé** contre les failles découvertes (Couche 7)

Et tout ça, mesuré, testé, répété.

Chaque couche est une barrière de plus. Aucune n'est parfaite seule, mais ensemble, elles forment un système dont la sécurité est plus grande que la somme de ses parties.

C'est ça, l'oignon.

---

*Document rédigé le 2026-05-25 — mis à jour au fil des EPICs*
