use clap::{ArgGroup, Args, Parser, Subcommand, ValueEnum};
use std::{
    io::{self, Write},
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
};
use text_video_studio::{
    export::{self, Event},
    render::Project,
    updates::{self, Edition},
};

#[derive(Parser)]
#[command(
    name = "text-video-cmd",
    version,
    about = "Création de texte vidéo en ligne de commande",
    after_help = "Sans argument : assistant interactif. FFmpeg est requis dans le PATH ou à côté de cet exécutable."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Action>,
}
#[derive(Subcommand)]
enum Action {
    /// Générer une vidéo MP4 ou WebM
    Render(RenderArgs),
    /// Saisir les paramètres dans le terminal
    Interactive,
    /// Rechercher et installer une mise à jour publique GitHub, sans token
    Update {
        /// Rechercher uniquement, sans modifier l’exécutable
        #[arg(long)]
        check: bool,
    },
}
#[derive(Clone, Copy, ValueEnum)]
enum Format {
    Mp4,
    Webm,
}
#[derive(Clone, Copy, ValueEnum)]
enum Preset {
    Bandeau,
    Sd,
    Hd,
    FullHd,
    Qhd,
    Uhd,
    Vertical,
    Square,
}
impl Preset {
    fn dimensions(self) -> (u32, u32) {
        match self {
            Self::Bandeau => (1280, 180),
            Self::Sd => (854, 480),
            Self::Hd => (1280, 720),
            Self::FullHd => (1920, 1080),
            Self::Qhd => (2560, 1440),
            Self::Uhd => (3840, 2160),
            Self::Vertical => (1080, 1920),
            Self::Square => (1080, 1080),
        }
    }
}
#[derive(Args)]
#[command(group(ArgGroup::new("input").required(true).args(["text", "text_file"])))]
struct RenderArgs {
    /// Texte à afficher
    #[arg(long, conflicts_with = "text_file")]
    text: Option<String>,
    /// Fichier texte UTF-8
    #[arg(long, conflicts_with = "text")]
    text_file: Option<PathBuf>,
    /// Fichier vidéo de destination (jamais écrasé)
    #[arg(short, long)]
    output: PathBuf,
    #[arg(long, value_enum)]
    format: Option<Format>,
    #[arg(long, value_enum, default_value = "bandeau")]
    preset: Preset,
    /// Largeur personnalisée en pixels (paire)
    #[arg(long)]
    width: Option<u32>,
    /// Hauteur personnalisée en pixels (paire)
    #[arg(long)]
    height: Option<u32>,
    #[arg(long, default_value_t = 60)]
    fps: u32,
    /// Vitesse en pixels par seconde
    #[arg(long, default_value_t = 240.0)]
    speed: f32,
    /// Taille du texte en pixels
    #[arg(long, default_value_t = 96.0)]
    size: f32,
    /// Couleur du texte en hexadécimal RGB
    #[arg(long, value_parser = color, default_value = "000000")]
    foreground: [u8; 3],
    /// Couleur du fond en hexadécimal RGB
    #[arg(long, value_parser = color, default_value = "FFFFFF")]
    background: [u8; 3],
}

fn color(value: &str) -> Result<[u8; 3], String> {
    let value = value.strip_prefix('#').unwrap_or(value);
    if value.len() != 6 || !value.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("Couleur attendue : RRGGBB, par exemple 112848.".into());
    }
    let n = u32::from_str_radix(value, 16).map_err(|e| e.to_string())?;
    Ok([(n >> 16) as u8, (n >> 8) as u8, n as u8])
}

