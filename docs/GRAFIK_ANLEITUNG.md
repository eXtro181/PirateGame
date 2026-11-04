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

---

## 7. Figuren-Maler (Menschen und Humanoide)

`chars.rs`: Jede Figur ist ein **64 × 72-Bild** (× Maßstab s, Könige 1,5, Malakor größer). Die Figur ist ≈ 48 px hoch mit großem Kopf wie im Referenzstil. Der Fußpunkt liegt bei `h − 4·s`.

**Richtungen**: vorn (`down`), hinten (`up`), Seite (`side`, links = gespiegelt).
**Animationen** (Bilder je Richtung): idle 4, walk 6, atk 5, roll 5, hurt 1, dead 1 (nur vorn), cast 3, drink 3, sit 1, shoot 2. NPCs brauchen nur idle/walk/sit/dead, Gegner kein roll/drink/sit.

**Aussehen (`Look`)**: Statur, Haut (4 Töne: E0B896, C49674, 9A6A4C, 6A4430), Haar (Glatze/kurz/lang/Zopf/wild, 6 Farben), Bart, Kopfbedeckung (Dreispitz, Kopftuch, Kapuze, Kapitänshut), Mantel (6 Farben: 8A2A30, 2E4466, 4A5A36, 6A4A30, 2E2E36, 6A5A7E), Hose, Hemd, Augenklappe, Waffe. Dazu Sonderteile: Skelett, Lumpen, ertrunken, Robe, Schürze, leuchtende Kapuze, Laterne, Krone, Aura, Hörner, Schärpe, Fischkopf, Hauer, Totenkopf-Bemalung, Flügel (Federn oder Uhrwerk), Vogelbeine, Schweben, Schlangenleib, Schlangenhaar, Uhrwerk-Körper, Glockenrumpf, kreisende Schädel, vier Arme, Tentakelbart.

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

**Boden-Deko** (kleine Sprites ohne Kollision): Gras- und Schneebüschel, Blumen, Klee, kleiner Farn (schwingen im Wind), dazu flach liegend Steinchen, Steine, Zweige, Laub, Tang, Treibholz, Wurzeln, Muscheln, Knochen, Glut, Seerosen; Pilze, Leuchtkristalle, Laternenstängel. Sie entsteht **je 256-px-Eimer erst beim Zeichnen** (`World::deco_bucket`, Zufall je Kachel aus den Kachelkoordinaten, also jedes Mal gleich): Dichte `dens·4 + rnd·1,2` (bis 6 je Kachel), Arten aus der Tabelle des Bioms, auf Wegen meist nichts, auf Wiesen Blumen, am Sand Muscheln und Treibholz; auffällige Stücke selten (Schädel 25 %, Knochen 35 %, Laternen 50 %). Nur jeder zweite Laternenstängel und jede dritte Glut werfen Licht.

**Seeungeheuer-Teile** (klein, zur Laufzeit zusammengesetzt und je Seegebiet getönt): Schlangenkopf (zu/offen, Schuppen, Stachelkamm, Leuchtauge) und Körperbuckel (Halbkuppel mit Stacheln) – eine Seeschlange sind ein Kopf und vier Buckel auf einer Sinuswelle, jeder Buckel mit eigenem `cut_bottom` (taucht auf und ab). Riesenkalmar: Mantel mit Leuchtflecken und acht Fangarmen aus je zehn dicht gesetzten Tentakelgliedern (Bogen `sin(u·π)·58` aus dem Wasser). Qualle (zwei Pulse, leuchtend), Rochen von oben (drei Flügelschläge, Glutflecken, nach Kurs gedreht). Kanonen-Angler, Skelett-Kraken, Panzerkröte und Riesenhai sind die vorhandenen Kreaturen-Bögen, bis zur Hüfte im Wasser; die Wyvern fliegt mit Bodenschatten. Tönung: Glut (255, 168, 136), Frost (190, 224, 255), Sturm (196, 204, 222), Tiefe (160, 224, 170).

**Zierschrift** (`font::display_glyph`): Antiqua-Versalien aus Linienzügen in einem Raster (Versalhöhe 14, Akzente bis −4). Strichbreite wie mit einer um −20° gekippten Breitfeder: `dünn + (dick − dünn)·|sin(Winkel + 0,35)|` mit dick 2,05·k, dünn 0,55·k – senkrecht und „\“ kräftig, waagrecht und „/“ fein. Stämme bekommen an Ober- und Unterkante **Keilserifen** (Länge 1,9, zur Mitte gekehlt). Kantenglättung über die Deckung, dann Fase (oben links +0,12, unten rechts −0,28) und ein Verlauf nach unten (−0,22), dunkler Umriss. Graustufen, damit sie beim Zeichnen eingefärbt wird; im Atlas mit k = 2,4, gezeichnet weich gefiltert in jeder Größe. Der Titel-Schriftzug setzt THRONE/DROWNED mit k = 4,4 und „OF THE“ mit 2,9 auf eine gemeinsame Grundlinie, färbt mit einem Goldverlauf (FFF0C0 → E0AC52 → 7A4A1C) und bekommt eine Zierlinie mit Rauten und einer Krone mit Seelenstein.

