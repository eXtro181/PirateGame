//! HUD- und Menügrafik: Messingrahmen mit Ankerhaken, Bullaugen, Symbole, Kompass, Emblem.

use crate::canvas::Canvas;
use crate::color::*;
use crate::noise::*;

fn br() -> [Rgba; 5] {
    ramp(hex(0xE8A443))
}
fn brd() -> [Rgba; 5] {
    ramp(hex(0x7A5A3E))
}

fn ui_canvas(w: i32, h: i32) -> Canvas {
    let mut c = Canvas::new(w, h);
    c.standing = false;
    c.cur_up = 0;
    c
}

pub fn brass_rect(c: &mut Canvas, x: i32, y: i32, w: i32, h: i32) {
    let (b, d) = (br(), brd());
    for i in 0..w {
        c.set(x + i, y, b[3]);
        c.set(x + i, y + h - 1, d[1]);
    }
    for j in 0..h {
        c.set(x, y + j, b[2]);
        c.set(x + w - 1, y + j, d[1]);
    }
    for i in 1..w - 1 {
        c.set(x + i, y + 1, d[2]);
        c.set(x + i, y + h - 2, d[0]);
    }
    for j in 1..h - 1 {
        c.set(x + 1, y + j, d[2]);
        c.set(x + w - 2, y + j, d[0]);
    }
}

pub fn anchor(c: &mut Canvas, x: i32, y: i32, col: &[Rgba; 5], s: i32) {
    c.vline(x, y + 2 * s, y + 10 * s, col[2]);
    c.vline(x + 1, y + 2 * s, y + 10 * s, col[1]);
    c.ellipse(x as f32 + 0.5, (y + s) as f32, 1.8 * s as f32, 1.8 * s as f32, col[3]);
    c.set(x, y + s, hex(0x0B1016));
    c.hline(x - 3 * s, x + 4 * s, y + 4 * s, col[3]);
    for k in 0..=4 * s {
        let dy = ((k as f32).sqrt() * 1.4).round() as i32;
        c.set(x - k, y + 10 * s - dy, col[2]);
        c.set(x + 1 + k, y + 10 * s - dy, col[1]);
    }
    c.set(x - 4 * s, y + 7 * s, col[4]);
    c.set(x + 5 * s, y + 7 * s, col[4]);
}

pub fn porthole(d: i32) -> Canvas {
    let mut c = ui_canvas(d, d);
    let (b, dd) = (br(), brd());
    let cc = (d - 1) as f32 / 2.0;
    for y in 0..d {
        for x in 0..d {
            let r = ((x as f32 - cc).powi(2) + (y as f32 - cc).powi(2)).sqrt();
            if r > cc + 0.5 {
                continue;
            }
            if r > cc - 2.5 {
                let l = -(x as f32 - cc) - (y as f32 - cc);
                c.set(x, y, if l > 2.0 { b[3] } else if l > -2.0 { b[2] } else { dd[1] });
                let a = (y as f32 - cc).atan2(x as f32 - cc);
                if ((a * 4.0).round() as i32) % 3 == 0 && r > cc - 1.5 {
                    c.set(x, y, dd[0]);
                }
            } else {
                c.set(x, y, if r < cc - 4.0 { hex(0x0E161E) } else { hex(0x16232E) });
            }
        }
    }
    c
}

