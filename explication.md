# Searchy — Explication complète du projet (soutenance)

**Projet :** Moteur de recherche local en Rust  
**Équipe :** 4 membres — EPITA S4  
**But :** indexer des fichiers d'un dossier et répondre à des recherches texte en classant les résultats par pertinence

---

## Vue d'ensemble : comment ça marche en 4 étapes

```
Dossier de fichiers
        │
        ▼
  1. LECTURE (reader.rs)
     Ouvre chaque fichier, extrait le texte brut
        │
        ▼
  2. NETTOYAGE (jesaispas.rs)
     Normalise, tokenise, supprime les mots inutiles
        │
        ▼
  3. INDEXATION (inverse_index.rs)
     Construit un index inversé (mot → liste de documents)
        │
        ▼
  4. RECHERCHE (search.rs)
     Calcule un score TF-IDF pour chaque document et trie les résultats
```

L'utilisateur interagit via une **interface graphique** (gui.rs) ou en **ligne de commande** (main.rs).

---

## Fichier par fichier

---

### `main.rs` — Point d'entrée du programme

C'est le fichier qui se lance quand on exécute `cargo run`. Il décide comment démarrer le programme.

**Logique :**
- Si l'utilisateur passe **2 arguments** en ligne de commande (un dossier + une requête), on lance le mode CLI.
- Sinon, on ouvre l'**interface graphique**.

**Mode CLI (`run_cli`) — déroulement pas à pas :**
1. Lecture des documents du dossier → `read_docs()`
2. Pour chaque document : nettoyage du texte → `preprocess()`
3. Construction de l'index inversé → `inverse_index()`
4. Lancement de la recherche → `search()`
5. Affichage des résultats avec le score et le chemin du fichier

**Exemple d'utilisation CLI :**
```bash
cargo run -- /mon/dossier "musique emotion"
```

---

### `reader.rs` — Lecture des fichiers

Ce module parcourt un dossier (et ses sous-dossiers) et extrait le texte de chaque fichier lisible.

**Structure `Document` :**
```
Document {
    id: usize,       // numéro unique du document (0, 1, 2, ...)
    path: String,    // chemin complet du fichier
    content: String, // texte brut extrait
}
```

**Fonction principale : `read_docs(dossier)`**  
Elle appelle `walk()` qui explore récursivement tous les dossiers.

**Formats supportés et comment ils sont lus :**

| Format | Méthode |
|--------|---------|
| `.txt` | Lecture directe avec `fs::read_to_string` |
| `.html` / `.htm` | Lecture puis suppression des balises HTML avec une regex `<[^>]+>` |
| `.pdf` | Extraction via la crate `pdf-extract` |
| `.docx` | C'est un ZIP : on ouvre `word/document.xml` et on extrait les balises `<w:t>` |
| `.odt` | C'est un ZIP : on ouvre `content.xml` et on enlève toutes les balises XML |
| Autres | Ignorés silencieusement |

**Pourquoi les `.docx` et `.odt` sont traités comme des ZIP ?**  
Ces formats sont en réalité des archives ZIP contenant des fichiers XML à l'intérieur. On utilise la crate `zip` pour les ouvrir sans les décompresser sur disque.

---

### `jesaispas.rs` — Nettoyage et normalisation du texte

> Note : le nom de ce fichier était provisoire lors du développement. Il contient le module de **prétraitement** du texte.

Ce module transforme un texte brut en une liste de mots utiles pour l'indexation.

**Fonction principale : `preprocess(texte)`**  
Enchaîne 4 étapes dans l'ordre :

**Étape 1 — `normalize(texte)`**  
- Passe tout en minuscules (`"Rust"` → `"rust"`)
- Supprime les accents (`"é"` → `"e"`, `"à"` → `"a"`)
- Technique : décomposition Unicode NFD + suppression des marques diacritiques

**Étape 2 — `clean_text(texte)`**  
- Remplace tout ce qui n'est pas une lettre ou un chiffre par un espace
- `"rust, c'est génial !"` → `"rust  c est genial  "`

**Étape 3 — `tokenize(texte)`**  
- Découpe le texte en mots sur les espaces
- `"rust est rapide"` → `["rust", "est", "rapide"]`

**Étape 4 — `remove_stopwords(tokens)`**  
- Supprime les mots trop courants qui n'ont pas de valeur pour la recherche
- Exemples de mots supprimés (français) : `"le"`, `"la"`, `"et"`, `"est"`, `"dans"`, `"avec"`...
- Exemples de mots supprimés (anglais) : `"the"`, `"is"`, `"in"`, `"and"`...
- Les mots d'une seule lettre sont aussi supprimés

