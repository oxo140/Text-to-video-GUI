//! Lecture des releases publiques : aucune authentification ni variable de token.
use reqwest::blocking::Client;
use semver::Version;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    io::{Read, Write},
    time::Duration,
};

pub const REPOSITORY: &str = "oxo140/Text-to-video-GUI";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
const MAX_DOWNLOAD: u64 = 256 * 1024 * 1024;

#[derive(Clone, Copy)]
pub enum Edition {
    Gui,
    Cmd,
}
impl Edition {
    pub fn asset(self) -> &'static str {
        match self {
            Self::Gui => "Text-Video-Studio-GUI.exe",
            Self::Cmd => "Text-Video-Studio-CMD.exe",
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
    size: u64,
}
#[derive(Deserialize)]
struct Release {
    tag_name: String,
    draft: bool,
    prerelease: bool,
    assets: Vec<Asset>,
}
#[derive(Clone, Debug)]
pub struct Update {
    pub version: String,
    binary: Asset,
    checksums: Asset,
}

fn client() -> Result<Client, String> {
    Client::builder()
        .https_only(true)
        .connect_timeout(Duration::from_secs(15))
        .timeout(Duration::from_secs(300))
        .user_agent(format!("Text-Video-Studio/{VERSION}"))
        .build()
        .map_err(|e| format!("Client de mise à jour : {e}"))
}

fn asset_for(release: &Release, name: &str) -> Result<Asset, String> {
    let prefix = format!(
        "https://github.com/{REPOSITORY}/releases/download/{}/",
        release.tag_name
    );
    let asset = release
        .assets
        .iter()
        .find(|a| a.name == name)
        .ok_or_else(|| format!("La release ne contient pas {name}."))?;
    if asset.browser_download_url != format!("{prefix}{name}")
        || asset.size == 0
        || asset.size > MAX_DOWNLOAD
    {
        return Err(format!("Fichier de mise à jour invalide : {name}."));
    }
    Ok(asset.clone())
}

fn parse_release(bytes: &[u8], edition: Edition, current: &str) -> Result<Option<Update>, String> {
    let release: Release =
        serde_json::from_slice(bytes).map_err(|e| format!("Réponse GitHub invalide : {e}"))?;
    if release.draft || release.prerelease {
        return Ok(None);
    }
    let version = Version::parse(
        release
            .tag_name
            .strip_prefix('v')
            .unwrap_or(&release.tag_name),
    )
    .map_err(|e| format!("Version publiée invalide : {e}"))?;
    let installed =
        Version::parse(current).map_err(|e| format!("Version installée invalide : {e}"))?;
    if version <= installed {
        return Ok(None);
    }
    let binary = asset_for(&release, edition.asset())?;
    let checksums = asset_for(&release, "SHA256SUMS.txt")?;
    Ok(Some(Update {
        version: version.to_string(),
        binary,
        checksums,
    }))
}

fn response_error(status: reqwest::StatusCode) -> String {
    if status.as_u16() == 403 || status.as_u16() == 429 {
        "Limite d’accès à GitHub atteinte. Réessayez plus tard.".into()
    } else {
        format!("GitHub a renvoyé l’erreur HTTP {status}.")
    }
}

pub fn check(edition: Edition) -> Result<Option<Update>, String> {
    if !cfg!(all(windows, target_arch = "x86_64")) {
        return Err("Les mises à jour automatiques sont disponibles pour Windows x64.".into());
    }
    let response = client()?
        .get(format!(
            "https://api.github.com/repos/{REPOSITORY}/releases/latest"
        ))
        .header("Accept", "application/vnd.github+json")
        .timeout(Duration::from_secs(30))
        .send()
        .map_err(|e| format!("Connexion à GitHub impossible : {e}"))?;
    if response.status().as_u16() == 404 {
        return Ok(None);
    }
    if !response.status().is_success() {
        return Err(response_error(response.status()));
    }
    let mut bytes = Vec::new();
    response
        .take(1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > 1024 * 1024 {
        return Err("Réponse GitHub trop volumineuse.".into());
    }
    parse_release(&bytes, edition, VERSION)
}

fn expected_hash(manifest: &str, filename: &str) -> Result<String, String> {
    let entries: Vec<_> = manifest
        .lines()
        .filter_map(|line| {
            let mut words = line.split_whitespace();
            let hash = words.next()?;
            let name = words.next()?.trim_start_matches('*');
            (name == filename && words.next().is_none()).then_some(hash)
        })
        .collect();
    if entries.len() != 1
        || entries[0].len() != 64
        || !entries[0].bytes().all(|c| c.is_ascii_hexdigit())
    {
        return Err(format!(
            "Empreinte SHA-256 absente ou invalide pour {filename}."
        ));
    }
    Ok(entries[0].to_ascii_lowercase())
}

fn download(client: &Client, asset: &Asset, output: &mut impl Write) -> Result<String, String> {
    let mut response = client
        .get(&asset.browser_download_url)
        .send()
        .map_err(|e| format!("Téléchargement impossible : {e}"))?;
    if !response.status().is_success() {
        return Err(response_error(response.status()));
    }
    let mut hasher = Sha256::new();
    let mut count = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let n = response
            .read(&mut buffer)
            .map_err(|e| format!("Téléchargement interrompu : {e}"))?;
        if n == 0 {
            break;
        }
        count += n as u64;
        if count > asset.size || count > MAX_DOWNLOAD {
            return Err("Taille du téléchargement incorrecte.".into());
        }
        hasher.update(&buffer[..n]);
        output
            .write_all(&buffer[..n])
            .map_err(|e| format!("Écriture de la mise à jour : {e}"))?;
    }
    if count != asset.size {
        return Err("Téléchargement incomplet.".into());
    }
    Ok(format!("{:x}", hasher.finalize()))
}

pub fn install(update: &Update, edition: Edition) -> Result<(), String> {
    if !cfg!(all(windows, target_arch = "x86_64")) {
        return Err("Installation automatique réservée à Windows x64.".into());
    }
    // Vérifie à nouveau la release au moment de l’installation.
    let latest = check(edition)?.ok_or("Aucune mise à jour disponible.")?;
    if latest.version != update.version
        || latest.binary.browser_download_url != update.binary.browser_download_url
    {
        return Err("La release a changé. Relancez la recherche de mise à jour.".into());
    }
    if latest.checksums.size > 64 * 1024 {
        return Err("Manifeste SHA-256 trop volumineux.".into());
    }
    let client = client()?;
    let mut manifest = Vec::new();
    download(&client, &latest.checksums, &mut manifest)?;
    let manifest = std::str::from_utf8(&manifest).map_err(|e| e.to_string())?;
    let expected = expected_hash(manifest, edition.asset())?;
    let mut file = tempfile::Builder::new()
        .suffix(".exe")
        .tempfile()
        .map_err(|e| e.to_string())?;
    let actual = download(&client, &latest.binary, &mut file)?;
    if actual != expected {
        return Err("L’empreinte SHA-256 ne correspond pas. Mise à jour refusée.".into());
    }
    file.as_file().sync_all().map_err(|e| e.to_string())?;
    self_replace::self_replace(file.path()).map_err(|e| format!("Remplacement impossible : {e}. Vérifiez les droits d’écriture du dossier de l’application."))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn release(version: &str) -> serde_json::Value {
        serde_json::json!({"tag_name":version,"draft":false,"prerelease":false,"assets":[
            {"name":"Text-Video-Studio-CMD.exe","size":123,"browser_download_url":format!("https://github.com/{REPOSITORY}/releases/download/{version}/Text-Video-Studio-CMD.exe")},
            {"name":"SHA256SUMS.txt","size":100,"browser_download_url":format!("https://github.com/{REPOSITORY}/releases/download/{version}/SHA256SUMS.txt")}
        ]})
    }
    #[test]
    fn versions_use_semver_and_ignore_prereleases() {
        assert!(parse_release(
            &serde_json::to_vec(&release("v0.10.0")).unwrap(),
            Edition::Cmd,
            "0.9.0"
        )
        .unwrap()
        .is_some());
        assert!(parse_release(
            &serde_json::to_vec(&release("v0.2.0")).unwrap(),
            Edition::Cmd,
            "0.3.0"
        )
        .unwrap()
        .is_none());
        let mut r = release("v0.4.0-beta.1");
        r["prerelease"] = true.into();
        assert!(
            parse_release(&serde_json::to_vec(&r).unwrap(), Edition::Cmd, "0.3.0")
                .unwrap()
                .is_none()
        );
    }
    #[test]
    fn rejects_wrong_repository_and_missing_edition() {
        let mut r = release("v0.4.0");
        r["assets"][0]["browser_download_url"] = "https://example.org/replacement.exe".into();
        assert!(parse_release(&serde_json::to_vec(&r).unwrap(), Edition::Cmd, "0.3.0").is_err());
        assert!(parse_release(
            &serde_json::to_vec(&release("v0.4.0")).unwrap(),
            Edition::Gui,
            "0.3.0"
        )
        .is_err());
    }
    #[test]
    fn checksums_are_exact_and_unique() {
        let hash = "a".repeat(64);
        assert_eq!(
            expected_hash(&format!("{hash}  app.exe"), "app.exe").unwrap(),
            hash
        );
        assert!(expected_hash(&format!("{hash}  other.exe"), "app.exe").is_err());
        assert!(expected_hash(&format!("{hash} app.exe\n{hash} app.exe"), "app.exe").is_err());
        assert!(expected_hash("invalid app.exe", "app.exe").is_err());
    }
}
