use crate::render::{Project, Scene};
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::Sender,
        Arc,
    },
    thread,
};

pub enum Event {
    Progress(f32),
    Finished(Result<PathBuf, String>),
}

pub fn command(program: &str) -> Command {
    let bundled = if program == "ffmpeg" {
        std::env::current_exe()
            .ok()
            .and_then(|exe| {
                exe.parent().map(|p| {
                    p.join(if cfg!(windows) {
                        "ffmpeg.exe"
                    } else {
                        "ffmpeg"
                    })
                })
            })
            .filter(|p| p.is_file())
    } else {
        None
    };
    let mut c = Command::new(bundled.as_deref().unwrap_or_else(|| Path::new(program)));
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        c.creation_flags(0x08000000);
    }
    c
}

pub fn available() -> bool {
    command("ffmpeg")
        .args(["-version"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

pub fn run(
    p: &Project,
    destination: &Path,
    cancel: &Arc<AtomicBool>,
    tx: &Sender<Event>,
) -> Result<PathBuf, String> {
    p.validate()?;
    if destination.exists() {
        return Err("Ce fichier existe déjà. Choisissez un autre nom pour le conserver.".into());
    }
    let parent = destination
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let temporary = tempfile::Builder::new()
        .prefix(".text-video-")
        .suffix(".partial")
        .tempfile_in(parent)
        .map_err(|e| format!("Dossier de sortie inaccessible : {e}"))?;
    let scene = Scene::new(p);
    let total = (scene.duration(p) * p.fps as f32).ceil() as u32;
    let mut cmd = command("ffmpeg");
    cmd.args([
        "-hide_banner",
        "-loglevel",
        "error",
        "-y",
        "-f",
        "rawvideo",
        "-pixel_format",
        "rgb24",
        "-video_size",
        &format!("{}x{}", p.width, p.height),
        "-framerate",
        &p.fps.to_string(),
        "-i",
        "pipe:0",
        "-an",
    ]);
    if p.webm {
        cmd.args([
            "-c:v",
            "libvpx-vp9",
            "-crf",
            "30",
            "-b:v",
            "0",
            "-deadline",
            "realtime",
            "-cpu-used",
            "6",
            "-f",
            "webm",
        ]);
    } else {
        cmd.args([
            "-c:v",
            "libx264",
            "-preset",
            "veryfast",
            "-crf",
            "20",
            "-movflags",
            "+faststart",
            "-f",
            "mp4",
        ]);
    }
    let mut child = cmd
        .args(["-pix_fmt", "yuv420p"])
        .arg(temporary.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            format!("Impossible de lancer FFmpeg : {e}. Installez-le et ajoutez-le au PATH.")
        })?;
    let mut stderr = child.stderr.take().expect("stderr redirigé");
    let logs = thread::spawn(move || {
        let mut s = String::new();
        let _ = stderr.read_to_string(&mut s);
        s
    });
    let mut input = child.stdin.take().expect("stdin redirigé");
    let result = (|| -> Result<(), String> {
        for n in 0..total {
            if cancel.load(Ordering::Relaxed) {
                return Err("Export annulé.".into());
            }
            let frame = scene.frame(
                p,
                n as f32 / p.fps as f32,
                p.width as usize,
                p.height as usize,
            );
            input
                .write_all(&frame)
                .map_err(|e| format!("Écriture vidéo interrompue : {e}"))?;
            if n % (p.fps / 5).max(1) == 0 {
                let _ = tx.send(Event::Progress(n as f32 / total as f32));
            }
        }
        Ok(())
    })();
    drop(input);
    if result.is_err() {
        let _ = child.kill();
    }
    let status = child.wait().map_err(|e| format!("Fin de FFmpeg : {e}"))?;
    let log = logs.join().unwrap_or_default();
    result.map_err(|e| {
        if cancel.load(Ordering::Relaxed) || log.is_empty() {
            e
        } else {
            format!("{e}\n{log}")
        }
    })?;
    if cancel.load(Ordering::Relaxed) {
        return Err("Export annulé.".into());
    }
    if !status.success() {
        return Err(format!("FFmpeg a échoué : {log}"));
    }
    temporary
        .persist_noclobber(destination)
        .map_err(|e| format!("Impossible de finaliser le fichier : {}", e.error))?;
    Ok(destination.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn never_overwrites_an_existing_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("existing.mp4");
        std::fs::write(&path, b"original").unwrap();
        let (tx, _) = std::sync::mpsc::channel();
        assert!(run(
            &Project::default(),
            &path,
            &Arc::new(AtomicBool::new(false)),
            &tx
        )
        .is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"original");
    }
    #[test]
    fn cancelled_export_leaves_no_partial_file() {
        if !available() {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cancelled.mp4");
        let (tx, _) = std::sync::mpsc::channel();
        let p = Project {
            width: 320,
            height: 180,
            ..Project::default()
        };
        assert!(run(&p, &path, &Arc::new(AtomicBool::new(true)), &tx).is_err());
        assert!(!path.exists());
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0);
    }
}
