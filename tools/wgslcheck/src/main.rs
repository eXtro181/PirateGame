//! Shader-Prüfung ohne Browser: jede Datei aus `src/gfx/shaders` (zusammen mit `common.wgsl`) wird
//! gelesen, validiert und für jeden Einstiegspunkt nach GLSL ES 3.00 (WebGL2) übersetzt.
//! Die Datei steht vor `common.wgsl` (in WGSL ist die Reihenfolge auf Modulebene frei), damit die
//! Zeilennummern der Meldungen zur Datei passen.
use std::fs;

fn main() {
    let dir = std::env::args().nth(1).unwrap_or_else(|| "src/gfx/shaders".into());
    let common = fs::read_to_string(format!("{dir}/common.wgsl")).expect("common.wgsl");
    let mut bad = 0;
    for f in ["scene.wgsl", "post.wgsl"] {
        let own = fs::read_to_string(format!("{dir}/{f}")).expect(f);
        let src = format!("{own}\n{common}");
        let lines = own.lines().count();
        let m = match naga::front::wgsl::parse_str(&src) {
            Ok(m) => m,
            Err(e) => {
                bad += 1;
                println!("{f}: FEHLER beim Lesen (ab Zeile {} beginnt common.wgsl)\n{}", lines + 2, e.emit_to_string_with_path(&src, f));
                continue;
            }
        };
        let mut v = naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::empty());
        let info = match v.validate(&m) {
            Ok(i) => i,
            Err(e) => {
                bad += 1;
                println!("{f}: FEHLER bei der Prüfung (ab Zeile {} beginnt common.wgsl)\n{}", lines + 2, e.emit_to_string_with_path(&src, f));
                continue;
            }
        };
        let mut ok = true;
        for ep in m.entry_points.iter() {
            let mut out = String::new();
            let opts = naga::back::glsl::Options { version: naga::back::glsl::Version::Embedded { version: 300, is_webgl: true }, ..Default::default() };
            let pl = naga::back::glsl::PipelineOptions { shader_stage: ep.stage, entry_point: ep.name.clone(), multiview: None };
            let res = match naga::back::glsl::Writer::new(&mut out, &m, &info, &opts, &pl, naga::proc::BoundsCheckPolicies::default()) {
                Ok(mut w) => w.write().map(|_| ()),
                Err(e) => Err(e),
            };
            if let Err(e) = res {
                ok = false;
                println!("{f}: {} lässt sich nicht nach GLSL ES 3.00 übersetzen: {e:?}", ep.name);
            }
        }
        if ok {
            println!("{f}: ok ({} Einstiegspunkte)", m.entry_points.len());
        } else {
            bad += 1;
        }
    }
    std::process::exit(if bad > 0 { 1 } else { 0 });
}
