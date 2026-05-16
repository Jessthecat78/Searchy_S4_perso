use std::collections::HashMap;
use std::process::Command;

use eframe::egui;

use crate::images::{collect_images, search_images, ImageEntry};
use crate::inverse_index::inverse_index;
use crate::jesaispas::preprocess;
use crate::reader::read_docs;
use crate::search::search;

pub struct SearchApp {
    folder: String,
    query: String,
    status: String,
    index: Option<HashMap<String, Vec<(i32, i32)>>>,
    id_to_path: Vec<String>,
    nb_docs: i32,
    results: Vec<(String, f64)>,
    // Recherche d'images PNG par nom de fichier (séparée de la recherche texte).
    images: Vec<ImageEntry>,
    image_query: String,
    image_results: Vec<(String, usize)>,
}

impl Default for SearchApp {
    fn default() -> Self {
        Self {
            folder: String::new(),
            query: String::new(),
            status: "Choisis un dossier puis clique sur Indexer.".to_string(),
            index: None,
            id_to_path: Vec::new(),
            nb_docs: 0,
            results: Vec::new(),
            images: Vec::new(),
            image_query: String::new(),
            image_results: Vec::new(),
        }
    }
}

impl eframe::App for SearchApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Drag & drop : un dossier glissé sur la fenêtre remplit le champ "Dossier".
        // Si l'utilisateur lâche un fichier, on prend son dossier parent.
        let dropped_folder = ctx.input(|i| {
            i.raw.dropped_files.iter().find_map(|f| {
                f.path.as_ref().and_then(|p| {
                    if p.is_dir() {
                        Some(p.to_string_lossy().to_string())
                    } else {
                        p.parent().map(|d| d.to_string_lossy().to_string())
                    }
                })
            })
        });
        if let Some(folder) = dropped_folder {
            self.folder = folder;
            self.status = format!("Dossier déposé : {}. Clique sur Indexer.", self.folder);
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(20.0))
            .show(ctx, |ui| {
            ui.heading(
                egui::RichText::new("🔎 Searchy")
                    .color(egui::Color32::from_rgb(120, 170, 250)),
            );
            ui.label("Moteur de recherche local (.txt / .pdf / .docx / .odt / .html) + images .png");
            ui.add_space(16.0);

            ui.horizontal(|ui| {
                ui.label("Dossier :");
                ui.add(
                    egui::TextEdit::singleline(&mut self.folder)
                        .desired_width(600.0)
                        .hint_text("Colle ici le chemin du dossier"),
                );
                if ui.button("📂 Choisir dossier").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.folder = path.to_string_lossy().to_string();
                        self.status = format!("Dossier sélectionné : {}. Clique sur Indexer.", self.folder);
                    }
                }
                if ui.button("📁 Indexer").clicked() {
                    self.do_index();
                }
            });

            ui.add_space(8.0);

            let mut launch_search = false;
            let mut clear_text = false;
            ui.horizontal(|ui| {
                ui.label("Requête texte :");
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.query)
                        .desired_width(600.0)
                        .hint_text("Mots-clés à chercher dans les documents"),
                );
                let enter =
                    response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                if ui.button("🔍 Rechercher").clicked() || enter {
                    launch_search = true;
                }
                if ui
                    .button("✖")
                    .on_hover_text("Effacer la requête et les résultats")
                    .clicked()
                {
                    clear_text = true;
                }
            });
            if clear_text {
                self.query.clear();
                self.results.clear();
            }
            if launch_search {
                self.do_search();
            }

            ui.add_space(8.0);

            // 2ᵉ barre de recherche : images PNG par nom de fichier (pas d'OCR).
            let mut launch_image_search = false;
            let mut clear_image = false;
            ui.horizontal(|ui| {
                ui.label("Requête image :");
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.image_query)
                        .desired_width(600.0)
                        .hint_text("Mots présents dans le nom de fichier .png"),
                );
                let enter =
                    response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                if ui.button("🖼 Rechercher image").clicked() || enter {
                    launch_image_search = true;
                }
                if ui
                    .button("✖")
                    .on_hover_text("Effacer la requête et les résultats")
                    .clicked()
                {
                    clear_image = true;
                }
            });
            if clear_image {
                self.image_query.clear();
                self.image_results.clear();
            }
            if launch_image_search {
                self.do_image_search();
            }

            ui.add_space(12.0);
            ui.separator();
            ui.label(&self.status);
            ui.separator();
            ui.add_space(8.0);

            // Deux colonnes côte à côte : résultats texte à gauche, images à droite.
            ui.columns(2, |cols| {
                cols[0].label("Résultats texte :");
                egui::ScrollArea::vertical()
                    .id_source("text_results")
                    .auto_shrink([false, false])
                    .show(&mut cols[0], |ui| {
                        if self.results.is_empty() {
                            ui.weak("(rien à afficher pour l'instant)");
                        }
                        for (path, score) in &self.results {
                            ui.horizontal(|ui| {
                                ui.monospace(format!("[{:.4}]", score));
                                if ui.link(path).clicked() {
                                    let _ = Command::new("xdg-open").arg(path).spawn();
                                }
                            });
                        }
                    });

                cols[1].label("Résultats images :");
                egui::ScrollArea::vertical()
                    .id_source("image_results")
                    .auto_shrink([false, false])
                    .show(&mut cols[1], |ui| {
                        if self.image_results.is_empty() {
                            ui.weak("(rien à afficher pour l'instant)");
                        }
                        for (path, score) in &self.image_results {
                            ui.horizontal(|ui| {
                                ui.monospace(format!("[{}]", score));
                                if ui.link(path).clicked() {
                                    let _ = Command::new("xdg-open").arg(path).spawn();
                                }
                            });
                        }
                    });
            });
        });
    }
}