**Titelmotiv** (`throne.rs`, in voller Größe gemalt und mit einem Kastenfilter auf 55 % verkleinert, damit Details beim Kameraschwenk nicht flimmern): Thron 540 × 680 – Spitzbogen-Lehne (`145·(1 − ((250 − y)/214)^1,7)`), vertiefte, kannelierte Innenfläche, Bronzeband mit Grünspan, Rosette (Radius 56) mit zwölf Speichen und grün leuchtendem Seelenstein, Schädelfries, Armlehnen aus Quadern mit Schädeln, drei Spitzbogennischen mit Seelenlichtern, Spitzen, Korallen (verzweigte Röhren), Seepocken, Tang, Risse, Wasserspuren, nasse Wasserlinie. Dazu kannelierte, gebrochene Säulen, eine Korallenkrone mit Steinen und Kettenglieder.

**PixelLab-Insel und Prolog** (`sw_art::pixellab`, `world/plisle.rs`): die einzige Stelle mit fertigen Bildern (PNGs in `crates/art/assets/pixellab/`, eingebettet, dekodiert mit der `png`-Crate). Eingebackene fremde Untergründe (Wasser am Wrack, Sandflecken, blaue Rinnen, grelles Grün) wurden vorab umgefärbt: Helligkeit des Bereichs auf die nach Helligkeit sortierte Farbrampe der passenden Bodenkachel abgebildet (24 Stufen; Schlamm, Fels bzw. alle Moos-/Halmfarben des Moos-Bogens). Requisiten: Farbe 1:1, Relief = (geglättete Helligkeit − 0,4)·3 + Wölbung quer zum Objekt `√(1 − u²)·3,5` (u = Lage in der Zeile von −1 bis 1), damit Punktlichter sie räumlich treffen; Fußpunkt = unterste deckende Zeile − 2, waagrecht die Mitte der deckenden Pixel der untersten 7 Zeilen; Bäume mit ausblendender Krone (ab 55–66 % der Höhe) und Wiegen im Wind, die Feuerstelle mit warmem Licht (r 170, 1,9/0,9/0,35, Flackern 0,3). **Piratenfackel** `plTorch` (Pfahl mit Schädel, Eisenkorb): Licht in Korbhöhe (`TORCH_FIRE` 92 px über dem Fuß, z 83, r 260, 2,6/1,45/0,6, Flackern 0,22), darauf die PixelFX-Flamme „Feuerschale“ (Maßstab 0,8, Helligkeit 1,7, 9 px über `TORCH_FIRE`) und drei aufsteigende Funken; die beiden Fackeln am Prolog-Sarg leuchten zusätzlich mit je einem starken Licht (z 70, r 320, 2,3/1,3/0,56), der offene Sarg mit einem warmen (r 160, 1,15/0,82/0,5).

---

## 10. Gelände: Materialien, Paletten, Chunk-Backen