fn render(args: RenderArgs) -> Result<(), String> {
    let text = match (args.text, args.text_file) {
        (Some(text), None) => text,
        (None, Some(path)) => std::fs::read_to_string(path)
            .map_err(|e| format!("Lecture du texte : {e}"))?
            .trim_start_matches('\u{feff}')
            .to_owned(),
        _ => return Err("Fournir --text ou --text-file.".into()),
    };
    let (w, h) = args.preset.dimensions();
    let inferred = args
        .output
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase);
    let format = args
        .format
        .unwrap_or(if inferred.as_deref() == Some("webm") {
            Format::Webm
        } else {
            Format::Mp4
        });
    let ext = match format {
        Format::Mp4 => "mp4",
        Format::Webm => "webm",
    };
    if inferred.as_deref().is_some_and(|e| e != ext) {
        return Err(format!("L’extension de sortie doit être .{ext}."));
    }
    let mut output = args.output;
    output.set_extension(ext);
    let project = Project {
        text,
        width: args.width.unwrap_or(w),
        height: args.height.unwrap_or(h),
        fps: args.fps,
        speed: args.speed,
        size: args.size,
        foreground: args.foreground,
        background: args.background,
        webm: matches!(format, Format::Webm),
    };
    project.validate()?;
    if !export::available() {
        return Err(
            "FFmpeg introuvable. Installez-le ou placez ffmpeg.exe à côté de l’application.".into(),
        );
    }
    if let Some(parent) = output.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).map_err(|e| format!("Création du dossier : {e}"))?;
    }
    let cancel = Arc::new(AtomicBool::new(false));
    let signal = cancel.clone();
    ctrlc::set_handler(move || signal.store(true, Ordering::Relaxed)).map_err(|e| e.to_string())?;
    let (tx, rx) = mpsc::channel();
    let worker = std::thread::spawn(move || {
        let result = export::run(&project, &output, &cancel, &tx);
        let _ = tx.send(Event::Finished(result));
    });
    let mut result = Err("Le traitement s’est interrompu.".into());
    for event in rx {
        match event {
            Event::Progress(p) => {
                eprint!("\rExport : {:3.0} %", p * 100.0);
                let _ = io::stderr().flush();
            }
            Event::Finished(r) => {
                eprintln!();
                result = r.map(|path| println!("Export terminé : {}", path.display()));
                break;
            }
        }
    }
    worker
        .join()
        .map_err(|_| "Erreur du moteur de rendu.".to_owned())?;
    result
}

fn prompt(label: &str, default: &str) -> Result<String, String> {
    print!(
        "{label}{} : ",
        if default.is_empty() {
            String::new()
        } else {
            format!(" [{default}]")
        }
    );
    io::stdout().flush().map_err(|e| e.to_string())?;
    let mut line = String::new();
    if io::stdin()
        .read_line(&mut line)
        .map_err(|e| e.to_string())?
        == 0
    {
        return Err("Saisie interrompue.".into());
    }
    let value = line.trim();
    Ok(if value.is_empty() {
        default.into()
    } else {
        value.into()
    })
}
fn number<T: std::str::FromStr>(label: &str, default: &str) -> Result<T, String> {
    prompt(label, default)?
        .parse()
        .map_err(|_| format!("Valeur incorrecte : {label}."))
}
fn interactive() -> Result<(), String> {
    println!("Text Video Studio — Console {}", updates::VERSION);
    let text = prompt("Texte à afficher", "")?;
    let width = number("Largeur en pixels", "1280")?;
    let height = number("Hauteur en pixels", "180")?;
    let fps = number("Images par seconde (24, 30, 60)", "60")?;
    let speed = number("Vitesse en pixels par seconde", "240")?;
    let size = number("Taille du texte en pixels", "96")?;
    let foreground = color(&prompt("Couleur du texte (RRGGBB)", "000000")?)?;
    let background = color(&prompt("Couleur du fond (RRGGBB)", "FFFFFF")?)?;
    let output = PathBuf::from(prompt("Fichier de sortie (.mp4 ou .webm)", "texte.mp4")?);
    render(RenderArgs {
        text: Some(text),
        text_file: None,
        output,
        format: None,
        preset: Preset::Bandeau,
        width: Some(width),
        height: Some(height),
        fps,
        speed,
        size,
        foreground,
        background,
    })
}
fn run() -> Result<(), String> {
    match Cli::parse().command {
        None | Some(Action::Interactive) => interactive(),
        Some(Action::Render(args)) => render(args),
        Some(Action::Update { check }) => {
            println!(
                "Version installée : {} · dépôt public {}",
                updates::VERSION,
                updates::REPOSITORY
            );
            if let Some(update) = updates::check(Edition::Cmd)? {
                println!("Version disponible : {}", update.version);
                if !check {
                    updates::install(&update, Edition::Cmd)?;
                    println!("Mise à jour installée. Relancez la commande.");
                }
            } else {
                println!("Aucune mise à jour disponible.");
            }
            Ok(())
        }
    }
}
fn main() {
    if let Err(e) = run() {
        eprintln!("Erreur : {e}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn colors_and_command_arguments() {
        assert_eq!(color("#112848").unwrap(), [17, 40, 72]);
        assert!(color("ééé").is_err());
        assert!(Cli::try_parse_from([
            "cmd",
            "render",
            "--text",
            "a",
            "--text-file",
            "a.txt",
            "-o",
            "a.mp4"
        ])
        .is_err());
        assert!(Cli::try_parse_from(["cmd", "render", "-o", "a.mp4"]).is_err());
        assert!(Cli::try_parse_from([
            "cmd", "render", "--text", "Été", "-o", "a.mp4", "--width", "640", "--height", "128"
        ])
        .is_ok());
    }
}
