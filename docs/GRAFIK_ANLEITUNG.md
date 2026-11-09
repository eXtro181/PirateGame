# Grafik von Throne of the Drowned – Analyse und 1:1-Anleitung für ein neues Projekt

Stand: 1. November 2026. Dieses Dokument zerlegt die komplette Grafik des Spiels bis auf Formeln, Zahlen und Farbwerte. Es ist so geschrieben, dass man den Look in einem **neuen Projekt** Schritt für Schritt exakt nachbauen kann. Die drei Shader stehen am Ende vollständig im Anhang.

Bezugsdateien im Repo: `crates/art/src/*` (Kunst aus Code), `src/gfx/mod.rs` + `src/gfx/render.rs` (Renderer), `src/gfx/shaders/*.wgsl` (Shader), `src/game/env.rs` (Stimmungen), `src/world/view.rs` (Welt zeichnen), `src/game/draw.rs` (Figuren, Nebel, Licht).

---

## Inhalt
1. Das Prinzip in einem Satz
2. Koordinaten, Kamera, Auflösung
3. Die Leinwand: vier Kanäle pro Pixel
4. Farbe: Rampen, Paletten, Umriss, Raster
5. Mal-Werkzeuge (Kuppel, Röhre, Umriss, Lichtkante)
6. Von der Leinwand zur GPU: Kodierung und Atlas
7. Figuren-Maler (Menschen und Humanoide)
8. Kreaturen-Rig (3D-Wesen als Pixelkunst)
9. Pflanzen und Requisiten
10. Gelände: Materialien, Paletten, Chunk-Backen
11. Der Renderer: Zeichenlisten und Vertexformat
12. Render-Durchgänge, Formate, Mischmodi
13. G-Puffer-Shader (Sprites, Boden, Wasser, Abgrund)
14. Licht-Shader (Umgebung, Sonne, Punktlichter, Schatten)
15. Bloom und Tiefenschärfe
16. Endbild: Diorama-Kippung, Farbkorrektur, Effekte
17. Stimmungen je Zone (alle Werte)
18. Nebel, Partikel, Spiegelungen, Schatten am Boden
19. Pixel-Modus gegen FullHD-Modus
19a. Ausbau 2026 (Runde 13): Licht, Wetter, Tageszeit, Bildwirkungen, Stilrichtlinie und Farbskript
19b. Runde 14: Wellen, Brandung, plastische Dächer, Titelbild „Sturmnacht“
20. Schritt-für-Schritt-Plan für das neue Projekt
21. Fallstricke
22. Anhang: Shader im Wortlaut

---

## 1. Das Prinzip in einem Satz

Alles wird als **Pixelkunst mit Zusatzdaten** erzeugt (Farbe + Relief → Normale + Höhe über Grund + Leuchten/Glanz). Der Renderer schreibt diese Daten in einen **G-Puffer** und beleuchtet jedes Pixel danach wie eine 3D-Szene: mit Punktlichtern, Sonne, Schatten aus dem Höhenfeld, Umgebungsverdeckung und Randlicht. Zum Schluss folgen Bloom, Tilt-Shift-Unschärfe, Diorama-Kippung und Farbkorrektur. So entsteht **HD-2D**: flache Pixelkunst, die wie ein beleuchtetes Miniaturmodell wirkt.

Drei Grundregeln ziehen sich durch alles:
- **Licht kommt von oben links** (`LIGHT = [-0.55, -0.62, 0.56]`, x rechts, y unten/Süden, z zur Kamera). Jede Schattierung in der Kunst folgt dieser Richtung.
- **Schatten werden kühl, Lichter warm** (Farbton-Verschiebung in allen Rampen).
- **Dark Fantasy**: dunkles Umgebungslicht, wenige helle Lichtinseln (Feuer, Laternen, Leuchten), Nebel.

---

## 2. Koordinaten, Kamera, Auflösung

**Welt**
- Weltpixel; eine Kachel = 32 px. y wächst nach Süden. Figuren und Requisiten stehen auf ihrem **Fußpunkt** (x, y).
- Höhe über Grund wird im Bild nach oben gezeichnet. Ein Punkt mit Bodenposition (x, y) und Höhe z erscheint bei Bildschirm-y = y − z (Schrägprojektion 1:1). Der Licht-Shader rechnet zurück mit `p3 = (x, y_bild + elev, elev)`.

**Auflösung**
- Die **kürzere Bildschirmseite zeigt 540 Weltpixel**. Bei einem Bildschirm `pw × ph` gilt `k = max(min(pw,ph) / 540, 0.5)` und `low = (pw/k, ph/k)` gerundet (bei 16:9 also 960 × 540).
- **UI-Skalierung**: `ui_scale = max(round(min(pw,ph) / 360), 1)`. UI-Koordinaten laufen in UI-Pixeln (kürzere Seite ≈ 360).
- **FullHD-Modus**: Ab kürzerer Seite ≥ 900 px sind alle Renderziele `low × rs` mit `rs = 2` groß. Die Kunst bleibt pixelgenau (Nearest-Sampling), aber Licht, Normalen, Nebel und Wasser werden doppelt so fein berechnet.
- **Kamera**: Sicht = Rechteck `view = (round(cx − w/2), round(cy − h/2), w, h)` in Weltpixeln. Der Ursprung wird auf ganze Pixel gerundet, damit nichts flimmert.

**Diorama-Kippung (TILT = 0,32)**: Das Endbild ist gekippt: oben ferner (gestaucht), unten näher (vergrößert). Bildschirm-uv → Quellbild-uv:
```
d   = 1 + TILT * v
src = ( 0.5 + (u - 0.5) / d ,  v * (1 + TILT) / d )
```
Die Umkehrung (Quelle → Bildschirm), z. B. für Maus und UI über Weltobjekten:
```
v = sy / (1 + TILT - TILT * sy)
u = 0.5 + (sx - 0.5) * (1 + TILT * v)
```
Örtliche Vergrößerung am Bildschirmpunkt: `(1 + TILT * v) / (1 + TILT * 0.5)`. Alle Umrechnungen Welt ↔ Bildschirm müssen über diese Formeln laufen.

---

## 3. Die Leinwand: vier Kanäle pro Pixel

Jedes Bild entsteht auf einer `Canvas` (`crates/art/src/canvas.rs`):

| Kanal | Typ | Bedeutung |
|---|---|---|
| `col` | RGBA8 | Farbe (Alpha 0 = leer) |
| `rel` | f32 | **Relief** in Pixeln (lokale Wölbung). Daraus entstehen die Normalen |
| `emi` | RGBA8 | rgb = **Leuchten**, a = **Glanz** (Spekular 0–255) |
| `up` | u8 | **Aufrecht-Anteil**: 0 = liegt flach (Boden), 255 = steht senkrecht zur Kamera (Wand, Figur) |
| `ev` | f32 | **Höhe über Grund** in px (−1 = automatisch aus dem Ankerpunkt) |

Zusätzlich: `cur_up` und `cur_spec` (Werte für neu gesetzte Pixel) sowie `standing` (aufrechte Figur oder flach liegendes Bild).

**Wichtige Operationen**
- `put(x, y, farbe, relief)` setzt Farbe und Relief und übernimmt `cur_up` und `cur_spec`.
- `set(x, y, farbe)` setzt nur die Farbe; halbdurchsichtige Farbe wird gemischt.
- `glow(x, y, farbe, k)` setzt das Leuchten (Farbe × k).
- `flat(w, h)` legt eine Bodenfläche an (`up = 0`, `standing = false`).

**Normale aus dem Relief** (`normal_at`, Stärke `k = 0.55`):
```
gx = (rel(x+1,y) - rel(x-1,y)) * 0.5      // am Bildrand einseitig ableiten
gy = (rel(x,y+1) - rel(x,y-1)) * 0.5      // leere Nachbarn zählen als rel - 1.5
gx = clamp(gx, -2.5, 2.5) * k ;  gy = clamp(gy, -2.5, 2.5) * k
u  = up / 255
Grundnormale = (0, 0.72*u, 1 - 0.31*u)    // flach: (0,0,1), aufrecht: kippt Richtung Betrachter
n  = normalize( -gx , 0.72*u - gy*(1 - 0.5*u) , 1 - 0.31*u )
```
Aufrechte Pixel bekommen also eine Normale, die zur Kamera (Süden) zeigt. Dadurch werden Wände und Figuren von vorn beleuchtet, Böden von oben.

---

## 4. Farbe: Rampen, Paletten, Umriss, Raster

**Rampe mit Farbton-Verschiebung** (`ramp_n(base, n, spread)`, meist 6 Stufen): Aus einer Grundfarbe entstehen n Töne von dunkel bis hell.
```
k  = (i - mid) / mid * 2          // -2 … +2
dL = k * 0.09 * spread            // Helligkeit
Farbton: Schatten (k<0) wandern bis zu -k*13° Richtung 245° (Blauviolett),
         Lichter  (k>0) wandern bis zu  k*11° Richtung 52° (Goldgelb)
Sättigung: Schatten × (1 - k*0.06) (steigt), Lichter × (1 - k*0.07) (sinkt)
L = clamp(L + dL, 0.02, 0.97)
```
**Umrissfarbe** (`outline_of`): Farbton 28° Richtung 255°, Sättigung × 1,1, Helligkeit × 0,5 (mindestens 0,03). Umrisse sind also nie schwarz, sondern dunkle, kühle Materialfarbe (selektiver Umriss).

**Palette** = Liste aus Hexwerten, dunkel → hell. **`pick(p, v)`** wählt mit v ∈ 0..1 und mischt weich zwischen benachbarten Stufen (Fotolook). Die alte Stufenvariante ist entfernt.

**Bayer-Raster 4×4** für Übergänge (`bayer(x,y) ∈ 0..1`):
```
 0  8  2 10
12  4 14  6
 3 11  1  9
15  7 13  5      (Wert / 16)
```
Typische Nutzung: `v += (bayer(x,y) - 0.5) * 0.15 … 0.6`, dann auf eine Palettenstufe runden.

**Zufall und Rauschen**: `h01(x, y, seed)` (Ganzzahl-Hash → 0..1), Perlin-Rauschen `Noise::n2`, fBm (`fbm`, periodisch `fbmp` für kachelbare Texturen), Worley-Zellen (`worley` → d1, d2, Zell-ID), xorshift-`Rng`. Alles ist deterministisch über Seeds, damit jedes Bild bei jedem Start gleich aussieht.

**Gamma**: Kunstfarben sind sRGB. Vertexfarben werden im Shader mit `pow(rgb, 2.2)` linearisiert. Die Albedo-Ziele sind `Rgba8UnormSrgb`. Farben für Uniforms gehen durch `lin()` (exakte sRGB-Kurve).

---

## 5. Mal-Werkzeuge

**Kuppel `dome(cx, cy, rx, ry, palette, höhe, sockel, dither, bias)`**: Grundform für Köpfe, Hände, Steine, Büschel.
```
nz = sqrt(1 - d²)                          // d = normierter Abstand zur Mitte
l  = dx*LIGHT.x + dy*LIGHT.y + nz*LIGHT.z
v  = (l*0.75 + 0.35 + bias) * (n-1) + (bayer-0.5)*dither
Farbe = palette[round(v)],  Relief = sockel + höhe*nz
```
**Röhre `tube(pfad, w0, w1, palette, höhe, sockel)`**: Äste, Arme, Seile, Tentakel. Die Breite läuft von w0 nach w1. Quer zur Laufrichtung: `side ∈ −1..1`, `nzz = sqrt(1 − side²)`, Licht nach der Normalen der Seite,
`v = ((lit*0.8 + nzz*0.55)*0.75 + 0.3)*(n-1) + (bayer-0.5)*0.6`, Relief = `sockel + höhe*nzz*r`.

**Umriss `outline(oben, stärke)`**: Leere Pixel neben gefüllten bekommen `mix(nachbar, outline_of(nachbar), stärke)` und das halbe Relief des Nachbarn. Figuren und Wesen: `outline(true, 0.95)`, Bäume `0.85`.

**Weitere**: `inner_shade(f)` dunkelt die untere und rechte Innenkante ab, `rim_light(t)` hellt die obere und linke Kante auf (+0,12 Helligkeit), `noise_fill`, `poly` (gefülltes Vieleck), `ellipse`/`disc`, `line`, `blit`/`crop`, `fill_ev` (Pixel ohne Höhe übernehmen sie von den Nachbarn).

---

## 6. Von der Leinwand zur GPU: Kodierung und Atlas

**Pixel kodieren** (`atlas::encode_px`). Pro Pixel entstehen zwei RGBA8-Texel:

| Textur | R | G | B | A |
|---|---|---|---|---|
| **Farbe** | r | g | b | 255 = deckend · 128–254 = deckend **und leuchtend** · 0 = leer |
| **Normale** | n.x·0,5+0,5 | n.y·0,5+0,5 | Höhe über Grund / 2 (bis 510 px) | Glanz |

- Leuchten: Ist der stärkste Leuchtkanal > 8, übernimmt die Farbe die Leuchtfarbe (`max(emi, col/2)`). Alpha wird `128 + min(k·0,5, 0,49)·254`, höchstens 254. Der Shader rechnet zurück: `glow = rgb * (a − 0.5) * 4` (bis ≈ 2).
- Höhe über Grund: aus `ev`, sonst bei aufrechten Bildern `Anker_y − y` (Abstand zum Fuß), bei flachen 0.

**Atlas** (`atlas.rs`): Seiten zu 2048 × 2048, Regalpackung (Start bei 4,4; Abstand 3 px; neue Zeile bei Überlauf; neue Seite bei vollem Blatt). Oben links liegt ein 2×2-weißes Texel für einfarbige Flächen. `add_trim` schneidet leere Ränder ab und verschiebt den Anker mit. Sprites tragen Seite, x, y, w, h und Anker (ax, ay = Fußpunkt).

**Dynamische Bereiche**: Feste Streifen (Spieler 208 px, eigenes Schiff 300 px, aktiver Boss 1520 px, zusammen eine Seite) werden zur Laufzeit neu gezeichnet und als Teilbereich hochgeladen. So lässt sich Kunst austauschen, ohne den Atlas neu zu bauen.

**Pixelstil wie PixelLab** (`pixstyle::pixel_style`, Nutzerwunsch 7.11.: „überall den PixelLab-Stil, sonst nachmachen“): Steht `Atlas.style`, wird jedes Bild beim Packen umgesetzt (es wächst um 1 px je Seite, der Anker um (1, 1)):
1. **Stufen**: deckende Pixel (Alpha ≥ 200) in HSL rastern – Farbton in 12°-Schritten, Sättigung in Fünfteln, Helligkeit `round(l^0,8·8)/8` und zurück `^1,25` (unten feiner, die meisten Dark-Fantasy-Farben liegen dort).
2. **Ausreißer**: teilt kein 4er-Nachbar die Stufe eines Pixels, übernimmt es die unter den 8 Nachbarn häufigste (mindestens dreimal) – saubere Pixelinseln statt Rauschen.
3. **Umriss in einer Farbe**: Farbe = mittlerer Farbton des Bildes, Sättigung `min(0,7·s, 0,45)`, Helligkeit 0,075. Sind ≥ 60 % der Randpixel schon dunkel (Leuchtdichte < 0,3 und < 0,8 × innerer Nachbar, der weiche `Canvas::outline`), werden nur diese umgefärbt; sonst bekommt jedes leere Pixel neben einem deckenden die Umrissfarbe (Relief/Höhe vom Nachbarn, kein Leuchten).

Ausgenommen (`style` aus): nahtlos gekachelte Mauern, Zäune und Brücken (`TILED` in `src/art.rs`), Flammenbilder, die PixelLab-Requisiten (sind es schon), Effekte, Tentakelbänder, Strudel und Seeungeheuer-Teile, Malakors Segmente, das Titelbild. An: Bäume, Requisiten, Bauten, Boden-Deko, alle Figuren und Gegner, Spieler-, Schiffs- und Bossbereich.

---

## 7. Figuren-Maler (Menschen und Humanoide)

`chars.rs`: Jede Figur ist ein **64 × 72-Bild** (× Maßstab s, Könige 1,5, Malakor größer). Die Figur ist ≈ 48 px hoch mit großem Kopf wie im Referenzstil. Der Fußpunkt liegt bei `h − 4·s`.

**Richtungen**: vorn (`down`), hinten (`up`), Seite (`side`, links = gespiegelt).
**Animationen** (Bilder je Richtung): idle 4, walk 6, atk 5, roll 5, hurt 1, dead 1 (nur vorn), cast 3, drink 3, sit 1, shoot 2. NPCs brauchen nur idle/walk/sit/dead, Gegner kein roll/drink/sit.

**Aussehen (`Look`)**: Geschlecht (`female`, seit 8.11.: Schultern −1·s, Taille bis 2,2·s schmaler um t ≈ 0,72, Hüfte −0,8·s unter dem Gürtel, Mieder mit Schnürung im Hemdausschnitt, Brustschatten, Mantelschöße 10 statt 6 lang und 3,4 statt 2 ausgestellt, Wimpern 140C10 und Lippen 8A3A3E, kurzes Haar als Bob, langes Haar bis 2,2 × Kopfradius, kein Bart), Statur, Haut (4 Töne: E0B896, C49674, 9A6A4C, 6A4430), Haar (Glatze/kurz/lang/Zopf/wild, 6 Farben), Bart, Kopfbedeckung (Dreispitz, Kopftuch, Kapuze, Kapitänshut), Mantel (6 Farben: 8A2A30, 2E4466, 4A5A36, 6A4A30, 2E2E36, 6A5A7E), Hose, Hemd, Augenklappe, Waffe. Dazu Sonderteile: Skelett, Lumpen, ertrunken, Robe, Schürze, leuchtende Kapuze, Laterne, Krone, Aura, Hörner, Schärpe, Fischkopf, Hauer, Totenkopf-Bemalung, Flügel (Federn oder Uhrwerk), Vogelbeine, Schweben, Schlangenleib, Schlangenhaar, Uhrwerk-Körper, Glockenrumpf, kreisende Schädel, vier Arme, Tentakelbart.

**Malreihenfolge** (`paint_human`):
1. Waffe hinter dem Körper (bei Blick nach hinten oder Ausholen von der Seite), Flügel.
2. **Beine**: 5 px breit, Hosenrampe quer schattiert (links Stufe 4, rechts Stufe 1, Mitte 3 bzw. 2 für das hintere Bein). Untere 40 % = Stiefel (eigene Rampe 0E0A0C … 705E62) mit Glanzlinie. Relief 3.
3. **Rumpf**: Mantel mit Schulterrundung, Schößen unterhalb der Hüfte (aufgeweitet), offener Hemdausschnitt vorn (38–62 % der Breite). Quer schattiert (links 4, Mitte 3, rechts 1, Schöße eine Stufe dunkler). Relief `3 + (1 − |rel−0,5|·2)·2` (Mitte gewölbt).
4. Revers, Knöpfe, Gürtel, Schärpe, Sonderteile. **Kleidungsdetails für alle Menschen** (nicht bei Robe, Skelett, Sitzen): Paspeln an der Mantelöffnung bei 37 % und 63 % der Breite (Stufe 5 links, 2 rechts), Taschenklappen mit Messingknopf (C9A04A) unter dem Gürtel, Saum mit dunkler Kante (Stufe 1) und hellem Umschlag (Stufe 4), Bandelier von der Schulter zur Hüfte (Lederrampe 1A100A … 9A6A44, alle 3 px ein Glanzpunkt, Schnalle B88A3A/F0D080 bei 45 %), Gürteltasche; an den Händen helle Manschetten aus der Hemdrampe; Stiefel mit umgeschlagener Stulpe bei 62 % und Schnalle bei 80 %; ein Glanzpunkt (E8E4F0, 0,35) im Auge.
5. **Arme** als Röhren, **Hände** als Kuppeln, **Kopf** als Kuppel mit Gesicht, Haar, Bart und Hut. Die Reihenfolge hängt von der Richtung ab: von hinten zuerst die Arme, dann der Kopf.
6. Waffe vor dem Körper, Zusatzarme, Schädel, Aura (Leuchtpunkte auf der Figur).
7. **Feinschliff** (`finish`, für jede Figur gleich): Formlicht – Pixel mit leerem Nachbarn oben/links ×1,1, unten/rechts ×0,87; Haare mit schrägen Strähnen (`(2x + y + seed) mod 5`: 0 → ×1,1, 3 → ×0,92); Haut glatt; alles andere als Gewebe `×(0,965 + n·0,07)` mit Köperbindung (`(x+y)` gerade ×1,02); Mantel mit senkrechten Falten (`sin((x − w/2)·1,35 + y·0,12 + seed·0,7)` > 0,84 → ×0,88, < −0,92 → ×1,07); Mikrorelief `rel += (n − 0,5)·0,9` außer Haut; ungesättigte helle Pixel (Stahl, Silber, Knochen) bekommen Glanz 150.
8. Zum Schluss `outline(true, 0.95)`.

**Posen** (`pose_for`): bob (Auf und Ab), Beinversatz, Armschwung, Neigung, Waffenwinkel. Beispiele:
- walk: Schritt `[0, 1.5, 2.5, 0, −1.5, −2.5]`, bob `[0, −1, 0, 0, −1, 0]`
- atk von vorn: Waffenwinkel `[−1.3, −0.4, 0.6, 1.8, 1.6]`, Neigung `[−1.5, −0.5, 1.5, 2.0, 0.5]`
- hurt: Neigung −3, bob 1, Waffe +0,7
- roll: Kugel aus Kuppeln (Mantel, Kopf, Haar, Stiefel), die sich in 5 Bildern dreht, mit Staub
- dead: Figur von vorn gemalt, um 90° gedreht hingelegt, um 18 % abgedunkelt, Blutlache (3A0A0E … 9A2A2E)

**Waffen** werden pixelweise entlang eines Winkels gezeichnet (Stahl-, Gold- und Holzrampen, Relief 6). Längen: Entermesser 20, Rapier 25, Dolch 11, Hammer 22, Knüppel 19, Fetisch 19, Dreizack 28.

---

## 8. Kreaturen-Rig (3D-Wesen als Pixelkunst)

`rig.rs`: Tiere, Monster, Schiffe und Bauwerke werden **räumlich** aus Formen gebaut und per Strahl gerendert. Ein Modell liefert so alle Richtungen und Posen.

**Raum**: x rechts, y vorwärts (Blickrichtung des Wesens), z oben; 1 Einheit = 1 px.
**Formen**: Ellipsoid (`ell`, `ball`, `ell_dir`, `ell_rot`), Quader (`cube`, `cube_rot`, `cube_dir`), senkrechter Zylinder (`cyl`), Dreieck/Viereck (Häute, Segel: `tri`, `quad`), Röhre als Kugelkette (`tube`, `curve` mit Catmull-Rom, `spike`), Punkt (`dot`, Pupillen und Funken), Auge (`eye` = Kugel + Pupille + Glanz). Dazu `ik` (zweigliedriges Bein mit Knie), `part()` (neue Konturgruppe), `clip(lo, hi)` (Höhenschnitt, z. B. Wasserlinie), `mark/rotate_since/shift_since` (Teilposen wie Kiefer oder Deckel), `scale`.

**Material** (`mat(farbe)` = 6-stufige Rampe) mit `.spec(0–255)`, `.glow(stärke)`, `.bias(±)`, `.pat(muster)`, `.soft()` (ohne Innenkontur), `.pal(liste)`.
**Muster** (verschieben die Rampenstufe, im Wesensraum ausgewertet): Schuppen, Streifen, Flecken, Bretter, Deckplanken, Mauerwerk, Nieten, Fell/Federn, Korn.

**Kamera**: orthografisch, Neigung `PITCH = 0,55` rad (sin 0,523, cos 0,852). Richtung → Gieren: vorn 0, Seite −π/2, hinten π.
```
Bildschirm: x = ox + p.x ,  y = oy + p.y*sin(P) - p.z*cos(P)       (ox = w/2, oy = h - Fuß)
Strahl zur Kamera b = (0, cos P, sin P)
```
**Ablauf je Pixel**: Jede Form wird gegen den Strahl getestet (Ellipsoid analytisch, Quader mit der Platten-Methode, Zylinder mit Mantel und Deckel, Dreieck baryzentrisch). Der nächste Treffer gewinnt (Tiefe d). Gespeichert werden Normale, Trefferpunkt im Wesensraum, Material und Gruppe.

**Schattieren**:
```
l  = n_bild · LIGHT                       // n_bild = (n.x, n·unten, n·b)
v  = (l*0.75 + 0.35 + bias)*(N-1) + muster(q) + (bayer-0.5)*0.55
Innenkontur: Nachbar weiter hinten als d+1.2 (anderes Teil) oder d+5 → v -= 1.6
Höhlenschatten: 12 Nachbarn in 2–3 px Abstand; jeder, der um dd > 1,5 näher liegt, verschattet → v -= 0.2 · Σ min(dd/6, 1)
Glanzlicht (spec > 90): h = norm(LIGHT + (0,0,1)) → v += (n·h)^24 · 1.6 · spec/255
Farbe = pal[round(v)] ;  Relief = (d - d_min)*0.6
up = (1 - max(n.z, 0))*255 ;   Höhe über Grund = max(q.z * cos P, 0)
Leuchten = Farbe × glow
```
Danach Punkte (Pupillen, Glanz), dann `outline(true, 0.95)` und `fill_ev()`. Wichtig: Die Höhe je Pixel ist die **echte Höhe über dem Boden**, deshalb folgen Schatten und Tiefenschärfe der Grundfläche des Wesens.

**Schiffe**: Rig mit 16 Kursen; gerendert werden 9 (Gieren `k·2π/16 + π`), die übrigen gespiegelt. Bossschiffe nur gerade Kurse. Maßstab eigenes Schiff 2,1, Bossschiffe 3,25, Holländer 6,9.

