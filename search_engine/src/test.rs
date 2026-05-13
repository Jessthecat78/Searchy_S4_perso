 mod inverse_index;
 mod search;
use search::search;
use inverse_index::inverse_index;

fn main() {
    let files = vec![
        vec![
            "rust".to_string(),
            "fast".to_string(),
            "safe".to_string(),
            "rust".to_string(),
        ],
        vec![
            "memory".to_string(),
            "safe".to_string(),
            "language".to_string(),
        ],
        vec![
            "rust".to_string(),
            "memory".to_string(),
            "system".to_string(),
        ],
    ];

    let index = inverse_index(files);

    let results = search("rust memory", &index, 3);

    for (doc_id, score) in results {
        println!("doc {} -> {}", doc_id, score);
    }
}
