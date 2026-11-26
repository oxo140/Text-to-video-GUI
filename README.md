# Text Video Studio — GUI et CMD

Application Rust regroupant [Text-to-video-GUI](https://github.com/oxo140/Text-to-video-GUI) et [Text-to-video-CMD](https://github.com/oxo140/Text-to-video-CMD). Les deux interfaces utilisent le même moteur de rendu et d’export.

## Télécharger

Les [releases publiques](https://github.com/oxo140/Text-to-video-GUI/releases/latest) proposent :

- `Text-Video-Studio-GUI.exe` : interface graphique en français, thème bleu marine, aperçu animé.
- `Text-Video-Studio-CMD.exe` : terminal, assistant interactif et commandes utilisables dans les scripts.
- `Text-Video-Studio-Windows.zip` : les deux exécutables et la documentation.
- `SHA256SUMS.txt` : empreintes des fichiers distribués.

Extraire le ZIP dans un dossier accessible en écriture. Les exécutables sont portables, sans installation de l’application ni copie de DLL dans System32. Le runtime C est lié statiquement.

**FFmpeg est nécessaire** : l’installer avec `winget install Gyan.FFmpeg`, ou placer `ffmpeg.exe` à côté des deux exécutables. FFmpeg n’est pas inclus dans l’archive.

## Interface graphique

Ouvrir `Text-Video-Studio-GUI.exe`, saisir le texte, régler l’apparence et le format de sortie, puis exporter. L’aperçu dispose d’une pause et d’un curseur temporel.

Formats proposés : bandeau d’origine 1280 × 180, SD 854 × 480, HD 1280 × 720, Full HD 1920 × 1080, QHD 2560 × 1440, 4K 3840 × 2160, vertical 1080 × 1920, carré 1080 × 1080, et **Personnalisé**. Dimensions paires, largeur de 64 à 3840 px et hauteur de 64 à 2160 px. Cadences : 24, 30 ou 60 fps.

Les réglages et le dernier dossier d’export sont mémorisés. La boîte d’export utilise par défaut un dossier `exports` accessible en écriture à côté de l’application, avec repli sur le dossier de travail ou un dossier temporaire. Les permissions Windows de tout autre emplacement choisi restent applicables.

## Ligne de commande

Sans argument, la version CMD ouvre un assistant interactif, comme le projet console d’origine. Pour les scripts :

```powershell
.\Text-Video-Studio-CMD.exe --help
.\Text-Video-Studio-CMD.exe render --text "Message de démonstration" --output bandeau.mp4
.\Text-Video-Studio-CMD.exe render --text "Information" --preset full-hd --fps 30 --output video.mp4
.\Text-Video-Studio-CMD.exe render --text-file message.txt --width 1280 --height 180 --speed 240 --size 96 --foreground 112848 --background FFFFFF --output bandeau.webm
```

`--text-file` lit un fichier UTF-8. `--text` et `--text-file` sont exclusifs. Les couleurs sont au format RGB hexadécimal. Les presets sont `bandeau`, `sd`, `hd`, `full-hd`, `qhd`, `uhd`, `vertical` et `square`. `--width` et `--height` permettent de personnaliser les dimensions.

Sans réglage, la version CMD produit un bandeau 1280 × 180 à 60 fps, texte noir sur fond blanc. L’extension de sortie détermine MP4/H.264 ou WebM/VP9 ; `--format mp4` ou `--format webm` permet de l’expliciter. Les fichiers existants ne sont jamais écrasés. `Ctrl+C` annule l’export et nettoie le fichier temporaire. Un échec renvoie un code de sortie non nul.

## Mises à jour sans token

La GUI dispose du bouton **Vérifier les mises à jour**, puis **Installer la version…** et **Redémarrer**. La console propose :

```powershell
.\Text-Video-Studio-CMD.exe update --check
.\Text-Video-Studio-CMD.exe update
```

Le programme interroge `https://api.github.com/repos/oxo140/Text-to-video-GUI/releases/latest` en HTTPS, sans token, sans compte et sans lire de variable d’authentification. Il ignore les brouillons, préversions et versions plus anciennes. L’installation est demandée explicitement ; elle télécharge l’exécutable de l’édition utilisée et le manifeste SHA-256, vérifie la taille et l’empreinte, puis remplace l’exécutable. La GUI et la CMD se mettent à jour séparément. Relancer l’édition après installation.

La mise à jour automatique concerne Windows x64. Le dossier de l’exécutable doit être accessible en écriture. Une absence de connexion, une release incomplète ou une limite d’accès GitHub est signalée sans demander de token. Le bouton n’interroge GitHub qu’à la demande.

Le workflow GitHub Actions utilise le jeton éphémère fourni par GitHub uniquement pour publier les releases. Aucun jeton n’est intégré dans les applications distribuées.

## Rendu

Défilement horizontal vers la gauche et une seconde de fond vide en fin de vidéo. Les vidéos sont muettes. L’aperçu et l’export partagent le même rasteriseur ; seul l’aperçu est réduit. Les accents français sont pris en charge. Les retours à la ligne deviennent des espaces ; les emojis et les écritures nécessitant un moteur de composition complexe ne sont pas pris en charge. Il ne s’agit pas de génération vidéo par IA.

Les images RGB sont transmises directement à FFmpeg, sans OpenCV ni fichier AVI intermédiaire. L’interface reste utilisable pendant l’export et l’annulation supprime les fichiers temporaires.

## Développement

Prérequis : Rust stable, FFmpeg et les outils de compilation C++ de Visual Studio avec SDK Windows.

```powershell
cargo test --locked
cargo clippy --all-targets --locked -- -D warnings
cargo run --bin text-video-studio
cargo run --bin text-video-cmd -- --help
cargo build --release --locked
# Version console sans dépendances graphiques :
cargo build --release --locked --no-default-features --bin text-video-cmd
# Préparer le dossier dist :
.\build.ps1
```

Le script reconnaît aussi la chaîne Rust locale du dossier `.tools` du workspace parent. Le workflow `.github/workflows/windows-release.yml` compile et vérifie les deux exécutables, génère le ZIP et les empreintes, puis publie une release pour les tags `v*`.

## Sources regroupées

- `src/lib.rs`, `render.rs`, `export.rs` : moteur commun.
- `src/main.rs` : application graphique.
- `src/bin/cmd.rs` : application console.
- `src/updates.rs` : mise à jour des deux éditions.
- `legacy/gui/` et `legacy/cmd/` : sources originales conservées pour référence.
- `PROVENANCE.md` : dépôts et commits d’origine.

Les dépôts d’origine n’ont pas de licence déclarée. Aucune nouvelle licence n’est imposée à leurs sources. Les dépendances conservent leurs licences respectives ; voir `THIRD_PARTY.md`.
