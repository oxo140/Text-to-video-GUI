# Text Video Studio v0.3.1 — GUI et CMD

Regroupement des deux projets Text-to-video-GUI et Text-to-video-CMD dans un seul dépôt Rust, avec conservation des sources historiques.

- `Text-Video-Studio-GUI.exe` : interface graphique en français, thème bleu marine professionnel sans drapeau tricolore, aperçu animé, export MP4/WebM.
- `Text-Video-Studio-CMD.exe` : assistant interactif ou commandes pour les scripts, lecture de fichiers texte UTF-8, couleurs, taille, vitesse et dimensions personnalisées, annulation Ctrl+C.
- Moteur commun, accents français, bandeau 1280 × 180 et résolutions SD à 4K, vertical, carré et personnalisé.
- Mise à jour depuis les releases publiques GitHub, sans token : bouton dans la GUI ou `update --check` / `update` dans la CMD. Vérification des tailles et des empreintes SHA-256 avant remplacement.
- Fichiers existants préservés, export asynchrone et nettoyage après annulation.

**Téléchargement** : extraire `Text-Video-Studio-Windows.zip`, puis ouvrir l’exécutable GUI ou CMD. Les deux exécutables seuls sont aussi proposés.

**FFmpeg reste nécessaire** : installer `Gyan.FFmpeg` ou placer `ffmpeg.exe` à côté des applications. Il n’est pas inclus dans l’archive. Aucun token ni DLL à placer dans System32 n’est requis.