**Eigenes Schiff: schwarze Galeone** (`ships::build_black`, 7.11., gut dreimal so viele Teile wie vorher; Maße l 47, b 14,5, Tiefe 16,5, Achterkastell (13, 7), Vorschiff 5,5). Werkstoffe: Rumpf 18151C mit Plankenfugen (Glanz 55), Unterwasserschiff 101618, Band 4E1216 (ochsenblutrot, mit den Pforten), Goldleisten C8963A (Glanz 230) bei `dz−6,1…−5,4` und `dz−2,5…−1,9`, Schandeck 15100E, Deck 3A2E26, Segel 1E1B22 (auf Tempo 20: 26222E) mit Flicken 3A3440, Wimpel 7A1A1C, Fenster FFA848 (Leuchten 1,5), Laternen FFB050 (1,9), Knochen D6CDB6. Rumpf: 46 Seepocken-Punkte und sechs Tangsträhnen am Unterwasserschiff, Kiel und Ruder. Heck: Kastell mit Band und zwei Goldleisten, Hütte (6 × 3) darüber, zwei Fensterreihen (5 bzw. 4 Fenster in Goldrahmen mit Sprossen), Totenkopf mit gekreuzten Knochen, Goldschnörkel (Catmull-Rom), Balkon mit 15 Balustern, Seitengalerien mit Fenster und Konsole, drei Laternen (zwei an Eisenarmen, eine große mit Goldspitze), Flaggenstock mit Totenkopfflagge. Reling mit Balustern alle 3 Einheiten, Goldhandlauf an Kastell und Hütte, Rüsten mit je fünf Jungfern, Kranbalken mit Anker (Schaft, Stock, Arme, Flunken) und Kette zur Klüse. Masten: drei Abschnitte mit Eisenringen, Mars (Ellipse 4 × 3 mit Rand), Saling, Goldknauf, langer roter Wimpel; Rahsegel (zerfetzt) mit Fußpferden und zwei bis drei Flicken, Besan als Gaffelsegel mit Flicken und Totenkopfflagge an der Gaffel. Wanten: je Mast und Seite fünf untere (Rüste → Mars) mit Webeleinen alle 2,4 Einheiten und drei obere (Mars → Saling) mit Webeleinen alle 2,6, Pardunen nach achtern, Stage zwischen den Masten. Bug: Bugspriet mit Goldringen, Klüverbaum, Stampfstock mit Stampfstagen, 2–4 Klüver, Galion als geflügelter Totenengel (Knochen, Kapuze, rot glühende Augen, Laterne). Deck: Ankerspill mit sechs Spaken, Steuerrad (16-Punkt-Ring, acht Speichen mit Goldknäufen) mit Kompasshaus, zwei Grätinge, Fässer mit Eisenreifen, Kisten, Taurollen, Glocke, drei Decklaternen. Ausbaustufen bis 20 (Tempo: Segelreihen 2–4, Leesegel ab 8 bzw. am Fockmast ab 16, Besantoppsegel ab 12, Klüver ab 6 und 14, schimmernde Säume auf 20; Verteidigung: Bänder/Platten/Vollpanzer ab 1/4/7, Gold und Rammsporn ab 10, Eisenkappe 12, Dornen 14, Heckpanzer 16, Ketten 18, leuchtende Runen 20; Angriff: 6–10 Pforten je Seite, zweite Batterie ab 7, Drehbassen ab 4, Mörser und Jagdgeschütze ab 10, Heckgeschütze und Karronaden ab 13, zweiter Mörser ab 18, Goldringe an den Rohren auf 20). Der Atlasstreifen fasst alle 9 Kurse (bis 274 von 300 px, Vorschau `preview <ordner> shipfit`).

**Auftauchen und Versinken** (`game/summon.rs`): Das Schiff wird um den untergetauchten Teil (`depth · Bildhöhe`) tiefer gezeichnet und unten um denselben Anteil abgeschnitten (`SprOpt.cut_bottom = depth`) – die Unterkante bleibt an der Wasserlinie, die Masten tauchen zuerst auf. An der Schnittkante 14 kleine Gischt-Glows, solange `depth > 0`; Schatten erst unter 60 % Tiefe. Davor ein wachsender dunkler Schatten unter Wasser, Blasen (Partikel, Größe 3–4, leuchtend) und aufblühende Schaumflecken. Nass: Färbung `(255 − 80w, 255 − 55w, 255 − 20w)`, `w` = Restzeit/4 s; Tropfen fallen aus Takelage und Segeln (Höhe bis `30 + 200w`, Schwerkraft 560), Rinnsale laufen gleichmäßig am Rumpf herab, kleine Spritzer an der Wasserlinie. Ablauf: Blasen 1,6 s, Aufsteigen 2,6 s (`depth = (1 − t)³`), 0,7 s danach an Bord; beim ersten Mal vorher Wurf der Flasche (1,6 s, Bogen mit Höhe `40(1 − k) + 150·sin(πk)`, drehend), beim Aufschlag Lichtblitz, Funkeln (PixelFX Hit), Platschen und 30 Glasscherben. Versinken beim Anlegen 2,6 s (`depth = t²`). Testlauf: `&summon`, `&summon=first,<sekunde>`, mit `&freeze&nobanner` für Bildschirmfotos.

**Riesige Meeresspinne** (Maßstab 2,47, Bild 370 × 296 px, nur im Boss-Bereich): acht Beine aus drei Röhren (3,4 → 0,6) mit Gelenkkugeln, rostroten Bändern (6A2A20), Stacheln und **Pelz** – je Schenkel 14, je Schiene 11, je Fuß 3 Borsten (`spike`, Breite 0,5 → 0,36, Länge 3–7, schräg nach außen und gegen die Gliedrichtung, Farbe 15110E). Aufgeblähter Hinterleib (Ellipsoid 20 × 23 × 15, Flecken 1A2220) mit runzligem Bauchfeld (56282A, Streifen 2,2), sechs **Adern** (300A0E, Röhre 0,45 → 0,2 auf der Oberfläche, gewellt, je ein Seitenast), **Pilzbefall** (7E8A5E gefleckt), elf **Eiterbeulen** (Kugel B4AE46 mit Glanz 210 auf dunklem Hof 5E5A22; jede dritte geplatzt: dunkles Loch, herabrinnender Eiter, sonst ein Glanzpunkt F2F0B0), drei sich hineinbohrenden **Würmern** (C2A496), einem **Eiersack** aus elf bleichen Eiern (D6CFB0) unter dem Hinterleib, Seepocken und Tang. Kopfbruststück mit zwei offenen **Wunden** (7C2222 auf Bauchfarbe), fünfzehn leuchtenden Augen (7CF0A0, Leuchten 2,2) und einem milchig-blinden (C8D6C2). **Geifer** (9CC296, Glanz 235): von jeder Klaue ein Faden mit Tropfen, ein schwer durchhängender Faden zwischen den Klauen, Fäden zwischen Knie und Knöchel. Im Spiel fallen zusätzlich grünliche Geifertropfen (Partikel, Schwerkraft 160) von den Klauen. **Bewuchs**: 26 dunkle Miesmuscheln (gedrehte Ellipsoide 3,4 × 1,6 × 1,4, 2E3A58, Streifen 0,9, Glanz 220) in Bänken auf Rücken und Flanken, fünf auf dem Kopfbruststück, je zwei an jedem Schenkel, zwölf weitere Seepocken (Zylinder 1,7 mit dunkler Öffnung), 18 herabhängende Tangsträhnen (Röhre 1,4 → 0,5, 5A6E26, jede dritte Seetang), sieben Moospolster (Ellipsoid 5 × 4,4 × 1,8, 56782C mit Pelzmuster) auf dem Rücken. **Zwei Holzbeine** (3. und 6. Bein): Balken aus Holz (5C3C20 bzw. 3E2814, Maserung) statt Gliedern, Eisenbänder an den Gelenken (2C2C32, Glanz 170), Nieten, ein eiserner Schuh als Fuß. **Kletterbilder** (Animation „cast“, nur Richtung 0, drei Bilder aus dem Laufzyklus): das Rig wird um 180° gegiert und um −90° gekippt – (x, y, z) → (−x, z, y), Bauch an der Wand, Rücken zur Kamera, Kopf nach oben –, auf den Bildgrund geschoben (+60) und auf einer 170 px höheren Leinwand gerendert. Im Spiel: in der Wand lauernd (Fußpunkt 200 px unter der Kante), beim Heraufklettern 2,2 s mit Staub bis zur Kante, dann Aufschlag; kopfüber hinab um π gedreht (Drehpunkt eine halbe Bildhöhe über dem Fußpunkt), zuletzt über die Alpha-Tönung ausgeblendet.

**Eingesponnene Leiche** (`wrapped_body`, Schrägsicht 1:1, liegend nach Osten 64 × 40 bzw. im Sarg nach Norden 36 × 66): Leib aus sieben Ellipsoiden längs einer Achse (Fuß 0 … Scheitel 46, Querschnitte von 3,5/2,2 bis 9,6/4,6), Seide B9B4A0 (Streifen 1,6), 15 schräg laufende Wickelbänder (8C866E) über die Oberseite, fünf lose Fäden flach am Boden (schräg längs des Leibs, damit sie nicht wie Beine wirken), nasse Speichelflecken (A6C2A0, Glanz 230), Verwesungsflecken, ein Blutfleck auf der Brust (441410), eine bleiche Hand (8C9282), die längs am Leib herausragt, und unter der Seide Augenhöhlen und Mund.

---

## 9. Pflanzen und Requisiten

**Baumkrone aus Büscheln** (`flora::clump`, `crown`): Ein Büschel ist eine gewellte Ellipse (Rand-Welle aus zwei Sinus über den Winkel), schattiert wie eine Kuppel mit Rauschen: `l = −0,6·dx − 0,7·dy + 0,4·nz`. Die Grundfläche liegt 0,8 Stufen dunkler, unten rechts zusätzlich −0,8. Darauf liegen viele kleine Blätter (Anzahl rx·ry·0,22), jeweils mit hellerer Oberkante (+0,6), Mitte und dunkler Unterkante (−1,2). Oben links kommen Glanzpunkte in der hellsten Farbe dazu.
Eine Krone besteht aus Randbüscheln (45 %) und Füllbüscheln. Sie werden **von hinten (oben) nach vorne (unten)** gemalt, vordere unten etwas dunkler. Davor liegt eine dunkle Unterkrone (Palette × 0,62) als Eigenschatten.

**Blütenbaum**: Wurzeln (Röhren), verdrehter Stamm aus 3 Strängen, zwei Äste (Bézier), dunkle Unterkrone, Krone, optional Glyzinien (hängende Perlenketten, oben hell, unten dunkel), Umriss 0,85. Paletten (dunkel → hell):

| Name | Hex |
|---|---|
| blossom | 6E3A5E 9E5E86 CC8EAE EAB8CE F8DCE6 FFF6F8 |
| pink | 5E2848 8E3E6A C25E92 E48AB4 F4B8D2 FDE0EC |
| violet | 2E1E52 4A2E7A 6E48A8 9670CC BA9CE4 DCC8F6 |
| blue | 14204E 1E3478 2E52A6 4A76CC 74A0E4 A8CCF6 |
| teal | 0A2E38 124650 1E6870 2E8E90 4AB2AC 7ED2C6 |
| green | 0E2E1A 184A24 24682E 388A3A 56AA4A 86CA62 |
| leaf | 122E1C 1C4626 2A6232 3E803C 5CA04A 8CC064 |
| red | 4A0E14 7A1A1E AE2E26 DA5034 F07E4A FAB074 |
| ember | 3A0E0C 6A1C14 A2341C D4582A F08A44 FFC274 |
| swamp | 0A2014 12321C 1C4826 2A6030 3E7A3A 5C9A4A |
| bark | 120C10 1E1418 2E2024 423034 5A4444 76605A |
| wood | 2E1E12 4A3220 6A4A30 8A6644 A8845C C4A47A |
| stone | 2E3238 464C54 646A70 868A8C A8AAA8 CACAC4 |
| gold | 4A3410 7A5A1C AA842C D2AE44 EACC6C F8E8A0 |

**Requisiten (`PropArt`)**: Bild + Anker + Bodenschatten (rx, ry) + Kollision (Kreis oder Kasten) + Licht (dx, dy, Höhe, Radius, Farbe, Flackern) + Wind (`sway`) + `overhead` (ab dieser Zeile blendet die Krone aus, wenn der Spieler dahinter steht) + Animationsbilder (Flammen, Gehängte).

**Boden-Deko** (kleine Sprites ohne Kollision): Gras- und Schneebüschel, Blumen, Klee, kleiner Farn (schwingen im Wind), dazu flach liegend Steinchen, Steine, Zweige, Laub, Tang, Treibholz, Wurzeln, Muscheln, Krebspanzer, Glut, Seerosen, Schilf, Pilze, Kristalle. Sie entsteht **je 256-px-Eimer erst beim Zeichnen** (`World::deco_bucket` in `world/nature.rs`, Zufall je Kachel aus den Kachelkoordinaten, also jedes Mal gleich) und folgt dem **Standort** (`World.nat`, siehe Kapitel 9 „Natürlicher Bewuchs“) statt einer Biom-Tabelle: Horste aus `patch = fbm(0,13·Kachel, 2)` (Gras 1 + (patch − 0,06)·12, höchstens 3, sonst 18 % ein Büschel), Blumen nur auf Wiesenflecken (`fbm(0,06·Kachel) + 0,12 auf Wiese > 0,14`, Anteil je Biom) und je 10 × 10 Kacheln **eine** Art (Kolonien), Klee über 0,22. Im **Schatten** (≥ 2) Waldboden: 1–2 Stück Laub, Zweige, Farnsprossen, Wurzeln, 5 % Pilze (Frost: Zweige/Schneehalme, Glut: Zweige/Glut/Kiesel). **Sand**: am Flutsaum (Landabstand 1) 40 % Tang, Muscheln, Treibholz, Kiesel, 3 % Krebspanzer/Gräten; Düne (2–3) Strandgras in Horsten. **Schnee**: Halme nur fleckenweise (patch > 0,12, 60 %), am Hangfuß Steine. **Asche**: Steine (14 %, am Hangfuß immer), 7 % Glut. **Fels**: 30 % Steine. **Hangfuß** überall Steine. **Wege**: 8 % Kiesel. **Pflaster/Planken**: 5 % Kiesel/Laub, Flaschen, Taue, Plankenstücke, Münzen nur auf Planken und in Siedlungen (5 %). **Wasser**: Seerosen (60 %) und Schilf nur in stillem Binnenwasser und im Mangrovensumpf, direkt am Ufer (22 % bzw. 30 %). Keine verstreuten Schädel, Knochen, Laternenstängel mehr.

**Natürlicher Bewuchs** (`world/nature.rs`, Nutzerwunsch 7.11.: „Flora und Deko sinnvoll setzen, wie es natürlich auf der Erde wäre“, danach „düsterer und abwechslungsreich, Dark Fantasy“). Statt der bunten Fantasiebäume wachsen die PixelLab-Arten und ihre düsteren Abwandlungen (`pixellab::VARIANTS`, beim Start umgefärbt: verdorrt `[32 + (h − 100)·0,12, s·0,62, l·0,8]`, blutrot `[352, min(s·0,95, 0,72), l·0,68]`, nachtgrün `[h + 18, s·0,85, l·0,72]`, knochenbleich `[h, s·0,12, 0,22 + l·0,78]`, Raureif (helle Pixel bis 65 % zu `[205°, 0,25, 0,86]`), verwelkt `[48, s·0,32, l·0,86]`, Blüten blutrot `[355, 0,72, l·0,5]` bzw. geisterblau `[196, 0,4, l]` – jeweils nur für Grün bzw. Blütenpixel) sowie Trauerweide und Leuchtpilze (kaltes cyanfarbenes Licht, Radius 120). Die Tabelle unten ist der Stand vor dem Dark-Fantasy-Durchgang; heute: Galgenstrand mit dunklen Kiefern, gebleichten Totbäumen und Weiden am Wasser, Aschebucht mit gebleichten Totbäumen, Frostgrotte mit bereiften Totbäumen, Rabenwacht und Mangroven mit verdorrten, blutroten und nachtgrünen Kronen, Weiden am Wasser, Leuchtpilzen im Unterholz und Geister-/Blutblumen auf den Lichtungen (Tabellen in `flora`).

| Biom | Wald | Höhen (Ebene ≥ 3) | am Wasser (≤ 3 Kacheln) | Sandstrand | Unterholz / Waldrand / Offenland | Waldschwelle |
|---|---|---|---|---|---|---|
| Galgenstrand | Zypresse 3, toter Baum 3, Kiefer 2 | Kiefer 3, tot 1 | tot 2, Zypresse 1 | Palme | Farn, Gras, Dornbusch / Dornen, Trockengras, Gras / Trockengras, Gras | 0,10 |
| Glutwacht | Zypresse 2, tot 2, Kiefer 1 | Kiefer 2, Zypresse 1 | Zypresse, tot | – | Farn, Gras / Dornen, Gras / Gras, Trockengras | 0,20 |
| Aschebucht | verkohlt 5, tot 3 | verkohlt 3, tot 2 | tot 2, verkohlt 1, Kiefer 1 | – | Trockengras / Trockengras, Dornen / Trockengras | 0,14 |
| Frostgrotte | Schneefichte 6, tot 1 | Schneefichte | Schneefichte 3, tot 1 | – | Schneebusch | 0,06 |
| Rabenwacht | Eiche 3, Laubbaum 3, großer Baum 2, Kiefer 2 | Kiefer 4, Zypresse 1, Laubbaum 1 | Laubbaum 2, Eiche 1, Zypresse 1 | – | Farn, Pilze / Dornen, Gras, Farn / Gras, Trockengras | 0,04 |
| Fluchmangroven | großer Baum 2, Laubbaum 2, Mangrove 2, Eiche 1, tot 1 | Laubbaum 2, großer Baum 2, Kiefer 1 | Mangrove (`swampTree`) 5, tot 1 | – | Farn, Pilze / Farn, Gras, Schilf / Gras | −0,04 |

**Standortfelder** (`World::compute_nat`, einmal nach `compute_shore`, ~20 ms): je Kachel ein Byte `nat` – Bits 0–3 Landabstand zum Wasser (Breitensuche vom Ufer, 1–15), Bits 4–5 Schatten (`stamp_shade` nach allem Bewuchs: Abstand zur Krone `hypot(dx, (dy − 14)·1,2)` < 1 / 1,8 / 2,6 Kacheln → 3 / 2 / 1, tote und verkohlte Bäume höchstens 1), Bit 6 Hangfuß (Wandkachel oben, oben-schräg oder seitlich), Bit 7 stilles Binnenwasser (Wasser, das nicht über Wasser mit Meer ≥ 8 Kacheln vom Land verbunden ist: Teiche, Seen; Bäche zum Meer zählen nicht).

**Bäume** (`scatter`): Waldrauschen `f = fbm(0,042·Kachel, 3)`, an Ufern (Landabstand 2–5) +0,05, direkt am Salzwasser −0,08; Walddichte `wd = smoothstep(Schwelle, Schwelle + 0,22, f)`. Je Kachel ein Versuch mit `0,2 + 0,45·wd` (Offenland: 0,6 % ein Solitärbaum), Art nach Standort (Landabstand ≤ 2 bzw. zur Hälfte ≤ 4 → „am Wasser“, Ebene ≥ 3 bzw. zu 35 % Ebene 2 → „Höhen“), Palmen nur auf Sand 2–5 Kacheln vom Wasser in lockeren Gruppen (`fbm(0,09) > 0,08`, 16 %). **Abstand der Stämme** je Art (Eiche 72, großer Baum 66, Mangrove 58, Laub-/toter/verkohlter Baum 48, Palme 46, Kiefer/Schneefichte 40, Zypresse 34 px) × (1,25 − 0,4·wd), Solitärbäume × 2,2 – im Kern dicht, am Rand licht. Kein Baum auf Eis, Fels, Wegen, Pflaster, Planken, Treppen und ihren Antritten (samt Nachbarkacheln), in Siedlungen, über Kanten (`free_t`), auf schiefem Fuß (`foot_ok`: Mitte und vier Punkte im Radius 9 px begehbar und auf einer Ebene).
**Unterholz** (zweiter Durchgang, Abstand zum nächsten Stamm): < 46 px Schatten → 30 % Unterholz in Flecken (`patch > −0,06`), Pilze nur nahe am Wasser, 1,2 % ein umgestürzter Stamm; 46–110 px Waldrand → 11 % Sträucher; Offenland → Blumenkolonien (2–4 Stück) auf Wiesenflecken, Gras in Horsten (11 %, 1–2), sonst 2 %, auf Höhen selten ein Findling. **Ufer** (Landabstand 1): Schilf (22 %, Sumpf 32 %) im seichten Wasser vor stillem Binnenwasser, im Sumpf und an Schlamm; Treibholz am Sandstrand (1,2 %), Steine an Felsufern (3 %). **Düne** (Sand bis 4 Kacheln): Trockengras in Horsten (14 %, bis 4). **Hangfuß**: 9 % Geröll (Findling 15 %, in der Frostgrotte 12 % Kristalle). Belegung in 8-px-Zellen schützt Bauten, Requisiten (Interaktives + 36 px), Feuer (110), Figuren (80), Gegner (40), Fundstücke (30), Anleger (70). Die Haine der Layouts (`grove`) nehmen die Waldarten und halten 38 px Abstand. Wilde Inseln haben statt verstreuter Schädel, Säulen und Holzstapel nur zwei Wegmarken am Weg zum Herzstück (Grabsteine bzw. alte Steine). Debug: `&floramap=<gebiet>` bzw. `w<n>` gibt eine Draufsicht als PNG (Base64, Konsolenzeile `FLORAMAP`) aus – Boden je Material und Ebene, Bäume nach Art, Unterholz, Deko als Punkte.

**Seeungeheuer-Teile** (klein, zur Laufzeit zusammengesetzt und je Seegebiet getönt): Schlangenkopf (zu/offen, Schuppen, Stachelkamm, Leuchtauge) und Körperbuckel (Halbkuppel mit Stacheln) – eine Seeschlange sind ein Kopf und vier Buckel auf einer Sinuswelle, jeder Buckel mit eigenem `cut_bottom` (taucht auf und ab). Riesenkalmar: Mantel mit Leuchtflecken und acht Fangarmen aus je zehn dicht gesetzten Tentakelgliedern (Bogen `sin(u·π)·58` aus dem Wasser). Qualle (zwei Pulse, leuchtend), Rochen von oben (drei Flügelschläge, Glutflecken, nach Kurs gedreht). Kanonen-Angler, Skelett-Kraken, Panzerkröte und Riesenhai sind die vorhandenen Kreaturen-Bögen, bis zur Hüfte im Wasser; die Wyvern fliegt mit Bodenschatten. Tönung: Glut (255, 168, 136), Frost (190, 224, 255), Sturm (196, 204, 222), Tiefe (160, 224, 170).

**Zierschrift** (`font::display_glyph`): Antiqua-Versalien aus Linienzügen in einem Raster (Versalhöhe 14, Akzente bis −4). Strichbreite wie mit einer um −20° gekippten Breitfeder: `dünn + (dick − dünn)·|sin(Winkel + 0,35)|` mit dick 2,05·k, dünn 0,55·k – senkrecht und „\“ kräftig, waagrecht und „/“ fein. Stämme bekommen an Ober- und Unterkante **Keilserifen** (Länge 1,9, zur Mitte gekehlt). Kantenglättung über die Deckung, dann Fase (oben links +0,12, unten rechts −0,28) und ein Verlauf nach unten (−0,22), dunkler Umriss. Graustufen, damit sie beim Zeichnen eingefärbt wird; im Atlas mit k = 2,4, gezeichnet weich gefiltert in jeder Größe. Der Titel-Schriftzug setzt THRONE/DROWNED mit k = 4,4 und „OF THE“ mit 2,9 auf eine gemeinsame Grundlinie, färbt mit einem Goldverlauf (FFF0C0 → E0AC52 → 7A4A1C) und bekommt eine Zierlinie mit Rauten und einer Krone mit Seelenstein.

**Titelmotiv** (`throne.rs`, in voller Größe gemalt und mit einem Kastenfilter auf 55 % verkleinert, damit Details beim Kameraschwenk nicht flimmern): Thron 540 × 680 – Spitzbogen-Lehne (`145·(1 − ((250 − y)/214)^1,7)`), vertiefte, kannelierte Innenfläche, Bronzeband mit Grünspan, Rosette (Radius 56) mit zwölf Speichen und grün leuchtendem Seelenstein, Schädelfries, Armlehnen aus Quadern mit Schädeln, drei Spitzbogennischen mit Seelenlichtern, Spitzen, Korallen (verzweigte Röhren), Seepocken, Tang, Risse, Wasserspuren, nasse Wasserlinie. Dazu kannelierte, gebrochene Säulen, eine Korallenkrone mit Steinen und Kettenglieder.

