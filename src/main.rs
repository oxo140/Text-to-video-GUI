#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use text_video_studio::{export, render, updates};

use eframe::egui::{self, Color32, RichText, Vec2};
use render::{Project, Scene};
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver},
        Arc,
    },
    time::{Duration, Instant},
};

const ACCENT: Color32 = Color32::from_rgb(110, 172, 240);
const MUTED: Color32 = Color32::from_rgb(166, 183, 205);
const PANEL: Color32 = Color32::from_rgb(18, 39, 67);
const RESOLUTIONS: &[(&str, u32, u32)] = &[
    ("Bandeau d’origine · 1280 × 180", 1280, 180),
    ("SD · 854 × 480", 854, 480),
    ("HD · 1280 × 720", 1280, 720),
    ("Full HD · 1920 × 1080", 1920, 1080),
    ("QHD · 2560 × 1440", 2560, 1440),
    ("4K UHD · 3840 × 2160", 3840, 2160),
    ("Vertical · 1080 × 1920", 1080, 1920),
    ("Carré · 1080 × 1080", 1080, 1080),
];

fn writable_export_directory() -> Option<PathBuf> {
    let mut candidates = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            candidates.push(parent.join("exports"));
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join("exports"));
    }
    candidates.push(
        std::env::temp_dir()
            .join("Text Video Studio")
            .join("exports"),
    );
    candidates.into_iter().find(|directory| {
        std::fs::create_dir_all(directory).is_ok() && tempfile::tempfile_in(directory).is_ok()
    })
}

struct Studio {
    project: Project,
    scene: Scene,
    texture: Option<egui::TextureHandle>,
    play: bool,
    time: f32,
    last: Instant,
    receiver: Option<Receiver<export::Event>>,
    cancel: Arc<AtomicBool>,
    progress: f32,
    message: String,
    output: Option<PathBuf>,
    ffmpeg: bool,
    custom_size: bool,
    export_directory: Option<PathBuf>,
    update_receiver: Option<Receiver<UpdateEvent>>,
    update_candidate: Option<updates::Update>,
    update_message: String,
    update_installed: bool,
}

enum UpdateEvent {
    Checked(Result<Option<updates::Update>, String>),
    Installed(Result<(), String>),
}

