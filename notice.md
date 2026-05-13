# Notice d'utilisation — Searchy

Document vivant : à compléter à chaque évolution du projet.
Dernière mise à jour : 2026-05-13.

---

## 1. Ce que fait le programme

Searchy est un **moteur de recherche local** écrit en Rust. Il indexe un dossier
contenant tes documents et te permet de chercher des mots à l'intérieur. Les
documents les plus pertinents sont remontés en premier grâce à l'algorithme **TF-IDF**.

Formats lus :

| Extension      | Méthode                                            |
| -------------- | -------------------------------------------------- |
| `.txt`         | Lecture directe.                                   |
| `.pdf`         | Extraction du texte via la crate `pdf-extract`.    |
| `.docx`        | ZIP → `word/document.xml` → balises `<w:t>`.       |
| `.odt`         | ZIP → `content.xml` → texte brut.                  |
| `.html` `.htm` | Suppression naïve des balises.                     |

En plus, Searchy gère une **recherche d'images** séparée :

| Extension      | Méthode                                                  |
| -------------- | -------------------------------------------------------- |
| `.png`         | Recherche par **nom de fichier** (pas d'OCR du contenu). |

Les autres formats (`.doc` ancien Word, `.rtf`, `.epub`, autres images, PDF
scannés sans OCR…) sont **ignorés silencieusement**.

Deux façons d'utiliser le programme :

- **Interface graphique** (recommandée pour la démo) : fenêtre avec sélecteur de
  dossier, champ de requête et liste de résultats.
- **Ligne de commande** : utile pour les tests rapides et les démos en terminal.

---

## 2. Prérequis

- **OS** : Linux. Sous WSL2, l'interface graphique fonctionne grâce à WSLg
  (Windows 11) ou via un serveur X (Windows 10).
- **Rust** : édition 2024. Installation :
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
- **Outils de compilation** (Debian/Ubuntu) :
  ```bash
  sudo apt install build-essential pkg-config
  ```
- **Dépendances graphiques** (Debian/Ubuntu) : déjà présentes sous WSLg.
  En cas d'erreur de lien à la compilation :
  ```bash
  sudo apt install libxkbcommon-dev libgtk-3-dev libwayland-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev
  ```

---

## 3. Compilation

Depuis la racine du dépôt :

```bash
cd search_engine
cargo build --release
```

Première compilation : 30 secondes à 2 minutes (téléchargement des crates GUI).
Compilations suivantes : quasi instantanées.

---

## 4. Préparer le jeu de données

Le moteur indexe **n'importe quel dossier** que tu lui donnes. **Il n'y a plus de
documents dans le dépôt** : tu fournis ton propre corpus.

Convention conseillée : un dossier en dehors du dépôt.

```bash
mkdir -p ~/searchy_corpus
```

Tu peux organiser à plat ou avec des **sous-dossiers** : le parcours est récursif.

Exemple d'arborescence :

```
~/searchy_corpus/
├── cours/
│   ├── algorithmique.pdf
│   └── rust_intro.docx
├── notes/
│   ├── todo.txt
│   └── reunion.odt
└── ressources.html
```

---

## 5. Utilisation avec l'interface graphique (mode recommandé)

### 5.1 Lancement

```bash
cd search_engine
cargo run --release
```

Une fenêtre **Searchy** s'ouvre.

### 5.2 Disposition de la fenêtre

```
┌────────────────────────────────────────────────────────────┐
│ Searchy                                                    │
│ Moteur de recherche local (.txt/.pdf/.docx/...) + .png     │
│                                                            │
│ Dossier        : [____________________] [Indexer]          │
│ Requête texte  : [____________________] [Rechercher]       │
│ Requête image  : [____________________] [Rechercher image] │
│                                                            │
│ État : 12 document(s) texte indexé(s), 4 image(s) PNG.     │
│ ---------------------------------------------------------- │
│ Résultats texte :        │ Résultats images :              │
│ [1.7918] /.../algo.pdf   │ [2] /.../carte_lyon.png         │
│ [0.6931] /.../reunion.odt│ [1] /.../lyon_2024.png          │
└────────────────────────────────────────────────────────────┘
```

### 5.3 Étapes pour faire une recherche

1. Dans le champ **Dossier**, **tape ou colle** le chemin du dossier de documents.
   Astuces pour récupérer un chemin :
   - Sous Linux : `pwd` dans un terminal ouvert dans le dossier.
   - Sous Windows (via WSL) : clic droit dans l'explorateur → « Copier en tant
     que chemin », puis remplacer `C:\` par `/mnt/c/` et les `\` par `/`.
2. Clique sur **« Indexer »**. L'application lit tous les fichiers du dossier
   et de ses sous-dossiers. **Deux index sont construits en même temps :**
   - l'index texte (TF-IDF) pour `.txt/.pdf/.docx/.odt/.html`,
   - la liste des **images `.png`** trouvées (indexées par nom de fichier).

   La ligne d'**État** affiche par exemple
   `12 document(s) texte indexé(s), 4 image(s) PNG trouvée(s).`
3. **Recherche texte** : tape ta requête dans le champ **Requête texte**
   (un ou plusieurs mots, casse et accents indifférents) puis clique sur
   **« Rechercher »** ou appuie sur **Entrée**.
4. **Recherche image** : tape un ou plusieurs mots dans **Requête image**
   puis clique sur **« Rechercher image »** ou appuie sur **Entrée**. La
   recherche compare aux mots du **nom de fichier** uniquement (pas au contenu
   de l'image — pas d'OCR).
5. Les résultats apparaissent dans deux colonnes : texte à gauche (trié par
   score TF-IDF), images à droite (trié par nombre de mots de la requête
   trouvés dans le nom de fichier).

### 5.4 Cas particuliers affichés dans la ligne d'État

| Message                                          | Signification                                              |
| ------------------------------------------------ | ---------------------------------------------------------- |
| `Choisis un dossier puis clique sur Indexer.`    | État initial au démarrage.                                 |
| `Indique un dossier d'abord.`                    | Tu as cliqué sur Indexer avec un champ vide.               |
| `Indexation de /chemin en cours…`                | L'indexation est lancée.                                   |
| `12 document(s) indexé(s). Tape une requête.`    | OK, tu peux chercher.                                      |
| `Aucun document lisible dans /chemin`            | Le dossier est vide ou ne contient aucune extension gérée. |
| `Aucun index. Clique sur Indexer d'abord.`       | Tu as cliqué sur Rechercher sans avoir indexé.             |
| `Saisis une requête.`                            | Champ requête texte vide.                                  |
| `Aucun résultat pour : "xxx"`                    | Aucun document ne contient les mots cherchés.              |
| `N résultat(s).`                                 | Recherche réussie, N documents remontés.                   |
| `Aucune image indexée. Clique sur Indexer d'abord.` | Tu as cliqué sur Rechercher image sans avoir indexé.    |
| `Saisis une requête image.`                      | Champ requête image vide.                                  |
| `Aucune image pour : "xxx"`                      | Aucun nom de fichier `.png` ne correspond.                 |
| `N image(s) trouvée(s).`                         | Recherche image réussie, N images remontées.               |

### 5.5 Notes pratiques

- Tu peux **changer de dossier et ré-indexer** à tout moment.
- Tu peux **enchaîner plusieurs requêtes** sur le même index sans ré-indexer.
- L'indexation est synchrone : sur un gros corpus la fenêtre peut « geler »
  quelques secondes le temps de lire les fichiers. C'est normal.

---

## 6. Utilisation en ligne de commande (mode rapide)

Pour ceux qui préfèrent le terminal, ou pour scripter.

```bash
cd search_engine
cargo run --release -- <dossier> "<requête>"
```

Exemples :

```bash
# Mot simple
cargo run --release -- ~/searchy_corpus "rust"

# Plusieurs mots
cargo run --release -- ~/searchy_corpus "index inversé tf-idf"
```

Sortie type :

```
12 document(s) indexé(s).

Résultats pour "index inversé" :
  [1.7918] /home/mathi/searchy_corpus/cours/algorithmique.pdf
  [0.6931] /home/mathi/searchy_corpus/notes/reunion.odt
```

Si **aucun argument** n'est passé, le programme ouvre l'interface graphique
(c'est le mode par défaut).

---

## 7. Comment lire les résultats

Chaque ligne :

```
[score] chemin_du_fichier
```

- **Score** = somme des `TF × IDF` pour chaque mot de la requête.
  - `TF` (term frequency) : nombre d'occurrences du mot dans ce document.
  - `IDF` (inverse document frequency) : `ln(nb_docs / nb_docs_qui_contiennent_le_mot)`
    → un mot rare dans le corpus pèse plus lourd qu'un mot fréquent.
- **Chemin** = chemin complet vers le fichier original.

Les résultats sont **toujours triés du plus pertinent au moins pertinent**.

Cas à connaître :
- Si un mot apparaît dans **tous** les documents, son IDF vaut `ln(N/N) = 0`,
  donc il n'influence pas le score (c'est correct, ce mot n'apporte aucune info).
- Si la requête ne contient que des **stop words** (`le`, `la`, `the`, `and`…),
  ils sont filtrés et il ne reste rien à chercher.

---

## 8. Scénarios de test recommandés

### 8.1 Sanity check (1 minute)

```bash
mkdir -p /tmp/searchy_sanity
echo "Le langage Rust est rapide et sûr." > /tmp/searchy_sanity/a.txt
echo "Python est interprété, lent mais pratique." > /tmp/searchy_sanity/b.txt
cd search_engine
cargo run --release -- /tmp/searchy_sanity "rust"
```
Attendu : `a.txt` ressort en premier, `b.txt` n'apparaît pas.

### 8.2 Test interface graphique avec PDF

```bash
mkdir -p /tmp/searchy_pdf
cp ../cahier_des_charges_projet_rust.pdf /tmp/searchy_pdf/
cd search_engine
cargo run --release
```
Dans la fenêtre :
1. Coller `/tmp/searchy_pdf` dans le champ **Dossier**.
2. **Indexer** → l'état doit afficher `1 document(s) indexé(s).`
3. Taper `moteur tf-idf` puis **Rechercher**.
4. Le PDF doit ressortir avec un score > 0.

### 8.3 Test DOCX + ODT

Créer deux documents avec LibreOffice (ou Word) :
- `algorithme.docx` qui contient le mot « tokenisation ».
- `reseaux.odt` qui ne le contient pas.

Lancer l'interface, sélectionner le dossier, indexer, chercher `tokenisation`.
Attendu : seul `algorithme.docx` apparaît.

### 8.4 Test corpus mixte (démo de soutenance)

```
/tmp/searchy_mixte/
├── pdfs/
│   ├── cours_algorithmique.pdf
│   └── cours_systeme.pdf
├── word/
│   └── rapport.docx
├── libre/
│   └── presentation.odt
└── notes.txt
```

Lancer l'interface, sélectionner `/tmp/searchy_mixte`, indexer.
Vérifier dans cet ordre :
1. **Compteur** : 5 document(s) indexé(s).
2. **Récursion** : les fichiers des sous-dossiers apparaissent dans les résultats.
3. **Pertinence** : les documents les plus pertinents sont en tête.

---

## 9. Limitations connues

- L'index est **reconstruit à chaque démarrage** (pas encore de sauvegarde sur disque).
- Pas d'opérateurs booléens explicites (`AND`, `OR`, `NOT`).
- Pas d'extrait de texte (snippet) autour des mots trouvés.
- Pas de surbrillance des mots dans les résultats.
- `.doc` ancien Word binaire, `.rtf`, `.epub`, PDF scannés sans OCR : non gérés.
- L'indexation est synchrone (la fenêtre peut figer quelques secondes sur un gros corpus).

---

## 10. Historique des modifications

- **2026-05-13 (suite)** :
  - **Polish GUI** : titre coloré « 🔎 Searchy », icônes sur les boutons
    (« 📁 Indexer », « 🔍 Rechercher », « 🖼 Rechercher image »).
  - **Bouton ✖** à côté de chaque champ de requête : vide la requête et ses
    résultats sans toucher à l'index.
  - **Drag & drop** : glisser un dossier (ou un fichier — on prend son dossier
    parent) sur la fenêtre remplit automatiquement le champ « Dossier ». Une
    ligne d'aide « Astuce : tu peux glisser-déposer… » est affichée sous le
    titre. Aucune dépendance ajoutée (eframe gère le drop nativement).

- **2026-05-13** :
  - Ajout d'une **2ᵉ barre de recherche** dédiée aux **images PNG**, séparée
    de la recherche texte. Le champ « Dossier » et le bouton « Indexer »
    sont **partagés** : un seul chemin sert aux deux index. Le bouton
    construit en même temps l'index texte TF-IDF et la liste des `.png`.
  - La recherche image se fait **uniquement sur le nom de fichier** (pas
    d'OCR). Algo simple : on découpe le nom de fichier en mots
    (séparateurs : tout ce qui n'est pas alphanumérique → `_`, `-`, `.`,
    espaces…), on fait pareil sur la requête, et on compte combien de mots
    de la requête sont retrouvés en sous-chaîne d'un token du nom de
    fichier. Plus le score est élevé, plus l'image remonte.
  - Nouveau fichier `src/images.rs` (collecte récursive + algo de score).
    Le cœur algorithmique TF-IDF n'est pas touché.
  - Layout GUI : les résultats sont maintenant affichés en **2 colonnes
    côte à côte** (texte à gauche, images à droite).

- **2026-05-12** :
  - Suppression du bouton « Parcourir… » et de la dépendance `rfd`. Raison : la
    boîte de dialogue native dépend d'un portail XDG ou de GTK3-dev qui ne sont
    pas installés partout (notamment sous WSL2 sans desktop). Le champ texte
    seul rend l'application portable sur n'importe quel ordinateur sans
    installation système préalable.
  - Agrandissement de l'interface graphique : fenêtre passée de 720×520 à
    1100×750, polices augmentées (titre 32 px, corps 18 px, boutons 18 px),
    boutons plus hauts (32 px) avec padding, champs texte élargis à 600 px,
    marge intérieure de 20 px autour du contenu. Lecture beaucoup plus confortable.

- **2026-05-11** :
  - Support PDF / DOCX / ODT / HTML (auparavant : `.txt` seulement, corpus factice).
  - CLI avec arguments `<dossier> "<requête>"`.
  - Interface graphique (eframe / egui) avec sélecteur de dossier, champ de
    requête, bouton Indexer, bouton Rechercher, liste des résultats.
  - Suppression des documents factices du dépôt.