**PixelLab-Insel und Prolog** (`sw_art::pixellab`, `world/plisle.rs`): die einzige Stelle mit fertigen Bildern (PNGs in `crates/art/assets/pixellab/`, eingebettet, dekodiert mit der `png`-Crate). Eingebackene fremde Untergründe (Wasser am Wrack, Sandflecken, blaue Rinnen, grelles Grün) wurden vorab umgefärbt: Helligkeit des Bereichs auf die nach Helligkeit sortierte Farbrampe der passenden Bodenkachel abgebildet (24 Stufen; Schlamm, Fels bzw. alle Moos-/Halmfarben des Moos-Bogens). Requisiten: Farbe 1:1, Relief = (geglättete Helligkeit − 0,4)·3 + Wölbung quer zum Objekt `√(1 − u²)·3,5` (u = Lage in der Zeile von −1 bis 1), damit Punktlichter sie räumlich treffen; Fußpunkt = unterste deckende Zeile − 2, waagrecht die Mitte der deckenden Pixel der untersten 7 Zeilen; Bäume mit ausblendender Krone (ab 55–66 % der Höhe) und Wiegen im Wind, die Feuerstelle mit warmem Licht (r 170, 1,9/0,9/0,35, Flackern 0,3). **Piratenfackel** `plTorch` (Pfahl mit Schädel, Eisenkorb): Licht in Korbhöhe (`TORCH_FIRE` 92 px über dem Fuß, z 83, r 260, 2,6/1,45/0,6, Flackern 0,22), darauf die PixelFX-Flamme „Feuerschale“ (Maßstab 0,8, Helligkeit 1,7, 9 px über `TORCH_FIRE`) und drei aufsteigende Funken; die beiden Fackeln am Prolog-Sarg leuchten zusätzlich mit je einem starken Licht (z 70, r 320, 2,3/1,3/0,56), der offene Sarg mit einem warmen (r 160, 1,15/0,82/0,5). Sarg und Fackeln erscheinen erst nach dem Kampf: Rauchwolke (PixelFX „Poof“ ×1,8, graublau), Rauchteilchen und aufsteigende Seelenfunken, dann blenden Sarg, Deckel und Fackeln über die Alpha-Tönung ein (`smooth(t / 1,36)`, insgesamt 1,7 s, die Requisiten selbst bleiben bis dahin verborgen); die Fackeln entzünden sich bei 0,6 bzw. 0,95 s mit Glutfunken und einer kleinen Flammensäule (×0,35), ihre Flamme wächst in 0,4 s auf volle Größe, das Licht mit.

---

## 10. Gelände: Materialien, Paletten, Chunk-Backen

**Bodentexturen im Pixelstil** (`pixstyle::pixel_tex`, statt der Körnung `grain`; `Materials.pixel`, Standard an): die kachelbare 256²-Textur wird wie eine PixelLab-Kachel umgesetzt – große weiche Helligkeitsflecken (Kastenfilter 25 × 25 mit Umlauf) zu 70 % eingeebnet (ruhiger Grund), dann jedes Pixel auf eine Tonleiter des Materials gerastert: Stufe `round((l − Mittel)/0,042)` (−7 … +5), Farbe = vorherrschender Farbton − Stufe·3° (dunkel kühler), Sättigung × (1 − Stufe·0,06), Helligkeit so, dass die Leuchtdichte der Stufe entspricht. Andersfarbige Einzelheiten (Sättigung > 0,12 und Farbton > 28° daneben oder viel satter: Blumen, Glut, Moos) behalten ihren Farbton (10°-Schritte, Sättigung in Fünfteln) und werden nur in der Helligkeit gestuft. Einzelpixel ohne gleiche Stufe im 4er-Nachbarn übernehmen die häufigste Stufe ringsum. Kostet etwa 6 ms je Material (einmalig beim ersten Gebrauch). Vorschau: `preview <ordner> ground` (oben ohne, unten mit).

**Fertige Kacheln (PixelLab-Insel Schädelhorn)**: `TerrainMap::art_px(px, py)` liefert für Landpixel eine Farbe aus zwei verbundenen Wang-Sätzen (Kiesstrand ↔ Moosgras, Moosgras ↔ Waldboden, je 16 Kacheln à 32 px). Jede **Kachelecke** trägt ein Terrain aus dem Uferabstand der vier angrenzenden Kacheln (Breitensuche): ≤ 2 Kacheln Sand, ab 6 Kacheln Wald, wo `fbm(0,075·Ecke) + (d − 6)·0,03 > 0,1`, sonst Gras; Wald neben Sand wird Gras (Sand und Wald teilen keinen Satz), ebenso um Landeplatz und Lager. Je Kachel: Satz = kleinstes Eckterrain, Maske NW·8 + NE·4 + SW·2 + SE·1 aus den Ecken darüber. Im Bäcker: Farbe übernehmen, Relief = (Helligkeit − 0,4)·4, am Wasser dieselbe nasse Uferkante (abdunkeln ×0,62, etwas 203848) und dunkle Umrisslinie wie überall; das Wasser selbst bleibt das des Shaders. Die Insel ist flach (keine Hochland-Ebenen, keine Wege, kein prozeduraler Bewuchs, keine Boden-Deko). **Prolog**: dieselbe Technik mit einer Kette aus drei Sätzen (Schlamm ↔ Schieferfels ↔ totes Moos ↔ dunkles Gras, alle um 20 % aufgehellt) auf der Platte der Prolog-Insel (Ebene 4, ohne Wasser, Abgrund und Planken). Abstandsfelder (Breitensuche, Kacheln): Rand der Platte, Bach, Pfade (Polylinien, seitlich geschlängelt `sin(0,31s)·1 + sin(0,83s)·0,5`, Breite 1,3 bzw. 1,0 Kacheln). Eckterrain: Fels am Erwachensplatz; Schlamm bis 1 Kachel vom Bach, bis 3 Kacheln wo `fbm(0,13·p) < −0,22`, und um die Wracks; Fels bis 2 Kacheln vom Rand bzw. von den Mauern, auf Pfaden, im Spinnenrevier und wo `fbm(0,06·p) > 0,42`; sonst Gras, wenn `−0,9·n1 + 0,022·min(Rand, 16) + 0,12 (3–8 Kacheln vom Bach) − 0,15 (nahe Pfad) + 0,25·n2 > 0,3`, sonst Moos; danach so lange absenken, bis keine Ecke mehr als eine Stufe über einem ihrer acht Nachbarn liegt. Vegetation: Baumgruppen an den besten Stellen (Rauschen + Bonus 4–12 Kacheln von der Mauer, 3–9 vom Bach, auf Gras), mindestens 13 Kacheln auseinander, je 3–6 Bäume im Umkreis von 3,6 Kacheln und 8–14 Pflanzen Unterholz (innen Farn/Pilze/Gras, außen Gras/Farn, höchstens ein Stamm); Schilf (36 %) bzw. Gras (9 %) an jeder Uferkachel; Grasbüschel wo `fbm(0,17·p) > 0,22` (42 %); Felsgruppen (Findling + 1–3 kleine) an Klippe und Mauerfuß, 9 Kacheln auseinander. Mindestabstände je Art (Bäume 34 px, Farn 14, Gras 11, Pilze 10, Stamm 36, Felsen 28). Die Pilze haben ein schwaches kaltes Licht (r 70). Die Prolog-Stimmung ist etwas heller (Umgebung 0,72, Mond 0,78), damit man Bewuchs und Pfade erkennt.

**Materialien** (Priorität für Überlagerung / „erhaben“ für Kanten):
Wasser 0/–, Abgrund 0/–, Sand 1, Schlamm 2, Asche 3, Erde 4, Eis 5, Fels 5,5, Moos 6 (erh. 3), Gras 7 (3), Wiese 7,5 (3), Schnee 7,5 (2,5), Basaltwaben 8 (1), Pflaster 9 (1,5), Marmor 10 (1,5), Planken 11 (1,5).

**Texturen**: je Material und Gebiet eine kachelbare **256 × 256**-Textur, erzeugt beim ersten Gebrauch. Danach folgt eine **Körnung** (feines fBm ±7 %, Hash ±7 %, warm/kalt ±6 in R/B).
- **Gras**: drei Palettenstufen mit großen weichen Flecken (fBm /64 + /16), 184 Büschel aus 3–5 Halmen (heller, dunkle Fußlinie), 280 helle Einzelhalme, Blumengruppen (4–9 Blüten aus Kreuz-Pixeln, 40 % mit gelber Mitte F2C84A).
- **Weg/Erde**: Rauschfeld, 500 Körner, Kiesel (kleine gewölbte Ellipsen aus der Steinpalette mit Licht von oben links und dunklem Schatten darunter), 4 Risse.
- **Sand**: Rauschfeld, Rippel (Sinus über y mit Rauschverzerrung), Körner, Muscheln.
- **Pflaster**: Worley-Zellen (Fuge bei d2−d1 < 0,08, optional Moos in den Fugen), jeder Stein zur Mitte gewölbt, Licht über den Zellgradienten.
- Weitere: Marmor (Glanz 60), Planken, Basaltwaben (mit Glutfugen), Asche, Schnee, Eis, Schlamm, Fels, Moos.

**Gebietspaletten** (je 6 Töne, dunkel → hell; Moos 4 Töne):

| Gebiet | Gras | Weg | Sand | Stein | Klippe |
|---|---|---|---|---|---|
| Küste (fahl) | 1C2016 2A301E 3A4228 4C5432 5E663C 747A4A | 221C16 30281E 403528 524434 645442 786852 | 3E3A34 504B42 635D52 767064 8A8476 9E988A | 26282C 36383E 4A4C52 5E6066 76787C 8E9094 | 16181C 22252A 30343A 40454C 525860 686E76 |
| Festung | 1A1E18 262A20 34382A 444632 56563C 6A6848 | 2A2628 3A3436 4C4446 5E5658 726A6A 88807E | 2E2A2C 403A3C 524C4C 665E5E 7A7270 908884 | 1E1A20 2C2830 3C3642 4E4854 625A66 78707A | 100E12 1C181E 2A242C 3A323C 4C424E 625664 |
| Sturm | 1A3028 24463A 345E44 4A7A4E 66965E 88B072 | 4A4036 625446 7C6C5A 968670 AEA088 C6B8A2 | 5E584A 7A7262 968C78 AEA490 C4BAA6 D8D0BE | 30363C 464E54 5E686E 7A8288 989EA2 B6BABA | 161A20 242A32 343C46 48525C 606C76 7E8A92 |
| Frost | 4E6672 6A8490 8AA4AE AAC2CA CCDDE2 EEF6F8 | 3E4E5C 546674 6C7E8C 8698A4 A2B2BC C0CED6 | 5E6E78 788894 94A2AC AEBAC2 C8D2D8 E0E8EC | 34424E 4A5A66 627480 7C8E98 98A8B0 B6C4CA | 141E2A 22303E 344658 4A6074 647C90 8299AA |
| Tiefe | 0E2418 163420 204A2A 2E6232 427A3C 5C944A | 2E261C 40362A 54483A 6A5C4A 80725C 988A72 | 3E3A2C 524C3A 68604A 7E765C 948C70 AAA286 | 262C28 363E38 4A544C 606A60 788276 929C8E | 121814 1E261E 2C362C 3C4A3A 506048 687A5C |
| Glut | 2E1820 44222A 5C3036 764242 905650 AA6E62 | 4E4242 6A5A58 86746E A08C84 B8A49A CCBAAE | 54464A 6E5E5E 887674 A28E8A BAA6A0 CEBCB4 | 28242A 3A363E 504A54 68626A 827C84 9C969C | 141016 221C24 322A34 443A46 5A4E5C 726674 |
| Ende | 160C12 22141C 341E2A 48283A 5E344A 76425C | 221A20 32282E 44383E 584A50 6E5E64 84747A | 261E22 382C32 4A3C42 5E4E54 726268 88767C | 1A141A 2A222A 3C323C 50444E 665862 7E6E78 | 0C080C 181018 261A26 362636 4A344A 60445E |

Glutfugen: Festung D8542A, Glut D8442E, Ende E85A2E.

**Materialgrenzen** (`mat_at`): An jedem Weltpixel werden die 4 umliegenden Kacheln bilinear gewichtet. Jedes Material bekommt `Gewicht + Rauschen(px + k·37, py + k·53)·0,42 + feines Rauschen(5×)·0,1 + Priorität·0,012`; das höchste gewinnt. So entstehen natürliche, ausgefranste Übergänge statt Kachelkanten.

**Chunk backen** (256 × 256 Weltpixel, `ChunkBake`):
1. Materialraster mit 4 px Rand, **Wasserabstand** per Chamfer (2/3-Gewichte). Dazu das **Ufer-Abstandsfeld** für die Wasserpixel: Zellen à 4 px mit 64 px Rand (96 × 96 Zellen, `mat_at` je Zellmitte), Chamfer-Abstand zum Land (4 / 5,66 px), je Wasserpixel bilinear gelesen.
2. **Ebenen je Pixel** mit Rand (seitlich 56, oben 32, unten 138 px) aus dem geglätteten Höhenfeld plus Kantenrauschen.
3. Je Spalte der Abstand nach unten zur tieferen Ebene (Hang/Mauer); je Zeile und Spalte die Abstände zur nächsten tieferen oder höheren Ebene.
4. Je Pixel, in dieser Reihenfolge:
   - **Kante nach Süden** (Tiefe 96 px = `FACE_PX`, seit 8.11. wie in A Link to the Past – vorher 168 px; Festung: 96-px-Mauer aus Quadern) als **Erd- und Felswand** (seit 7.11., vorher sanfter Grashang): je Spalte eine Lippe von `3 + (n+1)/2·5 + sin(0,45x)·1,2` px und ein bewachsener Hang bis `Lippe + 8 + (n+1)/2·22` px. Zeile 0: Abschlusslinie ×0,7; Lippe: Oberfläche ×(1,24 − 0,12k), `up` 40…100; 2,5 px Unterschnitt ×0,42 (`up` 230); Grashang ×(0,8 − 0,16k), senkrechte Halme (1/9 ×1,15) und Wurzeln (×0,62), `up` 130…180. Darunter Fels: Worley-Zellen mit Maßstab 1/17 (y ×2, also breiter als hoch, Rauschversatz ±6 px), Licht aus dem Zellgradienten `(Δx·0,7 + Δy·0,9)·12`, Helligkeit `0,6 − 0,32k + Zelltönung(±0,07) − Licht·0,35 + n2·0,04 − Regenspur` (Spur = `max(0, n(5x)) · 0,12 · (0,4 + k)`), Fuge bei `d2 − d1 < 0,07` (0,12…0,2, Relief −1), Farbe aus der Klippenpalette; oben Erde dazwischen (n > 0,3: 40 % Wegfarbe), Moos auf Zellen mit Licht von oben (grüne Gebiete, n2 > 0,4: 55 %), Eiszapfen unter der Lippe (Frost, jede 7. Spalte, 6–24 px), Glut in jeder 6. Fuge (Gebiete mit Glutfugen). `up` 190…235. Fuß (t > 0,8) bis 45 % dunkler, Geröllsteine, am Wasser nass (35 %) und Glanz 140.
   - Wasser/Abgrund: transparent; in `emi` stehen **Uferabstand** (0..255 ≙ 0..64 px) und **Art** (255 Wasser, 64 Abgrund) – `encode_px` schreibt sie für durchsichtige Pixel in Blau und Alpha der Normalen-Textur.
   - **Seitenwand** (`BANK_PX` 22 px, seit 8.11.) und **obere Kante** (`RIM_PX` 10 px): an der Seite außen eine helle Graskante (d > 0,8, ×1,2), innen eine dunkle Fuge (d < 0,12), dazwischen Fels aus Worley-Zellen, Licht von links (Westseite +0,12, Ostseite −0,14), `up` 200; oben eine dunkle Fuge (d < 0,3, ×0,55), eine helle Lichtkante (×1,22) und ein Saum (×1,06). Außerhalb von Festung und Siedlungen sind Aufgänge Steinstufen in der Wand. Auf Treppenkacheln: in Siedlungen Holzstufen, in der Natur ein Weg im Gebietsstil (Wegkacheln reichen drei Reihen über die Wandkacheln, weil der Hang tiefer ist als drei Kacheln).
   - Sonst: Materialtextur mit **Kantenschatten** unter erhabenen Nachbarn (bis 28 % dunkler + 8 % 101830), **dunkle Kante** zu tieferen Nachbarn (Umrissfarbe), **Lichtkante** oben (+0,08 Helligkeit), **nasse Ufer** (bis 3 px: 60 % dunkler gemischt + 15 % 203848), **Ufersteine** (Worley-Steine 5,5–8,5 px breit zwischen Gras/Erde und Wasser, Glanz 120 direkt am Wasser) und **Umgebungsverdeckung** am Fuß höherer Ebenen (oben bis 14 px, seitlich bis 9 px) und **Schlagschatten** der Wand auf der tieferen Ebene (`SHADOW_PX` 46 px über den Abstand `hi_u`: `k²·0,72 + Rauschen`, dafür reicht das Ebenenraster jetzt 54 px nach oben); zusammen bis 55 % dunkler und 12 % 0C1020.
5. Kodieren (`encode_px` mit Normalenstärke 0,55) und als eine Ebene des Chunk-Texturfelds hochladen.

---

## 11. Der Renderer: Zeichenlisten und Vertexformat

Das Spiel füllt jedes Bild eine Sofortmodus-Struktur `Frame` mit **Vierecken**:

| Liste | Inhalt | Reihenfolge |
|---|---|---|
| `ground` (+ `reflections`) | Wasserfläche, Boden-Chunks, Spiegelbilder | wie eingefügt |
| `decals` | weiche Bodenschatten | wie eingefügt |
| `world` | alle Sprites (Figuren, Requisiten, Deko) | **nach Fuß-y sortiert**, bei Gleichstand nach Einfügefolge |
| `over_alpha` | Nebel, dunkle Flächen | nach dem Licht, vormultipliziertes Alpha |
| `over_add` | Leuchtpartikel, Funken, Lichthöfe | nach dem Licht, additiv |
| `ui` | Oberfläche in UI-Pixeln | zuletzt, direkt auf den Bildschirm |
| `lights` | Punktlichter | die 48 nächsten zur Bildmitte |

**Vertex** (32 Byte): `pos f32×2` (Welt- bzw. UI-Pixel), `uv f32×2`, `col unorm8×4` (Tönung, a = Deckkraft), `ext u8×4` = (Atlasseite bzw. Chunk-Platz, **Modus**, **Wind**, **Flags**), `prm f32×2` = (Grundhöhe über Boden, Parameter). Ein Indexpuffer mit dem festen Muster `b, b+1, b+2, b, b+2, b+3` reicht für alle Vierecke.

**Modi**: 0 Atlas-Sprite, 1 Boden-Chunk (durchsichtige Pixel: Wasser bzw. Abgrund aus der Normalen-Textur), 2 einfarbige Fläche, 3 Ersatz-Wasser (wo kein Chunk liegt; Abgrund laut Maske), 4 Abgrund, 5 Nebelbank, 6 weicher Fleck (Schatten, Glühen), 7 Seekarte (UI), 8 Zierschrift/weiche Bilder (UI, eigene bilineare Mischung aus vier Texeln, vormultipliziert), 10 PixelFX-Effekt (Überlagerung, siehe Abschnitt 18).
**Flags**: 1 Aufblitzen (Treffer), 2 Umriss (Fokus), 4 Leuchten, 8 geisterhaft, 16 grau (UI), 32 Krone ausblenden, 64 Spiegelung.

**Sprite-Viereck** (`quad_spr`): links oben = Fuß − Anker (gespiegelt: Anker von rechts), optional skaliert, gedreht (um den Fuß) und unten abgeschnitten (`cut_bottom`, z. B. bis zur Hüfte im Wasser; die Grundhöhe wird dann auf die sichtbare Unterkante bezogen).
**Wind**: Die oberen beiden Ecken schwingen im Vertex-Shader:
`x += (sin(t·1,6 + x·0,031 + y·0,017)·0,7 + sin(t·2,9 + x·0,07)·0,3) · wind/64`.

**Packen** am Ende des Bildes: Weltliste sortieren, alle Vierecke in einen Vertexpuffer, Bereiche für die sechs Listen merken. Lichter außerhalb der Sicht verwerfen und nach Abstand zur Bildmitte sortieren. Puffer werden wiederverwendet (keine Speicheranlage pro Bild).

**Zusatztexturen**:
- **Rauschen** 256² (RGBA8): R = periodisches fBm (/64, 4 Oktaven), G = Worley-Netz `1 − clamp((d2−d1)/0,12)` (Kaustik), B = fBm (/32, 3 Oktaven), A = Hash.
- **Wassermaske**, grob: **eine Kachel (32 px) je Texel** über die ganze Karte (1776 × 1328): R = Uferabstand (0..64 px), G = Abgrund, B = Tiefe (0..320 px), A = Wasser. Sie liefert Tiefe, Abgrund, Spiegelungen und Ersatz-Wasser; den **feinen Uferabstand** (Schaum) tragen die Chunks selbst. So bleibt die Maske auch bei der zehnfachen Welt klein (9,4 MB statt 150 MB).
- **Seekarte** (RGBA8 sRGB, halbe Kachelauflösung 888 × 664): Papierbild aus Tiefe, Biom, Höhe, Wegen und Küstenlinien; unentdeckte Felder als dunkles, fleckiges Papier (neu hochgeladen, wenn sich die aufgedeckten Felder ändern).

---

## 12. Render-Durchgänge, Formate, Mischmodi

Renderziele in `low × rs` (ohne Tiefenpuffer, ohne Multisampling):

| Ziel | Format | Inhalt |
|---|---|---|
| `alb` | RGBA8 sRGB | Albedo |
| `nrm` | RGBA8 | Normale xy, Höhe/510, Glanz (Löschwert 0,5/0,5/0/0) |
| `emi` | RGBA16F (sonst RGBA8) | Leuchten rgb, **Alpha 0,5 = Wasseroberfläche** (sonst 1) |
| `lit` | HDR | beleuchtete Szene, **Alpha = Bodenzeile** |
| `bright`, `blur_a`, `blur_b` | HDR, halbe Größe | Bloom |
| `dof_a` (halbe Breite, volle Höhe), `dof_b` (halb) | HDR | Tiefenschärfe |

Sampler: `s_near` (Nearest, Clamp) für Kunst, `s_lin` (Linear, Repeat) für Rauschen und Nachbearbeitung. Rauschkoordinaten werden vor dem Lesen mit `fract` auf eine Texturkachel zurückgefaltet (in der großen Welt lägen sie sonst in den Hunderten; manche Mobil-GPUs filtern dann ungenau) – bei Repeat ist das nahtlos.

**Reihenfolge**:
1. **G-Puffer**, Boden: `fs_gbuf`, 3 Ziele, ohne Mischen (Discard bei Alpha < 0,45).
2. **Abziehbilder** (Schatten): `fs_decal`, nur in `alb`, Alpha-Mischung, **nur Farbe** wird geschrieben (Normale und Höhe bleiben).
3. **G-Puffer**, Welt (sortierte Sprites).
4. **Licht**: Vollbild-Dreieck `fs_light` → `lit`. Im selben Durchgang die Überlagerungen: `over_alpha` (PREMUL: One / OneMinusSrcAlpha) und `over_add` (One / One), beide **nur Farbe**, damit das Alpha (Bodenzeile) erhalten bleibt.
5. **Bloom**: `fs_bright` → `bright`, `fs_blur_h` → `blur_a`, `fs_blur_v` → `blur_b`.
6. **Tiefenschärfe** (bei hohen Effekten): `fs_dof_h` → `dof_a`, `fs_dof_v` → `dof_b`.
7. **Endbild** `fs_final` auf den Bildschirm (sRGB-Format), danach die **UI** (`vs_ui`/`fs_ui`, PREMUL).

---

## 13. G-Puffer-Shader

**Sprites und Chunks** (`fs_gbuf`):
- Farbe mit Nearest aus Atlas bzw. Chunk-Feld. Alpha < 0,45 wird verworfen; mit Flag 2 wird ein heller **Fokus-Umriss** (FFF2CC, leuchtend) um das Sprite gezeichnet.
- `rgb = textur × tönung`. Leuchten aus Alpha 0,5–0,99: `glow = rgb · (a − 0,5) · 4`. Flag 1 = Aufblitzen (75 % weiß + Leuchten), Flag 4 = leuchtet (+ rgb · 0,9), Flag 8 = geisterhaft (×0,4, kaltes Leuchten 0,2/0,55/0,5 × Helligkeit).
- **Krone ausblenden** (Flag 32): oberhalb `prm.y` Schachbrett-Discard.
- **Bodenvariation** (FullHD, nur Chunks): `rgb *= 0.84 + m1·0.3 + (m2 − 0.5)·0.1` und ein warm/kalt-Farbstich, damit sich die Texturen nicht sichtbar wiederholen.
- **Normalen** in FullHD: eigene bilineare Mischung der vier Nachbartexel (WebGL erlaubt nur einen Sampler je Textur). Höhe und Glanz bleiben pixelgenau. Bei Chunks wird auf die eigene Ebene geklemmt.
- Ausgabe: `alb = rgb`, `nrm = (n.xy, clamp((höhe·510 + prm.x)/510), glanz)`, `emi = glow · a`.

**Wasser** (`water`, in FullHD auf halben Weltpixeln gerastert):
1. Grundfarbe `mix(flach, tief, smoothstep(max(tiefe, ufer·0,85)))`.
2. **Wellenhöhe** = 0,55·Dünung + 0,3·Wellen + 0,15·Kräuselung (drei wandernde Rauschlagen). Täler dunkler (`× 0,86 + h·0,3`), Kämme heller und grüner.
3. **Kaustik** im Flachen (Worley-Kanal, zwei wandernde Lagen, smoothstep 0,86–0,97).
4. **Schaum**: Uferkante plus wandernde Schaumringe (`fract(ufer·6 − t·0,35)`) und in FullHD Gischt auf hohen Kämmen nahe der Küste.
5. **Normale** aus dem Wellengradienten (×9 × Wellenstärke, begrenzt auf ±0,6).
6. FullHD: **Himmelsspiegelung nach Fresnel** (`0,06 + 0,94·(1 − n·v)^5`), Horizont aus Nebel- und Umgebungsfarbe, Zenit aus Umgebung und Sonne, als Leuchten. Glanz 0,97. Pixel-Modus: Glitzerpunkte.
7. Ausgabe mit **Alpha 0,5 im Leuchtpuffer** als Kennung „Wasser“ für den Licht-Shader (auch bei Spiegelungen, Flag 64).

