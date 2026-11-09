//! Titelbild (Runde 14): Die Sturmnacht mit Galeone und Riesenkraken entsteht ganz im
//! Endbild-Shader (`title_scene` in `gfx/shaders/post.wgsl`, eingeschaltet über `Globals.fx6.y`).
//! Hier bleibt nur der Weltpunkt, an den die Kamera im Titel gestellt wird (die Titelwelt wird
//! schon erzeugt, aber nicht gezeichnet – das Spiel beginnt damit ohne Wartezeit).

/// Weltpunkt der Titel-Kamera (offenes Meer nordöstlich der Glutwacht).
pub const TITLE_SEA: (f32, f32) = (crate::world::TITLE_T.0 as f32 * 32.0, crate::world::TITLE_T.1 as f32 * 32.0);
