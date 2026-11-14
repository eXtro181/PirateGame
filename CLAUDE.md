# CLAUDE.md – Throne of the Drowned

Kurzfassung für jede Sitzung (Stand 14.11.2026). Alles Weitere steht in `docs/ai/` und wird **gezielt** nachgeschlagen –
nie auf Verdacht ganze Dateien lesen. Diese Datei wird in jeder Sitzung ganz geladen: kurz halten (unter 12 000 Zeichen).

## Projekt
- Soulslike-Piratenspiel im Browser, HD-Pixelart im HD-2D-Look. **Rust/Bevy 0.16**, eigener wgpu-Renderer (WebGL2/wasm).
- Paket heißt weiter `schwarzwasser` (alter Name, Wasm-Dateien `schwarzwasser*`).
- Alle Grafik, Musik und Klänge entstehen beim Start aus Code (`crates/art`, `src/synth.rs`). Einzige Ausnahme: PixelLab-PNGs in `crates/art/assets/pixellab/`.
- Repo `eXtro181/PirateGame`, live https://extro181.github.io/PirateGame/ (GitHub Pages baut nur bei Push auf `main`, 3–15 min).
- Rund 53 000 Zeilen Rust/WGSL in 74 Dateien; die größten: `src/world/mod.rs` 4100, `src/game/mod.rs` 2900, `crates/art/src/terrain.rs` 2100.

## Verbindlich (Nutzer)
- Antworten und Spieltexte auf **Deutsch**.
- Stil: Dark Fantasy (gedämpftes Licht, Lichtinseln, Nebel, Zonenstimmung), 3D-Wirkung an Hängen, Küsten und Abgründen.
- Alle Gegner in derselben Grafikqualität und mit denselben Animationen wie Spieler/NPCs; überall PixelLab-Stil; Bewuchs natürlich gesetzt.
- Auf Zweig `umbau` arbeiten und pushen; **nach `main` erst nach Zustimmung** (`tools/dev/merge-main`). Keine PRs ohne Auftrag.
- **Nie `cargo fmt`** (formatiert das ganze Repo um).
- Nie Passwörter auf der Live-Seite eingeben; Tests nur über `?debug&start=…`.
- Keine Bild-/Audiodateien ins Repo außer PixelLab-PNGs. Der PixelLab-Schlüssel kommt nie ins Repo und wird nie im Chat erfragt.
- Commits enden mit den Attributionszeilen der Sitzung; sonst keine Modellkennung in Repo-Dateien.
- Warnungsfrei bauen (eine bekannte Warnung: `AudioBufferSourceNode::stop` veraltet).
- Doku mitpflegen: Inhalt → `docs/SPIELKONZEPT.md`, Grafik → `docs/GRAFIK_ANLEITUNG.md`, Technik → passender Abschnitt in `docs/ai/` (ersetzen, nicht anhängen).

## Sparsam arbeiten (Tokens)
1. Nachschlagen statt lesen: `tools/dev/doc <stichwort>` (Doku), `tools/dev/code sym|ol|fn|refs` (Quelltext), `tools/dev/g <muster>` (Suche).
2. Keine Datei über ~300 Zeilen ganz lesen. Read nur mit `offset`/`limit` auf den Bereich, den `code ol` nennt.
   Ein Wächter lehnt Read ohne `limit` bei Dateien über 600 Zeilen bzw. 40 kB ab (`tools/dev/hooks/read_guard.py`).
3. Vor dem Ändern genau ein Element holen (`code fn <name>`, bei langen mit `-g <regex>`), dann gezielt per Edit ändern – große Dateien nie neu schreiben.
   Verlangt Edit ein vorheriges Read, genügt ein Read mit `limit` auf den Bereich.
4. Prüfen billig zuerst: `check` → `wgsl` → `build` + `smoke` → `shot -q -s` (Zustandszeile statt Bild) → erst dann Bilder, und dann das Kontaktblatt.
5. Ausgaben klein halten: keine rohen `cargo`-, `git diff`- oder `ls -R`-Ausgaben; `git diff --stat`, `| head`, `| cut -c1-160`.
6. Doku-Pflege: eine Tatsache je Zeile (≤ 160 Zeichen), jede Tatsache nur an einer Stelle; vor dem Commit `tools/dev/doc --check`.
7. Aufgabenliste klein halten (Erledigtes löschen); Berichte kurz; lange Läufe (Build, Bilder) im Hintergrund, derweil weiterarbeiten.
8. Nach Shader-Änderungen `tools/dev/docsync` (erzeugt `docs/GRAFIK_SHADER.md` neu).

## Werkzeuge (`tools/dev/…`, alle mit `-h`)
| Befehl | Zweck |
|---|---|
| `code map [pfad]` | Dateien mit Zeilenzahl und Kopfzeile |
| `code ol <datei> [muster]` | Gliederung einer Datei mit Zeilenbereichen |
| `code fn <name> [pfad]` | genau ein Element, auch `Typ::name` (`-s` Signatur, `-g <regex>` nur Trefferzeilen, `--all`) |
| `code sym <regex>` · `code refs <name>` | wo definiert · wer benutzt es (mit umgebender Funktion) |
| `g <muster> [pfad]` | Suche mit kurzen Treffern (`-d` Doku, `-a` überall) |
| `doc` · `doc <stichwort>` · `doc -s <überschrift>` | Doku-Übersicht · Volltextsuche · einen Abschnitt ausgeben |
| `check` · `check art` | Compiler-Prüfung (wasm · Kunst-Crate nativ), eine Zeile je Meldung |
| `wgsl` | Shader prüfen ohne Browser (naga, GLSL ES 3.00) |
| `build` | bauen → `dist/`, Testserver auf Port 8790 |
| `smoke` | Startprobe ohne Bild (Spiel + Titel, Atlasseiten, Fehler) |
| `shot [-q -s] "<parameter>"…` | Bilder/Zustand über Debug-Parameter; mehrere Bilder → Kontaktblatt `.shots/blatt.png` |
| `sheet` · `art <befehl>` | Lupe/Kontaktblatt aus Bildern · Kunst-Vorschau ohne Spiel |
| `live` · `merge-main --freigabe "…"` | Live-Stand prüfen · nach `main` zusammenführen (nur mit Zustimmung) |