**Exemple complet :**
```
Entrée : "Le chat est sur le tapis"
→ normalize : "le chat est sur le tapis"
→ clean_text : "le chat est sur le tapis"
→ tokenize : ["le", "chat", "est", "sur", "le", "tapis"]
→ remove_stopwords : ["chat", "tapis"]
```

---

### `inverse_index.rs` — Construction de l'index inversé

C'est le cœur de tout moteur de recherche.

**Qu'est-ce qu'un index inversé ?**  
Au lieu de stocker "document → liste de mots", on stocke l'inverse : **"mot → liste de documents qui le contiennent"**. Comme l'index à la fin d'un livre.

**Structure de l'index :**
```
HashMap<String, Vec<(i32, i32)>>
   mot      →   [(doc_id, tf), ...]

Exemple :
{
  "rust"    → [(0, 2), (2, 1)]   // "rust" apparaît 2 fois dans doc 0, 1 fois dans doc 2
  "memory"  → [(1, 1), (2, 1)]
  "safe"    → [(0, 1), (1, 1)]
}
```

- `doc_id` : identifiant du document
- `tf` (Term Frequency) : nombre de fois que le mot apparaît dans ce document

**Fonction `inverse_index(fichiers)` — déroulement :**
1. Pour chaque document, on compte combien de fois chaque mot apparaît (→ `word_list`)
2. On ajoute ces comptages dans l'index global
3. Si le mot n'existe pas encore dans l'index, on crée une nouvelle entrée
4. Si le mot existe déjà, on ajoute le nouveau `(doc_id, tf)` à sa liste

---

### `search.rs` — Moteur de recherche TF-IDF

Ce module calcule un score de pertinence pour chaque document et retourne les résultats triés.

**Qu'est-ce que TF-IDF ?**  
- **TF (Term Frequency)** : combien de fois le mot apparaît dans le document. Plus il apparaît, plus le document est pertinent.
- **IDF (Inverse Document Frequency)** : à quel point le mot est rare dans l'ensemble des documents. Un mot rare dans tous les docs (ex : `"oxymètre"`) a plus de valeur qu'un mot commun (ex : `"résultat"`).
- **Score = TF × IDF** : un document est pertinent si le mot y apparaît souvent ET si ce mot est rare dans les autres documents.

**Formule IDF utilisée :**
```
IDF(mot) = ln(nombre_total_de_docs / nombre_de_docs_contenant_le_mot) + 1
```

**Fonction `parse_query(requête)`**  
Nettoie la requête utilisateur : retire la ponctuation, met en minuscules.  
`"Musique, Émotions !"` → `["musique", "emotions"]`

**Fonction `idf(mot, index, nb_docs)`**  
Calcule le score IDF d'un mot. Si le mot n'est pas dans l'index, retourne 0.

**Fonction `search(requête, index, nb_docs)` — déroulement :**
1. Découpe la requête en mots avec `parse_query`
2. Pour chaque mot de la requête :
   - Calcule son IDF
   - Cherche dans l'index tous les documents qui contiennent ce mot
   - Pour chaque document trouvé, ajoute `TF × IDF` à son score total
3. Filtre les documents avec un score > 0
4. Trie les résultats du meilleur score au moins bon
5. Retourne la liste triée de `(doc_id, score)`

**Exemple :**
```
Requête : "rust memory"
→ "rust"   présent dans doc0 (tf=2) et doc2 (tf=1)
→ "memory" présent dans doc1 (tf=1) et doc2 (tf=1)
→ doc2 a des points pour les deux mots → score plus élevé
→ Résultats : doc2 > doc0 > doc1
```

---

### `images.rs` — Recherche d'images PNG par nom de fichier

Ce module gère un index séparé pour les images. Il n'analyse pas le contenu des images (pas d'OCR), mais recherche dans les **noms de fichiers**.

**Structure `ImageEntry` :**
```
ImageEntry {
    path: String,        // chemin complet de l'image
    tokens: Vec<String>, // mots extraits du nom de fichier
}
```

**`collect_images(dossier)`**  
Parcourt récursivement le dossier, collecte tous les `.png`, et tokenise leur nom.

**Tokenisation du nom :**  
Tout ce qui n'est pas alphanumérique sert de séparateur.  
`"carte_lyon_2024.png"` → tokens = `["carte", "lyon", "2024"]`

**`search_images(requête, images)`**  
Pour chaque image, compte combien de mots de la requête correspondent (même partiellement) à un token du nom de fichier. Trie par score décroissant.  
C'est une recherche simple par **sous-chaîne** : `"ly"` matche `"lyon"`.

---

### `gui.rs` — Interface graphique

C'est l'interface que l'utilisateur voit quand il lance le programme sans arguments. Construite avec la crate `eframe/egui`.