impl SearchApp {
    fn do_index(&mut self) {
        if self.folder.trim().is_empty() {
            self.status = "Indique un dossier d'abord.".to_string();
            return;
        }

        self.status = format!("Indexation de {} en cours…", self.folder);
        let docs = read_docs(&self.folder);

        // Les images PNG sont collectées dans le même dossier, en parallèle des
        // documents texte. C'est un index séparé : pas de fusion avec le TF-IDF.
        self.images = collect_images(&self.folder);
        self.image_results.clear();

        if docs.is_empty() && self.images.is_empty() {
            self.status = format!("Aucun document lisible dans {}", self.folder);
            self.index = None;
            self.id_to_path.clear();
            self.nb_docs = 0;
            self.results.clear();
            return;
        }

        self.id_to_path.clear();
        let mut contents = Vec::with_capacity(docs.len());
        for d in docs {
            self.id_to_path.push(d.path);
            contents.push((d.id, preprocess(&d.content)));
        }
        self.nb_docs = contents.len() as i32;
        self.index = if self.nb_docs > 0 {
            Some(inverse_index(contents))
        } else {
            None
        };
        self.results.clear();
        self.status = format!(
            "{} document(s) texte indexé(s), {} image(s) PNG trouvée(s).",
            self.nb_docs,
            self.images.len()
        );
    }

    fn do_image_search(&mut self) {
        if self.images.is_empty() {
            self.status = "Aucune image indexée. Clique sur Indexer d'abord.".to_string();
            self.image_results.clear();
            return;
        }
        if self.image_query.trim().is_empty() {
            self.status = "Saisis une requête image.".to_string();
            return;
        }

        self.image_results = search_images(&self.image_query, &self.images);

        if self.image_results.is_empty() {
            self.status = format!("Aucune image pour : {:?}", self.image_query);
        } else {
            self.status = format!("{} image(s) trouvée(s).", self.image_results.len());
        }
    }

    fn do_search(&mut self) {
        let index = match &self.index {
            Some(i) => i,
            None => {
                self.status = "Aucun index. Clique sur Indexer d'abord.".to_string();
                return;
            }
        };
        if self.query.trim().is_empty() {
            self.status = "Saisis une requête.".to_string();
            return;
        }

        let raw = search(&self.query, index, self.nb_docs);
        self.results.clear();
        for (doc_id, score) in raw {
            let path = self
                .id_to_path
                .get(doc_id as usize)
                .cloned()
                .unwrap_or_else(|| "?".to_string());
            self.results.push((path, score));
        }

        if self.results.is_empty() {
            self.status = format!("Aucun résultat pour : {:?}", self.query);
        } else {
            self.status = format!("{} résultat(s).", self.results.len());
        }
    }
}

fn setup_style(ctx: &egui::Context) {
    use egui::{FontFamily, FontId, TextStyle};
    let mut style = (*ctx.style()).clone();
    style.text_styles = [
        (TextStyle::Heading, FontId::new(32.0, FontFamily::Proportional)),
        (TextStyle::Body, FontId::new(18.0, FontFamily::Proportional)),
        (TextStyle::Button, FontId::new(18.0, FontFamily::Proportional)),
        (TextStyle::Monospace, FontId::new(16.0, FontFamily::Monospace)),
        (TextStyle::Small, FontId::new(14.0, FontFamily::Proportional)),
    ]
    .into();
    style.spacing.button_padding = egui::vec2(12.0, 8.0);
    style.spacing.item_spacing = egui::vec2(10.0, 8.0);
    style.spacing.interact_size.y = 32.0;
    ctx.set_style(style);
}

pub fn run() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1100.0, 750.0]).with_drag_and_drop(true),
        ..Default::default()
    };
    eframe::run_native(
        "Searchy",
        options,
        Box::new(|cc| {
            setup_style(&cc.egui_ctx);
            Box::<SearchApp>::default()
        }),
    )
}