pub fn icon(name: &str) -> Canvas {
    let mut p = ui_canvas(16, 16);
    let gold = hex(0xC9A04A);
    let tar = hex(0x4A3326);
    let old = hex(0x7A5A3E);
    let soul = hex(0x5FF2C8);
    let soul_l = hex(0xB8FFE8);
    let bone = hex(0xD8CFB8);
    match name {
        "saber" => {
            for t in 0..11 {
                let x = 3 + t;
                let y = 13 - t + ((t as f32 / 11.0).powi(2) * 2.0).round() as i32;
                p.set(x, y, hex(0xC8D0D8));
                p.set(x, y + 1, hex(0x6A7280));
            }
            p.line(1.0, 12.0, 5.0, 16.0, gold);
            p.set(2, 14, tar);
            p.set(1, 15, tar);
        }
        "pistol" => {
            p.rect(3, 6, 10, 3, hex(0x6A7280));
            p.hline(3, 12, 6, hex(0xB8C0C8));
            p.rect(2, 8, 4, 6, tar);
            p.set(2, 13, old);
            p.set(7, 9, hex(0x3A3E44));
            p.set(12, 5, hex(0xB8C0C8));
        }
        "rum" => {
            p.rect(5, 6, 6, 9, hex(0x3A5A3A));
            p.rect(6, 3, 4, 3, hex(0x3A5A3A));
            p.rect(6, 1, 4, 2, old);
            p.rect(5, 9, 6, 3, bone);
            p.set(7, 10, hex(0x7A1A22));
            p.vline(6, 7, 13, hex(0x5A8A5A));
        }
        "phial" => {
            p.rect(6, 1, 4, 3, old);
            p.ellipse(8.0, 10.0, 5.0, 5.0, hex(0x16232E));
            for y in 7..15 {
                for x in 4..13 {
                    if ((x - 8) as f32).hypot((y - 10) as f32) < 4.0 {
                        p.lit(x, y, if y < 9 { soul_l } else { soul }, 1.1);
                    }
                }
            }
            p.rect(7, 4, 2, 2, hex(0x4A6170));
        }
        "bomb" => {
            p.dome(8.0, 10.0, 5.0, 5.0, &ramp_n(hex(0x2A2E34), 5, 1.0), 2.0, 0.0, 0.4, 0.0);
            p.line(10.0, 5.0, 12.0, 2.0, old);
            p.lit(13, 1, hex(0xE8A443), 1.4);
            p.lit(12, 1, hex(0xC2502E), 1.0);
        }
        "bullets" => {
            for (x, y) in [(4, 6), (8, 9), (11, 5)] {
                p.ellipse(x as f32, y as f32, 2.0, 2.0, hex(0x8A9098));
                p.set(x - 1, y - 1, hex(0xD8DCE0));
            }
        }
        "fireball" => {
            for y in 2..15 {
                for x in 2..15 {
                    let r = ((x - 9) as f32).hypot((y - 9) as f32) + if x < 9 { (9 - x) as f32 * 0.3 } else { 0.0 };
                    if r < 5.5 {
                        p.lit(x, y, if r < 2.5 { hex(0xFFE0A0) } else if r < 4.0 { hex(0xE8A443) } else { hex(0xC2502E) }, 1.3);
                    }
                }
            }
        }
        "bolt" => {
            let c = hex(0xDFF7FF);
            p.line(10.0, 1.0, 6.0, 8.0, c);
            p.line(6.0, 8.0, 10.0, 8.0, c);
            p.line(10.0, 8.0, 5.0, 15.0, c);
            for y in 0..16 {
                for x in 0..16 {
                    if p.a(x, y) > 0 {
                        p.glow(x, y, soul_l, 1.3);
                    }
                }
            }
        }
        "hammer" => {
            p.line(3.0, 13.0, 10.0, 6.0, tar);
            p.rect(9, 2, 6, 5, hex(0x6A7280));
            p.hline(9, 14, 2, hex(0xB8C0C8));
        }
        "rapier" => {
            p.line(2.0, 14.0, 14.0, 2.0, hex(0xC8D0D8));
            p.ellipse(4.0, 12.0, 2.0, 2.0, gold);
        }
        "dagger" => {
            p.line(4.0, 12.0, 11.0, 5.0, hex(0xC8D0D8));
            p.line(2.0, 11.0, 5.0, 14.0, gold);
            p.set(3, 13, tar);
        }
        "club" => {
            p.line(3.0, 13.0, 12.0, 4.0, tar);
            p.line(4.0, 13.0, 13.0, 4.0, old);
            p.ellipse(12.0, 4.0, 2.5, 2.5, old);
        }
        "fetish" => {
            p.line(4.0, 14.0, 10.0, 6.0, hex(0x2A1E1A));
            p.ellipse(11.0, 5.0, 3.5, 3.0, bone);
            p.lit(11, 5, soul, 1.3);
        }
        "tide" => {
            for x in 2..15 {
                let y = 9 + ((x as f32 * 0.8).sin() * 2.0).round() as i32;
                p.lit(x, y, soul, 0.9);
                p.set(x, y + 1, hex(0x22508A));
            }
        }
        "pearl" => {
            p.ellipse(8.0, 8.0, 3.5, 3.5, hex(0xC8D0D8));
            p.set(7, 6, hex(0xFFFFFF));
            for y in 4..13 {
                for x in 4..13 {
                    if p.a(x, y) > 0 {
                        p.glow(x, y, soul_l, 0.5);
                    }
                }
            }
        }
        "figurehead" => {
            p.ellipse(8.0, 6.0, 4.0, 4.0, bone);
            p.ellipse(8.0, 12.0, 4.0, 3.0, bone);
            p.lit(7, 5, soul, 1.2);
            p.lit(9, 5, soul, 1.2);
        }
        "key" => {
            p.ellipse(5.0, 6.0, 3.0, 3.0, gold);
            p.set(5, 6, hex(0x0B1016));
            p.hline(7, 14, 6, gold);
            p.vline(12, 6, 9, gold);
            p.vline(14, 6, 8, gold);
        }
        "shard" => {
            p.line(8.0, 1.0, 12.0, 8.0, soul_l);
            p.line(12.0, 8.0, 8.0, 15.0, soul);
            p.line(8.0, 15.0, 4.0, 8.0, soul);
            p.line(4.0, 8.0, 8.0, 1.0, soul_l);
            for y in 0..16 {
                for x in 0..16 {
                    if p.a(x, y) > 0 {
                        p.glow(x, y, soul, 1.0);
                    }
                }
            }
        }
        "coat" => {
            p.rect(4, 3, 8, 11, hex(0x5A2A30));
            p.rect(2, 4, 2, 7, hex(0x5A2A30));
            p.rect(12, 4, 2, 7, hex(0x5A2A30));
            p.vline(8, 3, 13, bone);
        }
        "ring" => {
            for y in 0..16 {
                for x in 0..16 {
                    let r = ((x - 8) as f32).hypot((y - 9) as f32);
                    if (2.2..4.2).contains(&r) {
                        p.set(x, y, gold);
                    }
                }
            }
            p.lit(8, 4, hex(0x7A1A22), 1.2);
        }
        "shanty" => {
            p.vline(10, 2, 11, bone);
            p.ellipse(8.0, 12.0, 2.5, 2.0, bone);
            p.line(10.0, 2.0, 13.0, 5.0, bone);
        }
        _ => {}
    }
    p.outline(true, 0.9);
    p
}