**Fertige Kacheln (PixelLab-Insel Schädelhorn)**: `TerrainMap::art_px(px, py)` liefert für Landpixel eine Farbe aus zwei verbundenen Wang-Sätzen (Kiesstrand ↔ Moosgras, Moosgras ↔ Waldboden, je 16 Kacheln à 32 px). Jede **Kachelecke** trägt ein Terrain aus dem Uferabstand der vier angrenzenden Kacheln (Breitensuche): ≤ 2 Kacheln Sand, ab 6 Kacheln Wald, wo `fbm(0,075·Ecke) + (d − 6)·0,03 > 0,1`, sonst Gras; Wald neben Sand wird Gras (Sand und Wald teilen keinen Satz), ebenso um Landeplatz und Lager. Je Kachel: Satz = kleinstes Eckterrain, Maske NW·8 + NE·4 + SW·2 + SE·1 aus den Ecken darüber. Im Bäcker: Farbe übernehmen, Relief = (Helligkeit − 0,4)·4, am Wasser dieselbe nasse Uferkante (abdunkeln ×0,62, etwas 203848) und dunkle Umrisslinie wie überall; das Wasser selbst bleibt das des Shaders. Die Insel ist flach (keine Hochland-Ebenen, keine Wege, kein prozeduraler Bewuchs, keine Boden-Deko). **Prolog**: dieselbe Technik mit den Sätzen Schlamm ↔ Schieferfels und Schieferfels ↔ totes Moos auf der Platte der Prolog-Insel (Ebene 4, ohne Wasser, Abgrund und Planken); Eckterrain Schlamm am Wasser oder wo `fbm < −0,16`, Moos wo `fbm > 0,2`, sonst Fels, Moos neben Schlamm wird Fels.

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
   - **Hang nach Süden** (Tiefe 168 px, Festung: 96-px-Mauer aus Quadern): sanft – die Bodentextur läuft geneigt weiter, zum Fuß nur `0,99 − 0,07t − 0,08t²` dunkler. Oben ein heller Saum (2–5 px, ×1,05), darunter **flache, gewellte Grasstufen** alle 42 px (2 px Lichtsaum ×1,05, dann 4 px Hauch Schatten ×0,93…0,99, Welle aus Rauschen ±16 px), seltene Felsausbisse aus der Klippenpalette (Schwelle 0,56 − 0,08·Mitte), offene Erde (0,45), Halme (1/23), einzelne Steine (1/71). Am Fuß (t > 0,85) bis 12 % dunkler, am Wasser nass (1A3846, 30 %). Relief `(1 − t)·18`, `up = 22`.
   - Wasser/Abgrund: transparent; in `emi` stehen **Uferabstand** (0..255 ≙ 0..64 px) und **Art** (255 Wasser, 64 Abgrund) – `encode_px` schreibt sie für durchsichtige Pixel in Blau und Alpha der Normalen-Textur.
   - **Seitliche Böschung** (bis 64 px) und **oberer Rand** (20 px × 1,4): eigener Hangstreifen, ebenfalls sanft (`0,99 − e²·0,15`, Fels ab 0,52, Erde ab 0,4, `up = 20`). Auf Treppenkacheln: in Siedlungen Holzstufen, in der Natur ein Weg im Gebietsstil (Wegkacheln reichen drei Reihen über die Wandkacheln, weil der Hang tiefer ist als drei Kacheln).
   - Sonst: Materialtextur mit **Kantenschatten** unter erhabenen Nachbarn (bis 28 % dunkler + 8 % 101830), **dunkle Kante** zu tieferen Nachbarn (Umrissfarbe), **Lichtkante** oben (+0,08 Helligkeit), **nasse Ufer** (bis 3 px: 60 % dunkler gemischt + 15 % 203848), **Ufersteine** (Worley-Steine 5,5–8,5 px breit zwischen Gras/Erde und Wasser, Glanz 120 direkt am Wasser) und **Umgebungsverdeckung** am Fuß höherer Ebenen (oben bis 14 px, seitlich bis 9 px, bis 45 % dunkler).
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
| Stille (vor Malakor) | 7A86A8×0,5, 0,55 | C8D4F0×0,5 (0,3, −0,35, 0,85), 0,45 | 2A3448 (0,07) | 0,01/0,015/0,035 · 0,96/1/1,06 | – / 0,75, Bloom 0,6 | 3A4E66 / 0C1420 / C8D8F0 | 1,3 | 0,3 |
| Holländer (Gewitter) | 8A9AB0×0,82, 0,55 | B8D0E0×0,7 (0,35, −0,5, 0,7), 0,5 | 24363A (0,07) | 0,01/0,025/0,03 · 0,96/1,02/1 | 1,12 / 0,8, Bloom 0,75 | 2E5A5E / 0A1C22 / B8E0D8 | 1,3 | 0,5 |
| Meeresgrund | 8AA8A0×0,86, 0,5 | B8D8C8×0,66 (−0,3, −0,4, 0,75), 0,5 | 2A4440 (0,12) | 0,01/0,03/0,025 · 0,94/1,02/1 | 1,1 / 0,7, Bloom 0,75 | 2A5A54 / 0A1A18 / A8D8C8 | 1,4 | 0,7 |
| Offene See | 7898C0×0,75 | C8D8F0×0,7 (0,35, −0,45, 0,6) | 3A5468 (0,12) | – | – | 3AA4BC / 124A70 / D8F0F6 | 1 | 0,3 |
| Glutsee | 8A7470×0,7, 0,45 | FF9A6A×0,5 (−0,5, −0,3, 0,6), 0,4 | 4A2A2A (0,1) | 0,03/0,01/0 · 1,04/0,97/0,94 | – / 0,9, Bloom 0,6 | A8705A / 5A2C2A / E8A070 | 1 | 0,7 |
| Eismeer | 7A9AC8×0,66 | C8E0FF×0,7 (0,45, −0,35, 0,75) | 9AB8D0 (0,14) | 0,01/0,025/0,05 · 0,94/1/1,08 | – / 0,85 | 7EC8D8 / 1E5A7A / FFFFFF | 1 | 0,6 |
| Sturmsee | 6E7A94×0,62, 0,35 | B0C0D8×0,4, 0,2 | 4A5866 (0,2) | – | 1,12 / 0,75 | 3A6E7E / 10283C / B8D0D8 | 1,2 | 0,25 |
| Tiefengrund | 5A8474×0,62, 0,35 | 98C878×0,42, 0,35 | 2E5040 (0,2) | 0/0,03/0,02 · 0,94/1,04/0,98 | – | 1E5A44 / 061C18 / 7AE0A0 | 1,3 (Abgrund 1,3) | 0,5 |

