use regex::Regex;
use std::fs;
use std::io::Read;
use std::path::Path;

#[derive(Debug)]
pub struct Document {
    pub id: usize,
    pub path: String,
    pub content: String,
}

pub fn read_docs(folder_path: &str) -> Vec<Document> {
    let mut docs = Vec::new();
    let mut next_id: usize = 0;
    walk(Path::new(folder_path), &mut docs, &mut next_id);
    docs
}

fn walk(dir: &Path, docs: &mut Vec<Document>, next_id: &mut usize) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            walk(&path, docs, next_id);
            continue;
        }

        if !path.is_file() {
            continue;
        }

        if let Some(content) = extract_text(&path) {
            docs.push(Document {
                id: *next_id,
                path: path.to_string_lossy().to_string(),
                content,
            });
            *next_id += 1;
        }
    }
}

fn extract_text(path: &Path) -> Option<String> {
    let ext = path.extension()?.to_string_lossy().to_lowercase();
    match ext.as_str() {
        "txt" => fs::read_to_string(path).ok(),
        "html" | "htm" => {
            let raw = fs::read_to_string(path).ok()?;
            Some(strip_html(&raw))
        }
        "pdf" => match pdf_extract::extract_text(path) {
            Ok(s) => Some(s),
            Err(e) => {
                eprintln!("PDF illisible {}: {}", path.display(), e);
                None
            }
        },
        "docx" => read_docx(path),
        "odt" => read_odt(path),
        _ => None,
    }
}

fn strip_html(text: &str) -> String {
    let re = Regex::new(r"<[^>]+>").unwrap();
    re.replace_all(text, " ").to_string()
}

fn read_docx(path: &Path) -> Option<String> {
    let file = fs::File::open(path).ok()?;
    let mut zip = zip::ZipArchive::new(file).ok()?;
    let mut xml = zip.by_name("word/document.xml").ok()?;
    let mut buf = String::new();
    xml.read_to_string(&mut buf).ok()?;

    // On extrait uniquement le contenu visible des balises <w:t>...</w:t>
    let re = Regex::new(r"<w:t[^>]*>([^<]*)</w:t>").unwrap();
    let mut out = String::new();
    for cap in re.captures_iter(&buf) {
        out.push_str(&cap[1]);
        out.push(' ');
    }
    Some(out)
}

fn read_odt(path: &Path) -> Option<String> {
    let file = fs::File::open(path).ok()?;
    let mut zip = zip::ZipArchive::new(file).ok()?;
    let mut xml = zip.by_name("content.xml").ok()?;
    let mut buf = String::new();
    xml.read_to_string(&mut buf).ok()?;

    // Le texte d'un ODT est entre les balises XML ; on les retire toutes.
    let re = Regex::new(r"<[^>]+>").unwrap();
    Some(re.replace_all(&buf, " ").to_string())
}
