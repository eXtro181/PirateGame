# Fable52.md – Ideenvorrat und Übergabe (Stand 15.11.2026)

Telegrammstil, eine Idee je Zeile. Aufwand: S < 1 Std · M halber Tag · L mehrere Tage. Regeln und Werkzeuge: `CLAUDE.md`.
Prüfen je Paket: `tools/dev/check`, `wgsl`, `smoke`, `shot -q -s`, dann Kontaktblatt.

## 0 Stand
- **Erledigt (Opus 5.5, 15.11.)**: B1, B4 (Lichtschächte aus Szenenhelligkeit), B6, B8, B11 (Regen), C1 (Rest), C4, D2, D5
  (Wiedergänger: beim zweiten Tod steht der alte Leichnam auf), E1, E2, E3, E6 (Erzählung), E7, E8 (Wetter), E9 (Logbuch).
  Details: `docs/ai/SPIEL.md` (Abschnitt Versalzung …), `docs/ai/GRAFIK.md`, Grafik-Anleitung Kap. 19d.
- **Neuer Anfang (Opus 5.5, 15.11.)**: Startinsel ersetzt durch die Galgenfeste (Sarg-Überfahrt, Dungeon, Boss Kielholer) – siehe `docs/ai/INHALT.md`.
- 14./15.11.: neuer Bildlook (Grafik-Anleitung Kap. 19d): Nebel als Tiefe, Klarheit, farbtreue Tonkurve, Hauptlicht, Lichtinseln, Wind im Gras.
- Engpass für alles Weitere: Atlas 5,95 von 6 Seiten. Ohne A1 passt keine größere Figur und kein zusätzliches Animationsbild hinein.
- Ungeprüft: echte GPU, iPhone, Modus „Sparsam“ – alles nur in Swiftshader gesehen. Zuerst dort Helligkeit und Leistung messen (`&prof`).

## A Voraussetzungen (zuerst)
- A1 L **Atlas je Zone nachladen**: `DynSlot` (`src/art.rs`) verallgemeinern – Gegner, NPCs, Requisiten einer Zone als eigener Bereich,
  beim Zonenwechsel tauschen (wie der Boss-Streifen). Ziel: fester Atlas ≤ 3 Seiten, 2–3 Seiten frei. Maß: `tools/dev/art budget`.
- A2 M **Palettierte Farbebene**: 8-Bit-Index + Palette je Bogen (spart ¾ der Farbebene; Umfärbungen für Mob, Ertrunkene, NG+ gratis).
  Ein Nachschlag in `sample_sprite`; WebGL2: ein Sampler je Textur. Nur wenn A1 nicht reicht.
- A3 S **4096er Seiten** nur bei `MAX_TEXTURE_SIZE ≥ 4096` und genug Speicher (×4 je Seite), Rückfall 2048. Ebenenzahl nie durch 6 teilbar.
- A4 S Füllrate prüfen: Lichtradius ×1,18 und dichtere Boden-Deko (15.11.) kosten auf Handys Leistung – notfalls an `fx_high` binden.

## B Grafik: fünffaches Detail ohne fünffachen Speicher
Grundsatz: Detail im Shader erzeugen (auflösungsunabhängig), Atlasfläche nur für Formen und Bewegung ausgeben.
- B1 M **Gras als Halme** (`fs_gbuf`, Boden-Chunks, grüne Materialien): je 3-px-Zelle per Hash 1–2 Halme (Lage, Höhe, Neigung),
  Farbe aus Albedo ±15 %, Normale gekippt, Spitzen hell; Halme neigen sich mit `band` (Wind im Gras). Boden wirkt 4× feiner und lebt.
- B2 M **Materialdetail**: Glanzkanal (`nrm.a`) in 8 Stufen als Materialklasse lesen (Stein, Holz, Stoff, Metall, Haut, Laub, Knochen, Eis)
  → je Klasse prozedurale Mikronormale und Körnung in `fs_gbuf` (FullHD). `pixstyle::infer_spec` ordnet Materialien schon ein.
- B3 M **Kantenverfeinerung in FullHD** (rs = 2): Sprites nach Scale2x/xBR-Regel abtasten (4 Nachbarn in `sample_sprite`) –
  glatte Diagonalen bei doppelter Auflösung, die Kunst bleibt 1×. Als Option „Kanten: scharf/fein“; Normalen mitverfeinern.
- B4 M **Echte Lichtstrahlen**: `light_rays` entlang der Sonnenrichtung im Höhenpuffer verdecken (8 Schritte) –
  Strahlen fallen zwischen Bäumen und Dächern durch, Schattenbahnen im Dunst.