**Übergänge**: Die aktuelle Stimmung läuft jedes Bild mit `t = min(dt·1,5, 1)` auf die Zielstimmung zu. In Seegebieten wird die Gebietsstimmung nach Nähe zur offenen See gemischt.

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
14. Materialtexturen 256² mit Körnung, Gebietspaletten, `mat_at`, Chunk-Bäcker mit Hängen, Kanten, nassen Ufern und Verdeckung (Kapitel 10). Chunks streamen (nur sichtbare plus Ring, Texturfeld-Ebenen).

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
- Überlagerungen dürfen **kein Alpha schreiben**, das Alpha der beleuchteten Szene trägt die Bodenzeile für die Tiefenschärfe.
- Schatten-Abziehbilder schreiben **nur Farbe** in die Albedo, sonst zerstören sie Normalen und Höhen.
- Höhe im Normalen-Texel ist Höhe/2 (bis 510 px); größere Wesen brauchen Grundhöhe über `prm.x`.
- Höhe und Bodenort gehören zusammen: Verdecker zählen nur „am selben Bodenort“, sonst werfen Figuren im Vordergrund Schatten auf den Hintergrund.
- Kamera-Ursprung auf ganze Pixel runden, sonst flimmert die Pixelkunst.
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
fn vs_world(v: VIn) -> VOut {
    var o: VOut;
    var p = v.pos;
    // Wind: obere Eckpunkte schwingen
    if (v.ext.z > 0u) {
        let t = G.screen.z;
        let a = f32(v.ext.z) / 64.0;
        p.x += (sin(t * 1.6 + v.pos.x * 0.031 + v.pos.y * 0.017) * 0.7 + sin(t * 2.9 + v.pos.x * 0.07) * 0.3) * a;
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
    return a * 0.55 + b * 0.3 + c * 0.15;
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
    var c = mix(G.water_a.rgb, G.water_b.rgb, dd);
    // weite See: großflächig tiefere und hellere Stellen, langsam ziehende Strömungsbänder
    let big = noise_at(p * 0.011 + vec2<f32>(t * 0.21, t * 0.08)).r;
    let band = noise_at(vec2<f32>(p.x * 0.004 + p.y * 0.002, p.y * 0.017) + vec2<f32>(t * 0.05, 0.0)).b;
    c *= 0.9 + big * 0.18 * dd + (band - 0.5) * 0.08 * dd;
    let h0 = wave_h(p, t);
    // Wellentäler dunkler, Kämme heller und grüner (Licht dringt durch die dünne Wasserhaut)
    c *= 0.86 + h0 * 0.3;
    c += G.water_a.rgb * smoothstep(0.62, 0.8, h0) * 0.18 * (1.0 - dd * 0.5);
    // Kaustik im Flachen (Voronoi-Netz aus der Rauschtextur)
    let ca = noise_at(p * 0.7 + vec2<f32>(t * 5.0, t * 2.0)).g;
    let cb = noise_at(p * 0.55 - vec2<f32>(t * 3.5, t * 4.0)).g;
    let caust = smoothstep(0.86, 0.97, max(ca, cb)) * (1.0 - dd) * G.water_b.w;
    c += vec3<f32>(caust * 0.35);
    // Schaumringe zum Ufer, Gischt auf hohen Kämmen nahe der Küste
    let fw = G.water_c.w;
    let ring = fract(shore * 6.0 - t * 0.35);
    let foam_edge = 1.0 - smoothstep(0.0, fw * 0.5, shore);
    let foam_ring = step(0.82, ring) * (1.0 - smoothstep(0.0, fw * 1.5, shore)) * step(0.45, noise_at(p * 1.7 + vec2<f32>(t * 2.0, 0.0)).b);
    var foam = max(step(0.4, foam_edge + (noise_at(p * 2.0 + t * 7.0).b - 0.5) * 0.6), foam_ring * 0.85);
    if (hd) {
        let crest = smoothstep(0.7, 0.78, h0) * (1.0 - smoothstep(0.0, fw * 3.0, shore)) * step(0.5, noise_at(p * 2.6 - t * 5.0).b);
        foam = max(foam, crest * 0.6);
    }
    c = mix(c, G.water_c.rgb, foam * 0.9);
    // Normale aus dem Wellenfeld
    let e = 1.0;
    let hx = wave_h(p + vec2<f32>(e, 0.0), t) - h0;
    let hy = wave_h(p + vec2<f32>(0.0, e), t) - h0;
    let nxy = clamp(vec2<f32>(-hx, -hy) * 9.0 * G.water_a.w, vec2<f32>(-0.6), vec2<f32>(0.6));
    var em = vec3<f32>(caust * 0.06);
    var spec = select(0.75, 0.2, foam > 0.5);
    if (hd) {
        // Spiegelung des Himmels nach Fresnel: flach blickend und an Wellenflanken stärker
        let n = vec3<f32>(nxy, sqrt(max(0.0, 1.0 - dot(nxy, nxy))));
        let v = normalize(vec3<f32>(0.0, 0.55, 0.85));
        let r = reflect(-v, n);
        let horizon = G.fog.rgb * 1.1 + G.ambient.rgb * 0.4;
        let zenith = G.ambient.rgb * 1.3 + G.sun_col.rgb * 0.08;
        let sky = mix(horizon, zenith, clamp(r.z, 0.0, 1.0));
        let fres = 0.06 + 0.94 * pow(1.0 - clamp(dot(n, v), 0.0, 1.0), 5.0);
        em += sky * clamp(fres * 2.2 + 0.08, 0.0, 0.6) * (1.0 - foam);
        spec = select(0.97, 0.25, foam > 0.5);
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
    let c = nrm_texel(uv, layer, chunk);
    // FullHD: Neigung weich zwischen den Kunstpixeln (eigene bilineare Mischung, WebGL erlaubt
    // nur einen Sampler je Textur); Höhe und Glanz bleiben pixelgenau, die Farbe ohnehin
    if (G.hd3.x > 0.5) {
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
        let n00 = nrm_texel(b, layer, chunk).rg;
        let n10 = nrm_texel(vec2<f32>(b1.x, b.y), layer, chunk).rg;
        let n01 = nrm_texel(vec2<f32>(b.x, b1.y), layer, chunk).rg;
        let n11 = nrm_texel(b1, layer, chunk).rg;
        let n = mix(mix(n00, n10, fr.x), mix(n01, n11, fr.x), fr.y);
        return vec4<f32>(n, c.b, c.a);
    }
    return c;
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
    var c = sample_sprite(v.uv, v.ext.x, chunk);
    if (chunk && c.a < 0.45) {
        // Wasser und Abgrund im Boden-Chunk: Art und Uferabstand stehen in der Normalen-Textur
        let wn = nrm_texel(v.uv, v.ext.x, true);
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
            let a = sample_sprite(v.uv + vec2<f32>(ts.x, 0.0), v.ext.x, false).a
                + sample_sprite(v.uv - vec2<f32>(ts.x, 0.0), v.ext.x, false).a
                + sample_sprite(v.uv + vec2<f32>(0.0, ts.y), v.ext.x, false).a
                + sample_sprite(v.uv - vec2<f32>(0.0, ts.y), v.ext.x, false).a;
            if (a > 0.4) {
                o.alb = vec4<f32>(1.0, 0.95, 0.8, 1.0);
                o.nrm = vec4<f32>(0.5, 0.5, clamp(v.prm.x / 510.0, 0.0, 1.0), 0.0);
                o.emi = vec4<f32>(0.9, 0.8, 0.55, 1.0);
                return o;
            }
        }
        discard;
    }
    var n = sample_nrm(v.uv, v.ext.x, chunk);
    if (chunk && G.hd3.x > 0.5) {
        // FullHD: feine Bodenstruktur (Körnung, kleine Unebenheiten) in der Normale, ortsfest in der Welt
        let fq = noise_at(v.world * 0.73 + vec2<f32>(13.0, 7.0));
        n = vec4<f32>(n.rg + (fq.rb - vec2<f32>(0.5)) * 0.09, n.b, n.a);
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
        // Spiegelung: die Wasseroberfläche bleibt, das Bild schimmert hindurch
        var w = water(v.world);
        let k = v.col.a * 0.85;
        w.alb = vec4<f32>(mix(w.alb.rgb, rgb, k), 1.0);
        w.emi = vec4<f32>(w.emi.rgb * (1.0 - k * 0.6) + glow * 0.7, 0.5);
        return w;
    }
    let elev = n.b * 510.0 + v.prm.x;
    o.alb = vec4<f32>(rgb, 1.0);
    o.nrm = vec4<f32>(n.rg, clamp(elev / 510.0, 0.0, 1.0), n.a);
    // Alpha im Leuchtpuffer: 1 = Sprite (kann Effekte verdecken), 0,875 = Boden-Chunk, 0,5 = Wasser
    o.emi = vec4<f32>(glow * v.col.a, select(1.0, 0.875, chunk));
    return o;
}

// Schatten/Abziehbilder: nur Farbe abdunkeln (Mischung), Normale und Höhe bleiben.
@fragment
fn fs_decal(v: VOut) -> @location(0) vec4<f32> {
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
    return vec4<f32>(v.col.rgb, a);
}

// ---------------------------------------------------------------- Überlagerung (nach dem Licht)
@fragment
fn fs_overlay(v: VOut) -> @location(0) vec4<f32> {
    let mode = v.ext.y;
    if (mode == 5u) {
        // Nebelbank
        let t = G.screen.z;
        let p = floor(v.world);
        let n = noise_at(p * 0.25 + vec2<f32>(t * v.prm.y, t * v.prm.y * 0.3)).r;
        let n2 = noise_at(p * 0.12 - vec2<f32>(t * v.prm.y * 0.6, 0.0)).b;
        var a = smoothstep(0.35, 0.85, n * 0.6 + n2 * 0.5) * v.col.a;
        // weicher Rand am Quad
        let e = min(min(v.uv.x, 1.0 - v.uv.x), min(v.uv.y, 1.0 - v.uv.y));
        a *= smoothstep(0.0, 0.25, e);
        // Pixelmodus: gerasterte Stufen; FullHD: weicher, fotografischer Nebel
        if (G.hd3.x < 0.5) {
            a = floor(a * 6.0 + hash21(p) * 0.8) / 6.0;
        } else {
            let pf = v.world * 0.5;
            a = smoothstep(0.35, 0.85, noise_at(pf * 0.25 + vec2<f32>(t * v.prm.y, t * v.prm.y * 0.3)).r * 0.6 + noise_at(pf * 0.12 - vec2<f32>(t * v.prm.y * 0.6, 0.0)).b * 0.5) * v.col.a * smoothstep(0.0, 0.25, e) * 0.8;
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
// t_a = Farbe, t_b = Normale (xy, Höhe/510, Glanz), t_c = Leuchten
@fragment
fn fs_light(f: FOut) -> @location(0) vec4<f32> {
    let dim = vec2<f32>(textureDimensions(t_a));
    let ip = vec2<i32>(f.uv * dim);
    let alb = textureLoad(t_a, ip, 0).rgb;
    let nr = textureLoad(t_b, ip, 0);
    let emt = textureLoad(t_c, ip, 0);
    let em = emt.rgb;
    // Wasseroberfläche (Alpha 0,5 im Leuchtpuffer, siehe water_s)
    let wet = abs(emt.a - 0.5) < 0.2;
    let nxy = nr.rg * 2.0 - 1.0;
    let n = vec3<f32>(nxy, sqrt(max(0.0, 1.0 - dot(nxy, nxy))));
    let elev = nr.b * 510.0;
    let spec_k = nr.a;
    let sp = gpos((vec2<f32>(ip) + 0.5) / dim);
    let p3 = vec3<f32>(sp.x, sp.y + elev, elev);
    // interne Auflösungsstufe: Höhen (Welt-px) in Zielpixel umrechnen
    let rs = max(G.hd3.y, 1.0);

    // Bodenzeile (0..1, fern = oben): wo dieser Pixel auf dem Boden steht
    let ground = (f32(ip.y) + 0.5 + elev * rs) / dim.y;

    // Umgebungsverdeckung aus dem Höhenpuffer: Boden am Fuß von Wänden, Klippen und Stämmen
    // wird dunkler. Nur Verdecker zählen, die am selben Bodenort stehen (nicht Figuren davor).
    var ao = 1.0;
    // nur ebene Flächen empfangen Verdeckung: auf Wänden, Kronen und Figuren steigt die Höhe nach oben hin an
    let e_up = textureLoad(t_b, vec2<i32>(ip.x, max(ip.y - i32(2.0 * rs), 0)), 0).b * 510.0;
    let flat = 1.0 - smoothstep(0.8, 1.6, abs(e_up - elev));
    if (G.hd2.x > 0.0 && flat > 0.0) {
        var occ = 0.0;
        let g1 = f32(ip.y) / rs + elev;
        for (var k = 0; k < 12; k = k + 1) {
            let ang = f32(k) * 2.39996;
            let rad = 2.0 + f32(k) * 1.1;
            let o = vec2<f32>(cos(ang), sin(ang)) * rad * rs;
            let q = vec2<i32>(clamp(vec2<f32>(ip) + o, vec2<f32>(0.0), dim - 1.0));
            let e2 = textureLoad(t_b, q, 0).b * 510.0;
            let dh = e2 - elev;
            let g2 = f32(q.y) / rs + e2;
            if (dh > 2.0 && abs(g2 - g1) < 6.0 + rad) {
                occ += clamp(dh / (rad * 2.0 + 6.0), 0.0, 1.0);
            }
        }
        ao = 1.0 - clamp(occ / 12.0 * 1.6, 0.0, 0.6) * G.hd2.x * flat;
    }

    // Wolkenschatten: große, weiche Flecken ziehen langsam über Land und See (am Bodenort des
    // Pixels, damit Bäume und Wände mit ihrem Fuß im selben Schatten stehen)
    var cloud = 1.0;
    if (G.hd2.x > 0.0) {
        let wq = vec2<f32>(sp.x, sp.y + elev) * 0.00042 + vec2<f32>(t_now() * 0.0062, t_now() * 0.0024);
        // zurückgefaltet: große Weltkoordinaten (siehe noise_at)
        let c1 = textureSampleLevel(t_d, s_lin, fract(wq), 0.0).r;
        let c2 = textureSampleLevel(t_d, s_lin, fract(wq * 2.7 + vec2<f32>(0.37, 0.11)), 0.0).b;
        cloud = 1.0 - smoothstep(0.5, 0.74, c1 * 0.72 + c2 * 0.34) * 0.4;
    }
    // Umgebung: Himmel von oben, Boden reflektiert etwas
    let hemi = mix(0.75, 1.0, n.z * G.ambient.w + (1.0 - G.ambient.w));
    var li = G.ambient.rgb * hemi * ao * mix(1.0, cloud, 0.35);
    // Randlicht: erhöhte Kanten (Figuren, Kronen, Klippenkanten) fangen kühles Himmelslicht von hinten
    if (elev > 3.0) {
        let edge = pow(clamp(1.0 - n.z, 0.0, 1.0), 2.0);
        let back = clamp(-n.y * 0.8 + abs(n.x) * 0.35, 0.0, 1.0);
        li += (G.ambient.rgb * 1.6 + G.sun_col.rgb * 0.5) * edge * back * G.hd2.z;
    }
    var spec = vec3<f32>(0.0);
    let view_dir = normalize(vec3<f32>(0.0, 0.55, 0.85));

    // Sonne/Mond mit Bildschirm-Schatten (Höhenfeld abtasten)
    let sd = normalize(G.sun_dir.xyz);
    var nd = max(dot(n, sd), 0.0);
    var sh = 1.0;
    if (G.sun_dir.w > 0.0 && nd > 0.0) {
        var occl = 0.0;
        // versetzter Start je Pixel (kein Streifenmuster), Schatten werden zum Ende hin weicher
        let jit = hash21(vec2<f32>(ip) * 0.73 + vec2<f32>(3.1, 7.7));
        for (var k = 1; k <= 12; k = k + 1) {
            let tq = (f32(k) - jit) * 5.0;
            let q = p3 + sd * tq;
            let sp2 = vec2<f32>(q.x, q.y - q.z);
            let suv = (sp2 - G.view.xy) / G.view.zw;
            if (suv.x < 0.0 || suv.y < 0.0 || suv.x > 1.0 || suv.y > 1.0) { break; }
            let e2 = textureLoad(t_b, vec2<i32>(suv * dim), 0).b * 510.0;
            // Verdecker: höher als der Strahl und am selben Bodenort
            let ground2 = sp2.y + e2;
            if (e2 > q.z + 3.0 && abs(ground2 - q.y) < 22.0) {
                occl = 1.0 - f32(k) / 15.0;
                break;
            }
        }
        sh = 1.0 - occl * G.sun_dir.w;
    }
    li += G.sun_col.rgb * nd * sh * (0.5 + 0.5 * ao) * cloud;
    let hv = normalize(sd + view_dir);
    let hd = G.hd3.x > 0.5;
    // Glanz: glatte Flächen (Wasser, Metall) mit scharfen, hellen Spitzlichtern
    var sexp = 40.0;
    var spnorm = 1.0;
    if (hd) {
        sexp = mix(12.0, 220.0, spec_k * spec_k);
        spnorm = (sexp + 8.0) / 48.0;
    }
    if (wet) {
        // Wasser: Sonne/Mond nicht als großer Spiegelkegel. Mit dem überall gleichen Blick der
        // Schrägsicht stünde bei Gegenlicht fast jede Wellenflanke darin und die See würde flächig
        // weiß. Stattdessen blickt hier eine Kamera schräg über der Bildmitte: der Glanz bildet
        // einen örtlichen Lichtpfad (wandert mit Sonne und Kamera), darin ein weicher Schimmer und
        // funkelnde Glitzerpunkte (Zufall je Weltpixel aus dem Alphakanal der Rauschtextur,
        // springt mehrmals je Sekunde; Dichte höchstens 3,5 %).
        let cen = G.view.xy + G.view.zw * 0.5;
        let cd = G.view.w * 2.5;
        let vw = normalize(vec3<f32>(cen.x, cen.y + cd * 0.543, cd * 0.84) - p3);
        let ndh = max(dot(n, normalize(sd + vw)), 0.0);
        let lobe = pow(ndh, 70.0);
        let gq = vec2<i32>(select(floor(sp), floor(sp * 2.0), hd));
        let tk = i32(t_now() * 7.0);
        let r = textureLoad(t_d, vec2<i32>((gq.x + tk * 37) & 255, (gq.y + tk * 59) & 255), 0).a;
        let spark = step(1.0 - pow(ndh, 300.0) * 0.035, r) * select(2.2, 3.2, hd);
        spec += G.sun_col.rgb * (lobe * 0.3 + spark) * (spec_k * 1.03) * sh * G.sun_col.w * cloud * cloud;
    } else {
        spec += G.sun_col.rgb * pow(max(dot(n, hv), 0.0), sexp) * spec_k * sh * G.sun_col.w * spnorm * cloud * cloud;
    }

    // Punktlichter in 3D
    let nl = i32(G.counts.x);
    let t = G.screen.z;
    var shadows = 0;
    for (var i = 0; i < nl; i = i + 1) {
        let L = G.lights[i];
        let d3 = L.pos.xyz - p3;
        let dist = length(d3);
        if (dist > L.pos.w) { continue; }
        let ld = d3 / max(dist, 0.001);
        let ndl = max(dot(n, ld), 0.0) * 0.85 + 0.15;
        var att = 1.0 - dist / L.pos.w;
        att = att * att;
        var lsh = 1.0;
        if (hd) {
            // natürlicher Abfall: hell am Licht, langer weicher Ausläufer
            let x = dist / L.pos.w;
            let wnd = 1.0 - x * x;
            att = wnd * wnd / (1.0 + 7.0 * x * x);
            // Schlagschatten der nahen Lichter: Strahl zum Licht durch das Höhenfeld
            if (shadows < 8 && att > 0.04 && dist > 10.0) {
                shadows = shadows + 1;
                let jit2 = hash21(vec2<f32>(ip) * 1.31 + vec2<f32>(f32(i), 2.3));
                for (var k = 1; k <= 10; k = k + 1) {
                    let s2 = (f32(k) - jit2) / 10.5;
                    let q = p3 + d3 * s2;
                    if (q.z > L.pos.z - 4.0) { break; }
                    let sp2 = vec2<f32>(q.x, q.y - q.z);
                    let suv = (sp2 - G.view.xy) / G.view.zw;
                    if (suv.x < 0.0 || suv.y < 0.0 || suv.x > 1.0 || suv.y > 1.0) { break; }
                    let e2 = textureLoad(t_b, vec2<i32>(suv * dim), 0).b * 510.0;
                    if (e2 > q.z + 4.0 && abs(sp2.y + e2 - q.y) < 16.0) {
                        lsh = 0.25 + 0.75 * s2 * s2;
                        break;
                    }
                }
            }
        }
        let fl = 1.0 + (sin(t * 11.0 + f32(i) * 1.7) * 0.5 + sin(t * 17.0 + f32(i)) * 0.5) * L.col.w;
        li += L.col.rgb * ndl * att * fl * (0.6 + 0.4 * ao) * lsh;
        let h2 = normalize(ld + view_dir);
        spec += L.col.rgb * pow(max(dot(n, h2), 0.0), select(24.0, sexp, hd)) * spec_k * att * lsh * select(1.0, 0.4, hd);
    }
    if (!hd) {
        // weiche Stufen wie in Pixelkunst (leicht gedithert)
        let dith = (hash21(vec2<f32>(ip)) - 0.5) / 24.0;
        li = floor(li * 12.0 + dith * 12.0 + 0.5) / 12.0;
    }
    var c = alb * li + spec + em;
    // Streulicht im Dunst: jede Lichtquelle leuchtet als weicher Hof in der Luft um sich
    if (hd) {
        var halo = vec3<f32>(0.0);
        for (var i = 0; i < nl; i = i + 1) {
            let L = G.lights[i];
            let lp = vec2<f32>(L.pos.x, L.pos.y - L.pos.z);
            let dd = length(sp - lp) / max(L.pos.w, 1.0);
            if (dd > 1.0) { continue; }
            let fl2 = 1.0 + sin(t * 9.0 + f32(i) * 2.1) * 0.5 * L.col.w;
            halo += L.col.rgb * exp(-dd * 5.0) * fl2;
        }
        c += halo * (0.04 + G.fog.w * 0.3);
    }
    // Blitz (Gewitter)
    c += alb * G.counts.y * vec3<f32>(0.8, 0.85, 1.0);
    // Tiefennebel nach Bodenzeile (fern = oben), Ferne zusätzlich in Nebelfarbe getaucht
    let fogk = G.fog.w * (1.0 - ground * 0.6) + G.hd.y * 0.03 * (1.0 - smoothstep(0.0, 0.4, ground));
    c = mix(c, G.fog.rgb, clamp(fogk, 0.0, 0.85));
    // Alpha trägt die Bodenzeile für die Tiefenschärfe im Endbild
    return vec4<f32>(c * G.misc.w, clamp(ground * 0.5, 0.0, 1.0));
}

// ---------------------------------------------------------------- Bloom
@fragment
fn fs_bright(f: FOut) -> @location(0) vec4<f32> {
    let c = textureSampleLevel(t_a, s_lin, f.uv, 0.0).rgb;
    let l = max(c.r, max(c.g, c.b));
    let k = smoothstep(0.75, 1.6, l);
    return vec4<f32>(c * k, 1.0);
}

@fragment
fn fs_blur_h(f: FOut) -> @location(0) vec4<f32> {
    let ts = 1.0 / vec2<f32>(textureDimensions(t_a));
    var c = textureSampleLevel(t_a, s_lin, f.uv, 0.0).rgb * 0.227;
    c += textureSampleLevel(t_a, s_lin, f.uv + vec2<f32>(ts.x * 1.385, 0.0), 0.0).rgb * 0.316;
    c += textureSampleLevel(t_a, s_lin, f.uv - vec2<f32>(ts.x * 1.385, 0.0), 0.0).rgb * 0.316;
    c += textureSampleLevel(t_a, s_lin, f.uv + vec2<f32>(ts.x * 3.231, 0.0), 0.0).rgb * 0.07;
    c += textureSampleLevel(t_a, s_lin, f.uv - vec2<f32>(ts.x * 3.231, 0.0), 0.0).rgb * 0.07;
    return vec4<f32>(c, 1.0);
}
@fragment
fn fs_blur_v(f: FOut) -> @location(0) vec4<f32> {
    let ts = 1.0 / vec2<f32>(textureDimensions(t_a));
    var c = textureSampleLevel(t_a, s_lin, f.uv, 0.0).rgb * 0.227;
    c += textureSampleLevel(t_a, s_lin, f.uv + vec2<f32>(0.0, ts.y * 1.385), 0.0).rgb * 0.316;
    c += textureSampleLevel(t_a, s_lin, f.uv - vec2<f32>(0.0, ts.y * 1.385), 0.0).rgb * 0.316;
    c += textureSampleLevel(t_a, s_lin, f.uv + vec2<f32>(0.0, ts.y * 3.231), 0.0).rgb * 0.07;
    c += textureSampleLevel(t_a, s_lin, f.uv - vec2<f32>(0.0, ts.y * 3.231), 0.0).rgb * 0.07;
    return vec4<f32>(c, 1.0);
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
    // weiches Abrollen der Lichter (ACES-Näherung)
    let a = c * (2.51 * c + 0.03);
    let b = c * (2.43 * c + 0.59) + 0.14;
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
    return beam * top * G.hd.w;
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
    // Schrägsicht: Bildschirm → Low-Res-Bild
    let suv = tilt_src(f.uv);
    let k = G.hd.x;
    let dk = 1.0 + k * f.uv.y;
    let mag = vec2<f32>(dk, dk * dk / (1.0 + k));
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
        let bl = textureSampleLevel(t_c, s_lin, suv, 0.0).rgb;
        c = mix(c, bl, coc * 0.8);
        // Diorama: unscharfe Ränder etwas dunkler, der Blick bleibt in der Fokuszone
        c *= 1.0 - coc * 0.16;
    }
    if (G.hd.w > 0.0) {
        // Strahlen hellen auf, was schon da ist (kein Grauschleier auf dunklem Wasser)
        let r = light_rays(suv, f.uv.y);
        c = c * (1.0 + r * 0.6 * (vec3<f32>(0.4) + G.sun_col.rgb)) + G.sun_col.rgb * r * 0.004;
    }
    if (G.hd2.y > 0.0) {
        let fv = G.hd.z / (1.0 + k - k * G.hd.z);
        let band = smoothstep(0.16, 0.42, abs(f.uv.y - fv));
        c += (G.sun_col.rgb * 0.7 + G.fog.rgb * 0.4 + vec3<f32>(0.02)) * bokeh_dust(f.uv) * band * G.hd2.y * 0.07;
    }
    let bl = textureSampleLevel(t_b, s_lin, suv, 0.0).rgb;
    c += bl * G.misc.z;
    c = tonemap(c);
    // Farbkorrektur: Lift/Gain, Kontrast, Sättigung
    c = G.lift.rgb + c * (G.gain.rgb - G.lift.rgb);
    c = (c - 0.5) * G.lift.w + 0.5;
    let l = luma(c);
    c = mix(vec3<f32>(l), c, G.gain.w);
    // Blenden (Tod, Treffer)
    c = mix(c, vec3<f32>(1.0), G.counts.z);
    c = mix(c, vec3<f32>(0.5, 0.02, 0.04), G.counts.w);
    // Vignette
    let d = f.uv - 0.5;
    c *= 1.0 - dot(d, d) * G.misc.x;
    // Körnung
    let gr = hash21(floor(f.uv * G.screen.xy) + fract(G.screen.z) * 91.0) - 0.5;
    c += gr * G.misc.y;
    return vec4<f32>(clamp(c, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
```
