# For next time — état des lieux Searchy

Document de passation pour le prochain agent.
Dernière session : 2026-05-12.

---

## TL;DR

Projet **Searchy** = moteur de recherche textuel en Rust pour EPITA S4.
À aujourd'hui : on est passé d'un prototype avec corpus factice à un moteur qui
indexe de vrais PDF/DOCX/ODT avec une interface graphique simple. Estimation
d'avancement vs cahier des charges : **~65 %**.

Ce qui reste à faire est cadré dans la section « Reste à faire » plus bas.

---

## Contexte projet à connaître

- Projet de groupe (4 membres). L'utilisateur (Clément ou un coéquipier) bosse
  surtout la partie **moteur de requêtes / TF-IDF** d'après `CLAUDE_searchy.md`.
- Réparti en 4 modules + CLI/GUI dans le cahier des charges.
- Soutenance finale en mai 2026 → on est dans la phase « finalisation ».
- L'utilisateur écrit en français, code parfois en français parfois en anglais.
- Tout le code doit rester **niveau S4** (étudiant 2ᵉ année). Pas de sur-ingénierie.

Le fichier `CLAUDE_searchy.md` à la racine est le **guide de style officiel** du
projet — le lire avant de coder. Important : la règle « aucune crate externe »
a été **levée en session du 2026-05-11**, voir section « Décisions importantes ».

---

## Arborescence actuelle

```
search_engine_project/
├── CLAUDE_searchy.md        # guide de style projet (à respecter)
├── README.md                # quasi vide
├── notice.md                # notice d'utilisation (mise à jour à chaque évol)
├── for_nextime.md           # CE fichier
├── cahier_des_charges_projet_rust.pdf
└── search_engine/
    ├── Cargo.toml
    └── src/
        ├── main.rs          # lance GUI par défaut, CLI si args >= 3
        ├── gui.rs           # interface eframe/egui
        ├── reader.rs        # ingestion .txt/.pdf/.docx/.odt/.html, récursif
        ├── jesaispas.rs     # normalisation texte + stop words FR+EN
        ├── inverse_index.rs # construction HashMap<String, Vec<(doc_id, tf)>>
        ├── search.rs        # parse_query + IDF + TF-IDF + tri
        ├── test.rs          # ORPHELIN — a un fn main, n'est pas compilé
        └── docs/            # vide (corpus factice supprimé)
```

Note : `test.rs` est orphelin (déclare un second `fn main`). Il n'est pas compilé
parce qu'il n'est pas déclaré dans `main.rs`. Le supprimer un jour ou le
convertir en `#[cfg(test)]` quand on aura le temps.

---

## Dépendances actuelles (`Cargo.toml`)

```toml
regex = "1"
unicode-normalization = "0.1"
pdf-extract = "0.7"
zip = "0.6"
eframe = "0.27"
```

Toutes ajoutées avec accord explicite de l'utilisateur. Ne pas en ajouter
d'autres sans demander. `rfd` a été retirée le 2026-05-12 (voir section session).

---

## Ce qui a été fait dans cette session (2026-05-12)

### 1. Test avec un corpus PDF réel
- L'utilisateur a déposé 9 PDF dans `Donnees_test_searchy/` à la racine du dépôt.
- 9 fichiers `*:Zone.Identifier` (résidus NTFS de copie depuis Windows) ont été
  supprimés sur tout l'arbre du projet.