**Abgrund**: drei Felsebenen mit Parallaxe (Faktor 0,78 / 0,58 / 0,38, je tiefer nebliger), aufsteigender Nebel und Glutfunken.

**Abziehbilder** (`fs_decal`): weicher Fleck mit `1 − smoothstep(0,55, 1, d)`. Im Pixel-Modus in 4 Stufen gerastert.

---

## 14. Licht-Shader (`fs_light`)

Pro Pixel aus dem G-Puffer: Albedo, Normale (z rekonstruiert), Höhe `elev = b·510`, Glanz, Leuchten. Raumpunkt `p3 = (x, y + elev, elev)`.

1. **Bodenzeile** `ground = (y_pixel + 0,5 + elev·rs) / höhe` (0 oben/fern … 1 unten/nah) für Nebel und Tiefenschärfe.
2. **Umgebungsverdeckung** (nur auf ebenen Flächen): 12 Proben auf einer Spirale (Winkel 2,39996·k, Radius 2 + 1,1·k). Ein Verdecker zählt, wenn er > 2 px höher ist **und am selben Bodenort steht**. `ao = 1 − clamp(occ/12·1,6, 0, 0,6)·Stärke`.
3. **Wolkenschatten** (nur bei hohen Effekten): am Bodenort `(x, y + elev)` zwei Rauschlagen `r(q)·0,72 + b(2,7q + (0,37, 0,11))·0,34` mit `q = pos·0,00042 + (t·0,0062, t·0,0024)`, `cloud = 1 − smoothstep(0,5, 0,74, …)·0,4`. Dämpft Sonne (×cloud), Glanz (×cloud²) und Himmelslicht (×mix(1, cloud, 0,35)). Das Rauschen ist die vierte Textur des Lichtdurchgangs.
4. **Umgebung**: `ambient · mix(0,75, 1, n.z·Himmelsanteil + (1 − Himmelsanteil)) · ao · mix(1, cloud, 0,35)`.
5. **Randlicht** (Höhe > 3 px): `(ambient·1,6 + sonne·0,5) · (1 − n.z)² · clamp(−n.y·0,8 + |n.x|·0,35) · 0,55`.
6. **Sonne/Mond**: `max(n·sonne, 0)`, Schatten per Strahl durch das Höhenfeld (12 Schritte à 5 px mit Pixelversatz gegen Streifen). Verdecker = höher als der Strahl + 3 px und am selben Bodenort (±22 px); weicher zum Ende hin. Beitrag `sonne · nd · schatten · (0,5 + 0,5·ao) · cloud`.
7. **Glanz**: Blinn-Phong mit Blickrichtung `(0, 0,55, 0,85)`. FullHD: Exponent `mix(12, 220, glanz²)` mit Normierung `(exp + 8)/48`, Pixel-Modus 40.
   **Wasser** (Alpha 0,5 im Leuchtpuffer) bekommt Sonne/Mond anders: Der Blick kommt aus einer Kamera schräg über der Bildmitte (`mitte + (0, 0,543·d, 0,84·d)`, `d = Bildhöhe·2,5`), sodass der Glanz einen **örtlichen Lichtpfad** bildet, der mit Sonne und Kamera wandert. Darin ein weicher Schimmer `(n·h)^70 · 0,3` und **Glitzerpunkte**: Zufall je Weltpixel (FullHD: halbe Pixel) aus dem Alphakanal der Rauschtextur per `textureLoad`, Sprung 7× je Sekunde, Punkt wenn `r > 1 − (n·h)^300 · 0,035`, Helligkeit 3,2 (Pixel-Modus 2,2). Alles × glanz × schatten × 0,6 × cloud².
8. **Punktlichter** (bis 48): `ndl = max(n·l, 0)·0,85 + 0,15`. Abfall FullHD `(1 − x²)² / (1 + 7x²)` (natürlich mit langem Ausläufer), Pixel-Modus `(1 − x)²`. In FullHD werfen die **8 nächsten Lichter Schlagschatten** (10 Schritte zum Licht durch das Höhenfeld, Schatten 0,25–1). Flackern `1 + (sin(11t + 1,7i)·0,5 + sin(17t + i)·0,5)·flackern`. Beitrag `farbe · ndl · abfall · flackern · (0,6 + 0,4·ao) · schatten`.
9. Pixel-Modus: Licht in 12 Stufen mit leichtem Dithering (Pixelkunst-Look).
10. `c = albedo · licht + glanz + leuchten`.
11. FullHD: **Lichthöfe im Dunst** um jede Lichtquelle (`exp(−5·d)`, × (0,04 + Nebeldichte·0,3)).
12. **Blitz**: `+ albedo · blitz · (0,8, 0,85, 1)`.
13. **Tiefennebel**: `fogk = dichte·(1 − ground·0,6) + Tiefenschärfe·0,03·(1 − smoothstep(0, 0,4, ground))`, `c = mix(c, nebelfarbe, min(fogk, 0,85))`.
14. Ausgabe `rgb = c · belichtung`, **alpha = ground · 0,5**.

---

## 15. Bloom und Tiefenschärfe

**Bloom**: Helligkeit = max(r, g, b), Maske `smoothstep(0,75, 1,6)`. Danach Gauß-Unschärfe in halber Größe, waagrecht und senkrecht (Gewichte 0,227 / 0,316 bei 1,385 Texeln / 0,07 bei 3,231 Texeln). Im Endbild `+ bloom · Stärke` (je Zone 0,55–0,8).

**Tiefenschärfe (Tilt-Shift)**: breite Unschärfe in halber Größe mit Schritt 3 Texel, 9 Abgriffe (0,2 Mitte; 0,17 / 0,12 / 0,07 / 0,035 beidseitig). Helle Werte werden vorher auf 1,1 begrenzt, damit Funken nicht milchig verschmieren. Im Endbild:
```
fd  = ground*2 - fokus          // fokus = Bildzeile des Spielerfußes (0..1)
coc = fd < 0 ? smoothstep(0.2, 0.55, -fd) : smoothstep(0.22, 0.5, fd)
c   = mix(scharf, unscharf, coc*0.8) ;  c *= 1 - coc*0.16    // unscharfe Ränder etwas dunkler
```

---

## 16. Endbild (`fs_final`)

1. **Kippung**: Bildschirm-uv → Quell-uv (siehe Kapitel 2).
2. **Scharf-bilineare Vergrößerung**: Pixel bleiben scharf, Kanten flimmern nicht.
   `w = clamp((fract − 0,5) · Skalierung + 0,5, 0, 1)`, `uv = (floor + w)/Quellgröße`. Die Skalierung berücksichtigt die örtliche Vergrößerung durch die Kippung (`dk`, `dk²/(1 + TILT)`).
3. **Farbsaum** zum Rand (R und B gegenläufig versetzt, `dv·|dv|²·0,01·Stärke`).
4. **Tiefenschärfe** (siehe oben).
5. **Lichtstrahlen**: Bänder in Richtung der Sonne, in der Welt verankert, aus der Rauschtextur; nach oben stärker. Sie hellen vorhandenes Licht auf (`c·(1 + r·0,6·(0,4 + sonne))`), statt grau zu überdecken.
6. **Bokeh-Staub**: zwei Lagen Scheiben (Zellen 52 px und 88 px, 40 % besetzt), Parallaxe zur Kamera, nur außerhalb der Fokuszone.
7. **+ Bloom**.
8. **Tonemapping** (ACES-Näherung): `c(2,51c + 0,03) / (c(2,43c + 0,59) + 0,14)`.
9. **Farbkorrektur**: `c = lift + c·(gain − lift)`, Kontrast `(c − 0,5)·k + 0,5`, Sättigung `mix(luma, c, s)`.
10. **Blenden**: Weiß (Treffer) und Blutrot 0,5/0,02/0,04 (Tod).
11. **Vignette**: `c *= 1 − |uv − 0,5|²·stärke`.
12. **Körnung**: Hash je Bildschirmpixel und Zeit, ± 0,018 / 2.

---

## 17. Stimmungen je Zone

Eine Stimmung (`Env`) besteht aus: Umgebungslicht, Himmelsanteil, Sonnenrichtung, Sonnenfarbe, Schattenstärke, Nebelfarbe und -dichte, Lift, Gain, Kontrast, Sättigung, Bloom, Vignette, drei Wasserfarben (flach, tief, Schaum), Lichtfaktor für Punktlichter, Abgrund-Glut, Lichtstrahlen. Farben sind Hex-Werte × Faktor (linearisiert).

**Grundwerte** (gelten, wo unten nichts steht): Umgebung 8A94B8×0,55, Himmel 0,6, Sonne F2D2A4×0,75 aus (−0,55, −0,2, 0,8), Schatten 0,55, Nebel 4A6170 Dichte 0,06, Lift (0,012, 0,016, 0,03), Gain 1, Kontrast 1,06, Sättigung 1, Bloom 0,55, Vignette 0,75, Wasser 4EC4D0 / 1E6E9E / E8FAFA, Lichtfaktor 1, Strahlen 0,5.

| Zone | Umgebung | Sonne (Richtung) | Nebel (Dichte) | Lift / Gain | Kontr./Sätt. | Wasser flach/tief/Schaum | Licht | Strahlen |
|---|---|---|---|---|---|---|---|---|
| Galgenstrand | 7A88A8×0,76, Himmel 0,45 | B8C8E0×0,66 (0,4, −0,35, 0,7), Schatten 0,62 | 4E5C58 (0,12) | 0,02/0,026/0,03 · 0,96/1/1,02 | 1,1 / 0,7, Bloom 0,6 | 3A6E74 / 16384A / B8CCCC | 1,2 | 0,35 |
| Glutwacht | 7A78A0×0,56, 0,45 | FFA070×0,62 (−0,7, −0,15, 0,5), 0,5 | 3E3C4A (0,08) | 0,03/0,015/0,02 · 1,04/0,98/0,94 | – / 0,9, Bloom 0,7 | 2E7A86 / 1C5268 / C8E8E0 | 1,25 | 0,7 |
| Aschebucht | 8C7480×0,5, 0,4 | FF8A6A×0,3, 0,35 | 84343E (0,06) | 0,03/0,005/0,01 · 1,04/0,97/0,96 | – / 0,8 | 6A3A50 / 2A1426 / E8A0A8 | 1,2 | 0,6 |
| Frostgrotte | 5A78B8×0,38 | A8C8FF×0,55 (0,4, −0,3, 0,8) | 6A88A8 (0,05) | 0,01/0,02/0,05 · 0,95/1/1,06 | – / 0,9 | 6EA8C0 / 24506E / F0FAFF | 1 | 0,55 |
| Rabenwacht | 6E7A90×0,42 | C8D0E0×0,3, 0,25 | 50606E (0,16) | – | 1,1 / 0,82 | 3E8E9E / 18486A / C8E4EA | 1,15 | 0,3 |
| Fluchmangroven | 5A8068×0,4, 0,35 | B8D890×0,32, 0,4 | 4E6250 (0,12) | 0,01/0,03/0,015 · 0,97/1,03/0,95 | – | 2E5A4E / 12302E / 9AC0B0 | 1,2 | 0,75 |
| Ende der Welt | 8A6474×0,62, 0,45 | FF7A5A×0,42, 0,35 | 5A2232 (0,1) | 0,05/0,01/0,025 · 1,05/0,93/0,96 | 1,1 / –, Bloom 0,8 | 7A4A5A / 2E1624 / E08A90 | 1,25 (Abgrund 1,4) | 0,5 |
| Innenräume | 9A8A78×0,6, 0,2 | 6A4A30×0,1, 0,2 | 1A1210 (0) | 0,02/0,01/0 · 1,02/0,99/0,94 | Bloom 0,6, Vignette 1,1 | – | 1,2 | 0 |
| Kneipe (Prolog) | C8B49A×0,82, 0,35 | FFD8A8×0,35, 0,3 | 2A1E18 (0) | 0,025/0,018/0,01 · 1,04/1/0,94 | Bloom 0,7, Vignette 1 | – | 1,1 | 0 |
| Stillwasser (Prolog, Dorf) | C8D2DC×1,8, 0,85 | FFE6BC×1,9 (−0,5, −0,35, 0,8), 0,5 | A8BCC8 (0,02) | 0,05/0,05/0,055 · 1,18/1,15/1,08 | – / 1,08, Bloom 0,7, Vignette 0,6 | 5AAAB0 / 22667E / E8F6F2 | 0,4 | 1 |
| Spinnenschlund (Prolog) | 72809A×0,72, 0,45 | B0C0DC×0,78 (−0,5, 0,32, 0,8), 0,62 | 1E2630 (0,15) | 0,012/0,018/0,026 · 0,95/0,98/1,04 | – / 0,78 | 0C161C / 010203 / 6A7880 | 1,25 | 0,45 |
| Stille (vor Malakor) | 7A86A8×0,5, 0,55 | C8D4F0×0,5 (0,3, −0,35, 0,85), 0,45 | 2A3448 (0,07) | 0,01/0,015/0,035 · 0,96/1/1,06 | – / 0,75, Bloom 0,6 | 3A4E66 / 0C1420 / C8D8F0 | 1,3 | 0,3 |
| Holländer (Gewitter) | 8A9AB0×0,82, 0,55 | B8D0E0×0,7 (0,35, −0,5, 0,7), 0,5 | 24363A (0,07) | 0,01/0,025/0,03 · 0,96/1,02/1 | 1,12 / 0,8, Bloom 0,75 | 2E5A5E / 0A1C22 / B8E0D8 | 1,3 | 0,5 |
| Meeresgrund | 8AA8A0×0,86, 0,5 | B8D8C8×0,66 (−0,3, −0,4, 0,75), 0,5 | 2A4440 (0,12) | 0,01/0,03/0,025 · 0,94/1,02/1 | 1,1 / 0,7, Bloom 0,75 | 2A5A54 / 0A1A18 / A8D8C8 | 1,4 | 0,7 |
| Offene See | 7898C0×0,75 | C8D8F0×0,7 (0,35, −0,45, 0,6) | 3A5468 (0,12) | – | – | 3AA4BC / 124A70 / D8F0F6 | 1 | 0,3 |
| Glutsee | 8A7470×0,7, 0,45 | FF9A6A×0,5 (−0,5, −0,3, 0,6), 0,4 | 4A2A2A (0,1) | 0,03/0,01/0 · 1,04/0,97/0,94 | – / 0,9, Bloom 0,6 | A8705A / 5A2C2A / E8A070 | 1 | 0,7 |
| Eismeer | 7A9AC8×0,66 | C8E0FF×0,7 (0,45, −0,35, 0,75) | 9AB8D0 (0,14) | 0,01/0,025/0,05 · 0,94/1/1,08 | – / 0,85 | 7EC8D8 / 1E5A7A / FFFFFF | 1 | 0,6 |
| Sturmsee | 6E7A94×0,62, 0,35 | B0C0D8×0,4, 0,2 | 4A5866 (0,2) | – | 1,12 / 0,75 | 3A6E7E / 10283C / B8D0D8 | 1,2 | 0,25 |
| Tiefengrund | 5A8474×0,62, 0,35 | 98C878×0,42, 0,35 | 2E5040 (0,2) | 0/0,03/0,02 · 0,94/1,04/0,98 | – | 1E5A44 / 061C18 / 7AE0A0 | 1,3 (Abgrund 1,3) | 0,5 |

**Übergänge**: Die aktuelle Stimmung läuft jedes Bild mit `t = min(dt·1,5, 1)` auf die Zielstimmung zu. In Seegebieten wird die Gebietsstimmung nach Nähe zur offenen See gemischt. Im Prolog mischt sich Stillwasser → Spinnenschlund nach der Zeile des Spielers (`prolog_dark`: Glättung `t²(3 − 2t)` zwischen dem Dorfrand und der Mitte des Galgenwegs); die Nebelschicht folgt mit (Farbe C4D4DE → 70828E, Deckkraft 14 → 70).

**Übersetzung in die Shader-Werte** (`apply`, „Düster“-Option in Klammern):
```
ambient.rgb = Umgebung × 0.68 (0.5),  ambient.w = Himmelsanteil
sun_dir.w   = Schatten + 0.15 (nur bei hohen Effekten, sonst 0)
sun_col.rgb = Sonne × 0.72 (0.55),   sun_col.w = 0.6 (Glanz)
fog         = (Nebelfarbe, Dichte)
lift        = (Lift + (0.004, 0.008, 0.016), Kontrast + 0.06)
gain        = (Gain, Sättigung × 1 (0.75))
misc        = (min(Vignette + 0.2, 1), Körnung 0.018, Bloom (0.2 bei niedrigen Effekten), Belichtung × 1.15)
water_c.w   = 0.3 (Schaumbreite) ;  map.w = Abgrund-Glut
hd          = (TILT 0.32, Tiefenschärfe 1/0, Fokuszeile, Strahlen × (0.7 bei Düster))
hd2         = (Verdeckung 1, Bokeh 0.6, Randlicht 0.55, Farbsaum 0.5)   (bei niedrigen Effekten 0 außer Randlicht)
Punktlichter: Farbe × Lichtfaktor × 1.5
```

---

## 18. Nebel, Partikel, Spiegelungen, Schatten am Boden

- **Nebelschicht je Zone** (bildfüllende Nebelbank, Modus 5; auf See halbe Deckkraft):

| Zone | Farbe | Deckkraft | Tempo |
|---|---|---|---|
| Aschebucht | 200, 70, 80 | 42 | 14 |
| Ende der Welt | 140, 20, 40 | 60 | 10 |
| Fluchmangroven | 90, 130, 90 | 70 | 6 |
| Frostgrotte | 190, 215, 235 | 22 | 8 |
| Rabenwacht | 120, 130, 140 | 70 | 18 |
| Galgenstrand | 170, 190, 200 | 40 | 5 |
| sonst | 150, 190, 200 | 26 | 4 |

  Nebel = zwei wandernde Rauschlagen, `smoothstep(0,35, 0,85, n·0,6 + n2·0,5)`, weicher Rand am Viereck. Im Pixel-Modus 6 Stufen.
