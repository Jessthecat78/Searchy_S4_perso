use regex::Regex;
use std::collections::HashSet;
use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;

fn stopwords() -> HashSet<&'static str> {
    [
        // Français
        "alors", "au", "aucuns", "aussi", "autre", "avant", "avec", "avoir",
        "bon", "car", "ce", "cela", "ces", "ceux", "chaque", "ci", "comme",
        "comment", "dans", "des", "du", "dedans", "dehors", "depuis", "devrait",
        "doit", "donc", "dos", "droite", "debut", "elle", "elles", "en", "encore",
        "essai", "est", "et", "eu", "fait", "faites", "fois", "font", "force",
        "haut", "hors", "ici", "il", "ils", "je", "juste", "la", "le", "les",
        "leur", "la", "ma", "maintenant", "mais", "mes", "mine", "moins", "mon",
        "mot", "même", "ni", "nommes", "notre", "nous", "nouveaux", "ou",
        "par", "parce", "parole", "pas", "personnes", "peut", "peu", "piece",
        "plupart", "pour", "pourquoi", "quand", "que", "quel", "quelle", "quelles",
        "quels", "qui", "sa", "sans", "ses", "seulement", "si", "sien", "son",
        "sont", "sous", "soyez", "sujet", "sur", "ta", "tandis", "tellement",
        "tels", "tes", "ton", "tous", "tout", "trop", "tres", "tu", "valeur",
        "voie", "voient", "vont", "votre", "vous", "vu", "ça", "etaient", "etat",
        "etions", "ete", "etre", "un", "une", "d", "l", "c", "j", "m", "n", "s",
        "t", "y", "a",

        // Anglais
        "a", "an", "and", "are", "as", "at", "be", "by", "for", "from", "has",
        "he", "in", "is", "it", "its", "of", "on", "that", "the", "to", "was",
        "were", "will", "with", "this", "these", "those", "or", "not", "but",
        "we", "you", "they", "i", "me", "my", "our", "your", "their", "them",
    ]
    .iter()
    .copied()
    .collect()
}

pub fn remove_accents(text: &str) -> String {
    text.nfd()
        .filter(|c| !is_combining_mark(*c))
        .collect()
}

pub fn normalize(text: &str) -> String {
    remove_accents(&text.to_lowercase())
}
pub fn clean_text(text: &str) -> String {
    let re = Regex::new(r"[^a-zA-Z0-9\s]").unwrap();
    re.replace_all(text, " ").to_string()
}
pub fn tokenize(text: &str) -> Vec<String> {
    text.split_whitespace()
        .map(|s| s.to_string())
        .collect()
}
pub fn remove_stopwords(tokens: Vec<String>) -> Vec<String> {
    let stops = stopwords();

    tokens
        .into_iter()
        .filter(|token| {
            let len_ok = token.len() > 1;
            let not_stopword = !stops.contains(token.as_str());
            len_ok && not_stopword
        })
        .collect()
}
pub fn preprocess(text: &str) -> Vec<String> {
    let normalized = normalize(text);
    let cleaned = clean_text(&normalized);
    let tokens = tokenize(&cleaned);
    remove_stopwords(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_accents() {
        let input = "École naïve façade";
        let output = remove_accents(input);
        assert_eq!(output, "Ecole naive facade");
    }

    #[test]
    fn test_normalize() {
        let input = "RUST, C'est Génial !";
        let output = normalize(input);
        assert_eq!(output, "rust, c'est genial !");
    }

    #[test]
    fn test_clean_text() {
        let input = "rust, c'est genial ! 2025.";
        let output = clean_text(input);
        assert_eq!(output, "rust  c est genial   2025 ");
    }

    #[test]
    fn test_tokenize() {
        let input = "rust est rapide";
        let tokens = tokenize(input);
        assert_eq!(tokens, vec!["rust", "est", "rapide"]);
    }

    #[test]
    fn test_remove_stopwords() {
        let tokens = vec![
            "rust".to_string(),
            "est".to_string(),
            "rapide".to_string(),
            "et".to_string(),
            "sur".to_string(),
        ];

        let filtered = remove_stopwords(tokens);
        assert_eq!(filtered, vec!["rust", "rapide"]);
    }

}