impl Studio {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut style = (*cc.egui_ctx.style()).clone();
        style.visuals = egui::Visuals::dark();
        style.visuals.panel_fill = Color32::from_rgb(9, 25, 45);
        style.visuals.window_fill = PANEL;
        style.visuals.extreme_bg_color = Color32::from_rgb(9, 25, 45);
        style.visuals.selection.bg_fill = Color32::from_rgb(31, 79, 137);
        style.visuals.override_text_color = Some(Color32::from_rgb(238, 243, 250));
        style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(32, 56, 87);
        style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(41, 77, 121);
        style.visuals.widgets.active.bg_fill = ACCENT;
        style.spacing.item_spacing = Vec2::new(12.0, 12.0);
        style.spacing.button_padding = Vec2::new(16.0, 10.0);
        style
            .text_styles
            .insert(egui::TextStyle::Body, egui::FontId::proportional(15.0));
        style
            .text_styles
            .insert(egui::TextStyle::Button, egui::FontId::proportional(15.0));
        cc.egui_ctx.set_style(style);
        // Utilise la police système sans la redistribuer avec le portable.
        if let Some(windows) = std::env::var_os("WINDIR") {
            if let Ok(bytes) =
                std::fs::read(PathBuf::from(windows).join("Fonts").join("segoeui.ttf"))
            {
                let mut fonts = egui::FontDefinitions::default();
                fonts
                    .font_data
                    .insert("interface".into(), egui::FontData::from_owned(bytes));
                fonts
                    .families
                    .entry(egui::FontFamily::Proportional)
                    .or_default()
                    .insert(0, "interface".into());
                cc.egui_ctx.set_fonts(fonts);
            }
        }
        let mut project: Project = cc
            .storage
            .and_then(|s| eframe::get_value(s, "project"))
            .unwrap_or_default();
        if project.validate().is_err() {
            project = Project::default();
        }
        if project.text == "Les grandes idées commencent par quelques mots." {
            project.text = Project::default().text;
        }
        if project.foreground == [232, 225, 255] && project.background == [22, 19, 38] {
            project.foreground = Project::default().foreground;
            project.background = Project::default().background;
        }
        let scene = Scene::new(&project);
        let custom_size = cc
            .storage
            .and_then(|s| eframe::get_value(s, "custom_size"))
            .unwrap_or_else(|| {
                !RESOLUTIONS
                    .iter()
                    .any(|&(_, w, h)| project.width == w && project.height == h)
            });
        let export_directory: Option<PathBuf> = cc
            .storage
            .and_then(|s| eframe::get_value(s, "export_directory"));
        Self {
            project,
            scene,
            texture: None,
            play: true,
            time: 2.2,
            last: Instant::now(),
            receiver: None,
            cancel: Arc::new(AtomicBool::new(false)),
            progress: 0.0,
            message: String::new(),
            output: None,
            ffmpeg: export::available(),
            custom_size,
            export_directory,
            update_receiver: None,
            update_candidate: None,
            update_message: String::new(),
            update_installed: false,
        }
    }

    fn start_export(&mut self) {
        if let Err(e) = self.project.validate() {
            self.message = e;
            return;
        }
        let ext = if self.project.webm { "webm" } else { "mp4" };
        let directory = self
            .export_directory
            .clone()
            .filter(|p| p.is_dir() && tempfile::tempfile_in(p).is_ok())
            .or_else(writable_export_directory);
        let Some(directory) = directory else {
            self.message = "Aucun dossier accessible en écriture. Choisissez un emplacement accessible après avoir relancé l’application normalement.".into();
            return;
        };
        let Some(mut path) = rfd::FileDialog::new()
            .set_title("Exporter la vidéo")
            .set_directory(directory)
            .add_filter("Vidéo", &[ext])
            .set_file_name(format!("mon-texte.{ext}"))
            .save_file()
        else {
            return;
        };
        path.set_extension(ext);
        self.export_directory = path.parent().map(|p| p.to_path_buf());
        self.cancel = Arc::new(AtomicBool::new(false));
        self.progress = 0.0;
        self.output = None;
        self.message = "Encodage en cours…".into();
        let (tx, rx) = mpsc::channel();
        self.receiver = Some(rx);
        let p = self.project.clone();
        let cancel = self.cancel.clone();
        std::thread::spawn(move || {
            let result = export::run(&p, &path, &cancel, &tx);
            let _ = tx.send(export::Event::Finished(result));
        });
    }

    fn controls(&mut self, ui: &mut egui::Ui) {
        ui.label(
            RichText::new("01   COMPOSITION")
                .color(ACCENT)
                .size(11.0)
                .strong(),
        );
        ui.add_space(6.0);
        ui.heading("Paramètres du texte");
        ui.add_space(10.0);
        ui.label("Texte à faire défiler");
        ui.add(
            egui::TextEdit::multiline(&mut self.project.text)
                .desired_rows(5)
                .desired_width(f32::INFINITY)
                .char_limit(2000)
                .hint_text("Saisir le texte à afficher"),
        );
        ui.label(
            RichText::new(format!(
                "{} / 2 000 caractères · défilement vers la gauche",
                self.project.text.chars().count()
            ))
            .small()
            .color(MUTED),
        );
        ui.add_space(8.0);
        ui.separator();
        ui.label(
            RichText::new("02   APPARENCE")
                .color(ACCENT)
                .size(11.0)
                .strong(),
        );
        ui.horizontal(|ui| {
            for (name, fg, bg) in [
                ("Marine", [245, 247, 252], [12, 32, 60]),
                ("Clair", [17, 40, 72], [245, 247, 250]),
                ("Contraste", [255, 255, 255], [0, 0, 0]),
            ] {
                if ui.small_button(name).clicked() {
                    self.project.foreground = fg;
                    self.project.background = bg;
                }
            }
        });
        ui.horizontal(|ui| {
            ui.color_edit_button_srgb(&mut self.project.foreground);
            ui.label("Texte");
            ui.add_space(20.0);
            ui.color_edit_button_srgb(&mut self.project.background);
            ui.label("Fond");
        });
        ui.add(egui::Slider::new(&mut self.project.size, 24.0..=240.0).text("Taille · px"));
        ui.add(egui::Slider::new(&mut self.project.speed, 60.0..=720.0).text("Vitesse · px/s"));
        ui.add_space(8.0);
        ui.separator();
        ui.label(
            RichText::new("03   FORMAT DE SORTIE")
                .color(ACCENT)
                .size(11.0)
                .strong(),
        );
        egui::ComboBox::from_id_salt("resolution")
            .selected_text(if self.custom_size {
                "Personnalisé".to_owned()
            } else {
                RESOLUTIONS
                    .iter()
                    .find(|&&(_, w, h)| self.project.width == w && self.project.height == h)
                    .map(|&(name, _, _)| name.to_owned())
                    .unwrap_or_else(|| "Personnalisé".into())
            })
            .width(240.0)
            .show_ui(ui, |ui| {
                for &(name, w, h) in RESOLUTIONS {
                    if ui
                        .selectable_label(
                            !self.custom_size
                                && self.project.width == w
                                && self.project.height == h,
                            name,
                        )
                        .clicked()
                    {
                        self.project.width = w;
                        self.project.height = h;
                        self.custom_size = false;
                    }
                }
                ui.selectable_value(&mut self.custom_size, true, "Personnalisé…");
            });
        if self.custom_size {
            ui.horizontal(|ui| {
                ui.label("Largeur");
                ui.add(
                    egui::DragValue::new(&mut self.project.width)
                        .range(64..=3840)
                        .speed(2)
                        .suffix(" px"),
                );
            });
            ui.horizontal(|ui| {
                ui.label("Hauteur");
                ui.add(
                    egui::DragValue::new(&mut self.project.height)
                        .range(64..=2160)
                        .speed(2)
                        .suffix(" px"),
                );
            });
            ui.label(
                RichText::new("Dimensions paires · 64 × 64 à 3840 × 2160")
                    .small()
                    .color(MUTED),
            );
            if !self.project.width.is_multiple_of(2) || !self.project.height.is_multiple_of(2) {
                ui.colored_label(
                    Color32::LIGHT_RED,
                    "La largeur et la hauteur doivent être paires.",
                );
            }
        }
        ui.horizontal(|ui| {
            ui.label("Images / seconde");
            for fps in [24, 30, 60] {
                ui.selectable_value(&mut self.project.fps, fps, fps.to_string());
            }
        });
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.project.webm, false, "MP4 · H.264");
            ui.selectable_value(&mut self.project.webm, true, "WebM · VP9");
        });
    }
}