pub const ICONS: [&str; 21] = ["saber", "pistol", "rum", "phial", "bomb", "bullets", "fireball", "bolt", "hammer", "rapier", "dagger", "club", "fetish", "tide", "pearl", "figurehead", "key", "shard", "coat", "ring", "shanty"];

pub fn hp_frame() -> Canvas {
    let mut c = ui_canvas(158, 16);
    brass_rect(&mut c, 12, 2, 144, 12);
    anchor(&mut c, 5, 2, &br(), 1);
    c
}
pub fn bar_frame(w: i32) -> Canvas {
    let mut c = ui_canvas(w, 10);
    brass_rect(&mut c, 0, 0, w, 10);
    c
}
pub fn compass(f: i32) -> Canvas {
    let mut c = ui_canvas(9, 9);
    let b = br();
    c.line(1.0, 4.0, 7.0, 4.0, b[2]);
    c.line(5.0, 2.0, 7.0, 4.0, b[3]);
    c.line(5.0, 6.0, 7.0, 4.0, b[1]);
    c.lit(1 + f * 2, 4, hex(0xFFF0C0), 0.8);
    c
}
pub fn emblem() -> Canvas {
    let mut c = ui_canvas(34, 34);
    anchor(&mut c, 16, 3, &ramp(hex(0xD8CFB8)), 2);
    for t in 0..28 {
        let x = 4 + t;
        let y = 30 - t + ((t as f32 / 28.0).powi(2) * 4.0).round() as i32;
        c.set(x, y, hex(0xC8D0D8));
        c.set(x, y + 1, hex(0x6A7280));
    }
    c.line(2.0, 26.0, 9.0, 33.0, hex(0xC9A04A));
    c.lit(16, 5, hex(0x5FF2C8), 1.0);
    c.outline(true, 0.9);
    c
}
/// Wertes Schild/Plakette (9-Slice-Teile werden im Spiel zusammengesetzt).
pub fn panel_corner() -> Canvas {
    let mut c = ui_canvas(8, 8);
    let b = br();
    for i in 0..8 {
        c.set(i, 0, b[3]);
        c.set(0, i, b[2]);
    }
    c.set(1, 1, b[4]);
    c.set(2, 2, b[2]);
    c
}
/// Logo-Schriftzug mit Fase: jedes Schriftpixel wird zu 2×2 mit Licht von oben links.
pub fn logo(text: &str) -> Canvas {
    let glyphs: Vec<(Canvas, i32)> = text.chars().map(|ch| crate::font::glyph(ch).unwrap_or((Canvas::new(3, 8), 3))).collect();
    let w: i32 = glyphs.iter().map(|g| (g.1 + 1) * 3).sum::<i32>() + 6;
    let mut c = ui_canvas(w, 30);
    let lr = ramp_n(hex(0xC9B48A), 6, 1.1);
    let mut cx = 2;
    for (g, gw) in &glyphs {
        for y in 0..8 {
            for x in 0..*gw {
                if g.a(x, y) == 0 {
                    continue;
                }
                let t = y as f32 / 7.0;
                let base = if t < 0.3 { 5 } else if t < 0.6 { 4 } else { 3 };
                for (dx, dy, k) in [(0, 0, 0), (1, 0, 0), (2, 0, 1), (0, 1, 0), (1, 1, 1), (2, 1, 1), (0, 2, 1), (1, 2, 2), (2, 2, 2)] {
                    c.set(cx + x * 3 + dx, 2 + y * 3 + dy, lr[(base - k) as usize]);
                }
            }
        }
        cx += (gw + 1) * 3;
    }
    c.outline(true, 1.0);
    let mut c2 = ui_canvas(w + 2, 32);
    c2.blit(&c, 1, 1, false);
    c2.outline(true, 1.0);
    // leichtes Rauschen auf den Buchstaben
    for y in 0..c2.h {
        for x in 0..c2.w {
            if c2.a(x, y) > 0 && h01(x, y, 3) > 0.9 {
                c2.mul(x, y, 0.9);
            }
        }
    }
    c2
}