### 2. Retrait de `rfd` et du bouton « Parcourir… »
- Symptôme : sous WSL2 (et plus généralement sur tout système sans
  `xdg-desktop-portal` ni `libgtk-3-dev`), le bouton « Parcourir… » ne faisait
  rien (rfd échoue silencieusement quand son backend par défaut, le portail
  XDG, n'est pas joignable).
- Tests effectués : tentative de switch sur le backend `gtk3` → échec build car
  les headers `libgtk-3-dev` ne sont pas installés. Pose un problème de
  portabilité (le projet doit marcher sur n'importe quel ordi sans
  installation système).
- Décision de l'utilisateur : **retirer `rfd` entièrement**. Garder uniquement
  le champ texte où on tape/colle le chemin du dossier.
- `Cargo.toml` : ligne `rfd = "0.14"` supprimée.
- `src/gui.rs` : bloc `if ui.button("Parcourir…")...` supprimé, le champ
  texte « Dossier » a été élargi (440 → 600 px), le placeholder dit maintenant
  « Colle ici le chemin du dossier à indexer ».

### 3. Agrandissement de la GUI
- Avant la modif, la fenêtre était jugée trop petite par l'utilisateur.
- Ajout d'une fonction `setup_style(ctx)` dans `gui.rs` qui modifie les
  `TextStyle` egui : titre 32 px, corps 18 px, boutons 18 px,
  monospace 16 px, padding bouton 12×8, espacement 10×8, hauteur interaction 32 px.
- Marge intérieure de 20 px ajoutée au `CentralPanel` via `egui::Frame`.
- Fenêtre passée de 720×520 à **1100×750**.
- `setup_style` est appelée dans le callback de `eframe::run_native`.

### 4. Issue WSLg / ZINK (pas de fix en code)
- Au lancement de la GUI, l'utilisateur a eu une série d'erreurs Mesa
  (`libEGL warning: failed to get driver name`, `MESA: error: ZINK: failed to
  choose pdev`, `Broken pipe`, `winit EventLoopError: Exit Failure: 1`).
- C'est un bug **système WSLg**, indépendant de notre code. Le driver virtuel
  Mesa ZINK se met dans un mauvais état (fréquent après veille Windows ou
  beaucoup d'apps OpenGL).
- Workaround pour démarrage manuel : `LIBGL_ALWAYS_SOFTWARE=1 cargo run --release`
  (rendu logiciel — plus lent mais 100 % fiable).
- Workaround définitif côté système : `wsl --shutdown` depuis PowerShell, puis
  rouvrir WSL. Réinitialise la session graphique.
- **J'ai initialement codé une auto-détection** (`force_software_rendering_on_wsl`)
  qui posait `LIBGL_ALWAYS_SOFTWARE=1` automatiquement si `WSL_DISTRO_NAME` était
  détectée. **L'utilisateur a demandé d'annuler** car il veut que le code
  marche pareil sur tous les ordis sans logique conditionnelle. La fonction et
  son appel ont été retirés. Pas de trace dans `gui.rs` ni dans `notice.md`.

## Ce qui a été fait dans la session précédente (2026-05-11)

### 1. Audit initial du projet
- Lu tout le code + `cahier_des_charges_projet_rust.pdf` + `CLAUDE_searchy.md`.
- Établi un bilan : ~49 % d'avancement au début de la session.

### 2. Nettoyage
- Supprimé tous les fichiers `*:Zone.Identifier` (résidus Windows/WSL).
- Supprimé `.main.rs.swp` (swap Vim orphelin).
- Vidé `search_engine/src/docs/` (les 12 docs factices `.txt`).

### 3. Support multi-format (PDF/DOCX/ODT)
- Refactor complet de `reader.rs` :
  - `Document` retient désormais `path: String` (avant : juste `id` + `content`).
  - Parcours **récursif** des sous-dossiers.
  - Dispatch par extension : `.txt`, `.html`/`.htm`, `.pdf`, `.docx`, `.odt`.
  - PDF : `pdf_extract::extract_text(path)`.
  - DOCX : `zip` → lit `word/document.xml` → regex `<w:t[^>]*>([^<]*)</w:t>`.
  - ODT : `zip` → lit `content.xml` → regex `<[^>]+>` pour retirer toutes les balises.
  - HTML : regex `<[^>]+>` pour retirer les balises.
- Le **cœur algorithmique n'a PAS été touché** : `inverse_index.rs`, `search.rs`
  et `jesaispas.rs` sont identiques à avant la session.

### 4. CLI minimale
- `main.rs` accepte `<dossier> "<requête>"` en arguments.
- Affiche `[score] chemin` trié par pertinence.

### 5. Interface graphique
- Nouveau fichier `gui.rs` avec une app `eframe::App`.
- Layout : titre + ligne dossier (champ + bouton Parcourir + bouton Indexer) +
  ligne requête (champ + bouton Rechercher) + ligne d'état + zone de résultats
  scrollable.
- Sélecteur de dossier natif via `rfd::FileDialog::new().pick_folder()`.
- Touche **Entrée** dans le champ requête déclenche la recherche.
- `main.rs` : si aucun argument, lance la GUI ; sinon CLI.

### 6. Documentation
- `CLAUDE_searchy.md` : règle « aucune crate » remplacée par une liste contrôlée
  + nouvelle section « Formats de documents supportés ».
- `notice.md` : créé puis ré-écrit avec section GUI + CLI + scénarios de test.

---

## Décisions importantes (et leur *pourquoi*)

### Levée de la règle « aucune crate »
L'utilisateur a explicitement autorisé l'ajout de crates pour les formats de
documents. Raison : parser PDF/DOCX/ODT à la main est hors de portée S4
(DEFLATE + XML + objets PDF avec streams compressés). Le `Cargo.toml` violait
déjà la règle (`regex`, `unicode-normalization`), donc la règle a été
formalisée plutôt qu'inventée. **Ne pas re-réintroduire la règle** dans
`CLAUDE_searchy.md` sans demander.

### Choix eframe pour la GUI
- L'utilisateur voulait « simple et que n'importe qui puisse utiliser ».
- Alternatives envisagées : iced (trop complexe), tauri (trop), serveur web
  + HTML (deux couches à maintenir), fltk (moche).
- eframe = un seul crate, immediate mode, marche sous WSLg.
- Le sélecteur de dossier (`rfd`) est crucial pour le « anyone can use it » :
  taper un chemin absolu est trop hostile pour un non-tech.

### Cœur algorithmique laissé intact
L'utilisateur a explicitement vérifié à un moment que je n'avais pas modifié
l'algorithme central du cahier des charges. **Donc : ne pas refactorer
`inverse_index.rs`, `search.rs`, `jesaispas.rs` sans demander.** L'utilisateur
attache de l'importance à la stabilité du cœur algorithmique vs le cahier des charges.

### `notice.md` est un document vivant
À mettre à jour à **chaque** évolution fonctionnelle. La section « Historique
des modifications » est en bas. L'utilisateur l'a explicitement demandé comme tel.

---

## État vs cahier des charges (estimation au 2026-05-11)

| Module                       | Poids | Avancement | Contribution |
| ---------------------------- | ----- | ---------- | ------------ |
| Ingestion (multi-format)     | 15 %  | ~85 %      | 13 %         |
| Normalisation                | 20 %  | 85 %       | 17 %         |
| Indexation (sans persistence) | 25 %  | 50 %       | 12.5 %       |
| Requêtage (TF-IDF, sans bool / snippet) | 25 % | 55 %  | 14 %         |
| CLI + GUI                    | 10 %  | 90 %       | 9 %          |
| Doc / robustesse / livrables | 5 %   | 40 %       | 2 %          |

**Total ≈ 65 %**

---

## Reste à faire (priorités)

Par ordre d'impact sur la note de soutenance :

1. **Sauvegarde de l'index sur disque** (et chargement).
   Exigé explicitement par le cahier des charges. Choix de format à faire :
   texte custom (`key\tdoc_id:tf,doc_id:tf\n`) ou binaire fait main. **Pas de
   serde** par cohérence avec `CLAUDE_searchy.md`.
2. **Opérateurs booléens AND / OR / NOT** dans la requête.
3. **Snippets** : extrait de texte autour du mot trouvé dans le document (exigé
   par le cahier).
4. **Surbrillance** des mots dans les snippets (cosmétique mais joli en démo).
5. **Indexation asynchrone** dans la GUI (actuellement synchrone — la fenêtre
   gèle sur gros corpus).
6. **Stemming** basique (optionnel d'après le cahier).
7. **BM25** comme alternative à TF-IDF (optionnel d'après le cahier).
8. **README** correct + rapport de soutenance + site web (livrables exigés).
9. Supprimer ou nettoyer `test.rs`.

---

## Commandes utiles

```bash
# build
cd search_engine && cargo build --release

# lancer la GUI
cd search_engine && cargo run --release

# lancer en CLI
cd search_engine && cargo run --release -- ~/corpus "ma requête"

# test rapide sanity
mkdir -p /tmp/searchy_sanity
echo "Le langage Rust est rapide et sûr." > /tmp/searchy_sanity/a.txt
echo "Python est interprété, lent mais pratique." > /tmp/searchy_sanity/b.txt
cargo run --quiet -- /tmp/searchy_sanity "rust"
```

---

## Pièges connus

- L'utilisateur est sous **WSL2** (`Linux 6.6.87.2-microsoft-standard-WSL2`).
  La GUI marche grâce à **WSLg** sous Windows 11. Sous Windows 10 il faudrait
  un serveur X (VcXsrv).
- Les PDF **scannés sans OCR** ne retournent rien. C'est attendu, pas un bug.
- Sur les `.docx` complexes, certaines balises `<w:t xml:space="preserve">`
  sont gérées par la regex `<w:t[^>]*>([^<]*)</w:t>`. Mais si du jour au
  lendemain un docx avec textes vides ou caractères XML échappés casse
  l'extraction, c'est ici qu'il faut regarder.
- Le `fn main()` orphelin dans `test.rs` ne casse pas le build parce qu'il
  n'est pas déclaré comme module dans `main.rs`. Si quelqu'un essaie de faire
  `mod test;`, ça pétera.
- **WSLg / ZINK** : la GUI peut planter au démarrage avec des erreurs
  `libEGL warning` / `MESA: error: ZINK` / `Broken pipe`. Ce n'est pas
  notre code, c'est un bug WSLg connu. Solutions documentées plus haut
  (session 2026-05-12, point 4). Important : l'utilisateur **ne veut pas**
  de fix conditionnel WSL dans le code source — préférer `wsl --shutdown`
  ou le préfixe `LIBGL_ALWAYS_SOFTWARE=1` côté commande.

---

## Style de l'utilisateur

- Écrit en français, fautes de frappe occasionnelles.
- Préfère qu'on **valide les changements importants** avant d'agir (a demandé
  confirmation pour les crates).
- **IMPORTANT (2026-05-12)** : Pose souvent des **questions** qui ne sont PAS
  des demandes d'action (« je suis obligé de… ? », « ça doit marcher comment ? »).
  Y répondre, exposer les options, **et attendre un « applique » / « fais » /
  « vas-y » explicite** avant de toucher au code. Confondre question et ordre
  est une erreur que l'utilisateur a corrigée explicitement.
- Veut que le projet **fonctionne sur tous les ordis** sans config système.
  Refuse les fix conditionnels (du genre « si WSL alors… ») au profit de
  solutions universelles ou de workarounds côté utilisation.
- Apprécie les **explications structurées** (tableaux, sections).
- A **explicitement vérifié** à un moment que l'algorithme central n'avait pas
  été touché → soucieux de rester aligné avec le cahier des charges.
- Demande régulièrement des **récaps** et des **fichiers de doc vivants**
  (`notice.md`, `for_nextime.md`).
