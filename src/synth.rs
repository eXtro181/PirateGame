//! Synthesizer: Musik, Ambiente und Klangeffekte werden aus Code in PCM gerendert
//! (keine Audiodateien). Die Wiedergabe übernimmt `audio.rs`.

pub const SR: f32 = 22050.0;

// ------------------------------------------------------------------ Grundbausteine
#[derive(Clone, Copy)]
pub struct Rng(u32);
impl Rng {
    pub fn new(s: u32) -> Self {
        Rng(s.max(1))
    }
    pub fn f(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        (self.0 as f32 / u32::MAX as f32) * 2.0 - 1.0
    }
    pub fn u(&mut self) -> f32 {
        self.f() * 0.5 + 0.5
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Wave {
    Sine,
    Tri,
    Saw,
    Square,
}
fn osc(w: Wave, ph: f32) -> f32 {
    let p = ph - ph.floor();
    match w {
        Wave::Sine => (p * std::f32::consts::TAU).sin(),
        Wave::Tri => 1.0 - 4.0 * (p - 0.5).abs(),
        Wave::Saw => 2.0 * p - 1.0,
        Wave::Square => {
            if p < 0.5 {
                1.0
            } else {
                -1.0
            }
        }
    }
}

/// RBJ-Biquad (Tiefpass/Bandpass)
#[derive(Clone, Copy)]
pub struct Bq {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}
#[derive(Clone, Copy, PartialEq)]
pub enum Ft {
    Low,
    Band,
    High,
}
impl Bq {
    pub fn new(t: Ft, f: f32, q: f32) -> Self {
        let mut b = Bq { b0: 0.0, b1: 0.0, b2: 0.0, a1: 0.0, a2: 0.0, x1: 0.0, x2: 0.0, y1: 0.0, y2: 0.0 };
        b.set(t, f, q);
        b
    }
    pub fn set(&mut self, t: Ft, f: f32, q: f32) {
        let w = std::f32::consts::TAU * f.clamp(10.0, SR * 0.45) / SR;
        let (s, c) = w.sin_cos();
        let al = s / (2.0 * q.max(0.05));
        let a0 = 1.0 + al;
        let (b0, b1, b2) = match t {
            Ft::Low => ((1.0 - c) / 2.0, 1.0 - c, (1.0 - c) / 2.0),
            Ft::Band => (al, 0.0, -al),
            Ft::High => ((1.0 + c) / 2.0, -(1.0 + c), (1.0 + c) / 2.0),
        };
        self.b0 = b0 / a0;
        self.b1 = b1 / a0;
        self.b2 = b2 / a0;
        self.a1 = -2.0 * c / a0;
        self.a2 = (1.0 - al) / a0;
    }
    pub fn run(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.b1 * self.x1 + self.b2 * self.x2 - self.a1 * self.y1 - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = x;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }
}

/// Hüllkurve wie in WebAudio: exponentieller Anstieg, Halten, exponentieller Ausklang.
fn env(t: f32, a: f32, peak: f32, d: f32, rel: f32) -> f32 {
    if t < 0.0 || t > d {
        return 0.0;
    }
    let lo = 0.0001f32;
    if t < a {
        return lo * (peak / lo).powf(t / a.max(1e-4));
    }
    let hold = (d - rel).max(a);
    if t < hold {
        return peak;
    }
    peak * (lo / peak).powf((t - hold) / (d - hold).max(1e-4))
}
fn exp_decay(t: f32, v: f32, dur: f32) -> f32 {
    if t < 0.0 || t > dur {
        0.0
    } else {
        v * (0.0001f32 / v.max(1e-4)).powf(t / dur)
    }
}

fn add(buf: &mut [f32], i: usize, v: f32) {
    let n = buf.len();
    if n > 0 {
        buf[i % n] += v;
    }
}

// ------------------------------------------------------------------ Noten
const NOTES: [(&str, i32); 17] = [("C#", 1), ("Db", 1), ("D#", 3), ("Eb", 3), ("F#", 6), ("Gb", 6), ("G#", 8), ("Ab", 8), ("A#", 10), ("Bb", 10), ("C", 0), ("D", 2), ("E", 4), ("F", 5), ("G", 7), ("A", 9), ("B", 11)];
pub fn freq(n: &str) -> f32 {
    for (name, v) in NOTES {
        if let Some(rest) = n.strip_prefix(name) {
            if let Ok(o) = rest.parse::<i32>() {
                return 440.0 * 2f32.powf((v + (o + 1) * 12 - 69) as f32 / 12.0);
            }
        }
    }
    440.0
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Inst {
    Whistle,
    Fiddle,
    Accordion,
    AccordionSolo,
    Pad,
    Bass,
    Drum,
    Tick,
    Bell,
    /// gestrichenes Cello (langsamer Einsatz, Vibrato)
    Cello,
    /// gezupfte Harfe (Karplus-Strong)
    Harp,
    /// Streichergruppe (vier verstimmte Sägezähne, weich gefiltert)
    Strings,
    /// Chor „Aah“ (Formantfilter)
    Choir,
    /// Horn (Filter öffnet sich beim Anblasen)
    Horn,
    /// Pauke (gestimmt, langer Nachhall)
    Timpani,
    /// Becken
    Crash,
    /// Männerchor „Oh“ (dunkle Formanten)
    ChoirLo,
    /// Orgel (Pfeifenreihen)
    Organ,
    /// tiefe Blechbläser
    Brass,
    /// große Trommel
    Boom,
    /// Tremolo-Streicher
    Trem,
}
#[derive(Clone, Copy)]
pub struct Ev {
    pub t: f32,
    pub dur: f32,
    pub inst: Inst,
    pub f: f32,
    pub vel: f32,
}

fn seq(s: &str, inst: Inst, vel: f32, start: f32) -> (Vec<Ev>, f32) {
    let mut out = vec![];
    let mut t = start;
    for tok in s.split_whitespace() {
        if tok == "|" {
            continue;
        }
        let Some((n, l)) = tok.split_once(':') else { continue };
        let len: f32 = l.parse().unwrap_or(1.0);
        if n != "R" {
            for nn in n.split('+') {
                out.push(Ev { t, dur: len, inst, f: freq(nn), vel });
            }
        }
        t += len;
    }
    (out, t - start)
}
fn sq(s: &str, inst: Inst, vel: f32, start: f32) -> Vec<Ev> {
    seq(s, inst, vel, start).0
}
fn drum(t: f32, f: f32, vel: f32) -> Ev {
    Ev { t, dur: 1.0, inst: Inst::Drum, f, vel }
}
fn tick(t: f32, vel: f32) -> Ev {
    Ev { t, dur: 1.0, inst: Inst::Tick, f: 0.0, vel }
}

pub struct Song {
    pub unit: f32,
    pub len: f32,
    pub events: Vec<Ev>,
    pub gain: f32,
}

fn first(c: &str) -> &str {
    c.split('+').next().unwrap_or(c)
}
fn down(c: &str) -> String {
    first(c).replace('3', "2")
}

/// Harfen-Arpeggio über einen Akkord (Grundton, Quinte, Oktave, Terz darüber …) für `len` Schritte.
fn arp(chord: &str, len: f32, step: f32, t0: f32, vel: f32) -> Vec<Ev> {
    let notes: Vec<&str> = chord.split('+').collect();
    let pat = [0usize, 1, 2, 1, 2, 3, 2, 1];
    let mut out = vec![];
    let mut t = 0.0;
    let mut k = 0;
    while t < len - 0.01 {
        let n = notes[pat[k % pat.len()] % notes.len()];
        out.push(Ev { t: t0 + t, dur: step * 2.0, inst: Inst::Harp, f: freq(n), vel: vel * if k % 4 == 0 { 1.0 } else { 0.75 } });
        t += step;
        k += 1;
    }
    out
}

/// Oktavlage einer Notenfolge verschieben ("D4:2 A#3:1" um k Oktaven)
fn oct(s: &str, k: i32) -> String {
    s.split_whitespace()
        .map(|tok| match tok.split_once(':') {
            Some((n, l)) if n != "R" => {
                let ns: Vec<String> = n
                    .split('+')
                    .map(|x| {
                        let (name, o) = x.split_at(x.len() - 1);
                        format!("{name}{}", o.parse::<i32>().unwrap_or(4) + k)
                    })
                    .collect();
                format!("{}:{l}", ns.join("+"))
            }
            Some(_) => tok.to_string(),
            // einzelne Note ohne Länge (Bassgrundton)
            None if tok != "|" => {
                let (name, o) = tok.split_at(tok.len() - 1);
                format!("{name}{}", o.parse::<i32>().unwrap_or(2) + k)
            }
            None => tok.to_string(),
        })
        .collect::<Vec<_>>()
        .join(" ")
}
/// Akkorde der Titelmusik: (Bass, Mittellage, Hochlage)
fn tch(c: &str) -> (&'static str, &'static str, &'static str) {
    match c {
        "Bb" => ("A#1", "A#2+D3+F3", "A#3+D4+F4"),
        "Gm" => ("G1", "G2+A#2+D3", "G3+A#3+D4"),
        "A" => ("A1", "A2+C#3+E3", "A3+C#4+E4"),
        "C" => ("C2", "C3+E3+G3", "C4+E4+G4"),
        "F" => ("F2", "F2+A2+C3", "F3+A3+C4"),
        "Eb" => ("D#2", "D#3+G3+A#3", "D#4+G4+A#4"),
        _ => ("D2", "D3+F3+A3", "D4+F4+A4"),
    }
}

/// Alle Stücke: melancholische, rein instrumentale Seemannsmusik in Moll – Cello, Geige,
/// Blechflöte, Harfe, Akkordeon und Glocken, wenig Trommeln. Längen in Schritten, `unit` = Sekunden
/// je Schritt.
pub fn song(name: &str) -> Option<Song> {
    use Inst::*;
    let s = match name {
        // Titel „Thron der Ertrunkenen“ (20.11.): episch und düster, d-Moll mit phrygischem Es-Dur, 60 Schläge.
        // Totenglocken und Orgelbordun, eine einsame Stimme – Aufbau: Männerchor skandiert, Tremolo-Streicher,
        // Pauken wie Herzschlag, tiefes Blech – Hauptthema: der ganze Chor mit Blech, Orgel und großen Trommeln –
        // Einbruch: Cello allein, Glocken, Paukenwirbel – Höhepunkt: Blechfanfare über skandierendem Chor –
        // gewaltige Schlussakkorde, ein letzter Schlag, Stille – Ausklang mit Glocke und Stimme.
        "titel" => {
            let theme = "D4:4 A4:4 | A#4:4 A4:2 G4:2 | G4:4 F4:2 D#4:2 | D4:8 | D4:4 A4:4 | A#4:4 C5:2 D5:2 | D#5:4 D5:2 A#4:2 | A4:8";
            let lament = "R:8 | A4:4 D5:4 | C5:2 A#4:2 F4:4 | D5:4 A#4:4 | G4:4 F4:2 D#4:2 | E4:4 C#4:4";
            let solo = "A#3:4 D4:4 | D4:4 A#3:2 G3:2 | G3:4 F3:2 D#3:2 | E3:8";
            let fanfare = "D4:3 D4:1 F4:2 A4:2 | G4:3 G4:1 E4:2 C4:2 | F4:3 F4:1 D4:2 A#3:2 | A3:2 C#4:2 E4:2 A4:2 | A#4:3 A4:1 G4:2 D5:2 | D#5:4 D5:2 C5:2 | D5:3 C5:1 A#4:2 A4:2 | A4:4 C#5:4";
            let coda = "A4:4 D5:4 | C5:4 A#4:4 | A#4:4 F4:4 | E4:8 | D4:8";
            let bt = |bar: f32| bar * 8.0;
            let mut ev = sq(lament, Choir, 0.3, 0.0);
            ev.extend(sq(theme, Choir, 0.5, bt(14.0)));
            ev.extend(sq(&oct(theme, -1), ChoirLo, 0.55, bt(14.0)));
            ev.extend(sq(&oct(theme, -1), Brass, 0.45, bt(14.0)));
            ev.extend(sq(solo, Cello, 0.75, bt(22.0)));
            ev.extend(sq(fanfare, Brass, 0.6, bt(26.0)));
            ev.extend(sq(&oct(fanfare, 1), Horn, 0.4, bt(26.0)));
            ev.extend(sq(coda, Choir, 0.26, bt(38.0)));
            ev.extend(sq("D1+A1:48", Organ, 0.35, 0.0));
            ev.extend(sq("D1+A1:48", Organ, 0.3, bt(38.0)));
            // Totenglocken
            for bar in [0.0f32, 2.0, 4.0, 22.0, 23.0, 24.0, 25.0, 38.0, 40.0, 42.0] {
                ev.push(Ev { t: bt(bar), dur: 8.0, inst: Bell, f: freq(if bar % 2.0 == 0.0 { "D4" } else { "A3" }), vel: 0.35 });
            }
            let parts: [(u8, &[&str]); 7] = [
                (0, &["Dm", "Dm", "Bb", "Bb", "Eb", "A"]),
                (1, &["Dm", "Bb", "Eb", "Dm", "Dm", "Gm", "Eb", "A"]),
                (2, &["Dm", "Bb", "Eb", "Dm", "Dm", "Gm", "Eb", "A"]),
                (3, &["Bb", "Gm", "Eb", "A"]),
                (4, &["Dm", "C", "Bb", "A", "Gm", "Eb", "Dm", "A"]),
                (5, &["Dm", "Bb", "Eb", "Dm"]),
                (6, &["Dm", "Dm", "Bb", "A", "Dm", "Dm"]),
            ];
            let bars: Vec<(u8, &str)> = parts.iter().flat_map(|(s, l)| l.iter().map(move |c| (*s, *c))).collect();
            let timp = |t: f32, n: &str, v: f32| Ev { t, dur: 1.0, inst: Timpani, f: freq(n), vel: v };
            let boom = |t: f32, v: f32| Ev { t, dur: 1.0, inst: Boom, f: 46.0, vel: v };
            let crash = |t: f32, v: f32| Ev { t, dur: 1.0, inst: Crash, f: 0.0, vel: v };
            let chant = |t: f32, m: &[&str], pat: [usize; 8], v: f32| -> Vec<Ev> {
                pat.iter().enumerate().map(|(k, &p)| Ev { t: t + k as f32, dur: 0.9, inst: ChoirLo, f: freq(&oct(m[p], -1)), vel: v }).collect()
            };
            for (i, &(sec, c)) in bars.iter().enumerate() {
                let t = bt(i as f32);
                let (lo, mid, hi) = tch(c);
                let m: Vec<&str> = mid.split('+').collect();
                let tune = match c {
                    "A" | "C" => "A1",
                    "Gm" => "G1",
                    "Eb" => "D#2",
                    "Bb" => "A#1",
                    _ => "D2",
                };
                let bass = oct(lo, 1);
                match sec {
                    0 => {
                        if i >= 2 {
                            ev.extend(sq(&format!("{}:8", oct(mid, -1)), Trem, 0.12 + 0.03 * (i as f32 - 2.0), t));
                        }
                        if i == 5 {
                            for j in 0..16 {
                                ev.push(timp(t + j as f32 * 0.5, "A1", 0.1 + j as f32 * 0.03));
                            }
                        }
                    }
                    1 => {
                        // Aufbau: der Männerchor skandiert, alles wächst
                        let g = (i as f32 - 6.0) / 7.0;
                        ev.extend(chant(t, &m, [0, 0, 2, 0, 1, 2, 0, 2], 0.25 + 0.2 * g));
                        ev.extend(sq(&format!("{hi}:8"), Trem, 0.14 + 0.14 * g, t));
                        ev.extend([timp(t, tune, 0.3 + 0.25 * g), timp(t + 4.0, tune, 0.25 + 0.2 * g)]);
                        ev.extend(sq(&format!("{bass}:8"), Cello, 0.35, t));
                        if i % 2 == 0 {
                            ev.extend(sq(&format!("{}:8", oct(mid, -1)), Brass, 0.3 + 0.2 * g, t));
                        }
                        if i >= 10 {
                            ev.extend(sq(&format!("{mid}:8"), Organ, 0.25, t));
                        }
                        if i == 13 {
                            for j in 0..8 {
                                ev.push(boom(t + 4.0 + j as f32 * 0.5, 0.25 + j as f32 * 0.07));
                            }
                        }
                    }
                    2 | 4 => {
                        // volles Orchester: Hauptthema bzw. Höhepunkt
                        let s0 = if sec == 2 { 14 } else { 26 };
                        let k = if sec == 4 { 1.1 } else { 1.0 };
                        for (j, &p) in [0usize, 2, 1, 2, 0, 2, 1, 2].iter().enumerate() {
                            ev.push(Ev { t: t + j as f32, dur: 0.85, inst: Strings, f: freq(&oct(m[p], 1)), vel: 0.3 * k });
                        }
                        ev.extend(sq(&format!("{mid}:8"), Organ, 0.3 * k, t));
                        ev.extend(sq(&format!("{bass}:8"), Cello, 0.45, t));
                        ev.extend(sq(&format!("{bass}:2 R:2 {bass}:2 R:2"), Bass, 0.5, t));
                        for j in 0..4 {
                            ev.push(boom(t + j as f32 * 2.0, if j % 2 == 0 { 0.6 * k } else { 0.4 * k }));
                        }
                        ev.extend([timp(t, tune, 0.5 * k), timp(t + 4.0, tune, 0.4 * k)]);
                        if (i - s0) % 2 == 0 {
                            ev.push(crash(t, 0.45 * k));
                        }
                        if (i - s0) % 4 == 3 {
                            for (j, d) in [6.0f32, 6.5, 7.0, 7.5].iter().enumerate() {
                                ev.push(boom(t + d, (0.35 + j as f32 * 0.1) * k));
                            }
                        }
                        if sec == 4 {
                            ev.extend(sq(&format!("{hi}:8"), Choir, 0.45, t));
                            ev.extend(chant(t, &m, [0, 0, 2, 0, 0, 0, 2, 1], 0.45));
                        }
                    }
                    3 => {
                        // Einbruch: plötzlich leise
                        ev.extend(sq(&format!("{hi}:8"), Choir, 0.16, t));
                        ev.extend(sq(&format!("{}:8", oct(mid, -1)), Trem, 0.1, t));
                        if i == 25 {
                            for j in 0..16 {
                                ev.push(timp(t + j as f32 * 0.5, "A1", 0.1 + 0.045 * j as f32));
                            }
                            ev.extend([boom(t + 6.0, 0.4), boom(t + 7.0, 0.55), boom(t + 7.5, 0.7)]);
                        }
                    }
                    5 => {
                        // gewaltige Schlussakkorde, zuletzt ein Schlag und Stille
                        let last = i == 37;
                        let d = if last { 3 } else { 8 };
                        ev.extend(sq(&format!("{hi}:{d}"), Choir, 0.55, t));
                        ev.extend(sq(&format!("{}:{d}", oct(mid, -1)), ChoirLo, 0.55, t));
                        ev.extend(sq(&format!("{}:{d}", oct(mid, -1)), Brass, 0.55, t));
                        ev.extend(sq(&format!("{mid}:{d}"), Organ, 0.35, t));
                        ev.extend(sq(&format!("{hi}:{d}"), Trem, 0.3, t));
                        ev.extend(sq(&format!("{bass}:{d}"), Cello, 0.5, t));
                        ev.extend([boom(t, 0.8), timp(t, tune, 0.7), crash(t, 0.6)]);
                        if !last {
                            for j in 0..8 {
                                ev.push(timp(t + 4.0 + j as f32 * 0.5, tune, 0.2 + j as f32 * 0.05));
                            }
                        }
                    }
                    _ => {
                        ev.extend(sq(&format!("{mid}:8"), Strings, 0.14 * (1.0 - (i as f32 - 38.0) / 6.0), t));
                    }
                }
            }
            Song { unit: 60.0 / 60.0 / 2.0, len: 352.0, events: ev, gain: 0.85 }
        }
        // Strand der Ertränkten: Klagewalzer in a-Moll, Blechflöte und Akkordeon
        "salz" => {
            let a = "A4:2 C5:1 E5:3 | D5:2 C5:1 B4:3 | C5:2 B4:1 A4:2 G4:1 | A4:6 | E5:2 F5:1 E5:2 D5:1 | C5:2 B4:1 G4:3 | A4:2 B4:1 C5:2 B4:1 | A4:6";
            let b = "C5:2 D5:1 E5:3 | G5:2 F5:1 E5:3 | D5:2 C5:1 B4:2 G4:1 | E4:6 | A4:2 C5:1 E5:2 A5:1 | G5:2 E5:1 D5:3 | C5:2 B4:1 G#4:2 B4:1 | A4:6";
            let mut ev = sq(&format!("{a} | {b}"), Whistle, 0.45, 0.0);
            let chords = ["A2+E3+A3+C4", "G2+D3+G3+B3", "F2+C3+F3+A3", "A2+E3+A3+C4", "A2+E3+A3+C4", "E2+B2+E3+G3", "D2+A2+D3+F3", "A2+E3+A3+C4", "C3+G3+C4+E4", "G2+D3+G3+B3", "D2+A2+D3+F3", "E2+B2+E3+G3", "A2+E3+A3+C4", "C3+G3+C4+E4", "E2+B2+E3+G#3", "A2+E3+A3+C4"];
            for (i, c) in chords.iter().enumerate() {
                let t = i as f32 * 6.0;
                let up: Vec<&str> = c.split('+').skip(1).collect();
                let ch = up.join("+");
                ev.extend(sq(&format!("R:2 {ch}:2 {ch}:2"), Accordion, 0.1, t));
                ev.extend(sq(&format!("{}:6", first(c)), Cello, 0.55, t));
                ev.push(drum(t, 52.0, 0.22));
            }
            Song { unit: 60.0 / 66.0 / 3.0, len: 96.0, events: ev, gain: 0.5 }
        }
        // Auf See: langsames Seemannslied in d-Dorisch, Geige über Akkordeon und Cello
        "leine" => {
            let a = "D4:2 F4:1 A4:2 A4:1 | G4:2 F4:1 E4:3 | C4:2 E4:1 G4:2 G4:1 | F4:2 E4:1 D4:3 | D4:2 F4:1 A4:2 D5:1 | C5:2 A4:1 G4:3 | F4:2 E4:1 C4:2 E4:1 | D4:6";
            let b = "A4:2 C5:1 D5:3 | C5:2 A4:1 G4:3 | A4:2 G4:1 F4:2 E4:1 | D4:6 | F4:2 G4:1 A4:3 | G4:2 E4:1 C4:3 | D4:2 E4:1 F4:2 E4:1 | D4:6";
            let mut ev = sq(&format!("{a} | {b}"), Fiddle, 0.42, 0.0);
            ev.extend(sq(&format!("R:48 | {}", b.replace("4:", "5:").replace("5:6", "5:6")), Whistle, 0.16, 0.0));
            let chords = ["D3+F3+A3", "C3+E3+G3", "C3+E3+G3", "D3+F3+A3", "D3+F3+A3", "A2+C3+E3", "C3+E3+G3", "D3+F3+A3", "F3+A3+C4", "C3+E3+G3", "D3+F3+A3", "D3+F3+A3", "F3+A3+C4", "C3+E3+G3", "A2+C3+E3", "D3+F3+A3"];
            for (i, c) in chords.iter().enumerate() {
                let t = i as f32 * 6.0;
                ev.extend(sq(&format!("{c}:3 {c}:3"), Accordion, 0.11, t));
                ev.extend(sq(&format!("{}:6", down(c)), Cello, 0.5, t));
                ev.extend([drum(t, 55.0, 0.3), tick(t + 3.0, 0.08)]);
            }
            Song { unit: 60.0 / 80.0 / 3.0, len: 96.0, events: ev, gain: 0.5 }
        }
        // Am Säbelfeuer: Cello allein, darunter leise Harfe
        "atem" => {
            let mut ev = sq("D3:2 F3:2 A3:4 G3:1 F3:1 E3:2 D3:4 | A2:2 D3:2 F3:4 E3:2 C3:2 D3:4", Cello, 0.75, 0.0);
            for (i, c) in ["D3+A3+D4+F4", "A#2+F3+A#3+D4", "C3+G3+C4+E4", "D3+A3+D4+F4"].iter().enumerate() {
                ev.extend(arp(c, 8.0, 1.0, i as f32 * 8.0, 0.35));
                ev.extend(sq(&format!("{c}:8"), Pad, 0.08, i as f32 * 8.0));
            }
            Song { unit: 60.0 / 58.0 / 2.0, len: 32.0, events: ev, gain: 0.55 }
        }
        // Glutwacht: Glocken und Harfe wie in einem verlassenen Schrein, e-Moll
        "hain" => {
            let mel = "B4:3 G4:3 | A4:3 F#4:3 | G4:3 E4:3 | F#4:6 | B4:3 C5:3 | D5:3 B4:3 | A4:3 F#4:3 | E4:6";
            let mut ev = sq(mel, Bell, 0.45, 0.0);
            ev.extend(sq(&mel.replace("4:", "3:").replace("5:", "4:"), Cello, 0.45, 0.0));
            for (i, c) in ["E3+B3+E4+G4", "D3+A3+D4+F#4", "C3+G3+C4+E4", "B2+F#3+B3+D4", "E3+B3+E4+G4", "G2+D3+G3+B3", "D3+A3+D4+F#4", "E3+B3+E4+G4"].iter().enumerate() {
                let t = i as f32 * 6.0;
                ev.extend(arp(c, 6.0, 0.75, t, 0.4));
                ev.extend(sq(&format!("{c}:6"), Pad, 0.1, t));
            }
            Song { unit: 60.0 / 56.0 / 2.0, len: 48.0, events: ev, gain: 0.55 }
        }
        // Aschebucht / „Glutwind“: d-Moll, Geige, Cello-Pizzicato als Harfe, ferne Trommel
        "glut" => {
            let a = "D5:1 D5:1 F5:1 A5:2 F5:1 | G5:1 F5:1 E5:1 D5:3 | C5:1 C5:1 E5:1 G5:2 E5:1 | F5:1 E5:1 D5:1 C#5:3 | D5:1 F5:1 A5:1 D6:2 C6:1 | A#5:1 A5:1 G5:1 A5:3 | G5:1 F5:1 E5:1 F5:1 E5:1 C#5:1 | D5:6";
            let mut ev = sq(&a.replace("5:", "4:").replace("6:", "5:"), Fiddle, 0.45, 0.0);
            for (i, c) in ["D3+F3+A3", "A#2+D3+F3", "C3+E3+G3", "A2+C#3+E3", "D3+F3+A3", "G2+A#2+D3", "A2+C#3+E3", "D3+F3+A3"].iter().enumerate() {
                let t = i as f32 * 6.0;
                ev.extend(arp(c, 6.0, 1.0, t, 0.35));
                ev.extend(sq(&format!("{}:6", down(c)), Cello, 0.55, t));
                ev.extend([drum(t, 50.0, 0.4), drum(t + 3.0, 46.0, 0.25)]);
            }
            Song { unit: 60.0 / 70.0 / 3.0, len: 48.0, events: ev, gain: 0.5 }
        }
        // Frostgrotte / „Eisige Leine“: a-Moll, Flöte über Harfe und Flächen
        "frost" => {
            let mel = "A4:4 C5:2 E5:2 | D5:4 B4:4 | C5:2 B4:2 A4:2 G4:2 | A4:8 | E5:4 G5:2 E5:2 | F5:4 D5:4 | E5:2 D5:2 C5:2 B4:2 | A4:8";
            let mut ev = sq(mel, Whistle, 0.4, 0.0);
            for (i, c) in ["A2+E3+A3+C4", "G2+D3+G3+B3", "F2+C3+F3+A3", "E2+B2+E3+G#3", "A2+E3+A3+C4", "D2+A2+D3+F3", "E2+B2+E3+G#3", "A2+E3+A3+C4"].iter().enumerate() {
                let t = i as f32 * 8.0;
                ev.extend(arp(c, 8.0, 1.0, t, 0.3));
                ev.extend(sq(&format!("{c}:8"), Pad, 0.12, t));
            }
            Song { unit: 60.0 / 52.0 / 2.0, len: 64.0, events: ev, gain: 0.55 }
        }
        // Rabenwacht / „Rabensturm“: g-Moll, Geige ruft, das Cello antwortet
        "sturm" => {
            let call = "G4:1 G4:1 A#4:1 D5:1 G5:2 F5:2 | D#5:1 D5:1 C5:1 A#4:1 C5:4 | R:8 | A#4:1 A#4:1 D5:1 F5:1 A#5:2 A5:2 | G5:1 F5:1 D#5:1 D5:1 G4:4 | R:8";
            let answer = "R:16 | D3:2 D3:1 C3:1 A#2:4 | R:16 | G2:2 A2:2 A#2:2 G2:2";
            let mut ev = sq(call, Fiddle, 0.45, 0.0);
            ev.extend(sq(answer, Cello, 0.85, 0.0));
            for (i, c) in ["G2+A#2+D3", "D#2+G2+A#2", "D2+F#2+A2", "G2+A#2+D3", "F2+A2+C3", "G2+A#2+D3"].iter().enumerate() {
                let t = i as f32 * 8.0;
                ev.extend(sq(&format!("{c}:2 R:2 {c}:2 R:2"), Accordion, 0.12, t));
                ev.extend(sq(&format!("{}:8", first(c)), Cello, 0.4, t));
                ev.extend([drum(t, 58.0, 0.45), tick(t + 2.0, 0.12), drum(t + 4.0, 55.0, 0.3), tick(t + 6.0, 0.12)]);
            }
            Song { unit: 60.0 / 90.0 / 2.0, len: 48.0, events: ev, gain: 0.5 }
        }
        // Fluchmangroven / „Tiefer Grund“: c-Moll, Akkordeon und Cello in der Tiefe
        "tiefe" => {
            let mel = "C4:3 D#4:1 G4:4 | F4:2 D#4:2 D4:4 | C4:3 D#4:1 G4:2 G#4:2 | G4:8 | G#4:3 G4:1 F4:4 | D#4:2 D4:2 C4:4 | B3:2 C4:2 D4:2 D#4:2 | C4:8";
            let mut ev = sq(mel, AccordionSolo, 0.28, 0.0);
            ev.extend(sq(&mel.replace("4:", "3:").replace("3:3 ", "3:3 "), Cello, 0.55, 0.0));
            for (i, c) in ["C2+G2", "G#1+D#2", "C2+G2", "G1+D2", "G#1+D#2", "F1+C2", "G1+D2", "C2+G2"].iter().enumerate() {
                let t = i as f32 * 8.0;
                ev.extend(sq(&format!("{c}:8"), Pad, 0.18, t));
                ev.extend([drum(t, 42.0, 0.4), drum(t + 5.0, 40.0, 0.25)]);
            }
            Song { unit: 60.0 / 50.0 / 2.0, len: 64.0, events: ev, gain: 0.6 }
        }
        // Ende der Welt: Geigenklage und tiefe Flächen, d-Moll
        "ende" => {
            let mut ev = sq("A5:6 G5:2 F5:4 E5:4 | D5:8 R:8 | A#5:6 A5:2 G5:4 F5:4 | E5:12 R:4", Fiddle, 0.36, 0.0);
            ev.extend(sq("D3:16 | D3:8 C3:8 | A#2:16 | A2:16", Cello, 0.6, 0.0));
            for (i, c) in ["D2+A2+F3", "D2+A2+F3", "A#1+F2+D3", "A1+E2+C#3"].iter().enumerate() {
                let t = i as f32 * 16.0;
                ev.extend(sq(&format!("{c}:16"), Pad, 0.2, t));
                for b in 0..4 {
                    ev.push(drum(t + b as f32 * 4.0, 38.0, 0.35));
                }
            }
            ev.push(Ev { t: 0.0, dur: 8.0, inst: Bell, f: freq("D4"), vel: 0.3 });
            Song { unit: 60.0 / 60.0 / 2.0, len: 64.0, events: ev, gain: 0.55 }
        }
        // Bosse: drängend, aber düster – e-Phrygisch, Geige, Cello-Ostinato, Trommeln
        // Shanty (Nutzerwunsch 15.11.): episch und rein instrumental – die Arme holen den Sarg, die
        // Strömung trägt ihn zur Galgenfeste. d-Moll, Stampftakt, Geige und Blechflöte im Refrain.
        "shanty" => {
            let verse = "D4:2 D4:1 F4:1 A4:2 A4:2 | G4:1 F4:1 E4:1 D4:1 C4:2 A3:2 | D4:2 D4:1 F4:1 A4:2 D5:2 | C5:1 A4:1 G4:1 E4:1 D4:4 | F4:2 F4:1 A4:1 C5:2 C5:2 | A#4:1 A4:1 G4:1 F4:1 E4:2 C4:2 | D4:2 F4:1 A4:1 D5:2 C5:1 A4:1 | G4:1 E4:1 C#4:1 E4:1 D4:4";
            let chorus = "A4:2 D5:2 D5:1 E5:1 F5:2 | E5:1 D5:1 C5:1 A4:1 C5:4 | A#4:2 D5:2 D5:1 E5:1 F5:2 | G5:2 F5:1 E5:1 D5:4 | A4:2 D5:2 D5:1 E5:1 F5:2 | G5:1 F5:1 E5:1 D5:1 C5:2 A4:2 | A#4:1 A4:1 G4:1 F4:1 E4:2 C#5:2 | D5:8";
            let mut ev = sq(verse, Fiddle, 0.42, 0.0);
            ev.extend(sq(verse, AccordionSolo, 0.2, 0.0));
            ev.extend(sq(chorus, Fiddle, 0.5, 64.0));
            ev.extend(sq(chorus, Whistle, 0.22, 64.0));
            let chords = [
                "D2+A2+F3", "C2+G2+E3", "D2+A2+F3", "A1+E2+C3", "F2+A2+C3", "G2+A#2+D3", "D2+A2+F3", "A1+E2+C#3",
                "D2+A2+F3", "C2+G2+E3", "A#1+F2+D3", "G2+A#2+D3", "D2+A2+F3", "C2+G2+E3", "A1+E2+C#3", "D2+A2+F3",
            ];
            for (i, c) in chords.iter().enumerate() {
                let t = i as f32 * 8.0;
                let r = first(c);
                ev.extend(sq(&format!("{r}:8"), Cello, 0.5, t));
                ev.extend(sq(&format!("{r}:2 R:2 {r}:2 R:2"), Bass, 0.6, t));
                ev.extend(sq(&format!("R:2 {c}:1 R:3 {c}:1 R:1"), Accordion, 0.14, t));
                ev.extend([drum(t, 52.0, 0.55), tick(t + 2.0, 0.2), drum(t + 4.0, 50.0, 0.45), tick(t + 6.0, 0.2)]);
                if i >= 8 {
                    ev.extend(sq(&format!("{c}:8"), Pad, 0.14, t));
                    ev.extend([drum(t + 3.0, 56.0, 0.25), drum(t + 7.0, 56.0, 0.32)]);
                }
            }
            Song { unit: 60.0 / 104.0 / 2.0, len: 128.0, events: ev, gain: 0.55 }
        }
        "boss" => {
            let mel = "E5:1 F5:1 E5:1 G5:1 E5:1 F5:1 E5:2 | D5:1 E5:1 D5:1 C5:1 B4:4 | E5:1 F5:1 E5:1 G5:1 A5:1 G5:1 F5:2 | E5:1 D5:1 C5:1 D5:1 E5:4";
            let mut ev = sq(&format!("{mel} | {mel}"), Fiddle, 0.46, 0.0);
            for (i, r) in ["E2", "C2", "E2", "D2", "E2", "C2", "F2", "E2"].iter().enumerate() {
                let t = i as f32 * 8.0;
                let r3 = r.replace('2', "3");
                ev.extend(sq(&format!("{r}:1 {r}:1 R:1 {r}:1 {r}:1 R:1 {r}:2"), Bass, 0.6, t));
                ev.extend(sq(&format!("{r3}:2 {r3}:2 {r3}:2 {r3}:2"), Cello, 0.45, t));
                for b in 0..8 {
                    let v = if b % 4 == 0 { 0.85 } else if b % 2 == 1 { 0.25 } else { 0.5 };
                    ev.push(if b % 2 == 1 { tick(t + b as f32, v) } else { drum(t + b as f32, 56.0, v) });
                }
            }
            Song { unit: 60.0 / 118.0 / 2.0, len: 64.0, events: ev, gain: 0.5 }
        }
        _ => return None,
    };
    Some(s)
}
pub const SONGS: [&str; 12] = ["titel", "salz", "hain", "atem", "leine", "glut", "frost", "sturm", "tiefe", "ende", "boss", "shanty"];

/// Eine Note in den Puffer (zyklisch, damit die Schleife nahtlos ist).
pub fn render_event(buf: &mut [f32], e: &Ev, start: usize, dur: f32, rng: &mut Rng) {
    use Inst::*;
    let f = e.f;
    let v = e.vel;
    let tail = match e.inst {
        Bell => 1.5,
        Harp => 1.6,
        Cello => 0.25,
        Pad => 0.1,
        Strings | Trem => 0.3,
        Choir => 0.5,
        ChoirLo => 0.6,
        Organ => 0.4,
        Brass => 0.2,
        Horn => 0.15,
        _ => 0.05,
    };
    let total = match e.inst {
        Drum => 0.35,
        Tick => 0.04,
        Bass => dur.min(0.6) + 0.02,
        Bell => (dur + 1.4).min(2.4),
        Harp => (dur + 1.6).min(2.6),
        Timpani => 1.5,
        Boom => 2.2,
        Crash => 2.2,
        _ => dur + tail,
    };
    let n = (total * SR) as usize;
    let dt = 1.0 / SR;
    match e.inst {
        Whistle => {
            let (mut p1, mut p2, mut pv) = (0.0f32, 0.0f32, 0.0f32);
            for i in 0..n {
                let t = i as f32 * dt;
                pv += 5.8 * dt;
                let vib = (pv * std::f32::consts::TAU).sin() * f * 0.006 * (t / 0.25).min(1.0);
                p1 += (f + vib) * dt;
                p2 += 2.0 * f * dt;
                let s = osc(Wave::Tri, p1) + osc(Wave::Sine, p2) * 0.15;
                add(buf, start + i, s * env(t, 0.025, 0.22 * v, dur * 0.95, 0.06));
            }
        }
        Fiddle => {
            let mut lp = Bq::new(Ft::Low, 2400.0, 2.0);
            let (mut p, mut pv) = (0.0f32, 0.0f32);
            for i in 0..n {
                let t = i as f32 * dt;
                pv += 5.3 * dt;
                let vib = (pv * std::f32::consts::TAU).sin() * f * 0.008 * (t / 0.3).min(1.0);
                p += (f + vib) * dt;
                let s = lp.run(osc(Wave::Saw, p));
                add(buf, start + i, s * env(t, 0.07, 0.11 * v, dur * 0.97, 0.12));
            }
        }
        Cello => {
            let mut lp = Bq::new(Ft::Low, 1100.0, 0.9);
            let mut body = Bq::new(Ft::Band, 260.0, 1.2);
            let (mut p, mut q, mut pv) = (rng.u(), rng.u(), 0.0f32);
            for i in 0..n {
                let t = i as f32 * dt;
                pv += 5.0 * dt;
                let vib = (pv * std::f32::consts::TAU).sin() * f * 0.009 * ((t - 0.25) / 0.4).clamp(0.0, 1.0);
                p += (f + vib) * dt;
                q += (f * 1.003 + vib) * dt;
                let bow = 1.0 + (t * 3.1).sin() * 0.08;
                let raw = osc(Wave::Saw, p) * 0.7 + osc(Wave::Saw, q) * 0.3;
                let s = lp.run(raw) + body.run(raw) * 0.5;
                add(buf, start + i, s * bow * env(t, 0.18, 0.15 * v, dur + 0.2, 0.3));
            }
        }
        Harp => {
            // Karplus-Strong: Rauschstoß in einer Verzögerungsleitung, sanft gefiltert
            let len = ((SR / f.max(30.0)) as usize).max(2);
            let mut line: Vec<f32> = (0..len).map(|_| rng.f()).collect();
            for k in 1..len {
                line[k] = line[k] * 0.5 + line[k - 1] * 0.5;
            }
            let mut j = 0usize;
            let damp = 0.996f32;
            for i in 0..n {
                let t = i as f32 * dt;
                let a = line[j];
                let b = line[(j + 1) % len];
                let y = (a + b) * 0.5 * damp;
                line[j] = y;
                j = (j + 1) % len;
                let g = 0.32 * v * if t < 0.003 { t / 0.003 } else { 1.0 } * (1.0 - (t / total).powi(3)).max(0.0);
                add(buf, start + i, a * g);
            }
        }
        Accordion | AccordionSolo => {
            let solo = e.inst == AccordionSolo;
            let mut lp = Bq::new(Ft::Low, if solo { 1700.0 } else { 1300.0 }, 0.7);
            let (fa, fb) = (f * 2f32.powf(-7.0 / 1200.0), f * 2f32.powf(7.0 / 1200.0));
            let (mut a, mut b, mut c) = (rng.u(), rng.u(), 0.0f32);
            for i in 0..n {
                let t = i as f32 * dt;
                a += fa * dt;
                b += fb * dt;
                c += f * 0.5 * dt;
                let s = lp.run(osc(Wave::Saw, a) + osc(Wave::Saw, b) + osc(Wave::Square, c) * 0.25);
                let trem = 1.0 + (t * 5.2 * std::f32::consts::TAU).sin() * 0.25;
                add(buf, start + i, s * trem * env(t, if solo { 0.08 } else { 0.04 }, 0.07 * v, dur * 0.98, 0.1));
            }
        }
        Pad => {
            let mut lp = Bq::new(Ft::Low, 700.0, 0.7);
            let fs = [f * 2f32.powf(-9.0 / 1200.0), f, f * 2f32.powf(9.0 / 1200.0)];
            let mut ph = [rng.u(), rng.u(), rng.u()];
            let a = (dur * 0.3).min(1.2);
            let r = (dur * 0.4).min(1.2);
            for i in 0..n {
                let t = i as f32 * dt;
                let mut s = 0.0;
                for k in 0..3 {
                    ph[k] += fs[k] * dt;
                    s += osc(Wave::Saw, ph[k]);
                }
                add(buf, start + i, lp.run(s) * env(t, a, 0.05 * v, dur, r));
            }
        }
        Bass => {
            let d = dur.min(0.6);
            let (mut a, mut b) = (0.0f32, 0.0f32);
            for i in 0..n {
                let t = i as f32 * dt;
                a += f * dt;
                b += f * 0.5 * dt;
                let g = if t < 0.01 { 0.3 * v * t / 0.01 } else { exp_decay(t - 0.01, 0.3 * v, d) };
                add(buf, start + i, (osc(Wave::Tri, a) + osc(Wave::Sine, b)) * g);
            }
        }
        Drum => {
            let mut p = 0.0f32;
            let mut lp = Bq::new(Ft::Low, 300.0, 1.0);
            for i in 0..n {
                let t = i as f32 * dt;
                let fr = if t < 0.18 { f * 1.8 * (0.7f32 / 1.8).powf(t / 0.18) } else { f * 0.7 };
                p += fr * dt;
                let s = osc(Wave::Sine, p) * exp_decay(t, 0.5 * v, 0.3) + lp.run(rng.f()) * exp_decay(t, 0.12 * v, 0.06) * 3.0;
                add(buf, start + i, s);
            }
        }
        Tick => {
            let mut bp = Bq::new(Ft::Band, 2200.0, 1.0);
            for i in 0..n {
                let t = i as f32 * dt;
                add(buf, start + i, bp.run(rng.f()) * exp_decay(t, 0.18 * v, 0.03) * 2.0);
            }
        }
        Strings | Trem => {
            let trem = e.inst == Trem;
            let mut lp = Bq::new(Ft::Low, 1800.0, 0.7);
            let fs = [-11.0f32, -4.0, 4.0, 11.0].map(|c| f * 2f32.powf(c / 1200.0));
            let mut ph = [rng.u(), rng.u(), rng.u(), rng.u()];
            let a = (dur * 0.35).clamp(0.01, 0.22);
            let r = (dur * 0.3).clamp(0.04, 0.35);
            let p0 = rng.u();
            let (mut vib, mut g) = (1.0f32, 0.0f32);
            for i in 0..n {
                let t = i as f32 * dt;
                // Vibrato und Hüllkurve nur alle 8 Abtastwerte (spart die teuren sin/powf)
                if i % 8 == 0 {
                    vib = 1.0 + ((p0 + 5.5 * t) * std::f32::consts::TAU).sin() * 0.0035 * (t / 0.4).min(1.0);
                    g = env(t, a, 0.045 * v, dur + r * 0.5, r);
                    if trem {
                        // schnelles Hin- und Herstreichen
                        g *= 0.55 + 0.45 * ((p0 * 3.0 + 12.5 * t) * std::f32::consts::TAU).sin().abs();
                    }
                }
                let mut s = 0.0;
                for k in 0..4 {
                    ph[k] += fs[k] * vib * dt;
                    s += osc(Wave::Saw, ph[k]);
                }
                add(buf, start + i, lp.run(s) * g);
            }
        }
        Choir => {
            // „Aah“: drei verstimmte Sägezähne durch drei Formantfilter
            let (mut f1, mut f2, mut f3) = (Bq::new(Ft::Band, 730.0, 2.5), Bq::new(Ft::Band, 1090.0, 3.5), Bq::new(Ft::Band, 2440.0, 5.0));
            let fs = [f * 0.996, f, f * 1.004];
            let mut ph = [rng.u(), rng.u(), rng.u()];
            let a = (dur * 0.4).clamp(0.02, 0.45);
            let p0 = rng.u();
            let (mut vib, mut g) = (1.0f32, 0.0f32);
            for i in 0..n {
                let t = i as f32 * dt;
                if i % 8 == 0 {
                    vib = 1.0 + ((p0 + 4.6 * t) * std::f32::consts::TAU).sin() * 0.005 * (t / 0.6).min(1.0);
                    g = env(t, a, 0.16 * v, dur + 0.4, 0.5);
                }
                let mut s = 0.0;
                for k in 0..3 {
                    ph[k] += fs[k] * vib * dt;
                    s += osc(Wave::Saw, ph[k]);
                }
                add(buf, start + i, (f1.run(s) + f2.run(s) * 0.55 + f3.run(s) * 0.22) * g);
            }
        }
        Horn => {
            let mut lp = Bq::new(Ft::Low, 500.0, 0.9);
            let (mut p, mut q, mut pv) = (rng.u(), rng.u(), 0.0f32);
            let a = (dur * 0.3).clamp(0.01, 0.09);
            for i in 0..n {
                let t = i as f32 * dt;
                if i % 32 == 0 {
                    // Blechglanz: der Filter öffnet sich beim Anblasen und schließt sich etwas
                    let cut = 500.0 + 1300.0 * (t / 0.12).min(1.0) - 450.0 * ((t - 0.12).max(0.0) / 0.5).min(1.0);
                    lp.set(Ft::Low, cut.min(f * 6.0 + 300.0), 0.9);
                }
                pv += 4.8 * dt;
                let vib = (pv * std::f32::consts::TAU).sin() * f * 0.004 * ((t - 0.3) / 0.4).clamp(0.0, 1.0);
                p += (f + vib) * dt;
                q += (f * 1.002 + vib) * dt;
                let s = lp.run(osc(Wave::Saw, p) + osc(Wave::Saw, q) * 0.6);
                add(buf, start + i, s * env(t, a, 0.12 * v, dur + 0.08, 0.15));
            }
        }
        Timpani => {
            let mut lp = Bq::new(Ft::Low, 180.0, 0.8);
            let (mut p, mut p2) = (0.0f32, 0.0f32);
            for i in 0..n {
                let t = i as f32 * dt;
                let fr = f * (1.0 + 0.06 * (-t * 30.0).exp());
                p += fr * dt;
                p2 += fr * 1.5 * dt;
                let body = osc(Wave::Sine, p) + osc(Wave::Sine, p2) * 0.3 * (-t * 6.0).exp();
                let hit = lp.run(rng.f()) * (-t * 40.0).exp() * 2.0;
                let g = if t < 0.003 { t / 0.003 } else { 1.0 };
                add(buf, start + i, (body * exp_decay(t, 0.45 * v, 1.2) + hit * 0.4 * v) * g);
            }
        }
        Crash => {
            let (mut hp, mut bp) = (Bq::new(Ft::High, 4500.0, 0.7), Bq::new(Ft::Band, 7500.0, 0.8));
            for i in 0..n {
                let t = i as f32 * dt;
                let x = rng.f();
                let g = if t < 0.004 { t / 0.004 } else { 1.0 };
                add(buf, start + i, (hp.run(x) * 0.6 + bp.run(x) * 0.5) * exp_decay(t, 0.12 * v, 2.0) * g);
            }
        }
        ChoirLo => {
            // Männerchor „Oh“: vier verstimmte Sägezähne durch dunkle Formanten
            let (mut f1, mut f2, mut f3) = (Bq::new(Ft::Band, 460.0, 2.5), Bq::new(Ft::Band, 830.0, 3.5), Bq::new(Ft::Band, 2700.0, 6.0));
            let fs = [0.992f32, 0.998, 1.003, 1.008].map(|k| f * k);
            let mut ph = [rng.u(), rng.u(), rng.u(), rng.u()];
            let a = (dur * 0.4).clamp(0.02, 0.5);
            let p0 = rng.u();
            let (mut vib, mut g) = (1.0f32, 0.0f32);
            for i in 0..n {
                let t = i as f32 * dt;
                if i % 8 == 0 {
                    vib = 1.0 + ((p0 + 4.2 * t) * std::f32::consts::TAU).sin() * 0.006 * (t / 0.7).min(1.0);
                    g = env(t, a, 0.22 * v, dur + 0.5, 0.6);
                }
                let mut s = 0.0;
                for k in 0..4 {
                    ph[k] += fs[k] * vib * dt;
                    s += osc(Wave::Saw, ph[k]);
                }
                add(buf, start + i, (f1.run(s) + f2.run(s) * 0.5 + f3.run(s) * 0.1) * g);
            }
        }
        Organ => {
            // Orgel: Unteroktave, Grundton, Oktave, Quinte, Doppeloktave als Dreiecke, sachtes Schweben
            let parts = [(0.5f32, 0.45f32), (1.0, 1.0), (2.0, 0.55), (3.0, 0.3), (4.0, 0.22)];
            let mut ph = [0.0f32; 5];
            let mut lp = Bq::new(Ft::Low, 2600.0, 0.7);
            let (mut g, mut tr) = (0.0f32, 1.0f32);
            for i in 0..n {
                let t = i as f32 * dt;
                if i % 8 == 0 {
                    g = env(t, 0.09, 0.05 * v, dur + 0.25, 0.45);
                    tr = 1.0 + (t * 5.4 * std::f32::consts::TAU).sin() * 0.05;
                }
                let mut s = 0.0;
                for k in 0..5 {
                    ph[k] += f * parts[k].0 * dt;
                    s += osc(Wave::Tri, ph[k]) * parts[k].1;
                }
                add(buf, start + i, lp.run(s) * g * tr);
            }
        }
        Brass => {
            // tiefe Blechbläser: drei Sägezähne, der Filter reißt beim Einsatz auf
            let mut lp = Bq::new(Ft::Low, 300.0, 1.1);
            let fs = [0.997f32, 1.0, 1.004].map(|k| f * k);
            let mut ph = [rng.u(), rng.u(), rng.u()];
            let a = (dur * 0.3).clamp(0.02, 0.14);
            let mut g = 0.0f32;
            for i in 0..n {
                let t = i as f32 * dt;
                if i % 16 == 0 {
                    let cut = 300.0 + 1900.0 * (t / 0.25).min(1.0) - 700.0 * ((t - 0.25).max(0.0) / 0.8).min(1.0);
                    lp.set(Ft::Low, cut.min(f * 9.0 + 200.0), 1.1);
                    g = env(t, a, 0.1 * v, dur + 0.15, 0.25);
                }
                let mut s = 0.0;
                for k in 0..3 {
                    ph[k] += fs[k] * dt;
                    s += osc(Wave::Saw, ph[k]);
                }
                add(buf, start + i, lp.run(s) * g);
            }
        }
        Boom => {
            // große Trommel: tiefer Schlag mit fallender Tonhöhe, Rauschstoß, langes Nachgrollen
            let mut lp = Bq::new(Ft::Low, 220.0, 0.8);
            let (mut p, mut q) = (0.0f32, 0.0f32);
            for i in 0..n {
                let t = i as f32 * dt;
                let fr = f * (1.0 + 1.4 * (-t * 25.0).exp());
                p += fr * dt;
                q += fr * 0.5 * dt;
                let g = if t < 0.002 { t / 0.002 } else { 1.0 };
                let s = osc(Wave::Sine, p) * exp_decay(t, 0.6 * v, 1.4) + osc(Wave::Sine, q) * exp_decay(t, 0.3 * v, 1.8) + lp.run(rng.f()) * exp_decay(t, 0.5 * v, 0.12) * 2.0;
                add(buf, start + i, s * g);
            }
        }
        Bell => {
            let parts = [(1.0, 1.0), (2.76, 0.35), (5.4, 0.12)];
            let mut ph = [0.0f32; 3];
            let d = (dur + 1.4).min(2.4);
            for i in 0..n {
                let t = i as f32 * dt;
                let mut s = 0.0;
                for k in 0..3 {
                    ph[k] += f * parts[k].0 * dt;
                    s += osc(Wave::Sine, ph[k]) * parts[k].1;
                }
                let g = if t < 0.005 { 0.16 * v * t / 0.005 } else { exp_decay(t - 0.005, 0.16 * v, d) };
                add(buf, start + i, s * g);
            }
        }
    }
}

/// Schrittweises Rendern eines Liedes, damit kein Bild ruckelt.
pub struct SongRender {
    pub name: &'static str,
    pub song: Song,
    pub music: Vec<f32>,
    pub next: usize,
    rng: Rng,
    /// Titel: Hall und Begrenzer, in Stücken nach den Noten
    verb: Option<Verb>,
    post: usize,
}

/// Hall (Schroeder: vier gedämpfte Kammfilter, zwei Allpässe)
struct Verb {
    combs: Vec<(Vec<f32>, usize, f32)>,
    aps: Vec<(Vec<f32>, usize)>,
}
impl Verb {
    fn new() -> Self {
        Verb { combs: [558usize, 594, 638, 678].iter().map(|&d| (vec![0.0; d], 0, 0.0)).collect(), aps: [278usize, 113].iter().map(|&d| (vec![0.0; d], 0)).collect() }
    }
    fn run(&mut self, x: f32) -> f32 {
        let mut wet = 0.0;
        for (line, j, lp) in self.combs.iter_mut() {
            let y = line[*j];
            *lp = y * 0.62 + *lp * 0.38;
            line[*j] = x + *lp * 0.86;
            *j = (*j + 1) % line.len();
            wet += y;
        }
        let mut o = wet * 0.25;
        for (line, j) in self.aps.iter_mut() {
            let y = line[*j];
            let out = y - o * 0.5;
            line[*j] = o + out * 0.5;
            *j = (*j + 1) % line.len();
            o = out;
        }
        x * 0.8 + o * 0.55
    }
}
/// sanfter Begrenzer: bis 0,7 unverändert, darüber weich gesättigt
fn soft(x: f32) -> f32 {
    let a = x.abs();
    if a < 0.7 {
        x
    } else {
        x.signum() * (0.7 + 0.3 * ((a - 0.7) / 0.3).tanh())
    }
}
impl SongRender {
    pub fn new(name: &'static str) -> Option<Self> {
        let song = song(name)?;
        let n = (song.len * song.unit * SR) as usize;
        let verb = (name == "titel").then(Verb::new);
        Some(SongRender { name, music: vec![0.0; n], next: 0, rng: Rng::new(name.len() as u32 * 7919 + 13), song, verb, post: 0 })
    }
    /// gibt true zurück, wenn fertig
    pub fn step(&mut self, max_events: usize) -> bool {
        let unit = self.song.unit;
        if self.next < self.song.events.len() {
            let end = (self.next + max_events).min(self.song.events.len());
            for k in self.next..end {
                let e = self.song.events[k];
                let start = (e.t * unit * SR) as usize;
                let dur = e.dur * unit;
                render_event(&mut self.music, &e, start, dur, &mut self.rng);
            }
            self.next = end;
            if self.next < self.song.events.len() || self.verb.is_some() {
                return false;
            }
        }
        if let Some(v) = self.verb.as_mut() {
            // Hall und Begrenzer je halbe Sekunde, damit kein Bild ruckelt
            let g = self.song.gain;
            let end = (self.post + 11025).min(self.music.len());
            for x in &mut self.music[self.post..end] {
                *x = soft(v.run(*x) * g);
            }
            self.post = end;
            return end >= self.music.len();
        }
        {
            // leichte „64-Bit“-Färbung auf der Musikspur
            let g = self.song.gain;
            for s in self.music.iter_mut() {
                let x = (*s * 1.6).clamp(-1.0, 1.0);
                *s = ((x * 48.0).round() / 48.0 * 0.2 + x * 0.8) * g;
            }
            true
        }
    }
}

// ------------------------------------------------------------------ Klangeffekte
struct Fx {
    buf: Vec<f32>,
    rng: Rng,
}
impl Fx {
    fn new(len: f32, seed: u32) -> Self {
        Fx { buf: vec![0.0; (len * SR) as usize], rng: Rng::new(seed) }
    }
    /// Rauschstoß mit optional gleitender Filterfrequenz
    fn noise(&mut self, t0: f32, f0: f32, f1: f32, dur: f32, vol: f32, ft: Ft, q: f32, lin: bool) {
        let mut bq = Bq::new(ft, f0, q);
        let s0 = (t0 * SR) as usize;
        let n = (dur * SR) as usize;
        for i in 0..n {
            let t = i as f32 / SR;
            if i % 32 == 0 && f1 != f0 {
                let k = (t / dur).min(1.0);
                let f = if lin { f0 + (f1 - f0) * k } else { f0 * (f1 / f0).powf(k) };
                bq.set(ft, f, q);
            }
            let x = bq.run(self.rng.f());
            if s0 + i < self.buf.len() {
                self.buf[s0 + i] += x * exp_decay(t, vol, dur) * if ft == Ft::Band { 2.0 } else { 1.0 };
            }
        }
    }
    fn tone(&mut self, t0: f32, w: Wave, f0: f32, f1: f32, dur: f32, vol: f32) {
        let s0 = (t0 * SR) as usize;
        let n = (dur * SR) as usize;
        let mut p = 0.0;
        for i in 0..n {
            let t = i as f32 / SR;
            let f = f0 * (f1.max(1.0) / f0).powf(t / dur);
            p += f / SR;
            if s0 + i < self.buf.len() {
                self.buf[s0 + i] += osc(w, p) * exp_decay(t, vol, dur);
            }
        }
    }
}

pub fn sfx(name: &str) -> Vec<f32> {
    use Ft::*;
    use Wave::*;
    let len = match name {
        "death" | "rise" | "bell" | "stinger" => 3.2,
        "discover" => 2.6,
        "bubbles" => 1.6,
        "surge" => 2.4,
        "plunge" => 1.6,
        "glass" => 0.9,
        "kindle" | "roar" => 1.8,
        "cannon" | "crash" | "tide" | "level" | "pearls" | "chest" | "zap" | "fog" => 1.0,
        "slither" => 1.7,
        "hiss" | "creak" => 0.9,
        "rustle" => 1.1,
        "skitter" | "clink" => 0.8,
        _ => 0.6,
    };
    let mut x = Fx::new(len, name.bytes().map(|b| b as u32).sum::<u32>() + 3);
    match name {
        "swing" => x.noise(0.0, 900.0, 3200.0, 0.16, 0.35, Band, 1.5, false),
        // das Schiff bricht durch die Oberfläche: tiefes Grollen, rauschende Wassermassen, Blasen
        "surge" => {
            x.tone(0.0, Sine, 38.0, 62.0, 1.6, 0.9);
            x.tone(0.0, Saw, 55.0, 80.0, 0.9, 0.18);
            x.noise(0.0, 420.0, 160.0, 2.2, 0.9, Low, 1.0, false);
            x.noise(0.05, 1400.0, 700.0, 1.6, 0.45, Band, 0.7, false);
            for k in 0..12 {
                let t = 0.3 + k as f32 * 0.13 + x.rng.u() * 0.08;
                let (a, b) = (220.0 + x.rng.u() * 300.0, 700.0 + x.rng.u() * 500.0);
                x.tone(t, Sine, a, b, 0.08, 0.07);
            }
        }
        // das Schiff kracht zurück ins Meer: dumpfer Schlag, Gischt, nachrieselndes Wasser
        "plunge" => {
            x.tone(0.0, Sine, 70.0, 34.0, 0.5, 0.9);
            x.noise(0.0, 900.0, 300.0, 0.5, 0.9, Low, 0.9, false);
            x.noise(0.02, 2600.0, 1200.0, 1.3, 0.5, Band, 0.6, false);
            x.noise(0.4, 3800.0, 3800.0, 1.1, 0.16, Band, 2.0, false);
        }
        // Glas zerspringt: helles Klirren, rieselnde Scherben
        "glass" => {
            x.noise(0.0, 4200.0, 6500.0, 0.08, 0.5, Band, 4.0, false);
            for k in 0..10 {
                let t = k as f32 * 0.05 + x.rng.u() * 0.05;
                let f = 2400.0 + x.rng.u() * 3400.0;
                x.tone(t, Sine, f, f * 0.98, 0.14, 0.11);
            }
            x.noise(0.12, 3000.0, 3000.0, 0.4, 0.16, Band, 3.0, false);
        }
        // Blubbern: Blasen steigen aus der Tiefe auf und platzen
        "bubbles" => {
            for k in 0..16 {
                let t = k as f32 * 0.085 + x.rng.u() * 0.06;
                let (a, b) = (200.0 + x.rng.u() * 260.0, 520.0 + x.rng.u() * 520.0);
                x.tone(t, Sine, a, b, 0.06, 0.1);
            }
            x.noise(0.0, 260.0, 260.0, 1.4, 0.3, Low, 1.0, false);
        }
        // Spinnfaden: zischend, klebrig knisternd
        "silk" => {
            x.noise(0.0, 2400.0, 5200.0, 0.22, 0.22, Band, 3.0, false);
            for k in 0..3 {
                x.noise(0.05 + k as f32 * 0.06, 1800.0, 1800.0, 0.03, 0.18, Band, 6.0, false);
            }
        }
        // Fauchen der Spinne
        "hiss" => {
            for k in 0..6 {
                x.noise(k as f32 * 0.1, 3600.0, 4200.0, 0.14, 0.32 - k as f32 * 0.03, Band, 2.0, false);
            }
            x.tone(0.0, Saw, 95.0, 70.0, 0.5, 0.08);
        }
        // nasses Gleiten der Arme über Fels
        "slither" => {
            for k in 0..7 {
                let t = k as f32 * 0.18;
                x.noise(t, 220.0 + (k % 3) as f32 * 90.0, 520.0, 0.26, 0.45, Band, 1.6, false);
            }
            x.tone(0.0, Sine, 55.0, 42.0, 1.4, 0.25);
        }
        // Sargdeckel schabt über Holz
        "creak" => {
            x.tone(0.0, Saw, 210.0, 150.0, 0.55, 0.12);
            x.tone(0.05, Saw, 330.0, 250.0, 0.45, 0.06);
            x.noise(0.0, 700.0, 500.0, 0.6, 0.25, Band, 2.0, false);
        }
        // musikalischer Stinger (Boss besiegt, große Momente): aufsteigender Moll-Akkord mit
        // Paukenschlag, schwillt an und verklingt
        "stinger" => {
            x.tone(0.0, Sine, 55.0, 41.0, 0.9, 0.7);
            x.noise(0.0, 160.0, 90.0, 0.6, 0.6, Low, 1.0, false);
            for (k, f) in [146.8f32, 174.6, 220.0, 293.7].iter().enumerate() {
                let t0 = 0.08 + k as f32 * 0.09;
                x.tone(t0, Saw, *f, *f, 2.6 - t0, 0.06);
                x.tone(t0, Tri, f * 2.0, f * 2.0, 2.4 - t0, 0.05);
            }
            x.tone(0.5, Sine, 587.3, 587.3, 2.4, 0.06);
        }
        // Entdeckung (neuer Ort): leise Glockenfolge, zwei Töne nach oben
        "discover" => {
            for (k, (f, v)) in [(392.0f32, 0.12f32), (523.3, 0.1), (659.3, 0.08)].iter().enumerate() {
                let t0 = k as f32 * 0.22;
                x.tone(t0, Sine, *f, *f, 2.2 - t0, *v);
                x.tone(t0, Sine, f * 2.76, f * 2.76, 0.9, v * 0.25);
            }
        }
        // Kampflaut der Spielfigur (kein Wort: gefilterter Atemstoß mit Vokalfärbung)
        "grunt" => {
            x.noise(0.0, 720.0, 640.0, 0.22, 0.38, Band, 6.0, false);
            x.noise(0.0, 1150.0, 1050.0, 0.2, 0.22, Band, 7.0, false);
            x.tone(0.0, Saw, 128.0, 104.0, 0.2, 0.08);
        }
        "bell" => {
            // tiefe Glocke unter Wasser: unharmonische Teiltöne, langer Nachhall
            for (f, v, d) in [(98.0f32, 0.32f32, 3.0f32), (196.0, 0.16, 2.4), (270.0, 0.14, 2.0), (530.0, 0.07, 1.4), (812.0, 0.04, 0.9)] {
                x.tone(0.0, Sine, f, f * 0.995, d, v);
            }
            x.noise(0.0, 300.0, 300.0, 0.05, 0.25, Band, 1.0, false);
        }
        "chain" => {
            for k in 0..6 {
                x.noise(k as f32 * 0.06, 2600.0, 2200.0, 0.05, 0.35, Band, 1.0, false);
            }
        }
        // die Spinne in der Ferne: trockenes Rascheln, als schleife etwas durch Laub und Netze
        "rustle" => {
            for k in 0..5 {
                let t = k as f32 * 0.17 + (k % 2) as f32 * 0.05;
                x.noise(t, 1800.0 + (k % 3) as f32 * 500.0, 2600.0, 0.22, 0.32, Band, 1.1, false);
            }
            x.noise(0.1, 500.0, 300.0, 0.7, 0.12, Band, 0.8, false);
        }
        // ihre Schritte nah: hartes Klicken vieler Beine auf Fels, zwei Gruppen, dazu ein dumpfes Tappen
        "skitter" => {
            for k in 0..9 {
                let t = k as f32 * 0.055 + if k >= 5 { 0.16 } else { 0.0 };
                let f = 2600.0 + ((k * 7) % 5) as f32 * 420.0;
                x.noise(t, f, f * 0.8, 0.028, 0.55, Band, 3.0, false);
                x.tone(t, Sine, 210.0 - k as f32 * 6.0, 120.0, 0.05, 0.16);
            }
            x.noise(0.0, 260.0, 160.0, 0.5, 0.1, Low, 1.0, false);
        }
        // Krüge stoßen an (Kneipe): zwei helle, leicht verstimmte Klänge
        "clink" => {
            for (t, f) in [(0.0f32, 2350.0f32), (0.06, 3120.0)] {
                x.tone(t, Sine, f, f * 0.995, 0.45, 0.16);
                x.tone(t, Sine, f * 2.7, f * 2.68, 0.2, 0.05);
            }
            x.noise(0.0, 4200.0, 4200.0, 0.03, 0.12, Band, 2.0, false);
        }
        "thud" => {
            x.tone(0.0, Sine, 62.0, 40.0, 0.35, 0.5);
            x.noise(0.0, 120.0, 80.0, 0.2, 0.25, Low, 1.0, false);
        }
        "heavy" => x.noise(0.0, 400.0, 1800.0, 0.3, 0.5, Band, 1.2, false),
        "hit" => {
            x.noise(0.0, 1400.0, 1400.0, 0.08, 0.5, Band, 1.0, false);
            x.tone(0.0, Square, 220.0, 90.0, 0.09, 0.2);
        }
        "crit" => {
            x.noise(0.0, 900.0, 900.0, 0.25, 0.7, Band, 1.0, false);
            x.tone(0.0, Saw, 160.0, 40.0, 0.3, 0.35);
        }
        "hurt" => {
            x.noise(0.0, 500.0, 500.0, 0.2, 0.6, Low, 1.0, false);
            x.tone(0.0, Square, 140.0, 60.0, 0.18, 0.25);
        }
        "roll" => x.noise(0.0, 300.0, 900.0, 0.3, 0.3, Low, 1.0, true),
        "parry" => x.tone(0.0, Tri, 1800.0, 1500.0, 0.05, 0.12),
        "parried" => {
            x.tone(0.0, Square, 2400.0, 2300.0, 0.25, 0.25);
            x.tone(0.0, Tri, 3100.0, 3000.0, 0.4, 0.15);
            x.noise(0.0, 3000.0, 3000.0, 0.1, 0.4, Band, 1.0, false);
        }
        "drink" => {
            for (i, d) in [0.0, 0.12, 0.24, 0.36].iter().enumerate() {
                x.tone(*d, Sine, 320.0 + i as f32 * 17.0, 200.0, 0.08, 0.15);
            }
        }
        "heal" => {
            for i in 0..3 {
                x.tone(i as f32 * 0.08, Tri, 660.0 * (1.0 + i as f32 * 0.25), 700.0 * (1.0 + i as f32 * 0.25), 0.4, 0.12);
            }
        }
        "bones" => {
            for k in 0..6 {
                let t = k as f32 * 0.05 + x.rng.u() * 0.03;
                let f = 1500.0 + x.rng.u() * 1500.0;
                x.noise(t, f, f, 0.04, 0.35, Band, 4.0, false);
            }
        }
        "crush" => {
            x.noise(0.0, 600.0, 600.0, 0.3, 0.8, Low, 1.0, false);
            for k in 0..5 {
                x.noise(k as f32 * 0.03, 2000.0, 2000.0, 0.05, 0.4, Band, 3.0, false);
            }
        }
        "cannon" => {
            x.tone(0.0, Sine, 90.0, 30.0, 0.6, 0.9);
            x.noise(0.0, 500.0, 500.0, 0.7, 0.9, Low, 1.0, false);
        }
        "pistol" => {
            x.tone(0.0, Square, 200.0, 40.0, 0.15, 0.4);
            x.noise(0.0, 1500.0, 1500.0, 0.25, 0.8, Low, 1.0, false);
        }
        "tide" => {
            x.noise(0.0, 300.0, 2000.0, 0.6, 0.5, Band, 2.0, false);
            x.tone(0.0, Sine, 440.0, 880.0, 0.5, 0.15);
        }
        "crash" => {
            x.noise(0.0, 250.0, 250.0, 0.5, 0.9, Low, 1.0, false);
            x.tone(0.0, Saw, 80.0, 40.0, 0.4, 0.3);
        }
        "roar" => {
            x.tone(0.0, Saw, 70.0, 45.0, 1.2, 0.35);
            x.noise(0.0, 200.0, 200.0, 1.0, 0.5, Low, 1.0, false);
        }
        "death" => {
            x.tone(0.0, Sine, 110.0, 27.0, 3.0, 0.6);
            x.noise(0.0, 150.0, 150.0, 2.5, 0.6, Low, 1.0, false);
        }
        "pearls" => {
            for i in 0..5 {
                x.tone(i as f32 * 0.07, Tri, 880.0 + i as f32 * 220.0, 900.0 + i as f32 * 220.0, 0.3, 0.12);
            }
        }
        "kindle" => {
            x.noise(0.0, 800.0, 800.0, 1.2, 0.5, Band, 0.7, false);
            x.tone(0.0, Sine, 196.0, 196.0, 1.6, 0.2);
            x.tone(0.1, Sine, 293.7, 293.7, 1.6, 0.15);
        }
        "chest" => {
            x.tone(0.0, Square, 130.0, 90.0, 0.2, 0.2);
            for i in 0..3 {
                let f = 523.0 * (1.0 + i as f32 * 0.26);
                x.tone(0.2 + i as f32 * 0.1, Tri, f, f * 1.01, 0.35, 0.14);
            }
        }
        "level" => {
            for i in 0..3 {
                let f = 392.0 * 1.26f32.powi(i);
                x.tone(i as f32 * 0.1, Tri, f, f, 0.6, 0.15);
            }
        }
        "lock" => x.tone(0.0, Square, 1200.0, 1200.0, 0.04, 0.08),
        "rise" => {
            x.tone(0.0, Sine, 48.0, 70.0, 2.2, 0.7);
            x.noise(0.0, 180.0, 180.0, 2.4, 0.7, Low, 1.0, false);
            for k in 0..9 {
                let t = 0.4 + k as f32 * 0.17 + x.rng.u() * 0.1;
                let (a, b) = (300.0 + x.rng.u() * 500.0, 900.0 + x.rng.u() * 600.0);
                x.tone(t, Sine, a, b, 0.07, 0.08);
            }
            x.noise(1.4, 900.0, 900.0, 0.9, 0.5, Band, 0.8, false);
        }
        "zap" => {
            for k in 0..6 {
                let f = 3000.0 + x.rng.u() * 3000.0;
                x.noise(k as f32 * 0.035, f, f, 0.06, 0.45, Band, 2.0, false);
            }
            x.tone(0.0, Saw, 1400.0, 120.0, 0.35, 0.22);
            x.noise(0.05, 400.0, 400.0, 0.6, 0.6, Low, 1.0, false);
        }
        "charge" => {
            x.tone(0.0, Sine, 300.0, 900.0, 0.22, 0.12);
            x.noise(0.0, 2500.0, 2500.0, 0.2, 0.2, Band, 3.0, false);
        }
        "board" => {
            x.noise(0.0, 400.0, 400.0, 0.4, 0.4, Low, 1.0, false);
            x.tone(0.0, Tri, 180.0, 120.0, 0.3, 0.1);
        }
        "click" => x.tone(0.0, Square, 900.0, 900.0, 0.03, 0.06),
        "crackle" => {
            for k in 0..4 {
                let f = 1500.0 + x.rng.u() * 2500.0;
                let t = k as f32 * 0.09 + x.rng.u() * 0.05;
                x.noise(t, f, f, 0.03, 0.25, Band, 3.0, false);
            }
        }
        "fog" => {
            x.noise(0.0, 300.0, 1200.0, 0.9, 0.4, Band, 0.8, false);
            x.tone(0.0, Sine, 220.0, 110.0, 0.9, 0.12);
        }
        _ => {}
    }
    x.buf
}

/// Ambiente-Schleifen: Wellen (Rauschen mit langsamer Atmung), Wind, Grabesdröhnen
pub fn ambience(kind: &str) -> Vec<f32> {
    let mut rng = Rng::new(kind.len() as u32 * 31 + 5);
    match kind {
        "waves" => {
            let len = (9.090909 * SR) as usize; // eine LFO-Periode (0,11 Hz)
            let mut lp = Bq::new(Ft::Low, 520.0, 0.4);
            let mut out = vec![0.0; len];
            // Einschwingen
            for _ in 0..2000 {
                lp.run(rng.f());
            }
            for (i, o) in out.iter_mut().enumerate() {
                let t = i as f32 / SR;
                let m = 0.55 + 0.35 * (t * 0.11 * std::f32::consts::TAU).sin();
                *o = lp.run(rng.f()) * m * 1.4;
            }
            crossfade(&mut out, 2000);
            out
        }
        // Regen: dichtes, helles Rauschen mit einzelnen Tropfen
        "rain" => {
            let len = (5.0 * SR) as usize;
            let mut hp = Bq::new(Ft::Band, 2600.0, 0.5);
            let mut lp = Bq::new(Ft::Low, 900.0, 0.6);
            let mut out = vec![0.0; len];
            let mut drop = 0.0f32;
            let mut dph = 0.0f32;
            for o in out.iter_mut() {
                let n = rng.f();
                if rng.u() < 0.0009 {
                    drop = 0.5 + rng.u() * 0.5;
                    dph = 0.0;
                }
                dph += (1800.0 + drop * 1400.0) / SR;
                drop *= 0.9985;
                *o = hp.run(n) * 0.9 + lp.run(n) * 0.5 + (dph * std::f32::consts::TAU).sin() * drop * 0.12;
            }
            crossfade(&mut out, 3000);
            out
        }
        "wind" => {
            let len = (6.0 * SR) as usize;
            let mut bp = Bq::new(Ft::Band, 700.0, 0.8);
            let mut out = vec![0.0; len];
            for (i, o) in out.iter_mut().enumerate() {
                let t = i as f32 / SR;
                if i % 64 == 0 {
                    bp.set(Ft::Band, 600.0 + 250.0 * (t / 6.0 * std::f32::consts::TAU).sin(), 0.8);
                }
                *o = bp.run(rng.f()) * 1.5;
            }
            crossfade(&mut out, 3000);
            out
        }
        _ => {
            let len = (5.0 * SR) as usize;
            let mut lp = Bq::new(Ft::Low, 120.0, 0.7);
            let mut out = vec![0.0; len];
            let mut p = 0.0f32;
            for o in out.iter_mut() {
                p += 41.2 / SR;
                *o = lp.run(osc(Wave::Saw, p)) * 0.8;
            }
            out
        }
    }
}
fn crossfade(b: &mut Vec<f32>, n: usize) {
    let len = b.len();
    for i in 0..n {
        let k = i as f32 / n as f32;
        let tail = b[len - n + i];
        b[i] = b[i] * k + tail * (1.0 - k);
    }
    b.truncate(len - n);
}