- B5 M **Nasser Ufersaum**: `shore_field` auch landseitig in die Chunk-Normale schreiben → dunkler, glänzender Sand, auflaufender Schaum.
- B6 S **Fließende Glut**: Leuchtfugen (`crack_glow`) im Shader pulsieren und wandern lassen (Rauschen × Zeit); Hitzeflimmern daran koppeln.
- B7 M **Spuren**: Ringpuffer mit 64 Abziehbildern (Blut, Brandflecken, Schneespuren, Nässe hinter Figuren aus dem Wasser), verblassen in 30–90 s.
- B8 S **Farbige Schatten**: Schlagschatten in der Komplementärfarbe des Hauptlichts tönen (`fs_light`, `sh`) – Farbtiefe ohne Helligkeit.
- B9 M **Texturen 2.0** (`terrain.rs`): je Material 3 Varianten, große Formen (Risse, Platten, Wurzeln) über 256 px statt 128,
  Übergänge nach Eckterrain wie bei PixelLab (`WANG` in `pixellab.rs`). Kostet keine Atlasfläche (Chunks).
- B10 L **PixelLab gezielt** (5 Generierungen/Tag, erst nach A1): Held (8 Richtungen, Laufen/Hieb/Rolle) → vier Könige → Händler →
  Leitrequisiten je Zone. README in `crates/art/assets/pixellab/` pflegen.
- B11 S **Wetter am Boden**: Regen-Platscher und Schneetupfer als PixelFX (`Px::Splash` gibt es), Blitz-Nachleuchten auf nassem Boden.
- B12 S **Lichtspiele je Zone**: Polarlicht-Bänder (Frost), Sporenleuchten (Mangroven), Geisterlichter (Galgenstrand) als wandernde
  farbige Lichtflecken in `fs_light` (wie Wolkenschatten, nur additiv).

## C Animation
- C1 ✓ Stauchen/Strecken: `pose_layer` (`draw.rs`) hatte Atmen, Hieb, Treffer schon; ergänzt Schritt, Rolle, Landung (`Player.land_t`).
- C2 M **Waffe und Umhang als eigene Lage**: Waffe getrennt zeichnen (1–2 Bilder nachziehend, Schweif aus PixelFX), Umhang/Haar verzögert –
  wirkt wie doppelte Bildzahl, braucht kaum Atlas.
- C3 M **Zwischenbilder nur für den Helden** (nach A1): Lauf 8 → 12, Hieb 5 → 8, Rolle 6 → 8 (`chars.rs`, `wants_anim`).
- C4 S **Tode nach Schadensart**: zu Asche verbrennen, gefrieren und zersplittern, im Boden versinken, Seele steigt auf (PixelFX, `cut_bottom`).
- C5 S NPCs wenden Kopf/Blick zum Spieler (Richtungsbild), Blinzeln als 2-px-Überlagerung.
- C6 M **Boss-Mimik**: Augen und Kiefer als kleine eigene Sprites über dem Rig (`beasts.rs`): Ruhe, Zorn, Schmerz, Tod; Wechsel bei 50 % und 10 %.
- C7 S Umwelt: Fahnen, Ketten, Wäsche, Schilder mit `skew` im Wind; Krähen fliegen auf, Ratten huschen weg.
- C8 S Oberfläche: Beute fliegt zur Anzeige, Lebensbalken bricht in Splittern weg, Banner laufen wie Tinte ein (`fx0.z`).

## D Spielmechanik (Soulslike)
- D1 M **Haltung für alle**: `Boss.posture` auf Elitegegner und Spieler (Blockleiste) ausweiten; volle Leiste = Hinrichtung
  (Nahkamera `zoom_in`, Trefferstopp).
- D2 S **Sturzangriff**: Herunterspringen (`jump_target`) + Angriff = schwerer Hieb mit Aufschlagwelle; Hinterhalt ×2 auf Ahnungslose.
- D3 L **Waffenklassen** (`Weapon` in `data.rs`, `player_logic`): Säbel, Enterbeil (langsam, bricht Haltung), Rapier (Stich, Konter),
  Harpune (Reichweite, zieht heran), Donnerbüchse (Kegel, Nachladen) – je Combo, Ladehieb, Roll- und Laufangriff.
- D4 M **Zustände mit Aufbauleiste**: Brand, Frost (Ausdauer), Gift, Blutung, **Ertrinken** (Maximalleben schrumpft bis zur Rast).
- D5 M **Versalzung**: jeder Tod −5 % Maximalleben (bis −25 %), NPCs reagieren; Heilung durch Boss-Sieg oder seltenes „Süßwasser“.
  Nach zwei Toden am selben Fleck steht dort der eigene **Wiedergänger** (Kampf um doppelte Perlen).
- D6 M **Geliehene Flut** (Kern zur Geschichte): „Mondscherbe“ lässt am Säbelfeuer für Minuten Ebbe werden (`World.tide` verallgemeinern) –
  Sandbänke, Wrackkammern, Höhlen nur bei Ebbe. Nach dem Sieg laufen die Gezeiten von selbst: neue Wege im Nachspiel und NG+.
