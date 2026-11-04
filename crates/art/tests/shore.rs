//! Kodierung des Wassers im Boden-Chunk: durchsichtige Pixel tragen in der Normalen-Textur
//! b = Uferabstand (0..255 ≙ 0..64 px) und a = 255 (Wasser, 64 = Abgrund). Der Wasser-Shader
//! (`water_s` in scene.wgsl) liest genau diese Bytes – ändert sich die Kodierung, wird die Küste falsch.
use sw_art::terrain::*;

/// Gerade Küste: Land in den ersten vier Kachelspalten (0..128 px), rechts davon offenes Wasser
struct Half;
impl TerrainMap for Half {
    fn size(&self) -> (i32, i32) { (64, 64) }
    fn mat(&self, tx: i32, _ty: i32) -> Mat { if tx < 4 { Mat::Grass } else { Mat::Water } }
    fn level(&self, _tx: i32, _ty: i32) -> u8 { 0 }
    fn face(&self, _tx: i32, _ty: i32) -> Option<(i32, i32, u8, u8)> { None }
    fn stairs(&self, _tx: i32, _ty: i32) -> bool { false }
    fn biome(&self, _tx: i32, _ty: i32) -> Biome { Biome::Wind }
}

#[test]
fn shore_bytes() {
    let mut mats = Materials::new();
    let en = EdgeNoise::new();
    let c = bake_chunk(&Half, &mut mats, &en, 0, 0, 256);
    let (col, nrm) = sw_art::atlas::encode_canvas(&c, 0.55);
    let y = 128usize;
    // (Deckkraft, Uferabstand, Art) eines Pixels der Mittelzeile
    let px = |x: usize| {
        let o = (y * 256 + x) * 4;
        (col[o + 3], nrm[o + 2], nrm[o + 3])
    };
    assert!(px(64).0 > 0, "Land ist deckend");
    let mut last = 0u8;
    for x in 160..256 {
        let (a, shore, kind) = px(x);
        assert_eq!(a, 0, "Wasser ist durchsichtig (x={x})");
        assert_eq!(kind, 255, "Art = Wasser (x={x})");
        assert!(shore >= last, "Uferabstand wächst vom Ufer weg (x={x}: {shore} < {last})");
        last = shore;
    }
    assert!(px(160).1 < 255, "nahe am Ufer kein voller Abstand");
    assert_eq!(px(250).1, 255, "weit draußen voller Uferabstand");
}
