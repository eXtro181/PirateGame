# Projekt und Vorgaben (ausführlich)

## Stand und Pflege

Stand 14.11. (Token-Sparpaket; davor Runde 14 und der neue Endkampf vom 9.11.).
Das ausführliche Gesamtkonzept (Idee, Welt, Figuren, Systeme, Grafik, Klang, Technik) steht in `docs/SPIELKONZEPT.md`,
die komplette Grafik-Analyse mit allen Formeln, Werten und einer Nachbau-Anleitung in `docs/GRAFIK_ANLEITUNG.md`
– bei inhaltlichen bzw. grafischen Änderungen dort mitpflegen.
Bei Änderungen an Architektur, Systemen oder Entscheidungen den passenden Abschnitt in `docs/ai/` aktualisieren (nicht anhängen).
In die `CLAUDE.md` kommt nur, was in jeder Sitzung gebraucht wird; sie verweist auf diese Dateien.

## Projekt

**Throne of the Drowned**: Soulslike-Piratenspiel im Browser, HD-Pixelart im HD-2D-Look, **Rust/Bevy 0.16** mit eigenem wgpu-Renderer (WebGL2/wasm).
Paket heißt weiter `schwarzwasser` (alter Name, Wasm-Dateien `schwarzwasser*`).

## Keine Bild-/Audiodateien (Ausnahme PixelLab)

**Keine Bild-/Audiodateien**: alle Grafik, Musik und Klänge werden beim Start aus Code erzeugt (`crates/art`, `src/synth.rs`).
In `public/` liegen nur Icons und das Web-Manifest.
**Einzige Ausnahme** (Nutzerwunsch 4.11.): PixelLab-Grafik für die Insel Schädelhorn, die Prolog-Insel,
den natürlichen Bewuchs der übrigen Welt und (Nutzerwunsch 9.11.) den Fliegenden Holländer – PNGs in `crates/art/assets/pixellab/` (5 Bodensätze,
43 Requisiten plus 10 umgefärbte Abwandlungen, `dutchman.png` (128 px, Seitenansicht), Herkunft und Umfärbungen in der README dort;
das Probekontingent von 40 Generierungen ist aufgebraucht, täglich kommen bis zu 5 Bonus-Generierungen dazu), per `include_bytes!` eingebettet,
dekodiert mit der `png`-Crate.
**Nutzerwunsch 7.11.**: überall PixelLab-Stil – was nicht aus PixelLab stammt, wird nachgebildet (`pixstyle.rs`:
Sprites und Bodentexturen) –, und Flora/Deko natürlich setzen, wie sie auf der Erde wüchsen (`world/nature.rs`),
dabei düster und abwechslungsreich (Dark Fantasy).

## Repo, Live-Seite, Veröffentlichung

Repo `eXtro181/PirateGame`, live https://extro181.github.io/PirateGame/ (GitHub Pages;
`.github/workflows/deploy.yml` baut nur bei Push auf `main`, ca. 3–15 min – Pushes auf `umbau` lösen nichts aus;
die Pages-Umgebung erlaubt ohnehin nur `main`, ein Auslöser für `umbau` brach jedes Mal im Schritt „deploy“ ab).
`deploy-pages` wartet bis 30 min (`timeout`): bricht es ab, bricht es auch die Veröffentlichung ab,
und Wiederholungen **desselben Commits** hängen danach (Pages-Versionskennung = Commit-ID) – dann hilft nur ein neuer Commit.

## Verbindlich vom Nutzer

**Verbindlich vom Nutzer**: Antworten und Spieltexte auf **Deutsch**.
Dark Fantasy (gedämpftes Licht, Lichtinseln, Nebel, Zonenstimmung), 3D-Wirkung an Hängen, Küsten und Abgründen,
alle Gegner in derselben Grafikqualität und mit denselben Animationen wie Spieler/NPCs.
Grafik-Spezifikation: https://claude.ai/artifact/V4yVVaGGr1YaLyLo4UUA19.

## Arbeitsweise

**Arbeitsweise**: auf einem Zweig arbeiten, pushen, **erst nach Zustimmung nach `main` mergen**.
Nie `cargo fmt` (formatiert das ganze Repo um).
Keine PRs ohne Auftrag. Nie Passwörter auf der Live-Seite eingeben, Tests nur über `?debug&start=`.
Commits enden mit den Attributionszeilen der Sitzung.
