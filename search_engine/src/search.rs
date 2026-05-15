use std::collections::HashMap;
use std::cmp::Ordering;

pub fn parse_query(query: &str) -> Vec<String> {
    let mut words = Vec::<String>::new();

    for part in query.split_whitespace() {
        let clean = part
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect::<String>()
            .to_lowercase();

        if !clean.is_empty() {
            words.push(clean);
        }
    }

    words
}

pub fn idf(
    word: &String,
    index: &HashMap<String, Vec<(i32, i32)>>,
    nb_docs: i32,
) -> f64 {
    if let Some(list) = index.get(word) {
        let df = list.len() as f64;
        if df > 0.0 {
            return (nb_docs as f64 / df).ln() + 1.0;
        }
    }

    0.0
}

pub fn search(
    query: &str,
    index: &HashMap<String, Vec<(i32, i32)>>,
    nb_docs: i32,
) -> Vec<(i32, f64)> {
    let words = parse_query(query);
    let mut scores = HashMap::<i32, f64>::new();

    for word in words {
        let word_idf = idf(&word, index, nb_docs);

        if let Some(list) = index.get(&word) {
            for (doc_id, tf) in list {
                let value = (*tf as f64) * word_idf;

                if let Some(score) = scores.get_mut(doc_id) {
                    *score += value;
                } else {
                    scores.insert(*doc_id, value);
                }
            }
        }
    }

    let mut results = Vec::<(i32, f64)>::new();

    for (doc_id, score) in scores {
        if score > 0.0 {
            results.push((doc_id, score));
        }
    }

    results.sort_by(|a, b| {
        b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal)
    });

    results
}