- D7 M **Abkürzungen und Geheimnisse**: von innen entriegelte Tore, Winden, Leitern, Scheinwände (Hieb enthüllt), **Truhenkrabbe** (`chest_rig`).
- D8 S **Glutpfahl**: einmal setzbarer Wiedereinstieg vor Bossräumen.
- D9 M **Talismane** (2–4 Plätze) mit bauprägender Wirkung; Treibholz-Amulett als erster.
- D10 M **Gegnergruppen**: Schildträger, Harpunier, Gezeitenpriester (hebt Gefallene), Lauerer aus Pfützen, herabfallende Gehängte, Laternen-Patrouillen.
- D11 L **Entern**: Geisterschiff mit Enterhaken festmachen → Deckkampf als Arena, Beute im Laderaum; Harpunenleine gegen Seeungeheuer.
- D12 M **Bossräume mit Wasserstand**: Gezeiten-Shader im Kampf (Flut überspült Arena-Teile: langsamer, Blitz leitet) – zuerst Morgrim.
- D13 S **Flaschenpost**: feste Hinweise an Wegen, eigene Notizen lokal; „Echo“ zeigt den Tod eines Ertrunkenen als Geist vor Fallen.
- D14 M **Bünde** (schließen sich aus): Flutbrüder (Seelen heimführen), Zöllner (Mondsilber eintreiben), Laternenwacht (Feuer verteidigen).

## E Geschichte
- E1 M **Die vier Eide**: jeder König tauschte etwas gegen das Nicht-Ertrinken – Brandvogt seine Flotte, Ysra ihr Herz,
  Rabensturm seinen Namen, Morgrim seine Hand. Galionsfigur auf den Thron = stumme Erinnerung in drei Standbildern (`Ui.story`).
- E2 S **Gegenstandstexte** als Erzählweg: jede Waffe, jeder Fund zwei Zeilen Überlieferung (`Item` in `data.rs`).
- E3 M **Die Laternenmagd ist Malakors ertrunkene Tochter**: er stahl den Mond, damit sie nicht hinübermuss.
  Hinweise über Laternen und ein Schlaflied; Offenbarung vor dem Strudel.
- E4 M **Drei Enden**: Letzte Flut (jetzt) · **Neuer Zöllner** (Mond behalten, See bleibt still, dunkles NG+) ·
  **Hüter** (geheim: Spinne verschont + Magd eingeweiht → man bleibt am Feuer, sie zieht heim).
- E5 M **Stillwassers Wahrheit**: Spiegelscherben zeigen den Ertrunkenen ihr Gesicht – manche gehen heim, manche verzweifeln; das Dorf verändert sich.
- E6 S **Die Weberin**: die Meeresspinne wickelt Tote ein, um sie zur Flut zu tragen (Fährfrau, kein Ungeheuer).
  Verschont: Seidenbrücken als Abkürzungen, Seidenhandel. Erschlagen: ihre Brut legt Hinterhalte.
- E7 S **Stimme im Wasser**: das Spiegelbild flüstert an Feuern (`Ui::whisper`), reagiert auf Tode, Sünden (`hostile.rs`), Verschonen.
- E8 S **Welt nach jedem König**: Wetter der Zone klart auf, Zöllner rücken nach, NPCs ziehen um.
- E9 S **Logbuch des Geisterkapitäns**: Seiten auf wilden Inseln (gibt ihnen Sinn, siehe `docs/ai/OFFEN.md`); sein Flaschenschiff war Malakors erstes.

## F Reihenfolge
1. A1 → B1, C1, B6 (sofort sichtbar) → D2, D8, E2 (klein, großer Nutzen).
2. B2–B5, C2, C4 → D1, D4, D6 → E1, E3.
3. B9, B10, C3, C6 → D3, D11, D12 → E4, E5.

## G Befunde (Opus 5.5)
- B12 Lichtspiele (Polarlicht, Sporen, Geisterlichter) gebaut und wieder entfernt: über hellem Schnee und dunklem Wald kaum sichtbar.
- Ursache auch für andere Effekte: die Rauschtextur (`noise_texture` in `src/main.rs`) hat in den fbm-Kanälen r und b nur ±0,03 Streuung um 0,5.
  Effekte mit festen Schwellen (Wolkenschatten, Lichtstrahlen, Nebelbänke) wirken dadurch schwächer als gedacht.
  Vorschlag: Kanäle beim Erzeugen auf vollen Umfang normieren (Min/Max je Kanal), danach alle Zonen per Kontaktblatt neu abstimmen.
- B4 als Lichtschächte aus der Szenenhelligkeit (Endbild) gelöst; echte Verdeckung bräuchte den Höhenpuffer im Endbild-Durchgang.