## Wegweiser (wo steht was)
| Thema | Doku in `docs/ai/` | Quelltext |
|---|---|---|
| Bauen, Prüfen, Leistung, Atlas-Budget | `BAUEN.md` | `build.sh`, `tools/dev/` |
| Debug-Parameter (`?debug&start=…`) | `DEBUG.md` | `src/main.rs` |
| Prozedurale Kunst, PixelLab, Figuren, Rigs, Schiffe | `KUNST.md` | `crates/art/src/` |
| Start, Atlanten, Renderer, PixelFX, UI, Klang, Speichern | `KERN.md` | `src/main.rs`, `src/art.rs`, `src/gfx/`, `src/pfx.rs`, `src/ui/` |
| Welt, Inseln, Ebenen, Kollision, Streaming, Bewuchs | `WELT.md` | `src/world/` |
| Spielablauf, Kampf, Bosse, Schiff, Zwischensequenzen | `SPIEL.md` | `src/game/` |
| Orte, Geschichte, Bosse, Endkampf, Läden, NG+ | `INHALT.md` (lang: `docs/SPIELKONZEPT.md`) | `src/game/data.rs`, `src/game/dialog.rs` |
| Grafik kurz (lang: `docs/GRAFIK_ANLEITUNG.md`) | `GRAFIK.md` | `src/gfx/shaders/` |
| Vorgaben ausführlich, Veröffentlichung | `PROJEKT.md` | `.github/workflows/deploy.yml` |
| Offene Punkte · Token-Sparregeln und Einstellungen | `OFFEN.md` · `SPAREN.md` | `.claude/settings.json` |

## Harte Fallstricke
- **Atlas**: 5,95 von 6 Seiten belegt – neue große Teile vorher messen (`tools/dev/art budget`, danach `smoke`).
- **Texturfelder nie mit einer durch 6 teilbaren Ebenenzahl** (WebGL hält sie für Würfel-Felder: Sprites fehlen, Boden schwarz).
- **Boss-Bereich** fasst nur eine Atlasseite: Holländer (`build_malakor`) und Meeresspinne (`build_seabed`) nie zusammen laden
  („Bereich zu klein“ = Sprites unsichtbar). Die Streifen `PLAYER_STRIP` + `SHIP_STRIP` + `BOSS_STRIP` müssen zusammen genau eine Seite ergeben.
- **WebGL2**: nur ein Sampler je Textur; Pipelines mit mehreren Zielen brauchen für alle Ziele dieselbe Mischung und Schreibmaske.
- Neue `Globals`-Felder vor `lights`, in Rust und WGSL in derselben Reihenfolge; `env::apply` muss sie auf Grundwerte setzen.
- **Spielstand**: neue Felder immer `#[serde(default)]`, umbenannte per `#[serde(rename = "alt")]` – alte Profile müssen laden.
- **Veröffentlichung**: bricht `deploy-pages` ab, hängen Wiederholungen desselben Commits – dann hilft nur ein neuer Commit.
- Requisiten, deren Name mit `gibbet`/`gallows` beginnt, behandelt `view.rs` als Schwingkäfig.
- Swiftshader zeichnet ~1 Bild/s: lange Sequenzen mit `&tscale=<k>` prüfen, Wasser-Vergleiche auf See erst nach ~60 s.

## Konventionen
- Weltkoordinaten in Pixeln, 32 px je Kachel; Figuren stehen auf ihrem Fuß. UI in UI-Pixeln (kürzere Bildschirmseite ≈ 360).
- Schiffskurs `h`: Bogenmaß im Uhrzeigersinn von Nord, vorwärts = (sin h, −cos h).
- Spielstand (`Save` v7): `migrate` v5 verschiebt Perlenfleck und aufgehobene Gegenstände („id@x,y“) um `CORE` bzw. `END_OFF`;
  Liegeplätze über `Dock.key` („kueste“, „w12“); umbenanntes Feld z. B. `end_won` ↔ „megalodon“.
- Schriften: Fließtext in der Pixelschrift (`text::draw/t/tc/tr`), Titel/Banner/Überschriften/Knöpfe in der Zierschrift
  (`text::display`, Versalien, Größe = Versalhöhe in UI-Pixeln).
- Neue Requisiten: Kunst in `crates/art`, in `src/art.rs` registrieren, im Layout per `prop_t`/`prop_x` setzen.
  Neue Figuren: `chars::preset` + `PRESET_NAMES`. Danach Atlasseiten prüfen.
- Jede Quelldatei beginnt mit einem `//!`-Kopf (erste Zeile = Zweck) – `code map` zeigt ihn.
