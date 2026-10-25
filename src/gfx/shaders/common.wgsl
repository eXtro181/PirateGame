// Gemeinsame Daten aller Durchgänge.
struct Light {
    pos: vec4<f32>,   // x, y (Bodenposition), z (Höhe), Radius
    col: vec4<f32>,   // rgb (HDR), Flackern
};
struct Globals {
    view: vec4<f32>,      // Ursprung xy (Welt-px oben links), Größe zw (Low-Res)
    screen: vec4<f32>,    // Bildschirm b, h, Zeit, Skalierung
    ambient: vec4<f32>,   // Umgebungslicht rgb, Himmelsanteil
    sun_dir: vec4<f32>,   // Richtung xyz (zum Licht), Schattenstärke
    sun_col: vec4<f32>,   // rgb, Glanzstärke
    fog: vec4<f32>,       // Nebelfarbe, Dichte
    lift: vec4<f32>,      // Farbkorrektur Schatten rgb, Kontrast
    gain: vec4<f32>,      // Lichter rgb, Sättigung
    misc: vec4<f32>,      // Vignette, Körnung, Bloom, Belichtung
    water_a: vec4<f32>,   // flach rgb, Wellenstärke
    water_b: vec4<f32>,   // tief rgb, Klarheit
    water_c: vec4<f32>,   // Schaum/Glanz rgb, Schaumbreite
    map: vec4<f32>,       // Kartengröße px, Maskentexelgröße px, Abgrund-Tiefe
    counts: vec4<f32>,    // Lichteranzahl, Blitz (Gewitter), Weiß-Blende, Rot-Blende
    lights: array<Light, 48>,
};
@group(0) @binding(0) var<uniform> G: Globals;
@group(0) @binding(1) var s_near: sampler;
@group(0) @binding(2) var s_lin: sampler;

fn hash21(p: vec2<f32>) -> f32 {
    let h = dot(p, vec2<f32>(127.1, 311.7));
    return fract(sin(h) * 43758.5453);
}
fn luma(c: vec3<f32>) -> f32 {
    return dot(c, vec3<f32>(0.299, 0.587, 0.114));
}