- **PixelFX-Effekte** (`src/pfx.rs`, Bibliothek <https://github.com/patch-gg/pixelfx>, MIT): Der Original-Code läuft beim Start im Browser (`web/pfx.js`) und zeichnet jeden der 34 Effekte einmal Bild für Bild (Schleifen 8–12 Bilder, sonst rund 14 je Sekunde, große Effekte höchstens 12), ein Effekt je Bild des Spiels (~1,3 s zusammen). Jedes Bild ist `composite(farbe, licht)` über Transparenz: gerade Alpha, Licht als Deckkraft seiner hellsten Komponente – über dunklem Grund wirkt das wie additives Leuchten. Die Bilder werden auf die deckenden Pixel (Alpha > 3) zugeschnitten und mit einem Skyline-Packer (unten links) auf die Effektseite gelegt (79 % belegt). Paletten passend zur Welt: Säbelhieb Stahl (161A22 … FFFFFF), Doppelhieb Messing (24160C … FFFFFF), Seelen und Aufsammeln Türkis, Zersplittern Knochen, Staub und Rauch gedeckt grau-braun, Beutestrahl Gold, Fledermaus Leder. Effekte mit Darsteller (Seele, Heilung, Wiedergeburt, Zersplittern) bekommen die Gestalt der Spielerfigur bzw. eines Skeletts aus dem Figuren-Maler.
  **Zeichnen** (Modus 10, Überlagerung nach dem Licht, vormultipliziert): `farbe · tönung · helligkeit · a`. Richtungseffekte zeigen nach rechts: `dir(a)` dreht (Senkrechte ×0,8 für die Schrägsicht), nach links gespiegelt statt kopfüber; Bogenhiebe `face(a)` spiegeln nur und neigen höchstens ±0,6 (bei steilem Blick nach oben/unten wie `dir`). Helle Effekte setzen ein Punktlicht mit Hüllkurve `min(t/0,05, 1)·(1 − t/dauer)²`.
  **Verdeckung**: Der Überlagerungs-Durchgang bindet als Gruppe 2 die G-Puffer-Normale (b = Höhe/510) und das Leuchten (Alpha: 1 = Sprite, 0,875 = Boden-Chunk und Abgrund, 0,5 = Wasser). An Sprite-Pixeln gilt Bodenzeile `g = welt.y + höhe`; liegt sie vor dem Fuß des Effekts (`prm.x`), verschwindet er dort weich: `a *= 1 − smoothstep(fuß+1, fuß+7, g) · smoothstep(2, 6, höhe)`. Flacher Boden verdeckt nie; Hiebe schieben ihren Fuß 8 px vor den Angreifer.
- **Partikel**: additive weiche Flecken (Modus 6): Funken über Feuern (aufsteigend, 3 je Feuer), Glut, Gischt, Glühwürmchen, Staub. Schimmer über Gegenständen am Boden in der Seltenheitsfarbe.
- **Bodenschatten**: weiche Ellipse (Farbe 10, 12, 30; Deckkraft 0,42 bei Requisiten, 0,45 beim Spieler). Figuren: Radius 15·s, Höhe ⅓ davon. Requisiten: um (4, 1) versetzt.
- **Spiegelungen im Wasser** (`reflect`): Der Teil über dem Fuß wird kopfüber unter die Wasserlinie gezeichnet, in 24 waagrechten Streifen wellig verschoben (`sin((y + a)·0,09 + t·1,8)·(1,5 + a·0,03)`). Im Shader mischt Flag 64 das Bild mit der Wasseroberfläche (`k = Deckkraft·0,85`).
- **Arme aus der Tiefe** (Prolog-Ende, `coffin.rs`): je Arm eine Catmull-Rom-Bahn vom schwarzen Grund (Sarg + (±110, 620)) über zwei seitlich ausbauchende Zwischenpunkte bis neben den Sargfuß (±30, +26), dann eine Umschlingung um die Sargmitte (Ellipse 23 × 35, enger werdend 1,18 → 1,0, links im Uhrzeigersinn, rechts dagegen, je 265°). Alle 3 px ein **Querband** (`k:band0/1`, 40 × 7: quer schattiert – Ränder dunkel 12161A, Glanz oben, blasse Unterseite 8C767C, Haut 56626C –, ohne Kanten längs der Röhre; jedes 4. mit Saugnapf), quer zur Bahn gedreht (`rot = atan2(dx, −dy)`, rechter Arm gespiegelt, damit die Saugnäpfe zum Sarg zeigen) – so entsteht eine glatte Röhre ohne die Rillen, die runde Glieder mit Umriss ergäben. Maßstab nach Abstand zur Spitze `0,3 + 2,2·min(d/760, 1)^0,85` (am Grund gut 100 px, an der Klippe 50–90, um den Sarg ~30, Spitze 12), Schlängeln quer zur Bahn `sin(2,6t − 0,022s)·21` (in der Umschlingung keines), die Spitze tastet `sin(4,6t)·22`, bis sie zupackt. Sortierung: an der Klippe nach Bildschirm-y + 40, in der Umschlingung knapp vor dem Sarg. Jedes 8. Band trägt einen pulsierenden Leuchtpunkt (110/255/205, additiv – sichtbar auch im Schwarz der Tiefe), Spitze und Armmitte je ein kaltes Punktlicht (0,26/0,56/0,52, r 220–240), das die Klippe beleuchtet. Beim Hinabziehen fährt der Sarg (selbst gezeichnet, Requisite versteckt) mit `660·(0,3k + 0,7k²)` px nach Süden und neigt sich bis 0,16 rad.
- **Wasserfälle über Naturhänge** (Kaskade, Überlagerung): halb durchsichtige Bahnen à 3 px über die Hanghöhe (Mitte Deckkraft 165, Rand ×0,5; jede dritte hell 200/236/248, sonst 92/164/188), je Bahn zwei abwärts ziehende Lichter (`(t·(1,05 + (k mod 4)·0,13) + k·0,37 + j·0,5) mod 1`), Schaumstufen alle ~30 px (3 px hell + 2 px Nachschaum, leicht wackelnd, wechselnd eingerückt), Schaumkante 5 px am Hangrand, Spritzer an den Seiten, Gischt am Fuß; am Quellteich sanfte Ringe.
- **Brüstung der runden Ringmauer**: 72 Segmente à 5° auf dem Radius `OUT − 0,8`; jedes Segment ist ein Rig (Mauerkappe, Zinnen alle ~22 px, Moos) und wird mit `render_oblique` im Winkel der Kreistangente gerendert (18 Winkel 2,5° + k·5°, Tangenten über 90° gespiegelt), Länge = Sehne `2R·sin 2,5°` ≈ 151 px. Leinwand = gedrehter Grundriss (Schrägsicht 1:1) + 34 px Höhe; Anker in der Segmentmitte. An jedem Stoß ein quadratischer Pfeiler (`ringPost`), der die Fuge verdeckt.
- **Feine Bodenstruktur** (FullHD): Die Normale der Boden-Chunks bekommt ortsfestes Rauschen `(rausch(welt·0,73).rb − 0,5)·0,09` – Körnung und kleine Unebenheiten im Streiflicht.
- **Lichter** an Requisiten (Fackeln, Laternen, Feuer, Kristalle), an Figuren (der Spieler trägt ein schwaches warmes Licht: Höhe 70, Radius 180, 0,5/0,4/0,3), an Effekten (Treffer, Zauber, Explosionen).
- **Innenräume**: Alles außerhalb der Wände wird mit fast schwarzen Flächen überdeckt.

---

## 19. Pixel-Modus gegen FullHD-Modus

| | Pixel-Modus | FullHD |
|---|---|---|
| Renderziele | `low` | `low × 2` (ab 900 px kürzerer Seite) |
| Licht | 12 Stufen, gedithert | stufenlos |
| Punktlicht-Abfall | (1 − x)² | (1 − x²)²/(1 + 7x²) + Schlagschatten der 8 nächsten |
| Glanz | Exponent 40 | 12–220 nach Glanzwert, normiert |
| Normalen | pixelgenau | bilinear zwischen Kunstpixeln |
| Wasser | ganze Pixel, Glitzerpunkte | halbe Pixel, Fresnel-Himmel, Gischt |
| Nebel / Schatten | gerasterte Stufen | weich |
| Boden | – | großflächige Farbvariation |
| Lichthöfe im Dunst | – | ja |

Die Option **„Sparsam“** schaltet Tiefenschärfe, Verdeckung, Strahlen, Bokeh, Körnung und Sonnenschatten ab und lässt nur die Kippung an.

---

## 19a. Ausbau 2026 (Runde 13)

Zusätzliche Shader-Konstanten (in `Globals` und `common.wgsl` vor `lights`, beide Seiten gleich):

| Feld | Inhalt |
|---|---|
| `fx0` | Aufprallbild (0/1), wenig Leben (0..1), Tinten-Übergang (0..1), Kinobalken (0..1) |
| `fx1` | Druckwelle: Weltort x, y, Radius (px), Stärke |
| `fx2` | Wetter: Nässe, Schnee, Asche (0..1), Windstärke |
| `fx3` | Hitzeflimmern, Nacht (0..1), Zoom (1 = keiner), Sterne über dem Wasser |
| `fx4` | Rückstrahlung 0,22, leuchtende Flächen 0,55, Randlicht der Lichter 0,85, Lesbarkeit der Spielfigur 1 |
| `fx5` | Spieler (Weltort; Pflanzen biegen sich weg), Windrichtung |
| `fx6` | Wolkenbedeckung (0,2 + Front·0,55 + Regen·0,25) |
| `tone_lo`, `tone_hi` | Farbtrennung: Faktoren für Schatten und Lichter (1 = neutral) |
| `counts.y` | Blitz (harte Schatten, nur bei hohen Effekten) |

`env::apply` setzt alle Bildwirkungen auf Grundwerte (Titelbild!), das Spiel überschreibt sie danach in `Look::apply` (`src/game/look.rs`).

**Art des Pixels im Leuchtpuffer-Alpha**: 1 Sprite, **0,98 Laub** (Sprites mit Windstärke > 0), **0,96 Spielfigur** (Flag `HERO` = 128), 0,875 Boden/Abgrund, 0,5 Wasser. Die Windstärke steht seit Runde 13 auf allen vier Ecken; der Vertex-Shader schwenkt nur die oberen (Eckennummer `vertex_index & 3` – die flache Interpolation von `ext` ist unter WebGL und WebGPU verschieden: WebGL nimmt die letzte, WebGPU die erste Ecke).

**Licht-Pass** (zusätzlich zu Kapitel 14):
1. **Wetter auf Flächen** vor allem anderen: Nässe dunkelt die Albedo (×(1 − 0,26·Nässe)) und hebt den Glanz (mindestens 0,5); auf ebenem Boden (Art 0,875, `n.z > 0,94`) **Pfützen** aus zwei Rauschlagen (`smoothstep(0,6, 0,655)`), die den Himmel spiegeln und Regenringe tragen (`rain_ring`: je Zelle ein Tropfen, Ring wächst und verblasst). Schnee/Asche legen sich auf nach oben weisende Flächen (`smoothstep(0,8, 0,97, n.z)`, auf Requisiten voll, auf Boden 35 %), fleckig über ein Rauschfeld.
2. **Rückstrahlung und leuchtende Flächen** (FullHD): 8 Proben auf einer Spirale mit dem goldenen Winkel (`GI_KERNEL`, Radien 4–72 px, Gewicht `1/(1 + r²/900)`), je Pixel nach einem 4×4-Muster gedreht (geordnet, flimmert nicht). Mittlere Albedo der Umgebung × (Umgebung·0,55 + Sonne·0,3) × 0,22 färbt das Umgebungslicht (Farbbluten); mittleres Leuchten (ohne Wasser) × 0,55 macht Lava, Glut, Pilze und Runen zu Lichtquellen.
3. **Wolkenschatten nach Bedeckung**: Schwelle `0,62 − 0,3·Bedeckung`, Stärke `0,28 + 0,3·Bedeckung`.
4. **Lichtflecken unter Kronen**: Ist der stärkste Verdecker des Sonnenstrahls Laub (Art 0,98) und höher als 22 px, wird der Schatten in Flecken aufgehoben (`smoothstep(0,6, 0,68)` des b-Kanals bei 1/96, mit dem Wind wandernd, bis 82 %).
5. **Punktlichter**: flackernde Flammen bewegen ihr Licht (±2,4 px·Flackern), damit tanzen die Schatten; bis zu **12 Schattenlichter** je Pixel (ab dem 7. nur 7 Schritte); **weicher Halbschatten** über die Tiefe des Verdeckers (`smoothstep(2, 11)`) statt harter Kante; **farbiges Randlicht** an Kanten, die zum Licht weisen (`(1 − n.z)^1,6 · max(n̂·l̂, 0)`, von hinten stärker), Faktor `fx4.z`; **Lichtkegel im Dunst** (Hof × Strahlen aus dem Winkelrauschen um das Licht, nachts ×1,7); auf Wasser **Lichtpfade** (zitternder Streifen unter jedem Licht über 12 px Höhe).
6. **Spielfigur** (Art 0,96): kühler Rand `(1 − n.z)^1,4 · (Umgebung·0,9 + 0,05)` und Mindesthelligkeit (0,1/0,11/0,14).
7. **Wasser nachts**: Sterne spiegeln sich (Raster 3 px, von den Wellen verschoben, funkelnd, Dichte 0,32 %), Regenringe auf der Oberfläche.
8. **Blitz**: zweiter Strahl durch das Höhenfeld aus (0,42, −0,55, 0,72), 10 Schritte à 6,5 px → harte, lange Schatten.

**Endbild** (zusätzlich zu Kapitel 16): Zoom um die Bildmitte (Umrechnungen in `Frame::src_to_screen/screen_to_src/world_to_screen` berücksichtigen ihn); **Druckwelle** (Ring `exp(−((d − r)/16)²)`, 5 px nach außen); **Hitzeflimmern** (zwei wandernde Rauschlagen, Stärke = Zonenhitze + `smoothstep(0,05, 0,4, Bloom-Helligkeit)·0,9` → über Feuer und Lava von selbst); Lichtstrahlen nur durch **Wolkenlücken**; Unschärfe entsättigt um 22 % (Blickführung); **weicher Fuß** nach dem Kontrast (`max(kontrast, c·0,42)`: Schatten behalten Zeichnung); **Farbtrennung** `c *= mix(tone_hi, tone_lo, (1 − luma)^1,6)`; **wenig Leben** (Entsättigung 50 %, roter Rand im Herzschlag: zwei Schläge je 0,87 s); **Aufprallbild** (Schwelle 0,26 → fast Weiß/fast Schwarz); Dither ±1/255 gegen Farbstufen; **Tinten-Übergang** (Rauschen + Randabstand gegen die Schwelle `1,32 − 1,62·Blende`, dunkle Kante), die schwarze UI-Blende kommt erst ab 72 %; **Kinobalken** je 10,5 % oben und unten.

**Szene**: Nebel in **zwei Lagen** – die obere (heller, nachts kaum, schneller im Wind) mit Parallaxe `prm.x = 0,35` (die Rauschkoordinate wandert mit der Kamera mit); die dritte Nebel-Rauschlage nimmt den weichen b-Kanal (der Worley-Kanal g zeichnete ein Liniennetz). Wasser spiegelt die ziehenden Wolken (dasselbe Feld wie die Wolkenschatten, von den Wellen verzerrt). **Silhouette** (Überlagerung Modus 11): die Spielfigur wird ein zweites Mal gezeichnet, aber nur wo der G-Puffer ein Aufrechtes zeigt, dessen Bodenzeile vor ihrem Fuß liegt – kühl, halb durchsichtig, Rand heller, sanft pulsierend.

**Figuren**: `SprOpt.squash` (senkrechter Faktor, waagrecht gegenläufig, der Fuß bleibt) und `SprOpt.skew` (Oberkante seitlich versetzt) – `draw::pose_layer`: Atmen im Stand (±1,2 %), Ausholen (0,955, zurücklehnen) und Nachschwingen (1,045, vorlehnen) beim Hieb, Lehnen weg vom Treffer; Gegner sterben über mehrere Bilder (taumeln, sacken zusammen, dann liegen).

### Stilrichtlinie (Kurzfassung)

1. **Licht erzählt**: Jede Szene hat eine Hauptlichtquelle (Mond, Glut, Laterne) und Lichtinseln. Das Umgebungslicht bleibt gedämpft, Schatten behalten Zeichnung (weicher Fuß), reines Schwarz gibt es nur in der Tiefe (Abgrund, Prolog-Meer).
2. **Lesbarkeit vor Stimmung**: Spielfigur (Randlicht, Silhouette), Gegner (Ausholglanz, Wunden), Beute (Strahl) und Gefahren (Telegraphen) müssen in jeder Stimmung erkennbar sein.
3. **Pixel bleiben Pixel**: keine Drehung von Pixelkunst über wenige Grad (Schiff ≤ 2°), Bewegung lieber als Stauchen/Scheren; Kamera auf Zielpixel runden.
4. **Ein Effekt-Stil**: PixelFX für Kampf und Magie, eigene Teilchen nur für Wetter, Staub, Glut und Blut – alle beleuchtet oder leuchtend, nie flach eingefärbt.
5. **Bewegung in der Welt**: Wind (Laub, Halme, Nebel, Teilchen), Wasser (Wellen, Kielwasser, Treibgut, Fische) und Feuer (Glut, Flimmern) bewegen sich immer leicht – Stillstand nur als bewusster Moment (Zwischensequenz, Stille vor dem Boss).

### Farbskript (Farbtrennung je Zone, Schatten → Lichter)

| Zone | Schatten (`tone_lo`) | Lichter (`tone_hi`) | Wetter | Nacht |
|---|---|---|---|---|
| Galgenstrand | kühles Blaugrün 0,93/1/1,08 | Silber 1/1/1,02 | Regen 0,35, Wind 0,35 | 0,6 |
| Glutwacht | Violett 0,95/0,95/1,09 | Glut 1,06/0,99/0,9 | Asche 0,25 | 0,5 |
| Aschebucht | Rotbraun 1,02/0,97/0,99 | Glut 1,04/0,99/0,92 | Asche 0,8, Hitze 0,35 | 0,3 |
| Frostgrotte | Blau 0,95/0,99/1,07 | 1/1/1,02 | Schnee 0,85, Wind 0,5 | 0,6 |
| Rabenwacht | 0,95/1/1,06 | neutral | Regen 0,9, Wind 0,8 | 0,5 |
| Fluchmangroven | Grün 0,93/1,05/0,96 | fahles Gelb 1,03/1,03/0,91 | Regen 0,45 | 0,5 |
| Ende der Welt | Purpur 1,03/0,92/1 | Blutrot 1,06/0,96/0,94 | Asche 0,4 | – |
| Wilde Inseln | 0,95/1/1,05 | Bernstein 1,03/1/0,95 | Regen 0,35 | 0,7 |
| Offene See | 0,94/0,99/1,07 | 1,02/1/0,97 | Regen 0,3, Wind 0,6 | 0,8 |
| Stillwasser / Kneipe | warm 0,98/1/1,03 bzw. 1/0,97/0,93 | Morgen 1,04/1,01/0,95 bzw. Kerze 1,05/1/0,9 | – | – |
| Spinnenschlund | kalt 0,93/0,98/1,1 | 0,98/1/1,04 | – | – |

**Tageszeit**: ein Tag dauert 30 min; in der zweiten Hälfte sinkt die tiefe Nacht (`env::night`: Mond statt Sonne, Umgebung ×0,55/0,6/0,78, Nebel dunkler und +0,025, Laternen ×1,4, Strahlen ×0,45, Sterne) mit der Zonenstärke `nightk` über die Stimmung. **Bosskampf** (`env::arena`): Umgebung ×0,84–0,95, Kontrast +0,06, Vignette +0,25, Laternen ×1,15; in Phase 2 wandelt sich das Wetter mit dem Element (Feuer: Hitze und Asche, Eis: Schnee, Sturm: Regen, Wind und Blitze, Erde: Regen).

---

## 19b. Runde 14: Wellen, Brandung, plastische Dächer, Titelbild „Sturmnacht“

**Dünung** (`scene.wgsl: swell`): Richtung `d` = Windrichtung (`fx5.zw`), `ph = dot(p, d)·0,045 − t·1,2 + wob·6` mit `wob` = langsames Rauschen (biegt die Kämme), Höhe `sh = (0,5 + 0,5·sin ph)^2,4` (schmale Kämme, breite Täler), abbrechend über ein grobes Rauschen `seg`; Steigung analytisch `1,2·s^1,4·cos(ph)·k·d`. Stärke `swk = (0,3 + 0,7·Tiefe)·(0,55 + 0,65·Wind)·smoothstep(0, 0,35, Ufer)`. Wirkung: `c *= 0,9 + sh·0,26·swk`, Durchscheinen auf den Kämmen, heller Saum ab `sh` 0,82, Normale `−∇·8,5·swk`, Schaumkronen `smoothstep(0,86, 0,98, sh)·Rauschen·smoothstep(0,3, 0,85, Wind)`. **Brecher** am Ufer: `bw = fract(Ufer·2,4 − t·0,22 + n·0,12)`, Schaumlinie bei `bw` 0,84–0,995, milchiger Schwall davor (0,55–0,88), nur im Uferband (`fw·0,4 … fw·2,6`).

**Brandung** (`world/surf.rs`, `game/surf.rs`): Stellen = Wasserkacheln mit Uferabstand 1 am offenen Meer neben Fels/Basalt/Asche/Eis oder Land höherer Ebene; je Stelle Takt `3 + 3,2·seed` s. Beim Anschlag PixelFX `Splash` (Maßstab `(0,7 + 0,55·seed)·big·(0,85 + 0,45·Wind)`), dann 2 s lang: Schaumfleck (Radius `(14 + 36·√k)·big`, Deckkraft `(1 − k)²·110`), Gischtschleier als `glow` (steigt `(10 + 30k)·big`, verweht mit dem Wind), dazu ständig ein atmender Schaumsaum.

**Plastische Dächer** (`buildings.rs: roof_3d`): Relief-Rampe Rückseite `k·hr` (`up` 0), Vorderseite `(1 − k)·hr` (`up` 118), `hr = 0,9·(First − Oberkante)`; Walmflächen innerhalb `hip = 6 + hip_w·…` vom Rand (Relief `0,25·d + 2`, `up` 60, links ×1,42, rechts ×0,5), Grate ×1,5, Vorderseite ×(1,32 − 0,36k), am First ×1,45/1,22, Rückseite ×(0,5 + 0,25k), alles ×1,18; Traufschatten 12 Zeilen `0,52 + 0,48k²`, Gebäudeecken links ×1,2, rechts ×0,62…1.

**Titelbild „Sturmnacht“** (`post.wgsl: title_scene`, `fx6.y` = 1, `fx6.z` = Zeit): Koordinaten `q = ((u − 0,5)·Seitenverhältnis, 0,5 − v)` plus sanftes Schweben, Horizont `0,06`. Himmel: Verlauf `(0,055, 0,075, 0,095)` → `(0,005, 0,007, 0,018)`, Mond bei `(0,28·asp, 0,34)` Radius 0,036 mit zwei Höfen, Wolken `tfbm(q.x·cz·0,1 + t·0,011, cz·0,055 − t·0,0035)` mit `cz = 1/(hy + 0,1)`, Dichte `smoothstep(0,34, 0,74, n)`, beleuchtet vom Mond und vom Blitz. Blitz (`t_flash`): Periode 7 s, Zeitpunkt zufällig, `exp(−10d) + 0,7·exp(−9(d − 0,16))`, Strahl aus zwei Rauschlagen. See: Abstand `z = 0,12/dy`, vier Wellenzüge (Richtung, Frequenz 1,1/1,7/2,9/4,3, Amplituden ×0,55), Fresnel, Helligkeit `0,45 + 1,1·h`, Mondpfad `exp(−|x − Mond + n.x·0,6(0,2 + dy)|·(5 + 26dy))` mit Glitzern, Schaumkronen, Dunst `exp(−22dy)·0,75`. Galeone `ship_cov` im Schiffsraum (Länge 1, Maßstab 0,36, Rollen `0,045·sin 0,62t + 0,02·sin 0,27t`): Rumpf, drei Masten mit je drei geblähten, an der Unterkante zerfransten Segeln mit Löchern, Bugspriet, Takelage, Flagge; Randlicht = Deckung minus Deckung um 0,006 Richtung Mond. Kraken: Kopf `(sx + 0,34, hz − 0,06 + 0,07·rise)`, Ellipse 0,15, gefleckt; Augen bernstein mit waagrechter Pupille, Blinzeln `fract(0,07t)`; Arme `tentacle` (16 Glieder, Winkelzuwachs `(0,05 + 0,32u³)·sin(…) + (0,02 + 0,16u²)·Richtung`, Radius `r0·(1 − 0,88u)`, Seite für die helle Unterseite mit Saugnäpfen `fract(26u)`), drei hinter (r0 0,032), zwei vor dem Schiff (0,045). Endbild: `tonemap(c·1,3)`, kühle Schatten/warme Lichter, Vignette `1 − 0,6·v^2,4`, Körnung 0,025. Kosten: Arme und Schiff nur in ihren Begrenzungsrechtecken.

## 20. Schritt-für-Schritt-Plan für das neue Projekt

**Technik-Gerüst**
1. Projekt mit wgpu anlegen (hier Rust + Bevy 0.16 mit eigenem Render-Graph-Knoten; jede wgpu-Umgebung geht). Ziel WebGL2: keine Compute-Shader, ein Sampler je Textur, keine Tiefenpuffer nötig.
2. Uniform-Block `Globals` exakt wie im Anhang (`common.wgsl`) anlegen, inklusive 48 Lichter.
3. Vertexformat (Kapitel 11) und festen Indexpuffer bauen. `Frame` mit den sechs Listen plus Lichtliste, `pack_frame` mit Sortierung nach Fuß-y.
4. Renderziele und Pipelines exakt nach Kapitel 12 (Formate, Mischmodi, Schreibmasken!). Die drei Shader aus dem Anhang übernehmen.
5. Auflösung und Kamera nach Kapitel 2 (540 px kürzere Seite, ganzzahliger Kamera-Ursprung, UI-Skalierung, FullHD ab 900 px).

**Kunst-Bibliothek**
6. `Canvas` mit den fünf Kanälen, `put/set/glow`, `normal_at` (Kapitel 3).
7. Farbe: `hex`, `mix`, `shade`, HSL, `ramp_n`, `outline_of`, `pick`, `bayer`, Rauschen, Hash, Rng (Kapitel 4).
8. Werkzeuge `dome`, `tube`, `outline`, `inner_shade`, `rim_light`, `poly` (Kapitel 5).
9. `encode_px` und Atlas mit Regalpackung, Trimmen und Ankern (Kapitel 6).
10. Rauschtextur 256² und Wassermaske (Kapitel 11) erzeugen und hochladen.

**Inhalte**
11. Erste Figur mit dem Figuren-Maler (Beine, Rumpf, Arme, Kopf, Umriss), dann Posen und Richtungen.
12. Rig-Renderer (Formen, Strahltest, Schattierformel, Innenkontur, Höhe je Pixel), erstes Wesen aus Ellipsoiden.
13. Baum aus Büscheln (Kapitel 9), Steine und Büsche als Kuppeln.
14. Materialtexturen 256² im Pixelstil (`pixel_tex`: Grund einebnen, Tonleiter, Pixelgruppen), Gebietspaletten, `mat_at`, Chunk-Bäcker mit Hängen, Kanten, nassen Ufern und Verdeckung (Kapitel 10). Chunks streamen (nur sichtbare plus Ring, Texturfeld-Ebenen).

**Abstimmung**
15. Stimmungen (Kapitel 17) mit `apply` übernehmen, Übergänge weich (1,5/s).
16. Nebelschicht, Partikel, Schatten, Spiegelungen (Kapitel 18).
17. Vergleich: gleiche Szene, gleiche Stimmung, gleiche Kamerahöhe (540 px). Erst Albedo prüfen (Licht aus = nur Umgebung 1), dann Licht, dann Nachbearbeitung zuschalten.

**Faustregeln für neue Kunst**
- Jede Form braucht Relief, sonst wirkt sie im Licht flach.
- Aufrechtes (Wände, Figuren) mit `up = 255`, Liegendes mit `up = 0`, Hänge mit kleinen Werten (35).
- Höhe über Grund immer korrekt setzen (Fuß = 0). Davon hängen Schatten, Verdeckung, Nebel und Tiefenschärfe ab.
- Leuchtendes über `glow` (wird im Alpha kodiert), Glänzendes über `spec` (Wasser, Metall, Eis, Marmor).
- Paletten 6-stufig, dunkel → hell, mit kühlen Schatten. Umriss selektiv statt schwarz.

---

## 21. Fallstricke

- **Texturfelder nie mit einer durch 6 teilbaren Ebenenzahl** (WebGL-wgpu hält sie für Würfel-Felder; dann fehlen alle Sprites oder der Boden ist schwarz). Atlasseiten aufrunden, Chunk-Plätze z. B. 50.
- WebGL: **ein Sampler je Textur** (innerhalb eines Shader-Einstiegspunkts). Weiche Normalen und die weich gefilterte Zierschrift deshalb von Hand aus vier Texeln mischen statt einen zweiten (linearen) Sampler zu nehmen – sonst bricht die Shader-Übersetzung ab („A image was used with multiple samplers“) und das Spiel stürzt beim Start ab.
- Große Welten: keine Textur über die ganze Karte in feiner Auflösung (WebGL-Grenzen, Speicher). Feine Daten gehören in die gestreamten Chunks, global reicht eine Kachel je Texel.
- **Pipelines mit mehreren Renderzielen: gleiche Mischung und gleiche Schreibmaske für alle Ziele.** WebGL2 kennt ohne die Erweiterung `OES_draw_buffers_indexed` keine getrennte Mischung je Ziel (wgpu: `DownlevelFlags::INDEPENDENT_BLEND`); fehlt sie, bricht schon `create_render_pipeline` ab und das Spiel startet nicht (8.11.: die Abziehbild-Pipeline im gemeinsamen G-Puffer-Durchgang). Lösung: alle Ziele mit derselben Mischung (`ALPHA_BLENDING`, nur Farbe) und im Shader für die Ziele, die unverändert bleiben sollen, Alpha 0 ausgeben.
- Überlagerungen dürfen **kein Alpha schreiben**, das Alpha der beleuchteten Szene trägt die Bodenzeile für die Tiefenschärfe.
- Schatten-Abziehbilder schreiben **nur Farbe** in die Albedo, sonst zerstören sie Normalen und Höhen.
- Höhe im Normalen-Texel ist Höhe/2 (bis 510 px); größere Wesen brauchen Grundhöhe über `prm.x`.
- Höhe und Bodenort gehören zusammen: Verdecker zählen nur „am selben Bodenort“, sonst werfen Figuren im Vordergrund Schatten auf den Hintergrund.
- Kamera-Ursprung auf Zielpixel runden (Pixel-Modus ganze, FullHD halbe Weltpixel), sonst flimmert die Pixelkunst.
- **Flache Interpolation (`@interpolate(flat)`) nimmt unter WebGL die letzte, unter WebGPU die erste Ecke** eines Dreiecks. Werte, die nur an einzelnen Ecken stehen (früher die Windstärke an den oberen Ecken), kommen im Fragment-Shader je nach Plattform an oder nicht – solche Werte auf alle Ecken schreiben und im Vertex-Shader über `vertex_index` unterscheiden.
- Neue Felder in `Globals` immer **vor** `lights` und in Rust und WGSL in derselben Reihenfolge; `env::apply` muss sie auf Grundwerte setzen, sonst erbt das Titelbild die Werte des letzten Spielbilds (z. B. eine volle Tinten-Blende).
- Boden-Chunks schrittweise backen (Zeitbudget je Bild), sonst ruckelt das Laufen.
- Atlas beim Start in den Upload **verschieben**, nicht kopieren (Wasm-Speicher wächst nur).
- **Sonnenglanz auf Wasser nie mit der überall gleichen Blickrichtung**: Steht die Sonne „hinter“ dem Bild (Norden, Gegenlicht) oder läuft eine Stimmungs-Überblendung durch diese Lage, liegt der Halbvektor nahe der Senkrechten, fast jede Wellenflanke fällt in den Glanzkegel und die See wird flächig weiß-fleckig. Für Wasser deshalb eine Kamera über der Bildmitte (Lichtpfad) und Glitzerpunkte mit begrenzter Dichte statt eines breiten Kegels. Beim Testen ruhig 40–60 s warten: das Schiff fährt vom Ufer weg, und erst dann blendet die Hochsee-Stimmung ein.
- Texturkoordinaten für wiederholende Texturen mit `fract` zurückfalten, bevor sie riesig werden (Weltkoordinaten bis ~57 000 px).

---

## 22. Anhang: Shader im Wortlaut

Die drei Dateien aus `src/gfx/shaders/`. `common.wgsl` wird vor `scene.wgsl` bzw. `post.wgsl` gesetzt (eine Quelle je Pipeline-Gruppe).

### `common.wgsl`

```wgsl
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
    hd: vec4<f32>,        // HD-2D: Kippung, Tiefenschärfe, Fokuszeile, Lichtstrahlen
    hd2: vec4<f32>,       // HD-2D: Verdeckung, Bokeh-Staub, Randlicht, Farbsaum
    hd3: vec4<f32>,       // FullHD: weiche Normalen, Auflösungsstufe, frei, frei
    fx0: vec4<f32>,       // Aufprallbild, wenig Leben, Tinten-Übergang, Kinobalken
    fx1: vec4<f32>,       // Druckwelle: Weltort xy, Radius, Stärke
    fx2: vec4<f32>,       // Wetter: Nässe, Schnee, Asche, Windstärke
    fx3: vec4<f32>,       // Hitzeflimmern, Nacht, Zoom, Sterne
    fx4: vec4<f32>,       // Rückstrahlung, leuchtende Flächen, Randlicht der Lichter, Lesbarkeit
    fx5: vec4<f32>,       // Spieler xy (Pflanzen biegen sich weg), Windrichtung xy
    fx6: vec4<f32>,       // Wolkenbedeckung, frei, frei, frei
    tone_lo: vec4<f32>,   // Farbtrennung: Faktor Schatten
    tone_hi: vec4<f32>,   // Farbtrennung: Faktor Lichter
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

// Schrägsicht (HD-2D-Diorama): Bildschirm (0..1) → Low-Res-Bild (0..1).
// Oben liegt die Welt ferner (gestaucht), unten näher (vergrößert). Gegenstück: Frame::screen_to_src.
fn tilt_src(uv: vec2<f32>) -> vec2<f32> {
    let c = G.hd.x;
    let d = 1.0 + c * uv.y;
    return vec2<f32>(0.5 + (uv.x - 0.5) / d, uv.y * (1.0 + c) / d);
}
```

### `scene.wgsl`

```wgsl
// Szene: G-Puffer (Farbe, Normale+Höhe+Glanz, Leuchten), Überlagerung (Nebel, Partikel) und UI.

@group(1) @binding(0) var t_acol: texture_2d_array<f32>;
@group(1) @binding(1) var t_anrm: texture_2d_array<f32>;
@group(1) @binding(2) var t_ccol: texture_2d_array<f32>;
@group(1) @binding(3) var t_cnrm: texture_2d_array<f32>;
@group(1) @binding(4) var t_mask: texture_2d<f32>;
@group(1) @binding(5) var t_noise: texture_2d<f32>;
@group(1) @binding(6) var t_map: texture_2d<f32>;
// nur in der Überlagerung: G-Puffer (Normale + Höhe, Leuchten) für die Verdeckung der Effekte
@group(2) @binding(0) var t_gn: texture_2d<f32>;
@group(2) @binding(1) var t_ge: texture_2d<f32>;

struct VIn {
    @location(0) pos: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) col: vec4<f32>,
    @location(3) ext: vec4<u32>,   // Ebene, Modus, Wind, Flags
    @location(4) prm: vec2<f32>,   // Grundhöhe, Parameter
};
struct VOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) col: vec4<f32>,
    @location(2) @interpolate(flat) ext: vec4<u32>,
    @location(3) prm: vec2<f32>,
    @location(4) world: vec2<f32>,
};

@vertex
fn vs_world(v: VIn, @builtin(vertex_index) vid: u32) -> VOut {
    var o: VOut;
    var p = v.pos;
    // Wind: die oberen Eckpunkte schwingen (Eckennummer aus dem Vertex-Index; die Windstärke steht
    // auf allen vier Ecken, damit der Fragment-Shader Laub erkennt). Böen und Windrichtung kommen
    // aus dem Wetter.
    if (v.ext.z > 0u && (vid & 3u) < 2u) {
        let t = G.screen.z;
        let a = f32(v.ext.z) / 64.0;
        let wind = G.fx2.w;
        let gust = 1.0 + wind * (0.7 + 0.7 * sin(t * 0.63 + v.pos.x * 0.0041 + v.pos.y * 0.0023));
        p.x += (sin(t * 1.6 + v.pos.x * 0.031 + v.pos.y * 0.017) * 0.7 + sin(t * 2.9 + v.pos.x * 0.07) * 0.3) * a * gust;
        p += G.fx5.zw * wind * a * (1.6 + 0.8 * sin(t * 0.9 + v.pos.x * 0.01));
        // Halme, Farne und Schilf biegen sich vor der Spielfigur weg (hohe Kronen liegen zu weit oben)
        let dp = vec2<f32>(p.x - G.fx5.x, (p.y - (G.fx5.y - 12.0)) * 1.25);
        let dl = length(dp);
        if (dl < 34.0) {
            let bend = (1.0 - smoothstep(8.0, 34.0, dl)) * min(a * 2.6, 1.0) * 8.0;
            p.x += sign(dp.x + 0.001) * bend;
            p.y += bend * 0.35;
        }
    }
    let rel = (p - G.view.xy) / G.view.zw;
    o.clip = vec4<f32>(rel.x * 2.0 - 1.0, 1.0 - rel.y * 2.0, 0.0, 1.0);
    o.uv = v.uv;
    o.col = vec4<f32>(pow(v.col.rgb, vec3<f32>(2.2)), v.col.a);
    o.ext = v.ext;
    o.prm = v.prm;
    o.world = v.pos;
    return o;
}

struct GOut {
    @location(0) alb: vec4<f32>,
    @location(1) nrm: vec4<f32>,
    @location(2) emi: vec4<f32>,
};

fn decode_n(n: vec4<f32>) -> vec2<f32> {
    return n.rg;
}

fn noise_at(p: vec2<f32>) -> vec4<f32> {
    // auf eine Texturkachel zurückfalten: in der großen Welt lägen die Texturkoordinaten sonst in
    // den Hunderten, und manche Mobil-GPUs filtern dann ungenau. Bei wiederholender Textur
    // (s_lin: Repeat) ist fract() nahtlos.
    return textureSampleLevel(t_noise, s_lin, fract(p / 256.0), 0.0);
}

fn mask_at(p: vec2<f32>) -> vec4<f32> {
    return textureSampleLevel(t_mask, s_lin, p / G.map.xy, 0.0);
}

// ---------------------------------------------------------------- Wasser
// Wellenhöhe aus drei wandernden Rauschlagen (lange Dünung, mittlere Wellen, Kräuselung)
fn wave_h(p: vec2<f32>, t: f32) -> f32 {
    let a = noise_at(p * 0.32 + vec2<f32>(t * 2.2, t * 1.1)).r;
    let b = noise_at(p * 0.75 - vec2<f32>(t * 3.6, -t * 1.9)).r;
    let c = noise_at(p * 1.9 + vec2<f32>(-t * 6.0, t * 4.5)).r;
    let h = a * 0.54 + b * 0.31 + c * 0.15;
    // Trochoidale Schärfung: breite Wellentäler, markante Kämme
    return mix(h, h * h * (1.35 - 0.35 * h), 0.55);
}

fn water(p_in: vec2<f32>) -> GOut {
    return water_s(p_in, -1.0);
}

// Wasser; `shore_in` ≥ 0: pixelgenauer Uferabstand aus dem Boden-Chunk, sonst aus der groben Maske
fn water_s(p_in: vec2<f32>, shore_in: f32) -> GOut {
    var o: GOut;
    let hd = G.hd3.x > 0.5;
    // Pixelraster: in FullHD halbe Weltpixel, sonst ganze
    var p = floor(p_in) + 0.5;
    if (hd) { p = floor(p_in * 2.0) * 0.5 + 0.25; }
    let t = G.screen.z;
    let m = mask_at(p);
    var shore = m.r;          // 0 am Ufer → 1 weit weg (über ~64 px)
    if (shore_in >= 0.0) { shore = shore_in; }
    let depth = m.b;          // 0 flach → 1 tief
    // Grundfarbe mit Tiefe: im Flachen schimmert der Grund durch, in der Tiefe wird es dunkel
    let dd = smoothstep(0.0, 1.0, max(depth, shore * 0.85));
    var c = mix(G.water_a.rgb, G.water_b.rgb, dd * (0.85 + 0.15 * dd));
    // weite See: großflächig tiefere und hellere Stellen, langsam ziehende Strömungsbänder
    let big = noise_at(p * 0.011 + vec2<f32>(t * 0.21, t * 0.08)).r;
    let band = noise_at(vec2<f32>(p.x * 0.004 + p.y * 0.002, p.y * 0.017) + vec2<f32>(t * 0.05, 0.0)).b;
    c *= 0.88 + big * 0.20 * dd + (band - 0.5) * 0.09 * dd;
    let h0 = wave_h(p, t);
    // Wellentäler dunkler, Kämme heller und smaragdgrün durchschimmernd (Subsurface Scattering)
    c *= 0.84 + h0 * 0.32;
    let sss_col = mix(G.water_a.rgb * vec3<f32>(0.78, 1.26, 1.12), G.sun_col.rgb, 0.22);
    c += sss_col * smoothstep(0.52, 0.82, h0) * (0.22 + (1.0 - dd) * 0.15);
    // Kaustik nur im Flachen berechnen (Voronoi-Netz aus der Rauschtextur)
    var caust = 0.0;
    if (dd < 0.995) {
        let ca = noise_at(p * 0.7 + vec2<f32>(t * 5.0, t * 2.0)).g;
        let cb = noise_at(p * 0.55 - vec2<f32>(t * 3.5, t * 4.0)).g;
        let c_net = smoothstep(0.84, 0.965, max(ca, cb)) + smoothstep(0.88, 0.98, min(ca, cb)) * 0.55;
        caust = c_net * (1.0 - dd) * (1.0 - dd * 0.35) * G.water_b.w;
        c += mix(vec3<f32>(0.75, 0.98, 1.0), G.sun_col.rgb + vec3<f32>(0.3), 0.35) * (caust * 0.38);
    }
    // Schaumringe zum Ufer, Gischt auf hohen Kämmen nahe der Küste (nur in Ufernähe berechnen)
    let fw = G.water_c.w;
    var foam = 0.0;
    if (shore < fw * 3.2) {
        let ring = fract(shore * 6.0 - t * 0.35);
        let foam_edge = 1.0 - smoothstep(0.0, fw * 0.55, shore);
        let fn1 = noise_at(p * 1.7 + vec2<f32>(t * 2.0, 0.0)).b;
        let fn2 = noise_at(p * 2.0 + t * 7.0).b;
        if (hd) {
            let foam_ring = smoothstep(0.76, 0.92, ring) * (1.0 - smoothstep(0.0, fw * 1.6, shore)) * smoothstep(0.38, 0.62, fn1);
            let edge_lace = smoothstep(0.32, 0.52, foam_edge + (fn2 - 0.5) * 0.55);
            let crest = smoothstep(0.66, 0.78, h0) * (1.0 - smoothstep(0.0, fw * 3.0, shore)) * smoothstep(0.42, 0.62, noise_at(p * 2.6 - t * 5.0).b);
            foam = max(max(edge_lace, foam_ring * 0.88), crest * 0.68);
        } else {
            let foam_ring = step(0.82, ring) * (1.0 - smoothstep(0.0, fw * 1.5, shore)) * step(0.45, fn1);
            foam = max(step(0.4, foam_edge + (fn2 - 0.5) * 0.6), foam_ring * 0.85);
        }
        c = mix(c, G.water_c.rgb, foam * 0.92);
    }
    // Normale aus dem Wellenfeld
    let e = select(1.0, 0.65, hd);
    let hx = wave_h(p + vec2<f32>(e, 0.0), t) - h0;
    let hy = wave_h(p + vec2<f32>(0.0, e), t) - h0;
    let nxy = clamp(vec2<f32>(-hx, -hy) * (9.5 / e) * G.water_a.w, vec2<f32>(-0.68), vec2<f32>(0.68));
    var em = vec3<f32>(caust * 0.07);
    var spec = select(0.78, 0.18, foam > 0.5);
    if (hd) {
        // Spiegelung des Himmels nach Fresnel: flach blickend und an Wellenflanken stärker
        let n = vec3<f32>(nxy, sqrt(max(0.0, 1.0 - dot(nxy, nxy))));
        let v = normalize(vec3<f32>(0.0, 0.55, 0.85));
        let r = reflect(-v, n);
        let horizon = G.fog.rgb * 1.15 + G.ambient.rgb * 0.42 + G.sun_col.rgb * 0.12;
        let zenith = G.ambient.rgb * 1.35 + G.sun_col.rgb * 0.08;
        var sky = mix(horizon, zenith, clamp(r.z, 0.0, 1.0));
        // ziehende Wolken spiegeln sich (dasselbe Feld wie die Wolkenschatten, von den Wellen verzerrt)
        let wq = (p + nxy * 26.0) * 0.00042 + vec2<f32>(t * 0.0062 + 0.21, t * 0.0024 + 0.43);
        let cl = textureSampleLevel(t_noise, s_lin, fract(wq), 0.0).r * 0.72 + textureSampleLevel(t_noise, s_lin, fract(wq * 2.7), 0.0).b * 0.34;
        let cov = G.fx6.x;
        sky = mix(sky, G.fog.rgb * 1.4 + G.ambient.rgb * 0.7 + G.sun_col.rgb * 0.1, smoothstep(0.58 - cov * 0.25, 0.85 - cov * 0.25, cl) * 0.55);
        let fres = 0.05 + 0.95 * pow(1.0 - clamp(dot(n, v), 0.0, 1.0), 5.0);
        em += sky * clamp(fres * 2.35 + 0.07, 0.0, 0.65) * (1.0 - foam * 0.85);
        spec = mix(0.98, 0.22, foam);
    } else {
        // Glitzern (Pixelstil)
        let gl = noise_at(p * 1.3 + vec2<f32>(t * 9.0, t * 6.0)).a;
        let glint = step(0.985, gl) * (0.5 + 0.5 * sin(t * 6.0 + p.x));
        em += G.water_c.rgb * glint * 0.8;
    }
    o.alb = vec4<f32>(c, 1.0);
    o.nrm = vec4<f32>(nxy * 0.5 + 0.5, 0.0, spec);
    // Alpha 0,5 im Leuchtpuffer kennzeichnet die Wasseroberfläche für den Licht-Pass (Glitzern)
    o.emi = vec4<f32>(em, 0.5);
    return o;
}

// ---------------------------------------------------------------- Abgrund mit Tiefenebenen
fn abyss(p_in: vec2<f32>) -> GOut {
    var o: GOut;
    let p = floor(p_in) + 0.5;
    let t = G.screen.z;
    let cam = G.view.xy + G.view.zw * 0.5;
    var c = vec3<f32>(0.0);
    var em = vec3<f32>(0.0);
    let fogc = G.fog.rgb;
    // drei Felsebenen, je tiefer desto langsamer (Parallaxe) und nebliger
    for (var i = 0; i < 3; i = i + 1) {
        let k = 0.78 - f32(i) * 0.2;
        let q = cam + (p - cam) * k + vec2<f32>(0.0, f32(i) * 90.0);
        let n = noise_at(q * (0.35 + f32(i) * 0.1)).r;
        let rock = smoothstep(0.55, 0.62, n);
        let depthf = f32(i + 1) / 3.0;
        let rc = mix(vec3<f32>(0.10, 0.08, 0.11), fogc * 0.35, depthf);
        c = mix(c, rc, rock * (1.0 - f32(i) * 0.2));
    }
    // aufsteigender Nebel und Glut
    let mist = noise_at(p * 0.4 + vec2<f32>(0.0, t * 9.0)).r;
    em += fogc * (0.08 + mist * 0.18) * G.map.w;
    let spark = step(0.992, noise_at(floor(p / 2.0) * 2.0 * 0.9 + vec2<f32>(0.0, t * 40.0)).a);
    em += vec3<f32>(1.0, 0.45, 0.2) * spark * 0.8;
    o.alb = vec4<f32>(c, 1.0);
    o.nrm = vec4<f32>(0.5, 0.5, 0.0, 0.0);
    o.emi = vec4<f32>(em, 0.875);
    return o;
}

fn sample_sprite(uv: vec2<f32>, layer: u32, chunk: bool) -> vec4<f32> {
    if (chunk) {
        return textureSampleLevel(t_ccol, s_near, uv, i32(layer), 0.0);
    }
    return textureSampleLevel(t_acol, s_near, uv, i32(layer), 0.0);
}
fn nrm_texel(uv: vec2<f32>, layer: u32, chunk: bool) -> vec4<f32> {
    if (chunk) {
        return textureSampleLevel(t_cnrm, s_near, uv, i32(layer), 0.0);
    }
    return textureSampleLevel(t_anrm, s_near, uv, i32(layer), 0.0);
}
fn sample_nrm(uv: vec2<f32>, layer: u32, chunk: bool) -> vec4<f32> {
    if (G.hd3.x < 0.5) {
        return nrm_texel(uv, layer, chunk);
    }
    // FullHD: Neigung weich zwischen den Kunstpixeln (eigene Hermite-Mischung, WebGL erlaubt
    // nur einen Sampler je Textur); Höhe und Glanz bleiben pixelgenau aus dem nächsten Ecktexel
    var dim = vec2<f32>(textureDimensions(t_anrm).xy);
    if (chunk) {
        dim = vec2<f32>(textureDimensions(t_cnrm).xy);
    }
    let px = uv * dim - 0.5;
    let fr = fract(px);
    // Bodenstücke liegen je in einer eigenen Ebene: am Rand nicht auf die Gegenseite greifen
    let lo = 0.5 / dim;
    let hi = 1.0 - lo;
    let b = clamp((floor(px) + 0.5) / dim, lo, hi);
    let d = 1.0 / dim;
    let b1 = clamp(b + d, lo, hi);
    let t00 = nrm_texel(b, layer, chunk);
    let t10 = nrm_texel(vec2<f32>(b1.x, b.y), layer, chunk);
    let t01 = nrm_texel(vec2<f32>(b.x, b1.y), layer, chunk);
    let t11 = nrm_texel(b1, layer, chunk);
    let s = fr * fr * (3.0 - 2.0 * fr);
    let n = mix(mix(t00.rg, t10.rg, s.x), mix(t01.rg, t11.rg, s.x), s.y);
    let cx = select(t00, t10, fr.x >= 0.5);
    let cy = select(t01, t11, fr.x >= 0.5);
    let c = select(cx, cy, fr.y >= 0.5);
    return vec4<f32>(n, c.b, c.a);
}

@fragment
fn fs_gbuf(v: VOut) -> GOut {
    var o: GOut;
    let mode = v.ext.y;
    let flags = v.ext.w;
    if (mode == 3u) {
        if (mask_at(v.world).g > 0.5) {
            return abyss(v.world);
        }
        return water(v.world);
    }
    if (mode == 4u) {
        return abyss(v.world);
    }
    if (mode == 2u) {
        // Fläche
        o.alb = vec4<f32>(v.col.rgb, 1.0);
        o.nrm = vec4<f32>(0.5, 0.5, clamp(v.prm.x / 510.0, 0.0, 1.0), 0.0);
        var e = vec3<f32>(0.0);
        if ((flags & 4u) != 0u) { e = v.col.rgb; }
        o.emi = vec4<f32>(e, 1.0);
        return o;
    }
    let chunk = mode == 1u;
    // Krone ausblenden (Schachbrett-Dither), wenn der Spieler dahinter steht
    if ((flags & 32u) != 0u && v.uv.y < v.prm.y) {
        let q = vec2<i32>(floor(v.clip.xy));
        if (((q.x + q.y) & 1) == 0 || (q.y & 1) == 0) { discard; }
    }
    var uv = v.uv;
    if ((flags & 64u) != 0u) {
        // Wasserspiegelung: pixelgenaue Wellenverzerrung direkt im Shader (statt 24 CPU-Streifen)
        let t = G.screen.z;
        let wob = sin(v.world.y * 0.11 + t * 2.1 + v.world.x * 0.025) * 1.65
            + (noise_at(v.world * 0.62 + vec2<f32>(t * 3.1, -t * 1.8)).r - 0.5) * 2.4;
        uv.x -= wob / 2048.0;
        if (uv.x < v.prm.x || uv.x > v.prm.y) { discard; }
    }
    var c = sample_sprite(uv, v.ext.x, chunk);
    if (chunk && c.a < 0.45) {
        // Wasser und Abgrund im Boden-Chunk: Art und Uferabstand stehen in der Normalen-Textur
        let wn = nrm_texel(uv, v.ext.x, true);
        if (wn.a > 0.75) {
            return water_s(v.world, wn.b);
        }
        if (wn.a > 0.1) {
            return abyss(v.world);
        }
        discard;
    }
    if (c.a < 0.45) {
        // Umriss beim Fokus: Nachbarpixel prüfen
        if ((flags & 2u) != 0u) {
            let ts = 1.0 / vec2<f32>(textureDimensions(t_acol).xy);
            let a = sample_sprite(uv + vec2<f32>(ts.x, 0.0), v.ext.x, false).a
                + sample_sprite(uv - vec2<f32>(ts.x, 0.0), v.ext.x, false).a
                + sample_sprite(uv + vec2<f32>(0.0, ts.y), v.ext.x, false).a
                + sample_sprite(uv - vec2<f32>(0.0, ts.y), v.ext.x, false).a;
            if (a > 0.4) {
                o.alb = vec4<f32>(1.0, 0.95, 0.8, 1.0);
                o.nrm = vec4<f32>(0.5, 0.5, clamp(v.prm.x / 510.0, 0.0, 1.0), 0.0);
                o.emi = vec4<f32>(0.9, 0.8, 0.55, 1.0);
                return o;
            }
        }
        discard;
    }
    var n = sample_nrm(uv, v.ext.x, chunk);
    if (G.hd3.x > 0.5) {
        if (chunk) {
            // FullHD: feine Bodenstruktur (Körnung, kleine Unebenheiten) in der Normale, ortsfest in der Welt
            let fq = noise_at(v.world * 0.73 + vec2<f32>(13.0, 7.0));
            n = vec4<f32>(clamp(n.rg + (fq.rb - vec2<f32>(0.5)) * 0.11, vec2<f32>(0.02), vec2<f32>(0.98)), n.b, n.a);
        } else if (n.b > 0.02) {
            // FullHD: dezente Material-Mikrostruktur auf Felsen, Mauern, Stämmen und Schiffsrümpfen
            let fq = noise_at(uv * 512.0 + vec2<f32>(5.0, 11.0));
            n = vec4<f32>(clamp(n.rg + (fq.rg - vec2<f32>(0.5)) * 0.055, vec2<f32>(0.02), vec2<f32>(0.98)), n.b, n.a);
        }
    }
    var rgb = c.rgb * v.col.rgb;
    var glow = vec3<f32>(0.0);
    if (c.a < 0.995) {
        // Alpha 0.5..0.99 kodiert Leuchtstärke
        glow = rgb * (c.a - 0.5) * 4.0;
    }
    if ((flags & 1u) != 0u) {
        rgb = mix(rgb, vec3<f32>(1.0, 0.95, 0.9), 0.75);
        glow += vec3<f32>(0.6, 0.5, 0.45);
    }
    if ((flags & 4u) != 0u) {
        glow += rgb * 0.9;
    }
    if ((flags & 8u) != 0u) {
        // Geisterhaft: dunkel, leuchtend kalt
        glow += vec3<f32>(0.2, 0.55, 0.5) * luma(rgb);
        rgb *= 0.4;
    }
    if (chunk && G.hd3.x > 0.5) {
        // großflächige Abwechslung im Boden (Verfärbungen, Erdflecken, kühle und warme Stellen),
        // damit sich die Muster nicht sichtbar wiederholen
        let m1 = noise_at(v.world * 0.018).b;
        let m2 = noise_at(v.world * 0.061 + vec2<f32>(31.0, 17.0)).r;
        rgb *= 0.84 + m1 * 0.3 + (m2 - 0.5) * 0.1;
        rgb *= mix(vec3<f32>(1.05, 1.0, 0.93), vec3<f32>(0.95, 1.0, 1.06), smoothstep(0.3, 0.7, m2));
    }
    if ((flags & 64u) != 0u) {
        // Spiegelung: die Wasseroberfläche bleibt, das Bild schimmert wellig gebrochen hindurch
        var w = water(v.world);
        let wn = w.nrm.rg * 2.0 - 1.0;
        let ripple = 1.0 - clamp(length(wn) * 0.45, 0.0, 0.35);
        let k = v.col.a * 0.86 * ripple;
        w.alb = vec4<f32>(mix(w.alb.rgb, rgb * vec3<f32>(0.88, 0.95, 1.02), k), 1.0);
        w.emi = vec4<f32>(w.emi.rgb * (1.0 - k * 0.55) + glow * 0.75, 0.5);
        return w;
    }
    let elev = n.b * 510.0 + v.prm.x;
    o.alb = vec4<f32>(rgb, 1.0);
    o.nrm = vec4<f32>(n.rg, clamp(elev / 510.0, 0.0, 1.0), n.a);
    // Alpha im Leuchtpuffer: 1 = Sprite (kann Effekte verdecken), 0,98 = Laub (schwingt im Wind:
    // Lichtflecken unter Kronen), 0,96 = Spielfigur (Randlicht), 0,875 = Boden-Chunk, 0,5 = Wasser
    var kind = select(1.0, 0.875, chunk);
    if (!chunk && v.ext.z > 0u) { kind = 0.98; }
    if ((flags & 128u) != 0u) { kind = 0.96; }
    o.emi = vec4<f32>(glow * v.col.a, kind);
    return o;
}

// Schatten/Abziehbilder: nur Farbe abdunkeln (Mischung), Normale und Höhe bleiben.
@fragment
fn fs_decal(v: VOut) -> GOut {
    var a = v.col.a;
    if (v.ext.y == 6u) {
        let d = length(v.uv * 2.0 - 1.0);
        a *= 1.0 - smoothstep(0.55, 1.0, d);
        if (G.hd3.x < 0.5) {
            a = floor(a * 4.0 + hash21(floor(v.world)) * 0.9) / 4.0;
        }
    } else if (v.ext.y == 0u) {
        let c = textureSampleLevel(t_acol, s_near, v.uv, i32(v.ext.x), 0.0);
        a *= step(0.45, c.a);
    }
    // nur die Farbe: Normale und Leuchten mit Alpha 0 (gleiche Mischung für alle Ziele, siehe render.rs)
    var o: GOut;
    o.alb = vec4<f32>(v.col.rgb, a);
    o.nrm = vec4<f32>(0.0);
    o.emi = vec4<f32>(0.0);
    return o;
}

// ---------------------------------------------------------------- Überlagerung (nach dem Licht)
@fragment
fn fs_overlay(v: VOut) -> @location(0) vec4<f32> {
    let mode = v.ext.y;
    if (mode == 5u) {
        // Nebelbank: vor der Texturabfrage zwischen Pixel- und FullHD-Modus verzweigen (keine doppelten Abfragen).
        // prm.x = Parallaxe: höhere Lagen ziehen mit der Kamera mit (Tiefe zwischen den Lagen)
        let t = G.screen.z;
        let wpos = v.world + (G.view.xy + G.view.zw * 0.5) * v.prm.x;
        let e = min(min(v.uv.x, 1.0 - v.uv.x), min(v.uv.y, 1.0 - v.uv.y));
        let edge = smoothstep(0.0, 0.28, e);
        var a = 0.0;
        if (G.hd3.x < 0.5) {
            let p = floor(wpos);
            let n = noise_at(p * 0.25 + vec2<f32>(t * v.prm.y, t * v.prm.y * 0.3)).r;
            let n2 = noise_at(p * 0.12 - vec2<f32>(t * v.prm.y * 0.6, 0.0)).b;
            a = smoothstep(0.35, 0.85, n * 0.6 + n2 * 0.5) * v.col.a * edge;
            a = floor(a * 6.0 + hash21(p) * 0.8) / 6.0;
        } else {
            let pf = wpos * 0.5;
            let n = noise_at(pf * 0.25 + vec2<f32>(t * v.prm.y, t * v.prm.y * 0.3)).r;
            let n2 = noise_at(pf * 0.12 - vec2<f32>(t * v.prm.y * 0.6, 0.0)).b;
            // dritte Lage aus dem weichen Rauschkanal (der Worley-Kanal g zeichnete ein Liniennetz)
            let n3 = noise_at(pf * 0.52 + vec2<f32>(-t * v.prm.y * 0.35 + 17.0, t * v.prm.y * 0.45 + 5.0)).b;
            a = smoothstep(0.33, 0.86, n * 0.52 + n2 * 0.42 + n3 * 0.22) * v.col.a * edge * 0.84;
        }
        return vec4<f32>(v.col.rgb * a, a);
    }
    if (mode == 6u) {
        let d = length(v.uv * 2.0 - 1.0);
        var a = (1.0 - smoothstep(0.0, 1.0, d));
        a = a * a * v.col.a;
        return vec4<f32>(v.col.rgb * a, a);
    }
    if (mode == 2u) {
        return vec4<f32>(v.col.rgb * v.col.a, v.col.a);
    }
    if (mode == 10u) {
        // PixelFX-Effekt (Licht über Transparenz als Deckkraft, vormultipliziert gemischt)
        let c = textureSampleLevel(t_acol, s_near, v.uv, i32(v.ext.x), 0.0);
        var a = c.a * v.col.a;
        if (a < 0.004) { discard; }
        // Verdeckung: steht hier etwas Aufrechtes (Figur, Mauer, Requisit), dessen Bodenzeile vor dem
        // Fuß des Effekts liegt (prm.x), liegt der Effekt dahinter. Boden und Wasser verdecken nie.
        let ip = vec2<i32>(floor(v.clip.xy));
        if (textureLoad(t_ge, ip, 0).a > 0.95) {
            let e = textureLoad(t_gn, ip, 0).b * 510.0;
            a *= 1.0 - smoothstep(v.prm.x + 1.0, v.prm.x + 7.0, v.world.y + e) * smoothstep(2.0, 6.0, e);
            if (a < 0.004) { discard; }
        }
        return vec4<f32>(c.rgb * v.col.rgb * v.prm.y * a, a);
    }
    if (mode == 11u) {
        // Silhouette der Spielfigur hinter Verdeckern (Bäume, Mauern, Dächer): nur wo etwas
        // Aufrechtes vor ihrem Fuß (prm.x) steht, als kühler, halb durchsichtiger Umriss
        let c = textureSampleLevel(t_acol, s_near, v.uv, i32(v.ext.x), 0.0);
        if (c.a < 0.45) { discard; }
        let ip = vec2<i32>(floor(v.clip.xy));
        let ek = textureLoad(t_ge, ip, 0).a;
        if (ek < 0.95 || abs(ek - 0.96) < 0.01) { discard; }
        let e = textureLoad(t_gn, ip, 0).b * 510.0;
        let occ = smoothstep(v.prm.x + 3.0, v.prm.x + 10.0, v.world.y + e) * smoothstep(4.0, 9.0, e);
        if (occ < 0.05) { discard; }
        // Rand heller als die Fläche (Lesbarkeit), sanft pulsierend
        let ts = 1.0 / vec2<f32>(textureDimensions(t_acol).xy);
        let l = i32(v.ext.x);
        let nb = textureSampleLevel(t_acol, s_near, v.uv + vec2<f32>(ts.x, 0.0), l, 0.0).a
            * textureSampleLevel(t_acol, s_near, v.uv - vec2<f32>(ts.x, 0.0), l, 0.0).a
            * textureSampleLevel(t_acol, s_near, v.uv + vec2<f32>(0.0, ts.y), l, 0.0).a
            * textureSampleLevel(t_acol, s_near, v.uv - vec2<f32>(0.0, ts.y), l, 0.0).a;
        let rim = 1.0 - step(0.45, nb);
        let a = (0.22 + rim * 0.5) * occ * v.col.a * (0.85 + 0.15 * sin(G.screen.z * 3.0));
        return vec4<f32>(v.col.rgb * a, a);
    }
    let c = textureSampleLevel(t_acol, s_near, v.uv, i32(v.ext.x), 0.0);
    if (c.a < 0.45) { discard; }
    let a = v.col.a;
    var rgb = c.rgb * v.col.rgb;
    if ((v.ext.w & 4u) != 0u) { rgb *= 2.0; }
    return vec4<f32>(rgb * a, a);
}

// ---------------------------------------------------------------- UI (Bildschirm-Pixel)
@vertex
fn vs_ui(v: VIn) -> VOut {
    var o: VOut;
    let sc = G.screen.w;
    let p = v.pos * sc;
    o.clip = vec4<f32>(p.x / G.screen.x * 2.0 - 1.0, 1.0 - p.y / G.screen.y * 2.0, 0.0, 1.0);
    o.uv = v.uv;
    o.col = vec4<f32>(pow(v.col.rgb, vec3<f32>(2.2)), v.col.a);
    o.ext = v.ext;
    o.prm = v.prm;
    o.world = v.pos;
    return o;
}

@fragment
fn fs_ui(v: VOut) -> @location(0) vec4<f32> {
    let mode = v.ext.y;
    if (mode == 2u) {
        return vec4<f32>(v.col.rgb * v.col.a, v.col.a);
    }
    if (mode == 6u) {
        let d = length(v.uv * 2.0 - 1.0);
        let a = (1.0 - smoothstep(0.2, 1.0, d)) * v.col.a;
        return vec4<f32>(v.col.rgb * a, a);
    }
    if (mode == 8u) {
        // Zierschrift: eigene bilineare Mischung (WebGL erlaubt nur einen Sampler je Textur),
        // vormultipliziert gemischt, damit die Kanten nicht dunkel ausfransen
        let dim = vec2<f32>(textureDimensions(t_acol).xy);
        let px = v.uv * dim - 0.5;
        let fr = fract(px);
        let b = (floor(px) + 0.5) / dim;
        let d = 1.0 / dim;
        let l = i32(v.ext.x);
        let c00 = textureSampleLevel(t_acol, s_near, b, l, 0.0);
        let c10 = textureSampleLevel(t_acol, s_near, b + vec2<f32>(d.x, 0.0), l, 0.0);
        let c01 = textureSampleLevel(t_acol, s_near, b + vec2<f32>(0.0, d.y), l, 0.0);
        let c11 = textureSampleLevel(t_acol, s_near, b + d, l, 0.0);
        let p00 = vec4<f32>(c00.rgb * c00.a, c00.a);
        let p10 = vec4<f32>(c10.rgb * c10.a, c10.a);
        let p01 = vec4<f32>(c01.rgb * c01.a, c01.a);
        let p11 = vec4<f32>(c11.rgb * c11.a, c11.a);
        let m = mix(mix(p00, p10, fr.x), mix(p01, p11, fr.x), fr.y);
        if (m.a < 0.01) { discard; }
        return vec4<f32>(m.rgb * v.col.rgb * v.col.a, m.a * v.col.a);
    }
    if (mode == 7u) {
        // Seekarte: Papierbild, leicht weich gefiltert
        let m = textureSampleLevel(t_map, s_lin, v.uv, 0.0);
        return vec4<f32>(m.rgb * v.col.rgb * v.col.a, v.col.a);
    }
    let c = textureSampleLevel(t_acol, s_near, v.uv, i32(v.ext.x), 0.0);
    if (c.a < 0.45) { discard; }
    var rgb = c.rgb * v.col.rgb;
    if ((v.ext.w & 16u) != 0u) {
        // Graustufen (gesperrt)
        rgb = vec3<f32>(luma(rgb) * 0.6);
    }
    return vec4<f32>(rgb * v.col.a, v.col.a);
}
```

### `post.wgsl`

```wgsl
// Licht (aufgeschobene Beleuchtung aus dem G-Puffer), Bloom und Endbild.

@group(1) @binding(0) var t_a: texture_2d<f32>;
@group(1) @binding(1) var t_b: texture_2d<f32>;
@group(1) @binding(2) var t_c: texture_2d<f32>;
@group(1) @binding(3) var t_d: texture_2d<f32>;

struct FOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_full(@builtin(vertex_index) i: u32) -> FOut {
    var o: FOut;
    let x = f32((i << 1u) & 2u);
    let y = f32(i & 2u);
    o.clip = vec4<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
    o.uv = vec2<f32>(x, y);
    return o;
}

fn gpos(uv: vec2<f32>) -> vec2<f32> {
    return G.view.xy + uv * G.view.zw;
}

fn t_now() -> f32 {
    return G.screen.z;
}

// ---------------------------------------------------------------- Beleuchtung
// Vorberechnete Spirale für die Umgebungsverdeckung (spart 12 sin/cos-Aufrufe pro Pixel):
// xy = Richtung * Radius, z = Radius
const AO_KERNEL: array<vec3<f32>, 12> = array<vec3<f32>, 12>(
    vec3<f32>(2.0000, 0.0000, 2.0),
    vec3<f32>(-2.2858, 2.0940, 3.1),
    vec3<f32>(0.3672, -4.1839, 4.2),
    vec3<f32>(3.2247, 4.2061, 5.3),
    vec3<f32>(-6.3022, -1.1148, 6.4),
    vec3<f32>(6.3296, -4.0231, 7.5),
    vec3<f32>(-2.2326, 8.3051, 8.6),
    vec3<f32>(-4.4705, -8.6084, 9.7),
    vec3<f32>(10.1423, 3.7114, 10.8),
    vec3<f32>(-11.0142, 4.5053, 11.9),
    vec3<f32>(5.5105, -11.7743, 13.0),
    vec3<f32>(4.1678, 13.4699, 14.1),
);

// Physikalisch motivierter Glanz (GGX-Mikrofacetten + Schlick-Fresnel)
fn ggx_spec(n: vec3<f32>, v: vec3<f32>, l: vec3<f32>, gloss: f32) -> f32 {
    let h = normalize(l + v);
    let ndh = max(dot(n, h), 0.0);
    let ndl = max(dot(n, l), 0.0);
    let vdh = max(dot(v, h), 0.0);
    let rough = clamp(1.0 - gloss * 0.86, 0.09, 0.92);
    let a = rough * rough;
    let a2 = a * a;
    let d_denom = ndh * ndh * (a2 - 1.0) + 1.0;
    let d = a2 / max(3.14159 * d_denom * d_denom, 0.0001);
    let f0 = mix(0.04, 0.38, gloss * gloss);
    let fres = f0 + (1.0 - f0) * pow(1.0 - vdh, 5.0);
    return d * fres * ndl * gloss * 1.35;
}

// Rückstrahlung und leuchtende Flächen (Bildschirm-GI): Spirale mit dem goldenen Winkel,
// xy = Richtung * Radius (Low-Res-Pixel), z = Gewicht
const GI_KERNEL: array<vec3<f32>, 8> = array<vec3<f32>, 8>(
    vec3<f32>(4.00, 0.00, 0.98),
    vec3<f32>(-5.86, 5.37, 0.93),
    vec3<f32>(1.21, -13.75, 0.83),
    vec3<f32>(13.11, 17.10, 0.66),
    vec3<f32>(-30.72, -5.43, 0.48),
    vec3<f32>(36.07, -22.95, 0.33),
    vec3<f32>(-14.59, 54.27, 0.22),
    vec3<f32>(-32.98, -63.50, 0.15),
);

// Regenringe: je Zelle ein Tropfen, der sich ausbreitet und verblasst (0..1)
fn rain_ring(p: vec2<f32>, cs: f32, t: f32) -> f32 {
    let cell = floor(p / cs);
    let h = hash21(cell);
    let ph = fract(t * 1.3 + h * 7.0);
    let ctr = (cell + 0.5 + (vec2<f32>(hash21(cell + 1.7), hash21(cell + 4.3)) - 0.5) * 0.5) * cs;
    let rd = length(p - ctr);
    return (1.0 - smoothstep(0.0, 0.85, abs(rd - ph * cs * 0.42))) * (1.0 - ph) * step(0.35, h);
}

// t_a = Farbe, t_b = Normale (xy, Höhe/510, Glanz), t_c = Leuchten (Alpha: Art des Pixels)
@fragment
fn fs_light(f: FOut) -> @location(0) vec4<f32> {
    let dim = vec2<f32>(textureDimensions(t_a));
    let ip = vec2<i32>(f.uv * dim);
    var alb = textureLoad(t_a, ip, 0).rgb;
    let nr = textureLoad(t_b, ip, 0);
    let emt = textureLoad(t_c, ip, 0);
    let em = emt.rgb;
    // Art des Pixels: 0,5 Wasser, 0,875 Boden, 0,96 Spielfigur, 0,98 Laub, 1 übrige Sprites
    let kind = emt.a;
    let wet = abs(kind - 0.5) < 0.2;
    let hero = abs(kind - 0.96) < 0.009;
    let nxy = nr.rg * 2.0 - 1.0;
    let n = vec3<f32>(nxy, sqrt(max(0.0, 1.0 - dot(nxy, nxy))));
    let elev = nr.b * 510.0;
    var spec_k = nr.a;
    let sp = gpos((vec2<f32>(ip) + 0.5) / dim);
    let p3 = vec3<f32>(sp.x, sp.y + elev, elev);
    // interne Auflösungsstufe: Höhen (Welt-px) in Zielpixel umrechnen
    let rs = max(G.hd3.y, 1.0);
    let t = G.screen.z;
    let hd = G.hd3.x > 0.5;

    // Bodenzeile (0..1, fern = oben): wo dieser Pixel auf dem Boden steht
    let ground = (f32(ip.y) + 0.5 + elev * rs) / dim.y;

    // Wetter auf den Flächen: Nässe dunkelt und lässt glänzen, Pfützen auf ebenem Boden;
    // Schnee und Asche legen sich auf nach oben weisende Flächen (vor allem auf Requisiten)
    var puddle = 0.0;
    if (!wet) {
        if (G.fx2.x > 0.01) {
            let wk = G.fx2.x;
            alb *= 1.0 - 0.26 * wk;
            spec_k = mix(spec_k, max(spec_k, 0.5), wk * 0.75);
            if (kind > 0.86 && kind < 0.89 && n.z > 0.94) {
                let pn = textureSampleLevel(t_d, s_lin, fract(sp * 0.0021 + vec2<f32>(0.31, 0.77)), 0.0).r * 0.65
                    + textureSampleLevel(t_d, s_lin, fract(sp * 0.0087 + vec2<f32>(0.13, 0.41)), 0.0).b * 0.35;
                puddle = smoothstep(0.6, 0.655, pn) * wk;
            }
        }
        if (G.fx2.y > 0.01 || G.fx2.z > 0.01) {
            let up = smoothstep(0.8, 0.97, n.z) * select(0.35, 1.0, kind > 0.95);
            let sn = textureSampleLevel(t_d, s_lin, fract(sp * 0.0131 + vec2<f32>(0.21, 0.64)), 0.0).b;
            let ks = G.fx2.y * up * smoothstep(0.42, 0.6, sn + G.fx2.y * 0.25);
            alb = mix(alb, vec3<f32>(0.78, 0.83, 0.9), ks * 0.72);
            spec_k = max(spec_k, ks * 0.32);
            let ka = G.fx2.z * up * smoothstep(0.45, 0.62, sn + G.fx2.z * 0.2);
            alb = mix(alb, vec3<f32>(0.15, 0.14, 0.14), ka * 0.5);
        }
    }

    // Umgebungsverdeckung aus dem Höhenpuffer: Boden am Fuß von Wänden, Klippen und Stämmen
    // sowie Nischen in Klippen und Mauern werden plastisch abgedunkelt.
    var ao = 1.0;
    let e_up = textureLoad(t_b, vec2<i32>(ip.x, max(ip.y - i32(2.0 * rs), 0)), 0).b * 510.0;
    let flat = mix(0.38, 1.0, 1.0 - smoothstep(0.8, 2.2, abs(e_up - elev)));
    if (G.hd2.x > 0.0) {
        var occ = 0.0;
        let g1 = f32(ip.y) / rs + elev;
        for (var k = 0; k < 12; k = k + 1) {
            let ks = AO_KERNEL[k];
            let rad = ks.z;
            let q = vec2<i32>(clamp(vec2<f32>(ip) + ks.xy * rs, vec2<f32>(0.0), dim - 1.0));
            let e2 = textureLoad(t_b, q, 0).b * 510.0;
            let dh = e2 - elev;
            let g2 = f32(q.y) / rs + e2;
            if (dh > 1.5 && abs(g2 - g1) < 7.0 + rad) {
                occ += clamp(dh / (rad * 1.9 + 5.0), 0.0, 1.0);
            }
        }
        let n_crevice = (1.0 - n.z) * 0.14;
        ao = 1.0 - clamp((occ / 12.0 * 1.65 * flat) + n_crevice, 0.0, 0.65) * G.hd2.x;
    }

    // Rückstrahlung (farbiges Streulicht der Umgebung) und leuchtende Flächen (Lava, Glut, Pilze,
    // Runen) als Lichtquellen: Spirale je Pixel gedreht (geordnetes Muster, flimmert nicht)
    var bounce = vec3<f32>(0.0);
    var emis = vec3<f32>(0.0);
    if (hd && G.hd2.x > 0.0) {
        let bay = f32((ip.x & 3) * 4 + ((ip.y + (ip.x >> 2u)) & 3)) * 0.3927;
        let cs = cos(bay);
        let sn = sin(bay);
        var wsum = 0.0;
        for (var k = 0; k < 8; k = k + 1) {
            let kk = GI_KERNEL[k];
            let o = vec2<f32>(kk.x * cs - kk.y * sn, kk.x * sn + kk.y * cs) * rs;
            let q = vec2<i32>(clamp(vec2<f32>(ip) + o, vec2<f32>(0.0), dim - 1.0));
            let qe = textureLoad(t_c, q, 0);
            bounce += textureLoad(t_a, q, 0).rgb * kk.z;
            // Wasser spiegelt nur den Himmel: keine Lichtquelle
            emis += qe.rgb * kk.z * step(0.7, qe.a);
            wsum += kk.z;
        }
        bounce /= wsum;
        emis /= wsum;
    }

    // Wolkenschatten: große, weiche Flecken ziehen langsam über Land und See (am Bodenort des
    // Pixels, damit Bäume und Wände mit ihrem Fuß im selben Schatten stehen)
    var cloud = 1.0;
    if (G.hd2.x > 0.0) {
        let wq = vec2<f32>(sp.x, sp.y + elev) * 0.00042 + vec2<f32>(t_now() * 0.0062, t_now() * 0.0024);
        // zurückgefaltet: große Weltkoordinaten (siehe noise_at)
        let c1 = textureSampleLevel(t_d, s_lin, fract(wq), 0.0).r;
        let c2 = textureSampleLevel(t_d, s_lin, fract(wq * 2.7 + vec2<f32>(0.37, 0.11)), 0.0).b;
        // Bedeckung: bei aufziehender Front mehr und dunklere Wolken
        let cov = G.fx6.x;
        cloud = 1.0 - smoothstep(0.62 - cov * 0.3, 0.86 - cov * 0.3, c1 * 0.72 + c2 * 0.34) * (0.28 + cov * 0.3);
    }
    // Umgebung: Himmel von oben, warmer Bodenrückwurf von unten
    let hemi = mix(0.72, 1.04, n.z * G.ambient.w + (1.0 - G.ambient.w));
    var li = G.ambient.rgb * hemi * ao * mix(1.0, cloud, 0.35);
    li += bounce * (G.ambient.rgb * 0.55 + G.sun_col.rgb * 0.3 * cloud) * G.fx4.x * ao;
    li += emis * G.fx4.y;
    // Randlicht: erhöhte Kanten (Figuren, Kronen, Klippenkanten) fangen kühles Himmelslicht von hinten
    if (elev > 3.0) {
        let edge = pow(clamp(1.0 - n.z, 0.0, 1.0), 1.85);
        let back = clamp(-n.y * 0.82 + abs(n.x) * 0.38, 0.0, 1.0);
        li += (G.ambient.rgb * 1.65 + G.sun_col.rgb * 0.55) * edge * back * G.hd2.z;
    }
    var spec = vec3<f32>(0.0);
    let view_dir = normalize(vec3<f32>(0.0, 0.55, 0.85));

    // Sonne/Mond mit kontakthärtenden Bildschirm-Schatten (Höhenfeld abtasten)
    let sd = normalize(G.sun_dir.xyz);
    let raw_nd = dot(n, sd);
    let nd = select(max(raw_nd, 0.0), max((raw_nd + 0.15) / 1.15, 0.0), hd);
    var sh = 1.0;
    if (G.sun_dir.w > 0.0 && nd > 0.0) {
        var occl = 0.0;
        var canopy = false;
        // versetzter Start je Pixel (kein Streifenmuster), Schatten nahe am Werfer scharf, fern weicher
        let jit = hash21(vec2<f32>(ip) * 0.73 + vec2<f32>(3.1, 7.7));
        for (var k = 1; k <= 15; k = k + 1) {
            let tq = (f32(k) - jit) * 4.4;
            let q = p3 + sd * tq;
            let sp2 = vec2<f32>(q.x, q.y - q.z);
            let suv = (sp2 - G.view.xy) / G.view.zw;
            if (suv.x < 0.0 || suv.y < 0.0 || suv.x > 1.0 || suv.y > 1.0) { break; }
            let qi = vec2<i32>(suv * dim);
            let e2 = textureLoad(t_b, qi, 0).b * 510.0;
            // Verdecker: höher als der Strahl und am selben Bodenort
            let ground2 = sp2.y + e2;
            let over = e2 - (q.z + 2.5);
            if (over > 0.0 && abs(ground2 - q.y) < 23.0) {
                let contact = clamp(over * 0.25, 0.35, 1.0);
                let dist_fade = 1.0 - f32(k) / 18.5;
                if (dist_fade * contact > occl) {
                    occl = dist_fade * contact;
                    // Laubkronen lassen Licht in Flecken durch
                    canopy = e2 > 22.0 && abs(textureLoad(t_c, qi, 0).a - 0.98) < 0.009;
                }
                if (occl > 0.78) { break; }
            }
        }
        if (canopy) {
            let dq = sp * (1.0 / 96.0) + vec2<f32>(sin(t * 0.37) * 0.012 + t * 0.0016, cos(t * 0.29) * 0.01);
            let dap = smoothstep(0.6, 0.68, textureSampleLevel(t_d, s_lin, fract(dq), 0.0).b);
            occl *= 1.0 - dap * 0.82;
        }
        sh = 1.0 - occl * G.sun_dir.w;
    }
    // Warme Streuung an der Licht-Schatten-Grenze (Subsurface-Terminator für Laub, Segel, Haut)
    let term = select(0.0, smoothstep(0.0, 0.32, nd) * (1.0 - smoothstep(0.15, 0.65, max(raw_nd, 0.0))) * 0.14, hd && !wet);
    li += (G.sun_col.rgb * nd + G.sun_col.rgb * vec3<f32>(1.15, 0.85, 0.65) * term) * sh * (0.48 + 0.52 * ao) * cloud;
    let hv = normalize(sd + view_dir);
    // Glanz: glatte Flächen (Wasser, Metall, nasser Fels) mit physikalisch motivierten Spitzlichtern
    let sexp = 40.0;
    if (wet) {
        // Wasser: örtlicher Sonnen-/Mondpfad mit GGX-Schimmer und funkelnden Glitzerpunkten
        let cen = G.view.xy + G.view.zw * 0.5;
        let cd = G.view.w * 2.5;
        let vw = normalize(vec3<f32>(cen.x, cen.y + cd * 0.543, cd * 0.84) - p3);
        let ndh = max(dot(n, normalize(sd + vw)), 0.0);
        let lobe = pow(ndh, 64.0) * 0.34 + pow(ndh, 16.0) * 0.06;
        let gq = vec2<i32>(select(floor(sp), floor(sp * 2.0), hd));
        let tk = i32(t_now() * 7.0);
        let r = textureLoad(t_d, vec2<i32>((gq.x + tk * 37) & 255, (gq.y + tk * 59) & 255), 0).a;
        let spark = step(1.0 - pow(ndh, 280.0) * 0.038, r) * select(2.2, 3.4, hd);
        spec += G.sun_col.rgb * (lobe + spark) * (spec_k * 1.05) * sh * G.sun_col.w * cloud * cloud;
        // Sterne spiegeln sich in der nächtlichen See (von den Wellen verschoben, funkelnd)
        if (G.fx3.w > 0.01) {
            let sq = floor((sp + n.xy * 7.0) / 3.0);
            let hs = hash21(sq + vec2<f32>(0.37, 0.91));
            let tw = 0.55 + 0.45 * sin(t * (1.5 + hs * 3.0) + hs * 40.0);
            spec += vec3<f32>(0.72, 0.8, 1.0) * step(0.9968, hs) * tw * G.fx3.w * cloud * 1.7 * spec_k;
        }
        // Regen: Ringe auf der Oberfläche
        if (G.fx2.x > 0.01) {
            spec += (G.ambient.rgb * 1.6 + vec3<f32>(0.1)) * rain_ring(sp, 11.0, t) * G.fx2.x * 0.9;
        }
    } else if (hd) {
        spec += G.sun_col.rgb * ggx_spec(n, view_dir, sd, spec_k) * sh * G.sun_col.w * cloud * cloud;
    } else {
        spec += G.sun_col.rgb * pow(max(dot(n, hv), 0.0), sexp) * spec_k * sh * G.sun_col.w * cloud * cloud;
    }

    // Punktlichter in 3D + Streulicht-Hof im Dunst in einer gemeinsamen Schleife
    let nl = i32(G.counts.x);
    var shadows = 0;
    var halo = vec3<f32>(0.0);
    var rim_l = vec3<f32>(0.0);
    for (var i = 0; i < nl; i = i + 1) {
        let L = G.lights[i];
        let fi = f32(i);
        // flackernde Flammen bewegen ihr Licht: die Schatten tanzen mit
        let jl = vec2<f32>(sin(t * 13.0 + fi * 3.1), cos(t * 10.7 + fi * 1.7)) * L.col.w * 2.4;
        let lpos = vec3<f32>(L.pos.x + jl.x, L.pos.y + jl.y * 0.5, L.pos.z);
        if (hd) {
            let lp = vec2<f32>(lpos.x, lpos.y - lpos.z);
            let dp2 = sp - lp;
            let r_inv = 1.0 / max(L.pos.w, 1.0);
            let dd2 = dot(dp2, dp2) * (r_inv * r_inv);
            if (dd2 < 1.0) {
                let fl2 = 1.0 + sin(t * 9.0 + fi * 2.1) * 0.5 * L.col.w;
                // Lichtkegel im Dunst: Strahlen, die vom Licht ausgehen
                let ang = atan2(dp2.y, dp2.x) * 0.15915;
                let ray = textureSampleLevel(t_d, s_lin, fract(vec2<f32>(ang + fi * 0.37, fi * 0.13 + t * 0.004)), 0.0).r;
                halo += L.col.rgb * exp(-sqrt(dd2) * 5.0) * fl2 * (0.7 + 0.7 * smoothstep(0.48, 0.74, ray));
            }
        }
        let d3 = lpos - p3;
        let d2 = dot(d3, d3);
        let r2 = L.pos.w * L.pos.w;
        if (d2 > r2) { continue; }
        let dist = sqrt(d2);
        let ld = d3 / max(dist, 0.001);
        let ndl = max(dot(n, ld), 0.0) * 0.85 + 0.15;
        var att = 1.0 - dist / L.pos.w;
        att = att * att;
        var lsh = 1.0;
        if (hd) {
            // natürlicher Abfall: hell am Licht, langer weicher Ausläufer
            let x2 = d2 / max(r2, 0.001);
            let wnd = 1.0 - x2;
            att = wnd * wnd / (1.0 + 6.5 * x2);
            // Schlagschatten der nahen Lichter: Strahl zum Licht durch das Höhenfeld, mit weichem
            // Halbschatten (Tiefe des Verdeckers) statt harter Kante
            if (shadows < 12 && att > 0.04 && dist > 10.0) {
                shadows = shadows + 1;
                let steps = select(10, 7, shadows > 6);
                let jit2 = hash21(vec2<f32>(ip) * 1.31 + vec2<f32>(fi, 2.3));
                for (var k = 1; k <= steps; k = k + 1) {
                    let s2 = (f32(k) - jit2) / (f32(steps) + 0.5);
                    let q = p3 + d3 * s2;
                    if (q.z > lpos.z - 4.0) { break; }
                    let sp2 = vec2<f32>(q.x, q.y - q.z);
                    let suv = (sp2 - G.view.xy) / G.view.zw;
                    if (suv.x < 0.0 || suv.y < 0.0 || suv.x > 1.0 || suv.y > 1.0) { break; }
                    let e2 = textureLoad(t_b, vec2<i32>(suv * dim), 0).b * 510.0;
                    if (e2 > q.z + 2.0 && abs(sp2.y + e2 - q.y) < 16.0) {
                        let depth = smoothstep(2.0, 11.0, e2 - q.z);
                        lsh = min(lsh, mix(1.0, 0.2 + 0.8 * s2 * s2, depth));
                        if (lsh < 0.3) { break; }
                    }
                }
            }
            // farbiges Randlicht: Kanten, die zum Licht weisen, leuchten auf (Laternen hinter Figuren)
            if (elev > 3.0) {
                let edge = pow(clamp(1.0 - n.z, 0.0, 1.0), 1.6);
                let facing = max(dot(normalize(n.xy + vec2<f32>(0.0001, 0.0)), normalize(d3.xy + vec2<f32>(0.0001, 0.0))), 0.0);
                rim_l += L.col.rgb * edge * facing * att * lsh * (0.6 + 0.4 * smoothstep(0.0, 0.6, -ld.y));
            }
        }
        let fl = 1.0 + (sin(t * 11.0 + fi * 1.7) * 0.5 + sin(t * 17.0 + fi) * 0.5) * L.col.w;
        if (wet && hd && lpos.z > 12.0) {
            // Lichtpfad: die Laterne spiegelt sich als zitternder Streifen unter sich im Wasser
            let ry = lpos.y + lpos.z * 0.35;
            let dx = sp.x - lpos.x + n.x * 9.0;
            let dy = (sp.y - ry) / (lpos.z * 0.9 + 8.0);
            let streak = exp(-dx * dx / 40.0) * exp(-dy * dy * 2.2) * (0.6 + 0.4 * sin(sp.y * 0.9 + t * 4.0 + n.y * 6.0));
            spec += L.col.rgb * streak * 0.55 * fl;
        }
        li += L.col.rgb * ndl * att * fl * (0.58 + 0.42 * ao) * lsh;
        if (hd) {
            spec += L.col.rgb * ggx_spec(n, view_dir, ld, spec_k) * att * fl * lsh * 0.65;
        } else {
            let h2 = normalize(ld + view_dir);
            spec += L.col.rgb * pow(max(dot(n, h2), 0.0), 24.0) * spec_k * att * lsh;
        }
    }
    li += rim_l * G.fx4.z;
    // Lesbarkeit: die Spielfigur behält im Dunkeln eine Mindesthelligkeit und einen kühlen Rand
    if (hero) {
        let edge = pow(clamp(1.0 - n.z, 0.0, 1.0), 1.4);
        li += (G.ambient.rgb * 0.9 + vec3<f32>(0.05, 0.06, 0.08)) * edge * G.fx4.w;
        li = max(li, vec3<f32>(0.1, 0.11, 0.14) * G.fx4.w);
    }
    if (!hd) {
        // weiche Stufen wie in Pixelkunst (leicht gedithert)
        let dith = (hash21(vec2<f32>(ip)) - 0.5) / 24.0;
        li = floor(li * 12.0 + dith * 12.0 + 0.5) / 12.0;
    }
    var c = alb * li + spec + em;
    if (hd) {
        c += halo * (0.045 + G.fog.w * 0.32) * (1.0 + G.fx3.y * 0.7);
    }
    // Pfützen: spiegeln den Himmel, Regenringe darauf
    if (puddle > 0.0) {
        let sky = G.fog.rgb * 0.8 + G.ambient.rgb * 0.55;
        c = mix(c, sky * (0.62 + rain_ring(sp, 8.0, t) * 1.2) + spec * 1.4, puddle * 0.55);
    }
    // Blitz (Gewitter): hartes Licht von schräg oben mit langen, scharfen Schatten
    if (G.counts.y > 0.01) {
        let ldl = normalize(vec3<f32>(0.42, -0.55, 0.72));
        var occ2 = 0.0;
        for (var k = 1; k <= 10; k = k + 1) {
            let q = p3 + ldl * f32(k) * 6.5;
            let sp2 = vec2<f32>(q.x, q.y - q.z);
            let suv = (sp2 - G.view.xy) / G.view.zw;
            if (suv.x < 0.0 || suv.y < 0.0 || suv.x > 1.0 || suv.y > 1.0) { break; }
            let e2 = textureLoad(t_b, vec2<i32>(suv * dim), 0).b * 510.0;
            if (e2 > q.z + 2.5 && abs(sp2.y + e2 - q.y) < 23.0) { occ2 = 1.0; break; }
        }
        let ndl2 = max(dot(n, ldl), 0.0) * 0.8 + 0.2;
        c += alb * G.counts.y * vec3<f32>(0.8, 0.85, 1.0) * ndl2 * (1.0 - occ2 * 0.88) * 1.6;
    }
    // Tiefennebel nach Bodenzeile (fern = oben), Ferne zusätzlich in Nebelfarbe getaucht
    let fogk = G.fog.w * (1.0 - ground * 0.6) + G.hd.y * 0.03 * (1.0 - smoothstep(0.0, 0.4, ground));
    c = mix(c, G.fog.rgb, clamp(fogk, 0.0, 0.85));
    // Alpha trägt die Bodenzeile für die Tiefenschärfe im Endbild
    return vec4<f32>(c * G.misc.w, clamp(ground * 0.5, 0.0, 1.0));
}

// ---------------------------------------------------------------- Bloom (breiter 9-Tap-Gaußkern)
@fragment
fn fs_bright(f: FOut) -> @location(0) vec4<f32> {
    let c = textureSampleLevel(t_a, s_lin, f.uv, 0.0).rgb;
    let l = max(c.r, max(c.g, c.b));
    let k = smoothstep(0.64, 1.5, l);
    return vec4<f32>(c * k, 1.0);
}

fn bloom_blur(uv: vec2<f32>, step: vec2<f32>) -> vec4<f32> {
    var c = textureSampleLevel(t_a, s_lin, uv, 0.0).rgb * 0.196;
    let o1 = step * 1.485;
    let o2 = step * 3.465;
    let o3 = step * 5.445;
    let o4 = step * 7.425;
    c += (textureSampleLevel(t_a, s_lin, uv + o1, 0.0).rgb + textureSampleLevel(t_a, s_lin, uv - o1, 0.0).rgb) * 0.244;
    c += (textureSampleLevel(t_a, s_lin, uv + o2, 0.0).rgb + textureSampleLevel(t_a, s_lin, uv - o2, 0.0).rgb) * 0.108;
    c += (textureSampleLevel(t_a, s_lin, uv + o3, 0.0).rgb + textureSampleLevel(t_a, s_lin, uv - o3, 0.0).rgb) * 0.038;
    c += (textureSampleLevel(t_a, s_lin, uv + o4, 0.0).rgb + textureSampleLevel(t_a, s_lin, uv - o4, 0.0).rgb) * 0.012;
    return vec4<f32>(c, 1.0);
}

@fragment
fn fs_blur_h(f: FOut) -> @location(0) vec4<f32> {
    let ts = 1.0 / vec2<f32>(textureDimensions(t_a));
    return bloom_blur(f.uv, vec2<f32>(ts.x, 0.0));
}
@fragment
fn fs_blur_v(f: FOut) -> @location(0) vec4<f32> {
    let ts = 1.0 / vec2<f32>(textureDimensions(t_a));
    return bloom_blur(f.uv, vec2<f32>(0.0, ts.y));
}

// ---------------------------------------------------------------- Tiefenschärfe (halbe Größe, breiter Kern)
// Waagrecht: volles Bild → halbe Breite; senkrecht: → halbe Höhe.
// Lichter werden begrenzt, damit Funken und Laternen nicht zu milchigen Flecken verschmieren.
fn dof_tap(uv: vec2<f32>) -> vec3<f32> {
    return min(textureSampleLevel(t_a, s_lin, uv, 0.0).rgb, vec3<f32>(1.1));
}
fn dof_blur(uv: vec2<f32>, dir: vec2<f32>) -> vec4<f32> {
    var c = dof_tap(uv) * 0.2;
    var w = array<f32, 4>(0.17, 0.12, 0.07, 0.035);
    for (var k = 0; k < 4; k = k + 1) {
        let o = dir * (f32(k) + 1.0);
        c += dof_tap(uv + o) * w[k];
        c += dof_tap(uv - o) * w[k];
    }
    return vec4<f32>(c / 0.99, 1.0);
}
@fragment
fn fs_dof_h(f: FOut) -> @location(0) vec4<f32> {
    // liest das volle Bild: 3 Texel Abstand
    let ts = 1.0 / vec2<f32>(textureDimensions(t_a));
    return dof_blur(f.uv, vec2<f32>(ts.x * 3.0, 0.0));
}
@fragment
fn fs_dof_v(f: FOut) -> @location(0) vec4<f32> {
    // liest halbe Breite, volle Höhe: 3 Texel Abstand
    let ts = 1.0 / vec2<f32>(textureDimensions(t_a));
    return dof_blur(f.uv, vec2<f32>(0.0, ts.y * 3.0));
}

// ---------------------------------------------------------------- Endbild
// t_a = beleuchtete Szene (Low-Res, Alpha = Bodenzeile), t_b = Bloom, t_c = Unschärfe, t_d = Rauschen
fn tonemap(c: vec3<f32>) -> vec3<f32> {
    // ACES-Kurve mit sanfter Entsättigung in den Spitzenlichtern (AgX-inspiriert)
    let lum = luma(c);
    let desat = smoothstep(0.85, 3.2, lum) * 0.42;
    let x = mix(c, vec3<f32>(lum), desat);
    let a = x * (2.51 * x + 0.03);
    let b = x * (2.43 * x + 0.59) + 0.14;
    return clamp(a / b, vec3<f32>(0.0), vec3<f32>(1.0));
}

// Lichtstrahlen durch Dunst: Bänder in Richtung der Sonne, an der Welt verankert (Stärke 0..1)
fn light_rays(suv: vec2<f32>, sy: f32) -> f32 {
    let wp = G.view.xy + suv * G.view.zw;
    let sd = G.sun_dir.xyz;
    let dir = normalize(vec2<f32>(sd.x, sd.y - sd.z) + vec2<f32>(0.0001, 0.0));
    let perp = vec2<f32>(-dir.y, dir.x);
    let across = dot(wp, perp);
    let along = dot(wp, dir);
    let t = G.screen.z;
    // zurückgefaltet (große Welt: sonst ungenaue Texturkoordinaten)
    let n1 = textureSampleLevel(t_d, s_lin, fract(vec2<f32>(across * 0.0035 + t * 0.004, 0.37)), 0.0).r;
    let n2 = textureSampleLevel(t_d, s_lin, fract(vec2<f32>(across * 0.009 - t * 0.006, along * 0.0015 + 0.71)), 0.0).r;
    let beam = smoothstep(0.56, 0.85, n1) * (0.4 + 0.6 * n2);
    let top = clamp(0.35 + (1.0 - sy) * 0.65, 0.0, 1.0);
    // Strahlen fallen durch Wolkenlücken (dasselbe Rauschen wie die Wolkenschatten im Licht-Pass)
    let wq = wp * 0.00042 + vec2<f32>(t * 0.0062, t * 0.0024);
    let c1 = textureSampleLevel(t_d, s_lin, fract(wq), 0.0).r;
    let c2 = textureSampleLevel(t_d, s_lin, fract(wq * 2.7 + vec2<f32>(0.37, 0.11)), 0.0).b;
    let cov = G.fx6.x;
    let gap = 1.0 - smoothstep(0.62 - cov * 0.3, 0.86 - cov * 0.3, c1 * 0.72 + c2 * 0.34);
    return beam * top * G.hd.w * (0.4 + 0.95 * gap) * (1.15 - cov * 0.6);
}

// schwebender Staub im Vordergrund als unscharfe Bokeh-Scheiben (außerhalb der Fokuszone)
fn bokeh_dust(uv: vec2<f32>) -> f32 {
    let sp = uv * G.screen.xy / G.screen.w;
    let pxk = G.screen.x / G.view.z / G.screen.w;
    let t = G.screen.z;
    var dust = 0.0;
    for (var l = 0; l < 2; l = l + 1) {
        let fl = f32(l);
        let sc = 52.0 * (1.0 + fl * 0.7);
        let drift = vec2<f32>(sin(t * 0.13 + fl) * 8.0 + t * (3.0 + fl * 2.0), -t * (5.0 + fl * 3.0));
        let q = sp + G.view.xy * pxk * (1.25 + fl * 0.35) + drift;
        let id = floor(q / sc);
        let h = hash21(id + fl * 17.0);
        if (h > 0.6) {
            let r = sc * (0.07 + 0.11 * hash21(id + 3.1));
            let jit = vec2<f32>(hash21(id + 5.7), hash21(id + 9.3)) - 0.5;
            let ctr = (id + 0.5 + jit * (1.0 - 2.0 * r / sc)) * sc;
            let dd = length(q - ctr) / r;
            let disc = (1.0 - smoothstep(0.75, 1.0, dd)) * (0.6 + 0.4 * smoothstep(0.4, 0.95, dd));
            dust += disc * (0.55 + 0.45 * sin(t * 0.8 + h * 30.0));
        }
    }
    return dust;
}

@fragment
fn fs_final(f: FOut) -> @location(0) vec4<f32> {
    // Zoom (Kamera rückt nah heran): um die Bildmitte vergrößern
    let zoom = max(G.fx3.z, 0.5);
    let fuv = (f.uv - 0.5) / zoom + 0.5;
    // Schrägsicht: Bildschirm → Low-Res-Bild
    var suv = tilt_src(fuv);
    let k = G.hd.x;
    let dk = 1.0 + k * fuv.y;
    let mag = vec2<f32>(dk, dk * dk / (1.0 + k)) * zoom;
    // Druckwelle: ein Ring verschiebt das Bild nach außen
    let wp = G.view.xy + suv * G.view.zw;
    if (G.fx1.w > 0.001) {
        let d = wp - G.fx1.xy;
        let dl = length(d) + 0.001;
        let x = (dl - G.fx1.z) / 16.0;
        suv -= d / dl * exp(-x * x) * G.fx1.w * 5.0 / G.view.zw;
    }
    // Hitzeflimmern: in heißen Gebieten überall leicht, über Feuer, Glut und Lava stark
    let blm0 = textureSampleLevel(t_b, s_lin, suv, 0.0).rgb;
    let heat = G.fx3.x + smoothstep(0.05, 0.4, luma(blm0)) * 0.9 * G.hd.y;
    if (heat > 0.002) {
        let tt = G.screen.z;
        let hn = textureSampleLevel(t_d, s_lin, fract(wp * vec2<f32>(0.011, 0.027) + vec2<f32>(0.0, tt * 0.31)), 0.0).r;
        let hn2 = textureSampleLevel(t_d, s_lin, fract(wp * vec2<f32>(0.019, 0.046) - vec2<f32>(tt * 0.04, tt * 0.47)), 0.0).b;
        suv += vec2<f32>((hn - 0.5 + (hn2 - 0.5) * 0.6) * 1.6 / G.view.z, (hn2 - 0.5) * 0.8 / G.view.w) * heat;
    }
    // scharf-bilineare Vergrößerung: Pixel bleiben scharf, Kanten ohne Flimmern
    let src = vec2<f32>(textureDimensions(t_a));
    let px = suv * src;
    let scale = G.screen.xy / src * mag;
    let fl = floor(px - 0.5) + 0.5;
    let fr = px - 0.5 - floor(px - 0.5);
    let w = clamp((fr - 0.5) * scale + 0.5, vec2<f32>(0.0), vec2<f32>(1.0));
    let uv = (fl + w) / src;
    let s0 = textureSampleLevel(t_a, s_lin, uv, 0.0);
    var c = s0.rgb;
    // leichter Farbsaum zum Rand hin (Linse)
    if (G.hd2.w > 0.0) {
        let dv = f.uv - 0.5;
        let off = dv * dot(dv, dv) * 0.01 * G.hd2.w;
        c.r = textureSampleLevel(t_a, s_lin, uv + off, 0.0).r;
        c.b = textureSampleLevel(t_a, s_lin, uv - off, 0.0).b;
    }
    // Tiefenschärfe (Tilt-Shift): nach Bodenzeile, Ferne früher unscharf als die Nähe
    if (G.hd.y > 0.0) {
        let fd = s0.a * 2.0 - G.hd.z;
        let coc = select(smoothstep(0.22, 0.5, fd), smoothstep(0.2, 0.55, -fd), fd < 0.0) * G.hd.y;
        if (coc > 0.001) {
            let bl = textureSampleLevel(t_c, s_lin, suv, 0.0).rgb;
            c = mix(c, bl, coc * 0.8);
            // Diorama: unscharfe Ränder etwas dunkler und blasser, der Blick bleibt in der Fokuszone
            c *= 1.0 - coc * 0.16;
            c = mix(vec3<f32>(luma(c)), c, 1.0 - coc * 0.22);
        }
    }
    if (G.hd.w > 0.0) {
        // Strahlen hellen auf, was schon da ist (kein Grauschleier auf dunklem Wasser)
        let r = light_rays(suv, f.uv.y);
        c = c * (1.0 + r * 0.6 * (vec3<f32>(0.4) + G.sun_col.rgb)) + G.sun_col.rgb * r * 0.004;
    }
    if (G.hd2.y > 0.0) {
        let fv = G.hd.z / (1.0 + k - k * G.hd.z);
        let band = smoothstep(0.16, 0.42, abs(f.uv.y - fv));
        if (band > 0.001) {
            c += (G.sun_col.rgb * 0.7 + G.fog.rgb * 0.4 + vec3<f32>(0.02)) * bokeh_dust(f.uv) * band * G.hd2.y * 0.07;
        }
    }
    c += blm0 * G.misc.z;
    c = tonemap(c);
    // Farbkorrektur: Lift/Gain, Kontrast mit weichem Fuß (Schatten behalten Zeichnung statt ins
    // Schwarze zu laufen), Farbtrennung je Zone, Sättigung
    c = G.lift.rgb + c * (G.gain.rgb - G.lift.rgb);
    let cc = (c - 0.5) * G.lift.w + 0.5;
    c = max(cc, c * 0.42);
    let l0 = clamp(luma(c), 0.0, 1.0);
    c *= mix(G.tone_hi.rgb, G.tone_lo.rgb, pow(1.0 - l0, 1.6));
    let l = luma(c);
    c = mix(vec3<f32>(l), c, G.gain.w);
    // wenig Leben: die Welt verliert Farbe, am Rand pocht es rot im Herzschlag
    if (G.fx0.y > 0.001) {
        let lh = G.fx0.y;
        let ph = fract(G.screen.z * 1.15) * 8.0;
        let beat = exp(-(ph - 1.0) * (ph - 1.0) * 3.0) + 0.6 * exp(-(ph - 2.4) * (ph - 2.4) * 3.0);
        let dv = f.uv - 0.5;
        let edge = smoothstep(0.1, 0.42, dot(dv, dv) * 2.2);
        c = mix(c, vec3<f32>(luma(c)), lh * 0.5);
        c = mix(c, vec3<f32>(0.42, 0.02, 0.03), edge * lh * (0.3 + 0.38 * beat));
    }
    // Blenden (Tod, Treffer)
    c = mix(c, vec3<f32>(1.0), G.counts.z);
    c = mix(c, vec3<f32>(0.5, 0.02, 0.04), G.counts.w);
    // Aufprallbild: für einen Augenblick nur Licht und Schatten (harte Treffer, Paraden)
    if (G.fx0.x > 0.001) {
        let ink = select(vec3<f32>(0.02, 0.015, 0.03), vec3<f32>(1.0, 0.95, 0.86), luma(c) > 0.26);
        c = mix(c, ink, G.fx0.x);
    }
    // Vignette
    let d = f.uv - 0.5;
    c *= 1.0 - dot(d, d) * G.misc.x;
    // Körnung und feines Dither (keine Farbstufen in dunklen Verläufen)
    let fp = floor(f.uv * G.screen.xy);
    let gr = hash21(fp + fract(G.screen.z) * 91.0) - 0.5;
    c += gr * G.misc.y;
    c += (hash21(fp * 1.37 + vec2<f32>(17.0, 3.0)) + hash21(fp * 0.71 + vec2<f32>(5.0, 29.0)) - 1.0) / 255.0;
    // Übergang wie Tinte im Wasser: vom Rand her läuft Schwärze ins Bild
    if (G.fx0.z > 0.001) {
        let n1 = textureSampleLevel(t_d, s_lin, fract(f.uv * vec2<f32>(1.3, 0.8) + vec2<f32>(0.13, G.screen.z * 0.01)), 0.0).r;
        let n2 = textureSampleLevel(t_d, s_lin, fract(f.uv * vec2<f32>(3.7, 2.1) + vec2<f32>(0.57, 0.21)), 0.0).b;
        let m = n1 * 0.55 + n2 * 0.25 + dot(d, d) * 0.9;
        let th = 1.32 - G.fx0.z * 1.62;
        let ink = smoothstep(th - 0.05, th + 0.03, m);
        let edge = 1.0 - smoothstep(0.0, 0.04, abs(m - th));
        c = mix(c * (1.0 - edge * 0.35), vec3<f32>(0.008, 0.01, 0.018), ink);
    }
    // Kinobalken (Zwischensequenzen, Bossauftritte)
    let lb = G.fx0.w * 0.105;
    if (f.uv.y < lb || f.uv.y > 1.0 - lb) {
        c = vec3<f32>(0.0);
    }
    return vec4<f32>(clamp(c, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
```