impl eframe::App for Studio {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        if let Some(event) = self
            .update_receiver
            .as_ref()
            .and_then(|rx| rx.try_recv().ok())
        {
            self.update_receiver = None;
            match event {
                UpdateEvent::Checked(Ok(Some(candidate))) => {
                    self.update_message = format!("Version {} disponible.", candidate.version);
                    self.update_candidate = Some(candidate);
                }
                UpdateEvent::Checked(Ok(None)) => {
                    self.update_message = "Aucune mise à jour disponible.".into();
                    self.update_candidate = None;
                }
                UpdateEvent::Checked(Err(e)) | UpdateEvent::Installed(Err(e)) => {
                    self.update_message = e
                }
                UpdateEvent::Installed(Ok(())) => {
                    self.update_installed = true;
                    self.update_candidate = None;
                    self.update_message = "Mise à jour installée. Redémarrez l’application.".into();
                }
            }
        }
        let mut done = None;
        if let Some(rx) = &self.receiver {
            while let Ok(event) = rx.try_recv() {
                match event {
                    export::Event::Progress(p) => self.progress = p,
                    export::Event::Finished(r) => done = Some(r),
                }
            }
        }
        if let Some(result) = done {
            self.receiver = None;
            match result {
                Ok(path) => {
                    self.progress = 1.0;
                    self.message = "Export terminé.".into();
                    self.output = Some(path);
                }
                Err(e) => self.message = e,
            }
        }
        let elapsed = self.last.elapsed().as_secs_f32().min(0.1);
        self.last = Instant::now();
        if self.play {
            self.time = (self.time + elapsed) % self.scene.duration(&self.project);
        }
        egui::TopBottomPanel::top("header")
            .frame(
                egui::Frame::none()
                    .fill(PANEL)
                    .inner_margin(egui::Margin::symmetric(26.0, 18.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("T / V").size(24.0).strong().color(ACCENT));
                    ui.add_space(10.0);
                    ui.label(RichText::new("Text Video Studio").size(22.0).strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new("ÉDITEUR DE TEXTE VIDÉO")
                                .color(MUTED)
                                .size(11.0),
                        );
                    });
                });
            });
        egui::TopBottomPanel::bottom("footer")
            .frame(
                egui::Frame::none()
                    .fill(PANEL)
                    .inner_margin(egui::Margin::symmetric(24.0, 12.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(if self.ffmpeg {
                            "● FFmpeg prêt"
                        } else {
                            "● FFmpeg introuvable"
                        })
                        .color(if self.ffmpeg {
                            Color32::from_rgb(133, 214, 169)
                        } else {
                            Color32::LIGHT_RED
                        }),
                    );
                    if !self.ffmpeg && ui.small_button("Revérifier").clicked() {
                        self.ffmpeg = export::available();
                    }
                    ui.label(RichText::new("Traitement local").color(MUTED).small());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if self.update_installed {
                            if ui
                                .add_enabled(
                                    self.receiver.is_none(),
                                    egui::Button::new("Redémarrer"),
                                )
                                .clicked()
                            {
                                match std::env::current_exe()
                                    .and_then(|exe| std::process::Command::new(exe).spawn())
                                {
                                    Ok(_) => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
                                    Err(e) => {
                                        self.update_message =
                                            format!("Redémarrage impossible : {e}")
                                    }
                                }
                            }
                        } else if let Some(candidate) = self.update_candidate.clone() {
                            if ui
                                .add_enabled(
                                    self.update_receiver.is_none() && self.receiver.is_none(),
                                    egui::Button::new(format!(
                                        "Installer la version {}",
                                        candidate.version
                                    )),
                                )
                                .clicked()
                            {
                                let (tx, rx) = mpsc::channel();
                                self.update_receiver = Some(rx);
                                self.update_message =
                                    "Téléchargement et vérification de la mise à jour…".into();
                                std::thread::spawn(move || {
                                    let _ = tx.send(UpdateEvent::Installed(updates::install(
                                        &candidate,
                                        updates::Edition::Gui,
                                    )));
                                });
                            }
                        } else if ui
                            .add_enabled(
                                self.update_receiver.is_none() && self.receiver.is_none(),
                                egui::Button::new("Vérifier les mises à jour"),
                            )
                            .clicked()
                        {
                            let (tx, rx) = mpsc::channel();
                            self.update_receiver = Some(rx);
                            self.update_message = "Recherche sur GitHub…".into();
                            std::thread::spawn(move || {
                                let _ = tx.send(UpdateEvent::Checked(updates::check(
                                    updates::Edition::Gui,
                                )));
                            });
                        }
                        ui.label(
                            RichText::new(format!("v{}", updates::VERSION))
                                .small()
                                .color(MUTED),
                        );
                    });
                });
                if !self.update_message.is_empty() {
                    ui.label(RichText::new(&self.update_message).small().color(MUTED));
                }
            });
        egui::SidePanel::left("controls")
            .exact_width(365.0)
            .resizable(false)
            .frame(egui::Frame::none().fill(PANEL).inner_margin(22.0))
            .show(ctx, |ui| {
                let before = self.project.clone();
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add_enabled_ui(self.receiver.is_none(), |ui| self.controls(ui));
                });
                if before.text != self.project.text || before.size != self.project.size {
                    self.scene = Scene::new(&self.project);
                }
            });
        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(Color32::from_rgb(9, 25, 45))
                    .inner_margin(28.0),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Aperçu");
                    ui.label(RichText::new("EN DIRECT").color(ACCENT).small());
                });
                ui.add_space(14.0);
                let available = ui.available_size();
                let ratio = self.project.width as f32 / self.project.height as f32;
                let w = available.x.min((available.y - 245.0).max(160.0) * ratio);
                let size = Vec2::new(w, w / ratio);
                let rw = w.clamp(64.0, 960.0) as usize;
                let rh = (rw as f32 / ratio).max(1.0) as usize;
                let pixels = self.scene.frame(&self.project, self.time, rw, rh);
                let image = egui::ColorImage::from_rgb([rw, rh], &pixels);
                if let Some(texture) = &mut self.texture {
                    texture.set(image, egui::TextureOptions::LINEAR);
                } else {
                    self.texture =
                        Some(ctx.load_texture("preview", image, egui::TextureOptions::LINEAR));
                }
                ui.vertical_centered(|ui| {
                    ui.add(
                        egui::Image::new(self.texture.as_ref().unwrap())
                            .fit_to_exact_size(size)
                            .rounding(12.0),
                    );
                });
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui
                        .button(if self.play {
                            "Ⅱ  Pause"
                        } else {
                            "▶  Lecture"
                        })
                        .clicked()
                    {
                        self.play = !self.play;
                    }
                    if ui.button("↺").on_hover_text("Recommencer").clicked() {
                        self.time = 0.0;
                    }
                    let duration = self.scene.duration(&self.project);
                    ui.add(egui::Slider::new(&mut self.time, 0.0..=duration).show_value(false));
                    ui.label(
                        RichText::new(format!("{:.1} / {:.1} s", self.time, duration)).color(MUTED),
                    );
                });
                ui.add_space(10.0);
                ui.separator();
                ui.label(
                    RichText::new(format!(
                        "{} × {}   /   {} fps   /   {}   /   {:.1} secondes",
                        self.project.width,
                        self.project.height,
                        self.project.fps,
                        if self.project.webm { "WEBM" } else { "MP4" },
                        self.scene.duration(&self.project)
                    ))
                    .color(MUTED)
                    .small(),
                );
                if self.receiver.is_some() {
                    ui.add(
                        egui::ProgressBar::new(self.progress)
                            .show_percentage()
                            .animate(true),
                    );
                    if ui.button("Annuler l’export").clicked() {
                        self.cancel.store(true, Ordering::Relaxed);
                    }
                } else if ui
                    .add_enabled(
                        self.ffmpeg && self.update_receiver.is_none(),
                        egui::Button::new(
                            RichText::new("Exporter la vidéo  ↗")
                                .color(Color32::from_rgb(9, 25, 45))
                                .strong(),
                        )
                        .fill(ACCENT)
                        .min_size(Vec2::new(220.0, 46.0)),
                    )
                    .clicked()
                {
                    self.start_export();
                }
                if !self.message.is_empty() {
                    ui.label(&self.message);
                }
                if let Some(path) = &self.output {
                    ui.label(
                        RichText::new(path.display().to_string())
                            .small()
                            .color(MUTED),
                    );
                    ui.horizontal(|ui| {
                        if ui.button("Lire la vidéo").clicked() {
                            if let Err(e) = open_path(path) {
                                self.message = e;
                            }
                        }
                        if ui.button("Ouvrir le dossier").clicked() {
                            if let Some(parent) = path.parent() {
                                if let Err(e) = open_path(parent) {
                                    self.message = e;
                                }
                            }
                        }
                    });
                }
            });
        if self.play || self.receiver.is_some() || self.update_receiver.is_some() {
            ctx.request_repaint_after(Duration::from_millis(33));
        }
    }
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, "project", &self.project);
        eframe::set_value(storage, "custom_size", &self.custom_size);
        eframe::set_value(storage, "export_directory", &self.export_directory);
    }
    fn on_exit(&mut self, _: Option<&eframe::glow::Context>) {
        self.cancel.store(true, Ordering::Relaxed);
    }
}

fn open_path(path: &std::path::Path) -> Result<(), String> {
    #[cfg(windows)]
    let program = "explorer.exe";
    #[cfg(target_os = "macos")]
    let program = "open";
    #[cfg(all(unix, not(target_os = "macos")))]
    let program = "xdg-open";
    export::command(program)
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("Ouverture impossible : {e}"))
}

fn main() -> eframe::Result<()> {
    if let Some(path) = std::env::args()
        .nth(1)
        .filter(|_| std::env::args().any(|a| a == "--smoke-test"))
    {
        let p = Project {
            text: "Été à Paris !".into(),
            width: 320,
            height: 180,
            size: 32.0,
            speed: 720.0,
            webm: path.ends_with(".webm"),
            ..Project::default()
        };
        let (tx, _) = mpsc::channel();
        if let Err(e) = export::run(
            &p,
            &PathBuf::from(path),
            &Arc::new(AtomicBool::new(false)),
            &tx,
        ) {
            eprintln!("{e}");
            std::process::exit(1);
        }
        return Ok(());
    }
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 660.0])
            .with_min_inner_size([960.0, 620.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Text Video Studio",
        options,
        Box::new(|cc| Ok(Box::new(Studio::new(cc)))),
    )
}
