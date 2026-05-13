use std::fs;
use std::path::Path;

// Une image indexée = son chemin complet + les "mots" de son nom de fichier.
// Exemple : "carte_lyon_2024.png" -> tokens = ["carte", "lyon", "2024"]
#[derive(Debug)]
pub struct ImageEntry {
    pub path: String,
    pub tokens: Vec<String>,
}

// Parcourt récursivement le dossier et collecte tous les .png trouvés.
pub fn collect_images(folder_path: &str) -> Vec<ImageEntry> {
    let mut images = Vec::new();
    walk(Path::new(folder_path), &mut images);
    images
}

fn walk(dir: &Path, out: &mut Vec<ImageEntry>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            walk(&path, out);
            continue;
        }

        if !path.is_file() {
            continue;
        }

        let ext = match path.extension() {
            Some(e) => e.to_string_lossy().to_lowercase(),
            None => continue,
        };
        if ext != "png" {
            continue;
        }

        let stem = path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();

        out.push(ImageEntry {
            path: path.to_string_lossy().to_string(),
            tokens: tokenize_name(&stem),
        });
    }
}

// Découpe un nom de fichier en mots minuscules : tout ce qui n'est pas
// alphanumérique sert de séparateur (espaces, "_", "-", ".", etc.).
fn tokenize_name(name: &str) -> Vec<String> {
    name.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

// Recherche très simple : pour chaque image, on compte combien de mots
// de la requête correspondent à un token de son nom (sous-chaîne acceptée).
// Le score = nombre de mots de la requête qui ont matché.
pub fn search_images(query: &str, images: &[ImageEntry]) -> Vec<(String, usize)> {
    let q_tokens = tokenize_name(query);
    if q_tokens.is_empty() {
        return Vec::new();
    }

    let mut scored: Vec<(String, usize)> = Vec::new();
    for img in images {
        let mut score = 0;
        for qt in &q_tokens {
            for nt in &img.tokens {
                if nt.contains(qt.as_str()) {
                    score += 1;
                    break;
                }
            }
        }
        if score > 0 {
            scored.push((img.path.clone(), score));
        }
    }

    // Plus de mots de la requête matchés = meilleur score, on trie décroissant.
    scored.sort_by(|a, b| b.1.cmp(&a.1));
    scored
}