**Structure `SearchApp` — l'état de l'application :**
```
SearchApp {
    folder: String,                          // chemin du dossier sélectionné
    query: String,                           // requête texte saisie
    status: String,                          // message de statut affiché
    index: Option<HashMap<...>>,             // l'index inversé (None si pas encore indexé)
    id_to_path: Vec<String>,                 // correspondance doc_id → chemin fichier
    nb_docs: i32,                            // nombre de documents indexés
    results: Vec<(String, f64)>,             // résultats texte (chemin, score)
    images: Vec<ImageEntry>,                 // liste des images indexées
    image_query: String,                     // requête image saisie
    image_results: Vec<(String, usize)>,     // résultats images (chemin, score)
}
```

**Fonctionnement de l'interface (méthode `update`) :**  
`egui` appelle `update()` à chaque frame (comme un jeu vidéo). On dessine l'interface et on gère les actions à l'intérieur.

**Éléments de l'interface :**
1. **Titre** "Searchy" en bleu
2. **Ligne "Dossier"** : champ texte + bouton "Choisir dossier" (ouvre un explorateur de fichiers via `rfd`) + bouton "Indexer"
3. **Drag & drop** : on peut glisser-déposer un dossier directement sur la fenêtre
4. **Ligne "Requête texte"** : champ + bouton "Rechercher" + bouton ✖ (effacer)
5. **Ligne "Requête image"** : idem pour les images PNG
6. **Barre de statut** : message d'état (indexation en cours, nombre de résultats, erreurs)
7. **Deux colonnes** : résultats texte à gauche, résultats images à droite
8. **Liens cliquables** : cliquer sur un résultat ouvre le fichier avec `xdg-open` (l'application par défaut du système)

**`do_index()`** — déclenché par le bouton "Indexer" :
- Appelle `read_docs()` pour lire les fichiers texte
- Appelle `collect_images()` pour collecter les images
- Appelle `preprocess()` sur chaque document
- Appelle `inverse_index()` pour construire l'index

**`do_search()`** — déclenché par "Rechercher" ou la touche Entrée :
- Vérifie qu'un index existe
- Appelle `search()` et convertit les `doc_id` en chemins de fichiers

**`do_image_search()`** — déclenché par "Rechercher image" :
- Appelle `search_images()` et stocke les résultats

---

### `test.rs` — Fichier de test manuel

Petit programme indépendant (avec son propre `main`) utilisé pendant le développement pour tester rapidement `inverse_index` et `search` sans lancer l'interface graphique. Il crée des documents fictifs en dur et lance une recherche `"rust memory"`.

> Ce fichier n'est pas compilé dans le binaire final, c'est juste un outil de debug.

---

## Architecture globale — schéma des dépendances

```
main.rs
├── reader.rs       (lit les fichiers)
├── jesaispas.rs    (nettoie le texte)
├── inverse_index.rs (construit l'index)
├── search.rs       (calcule les scores TF-IDF)
└── gui.rs
    ├── reader.rs
    ├── jesaispas.rs
    ├── inverse_index.rs
    ├── search.rs
    └── images.rs   (recherche par nom de fichier PNG)
```

---

## Dépendances externes (`Cargo.toml`)

| Crate | Utilisation |
|-------|-------------|
| `regex` | Suppression des balises HTML, extraction XML des DOCX/ODT |
| `unicode-normalization` | Suppression des accents (décomposition NFD) |
| `pdf-extract` | Extraction du texte des fichiers PDF |
| `zip` | Lecture des archives DOCX et ODT |
| `eframe` | Framework pour l'interface graphique (egui) |
| `rfd` | Boîte de dialogue pour choisir un dossier |

Tout le reste (collections, fichiers, entrées/sorties) utilise la **bibliothèque standard Rust** (`std`).

---

## Données de test

| Dossier | Contenu |
|---------|---------|
| `Donnees_test_searchy_pdf/` | PDFs variés (règles de jeu, documents longs, emails...) |
| `Donnees_test_searchy_docx/` | Fichiers Word (textes sur la musique, les rêves, la solitude...) |
| `Donnees_test_searchy_png/` | Images PNG pour tester la recherche par nom |

---

## Points importants pour la soutenance

1. **L'index inversé est en mémoire** : il n'est pas sauvegardé sur disque entre deux sessions. À chaque lancement, il faut re-indexer.

2. **TF-IDF sans normalisation de longueur** : un long document avec beaucoup d'occurrences d'un mot aura mécaniquement un score plus élevé. C'est une simplification volontaire (niveau S4).

3. **La recherche d'images est totalement séparée** de la recherche texte. C'est un index distinct, sans TF-IDF, basé uniquement sur le nom des fichiers.

4. **Pas de persistance** : le programme ne retient rien entre deux fermetures. C'est une décision de conception pour rester simple.

5. **Deux modes d'utilisation** : GUI (par défaut) et CLI (avec arguments). La logique de fond est exactement la même.
