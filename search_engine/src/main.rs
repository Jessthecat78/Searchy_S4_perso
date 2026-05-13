mod gui;
mod images;
mod inverse_index;
mod jesaispas;
mod reader;
mod search;

use std::env;
use std::process;

use inverse_index::inverse_index;
use jesaispas::preprocess;
use reader::read_docs;
use search::search;

fn main() {
    let args: Vec<String> = env::args().collect();

    // Mode CLI si l'utilisateur passe un dossier et une requête.
    // Sinon, on ouvre l'interface graphique.
    if args.len() >= 3 {
        run_cli(&args[1], &args[2]);
    } else {
        if let Err(e) = gui::run() {
            eprintln!("Impossible de lancer l'interface : {e}");
            process::exit(1);
        }
    }
}

fn run_cli(folder: &str, query: &str) {
    let docs = read_docs(folder);
    if docs.is_empty() {
        eprintln!("Aucun document lisible trouvé dans {}", folder);
        process::exit(1);
    }
    println!("{} document(s) indexé(s).", docs.len());

    let mut id_to_path = Vec::<String>::with_capacity(docs.len());
    let mut contents = Vec::<(usize, Vec<String>)>::with_capacity(docs.len());
    for doc in docs {
        id_to_path.push(doc.path);
        contents.push((doc.id, preprocess(&doc.content)));
    }

    let nb_docs = contents.len() as i32;
    let index = inverse_index(contents);

    let results = search(query, &index, nb_docs);
    if results.is_empty() {
        println!("Aucun résultat pour : {:?}", query);
        return;
    }

    println!("\nRésultats pour {:?} :", query);
    for (doc_id, score) in results {
        let path = id_to_path
            .get(doc_id as usize)
            .map(|s| s.as_str())
            .unwrap_or("?");
        println!("  [{:.4}] {}", score, path);
    }
}
