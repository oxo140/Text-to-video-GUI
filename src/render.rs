use fontdue::{
    layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle},
    Font, FontSettings,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Project {
    pub text: String,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub speed: f32,
    pub size: f32,
    pub foreground: [u8; 3],
    pub background: [u8; 3],
    pub webm: bool,
}

impl Default for Project {
    fn default() -> Self {
        Self {
            text: "Texte de démonstration.".into(),
            width: 1920,
            height: 1080,
            fps: 30,
            speed: 240.0,
            size: 96.0,
            foreground: [245, 247, 252],
            background: [12, 32, 60],
            webm: false,
        }
    }
}

impl Project {
    pub fn validate(&self) -> Result<(), String> {
        if self.text.trim().is_empty() {
            return Err("Écrivez un texte avant d’exporter.".into());
        }
        if self.text.chars().count() > 2000 {
            return Err("Le texte est limité à 2 000 caractères.".into());
        }
        if !(64..=3840).contains(&self.width)
            || !(64..=2160).contains(&self.height)
            || !self.width.is_multiple_of(2)
            || !self.height.is_multiple_of(2)
        {
            return Err("Dimensions paires requises : de 64 × 64 à 3840 × 2160.".into());
        }
        if ![24, 30, 60].contains(&self.fps)
            || !self.speed.is_finite()
            || !(60.0..=720.0).contains(&self.speed)
            || !self.size.is_finite()
            || !(24.0..=240.0).contains(&self.size)
        {
            return Err("Paramètres de rendu invalides.".into());
        }
        Ok(())
    }
}

pub struct Glyph {
    x: f32,
    y: f32,
    width: usize,
    height: usize,
    alpha: Vec<u8>,
}
pub struct Scene {
    glyphs: Vec<Glyph>,
    pub text_width: f32,
    pub text_height: f32,
}

impl Scene {
    pub fn new(project: &Project) -> Self {
        let font = Font::from_bytes(epaint_default_fonts::UBUNTU_LIGHT, FontSettings::default())
            .expect("police embarquée valide");
        let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
        layout.reset(&LayoutSettings::default());
        let text = project
            .text
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        layout.append(&[&font], &TextStyle::new(&text, project.size, 0));
        let mut left = 0.0_f32;
        let mut top = f32::MAX;
        let mut right = 0.0_f32;
        let mut bottom = 0.0_f32;
        let mut glyphs = Vec::new();
        for g in layout.glyphs() {
            let (m, alpha) = font.rasterize_config(g.key);
            if m.width == 0 || m.height == 0 {
                continue;
            }
            left = left.min(g.x);
            top = top.min(g.y);
            right = right.max(g.x + m.width as f32);
            bottom = bottom.max(g.y + m.height as f32);
            glyphs.push(Glyph {
                x: g.x,
                y: g.y,
                width: m.width,
                height: m.height,
                alpha,
            });
        }
        if glyphs.is_empty() {
            top = 0.0;
        }
        for g in &mut glyphs {
            g.x -= left;
            g.y -= top;
        }
        Self {
            glyphs,
            text_width: right - left,
            text_height: bottom - top,
        }
    }

    pub fn duration(&self, p: &Project) -> f32 {
        (p.width as f32 + self.text_width) / p.speed + 1.0
    }

    // Le même rasteriseur sert à l’aperçu et aux images envoyées à FFmpeg.
    pub fn frame(&self, p: &Project, time: f32, width: usize, height: usize) -> Vec<u8> {
        let mut pixels = p.background.repeat(width * height);
        let sx = width as f32 / p.width as f32;
        let sy = height as f32 / p.height as f32;
        let origin_x = p.width as f32 - time * p.speed;
        let origin_y = (p.height as f32 - self.text_height) / 2.0;
        for g in &self.glyphs {
            let x0 = ((origin_x + g.x) * sx).floor() as i32;
            let y0 = ((origin_y + g.y) * sy).floor() as i32;
            let w = (g.width as f32 * sx).ceil() as i32;
            let h = (g.height as f32 * sy).ceil() as i32;
            for dy in 0.max(-y0)..h.min(height as i32 - y0) {
                for dx in 0.max(-x0)..w.min(width as i32 - x0) {
                    let gx = ((dx as f32 / sx) as usize).min(g.width - 1);
                    let gy = ((dy as f32 / sy) as usize).min(g.height - 1);
                    let a = g.alpha[gy * g.width + gx] as u32;
                    let i = ((y0 + dy) as usize * width + (x0 + dx) as usize) * 3;
                    for c in 0..3 {
                        pixels[i + c] = ((p.foreground[c] as u32 * a
                            + pixels[i + c] as u32 * (255 - a))
                            / 255) as u8;
                    }
                }
            }
        }
        pixels
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_input() {
        let mut p = Project::default();
        assert!(p.validate().is_ok());
        p.width = 1919;
        assert!(p.validate().is_err());
        p.width = 1920;
        p.text = " \n ".into();
        assert!(p.validate().is_err());
    }
    #[test]
    fn accented_text_scrolls_completely_out() {
        let p = Project {
            text: "Été, déjà !".into(),
            ..Project::default()
        };
        let scene = Scene::new(&p);
        let blank = p.background.repeat(320 * 180);
        assert_eq!(scene.frame(&p, 0.0, 320, 180), blank);
        let mid = (p.width as f32 + scene.text_width) / p.speed / 2.0;
        assert_ne!(scene.frame(&p, mid, 320, 180), blank);
        assert_eq!(scene.frame(&p, scene.duration(&p), 320, 180), blank);
    }
}
