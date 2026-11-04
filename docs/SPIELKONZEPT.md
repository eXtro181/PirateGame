# Throne of the Drowned – Spielkonzept

Gesamtdokument zu Idee, Spielablauf, Welt, Figuren, Systemen, Grafik, Klang und Technik.
Stand: 1. November 2026 (Version 0.6, Zweig `perf`). Technische Kurzreferenz für die Entwicklung: `CLAUDE.md`.

---

## 1. Überblick

| | |
|---|---|
| **Titel** | Throne of the Drowned (Arbeitstitel bis Oktober 2026: „Schwarzwasser“) |
| **Genre** | Soulslike-Action-RPG mit Seefahrt, Top-Down-3/4-Sicht |
| **Plattform** | Browser (Desktop und iPhone), WebGL2, läuft ohne Installation |
| **Live** | https://extro181.github.io/PirateGame/ |
| **Sprache** | Deutsch (alle Texte, Dialoge, Menüs) |
| **Technik** | Rust + Bevy 0.16 mit eigenem wgpu-Renderer, kompiliert nach WebAssembly |
| **Besonderheit** | Keine einzige Bild- oder Audiodatei: Grafik, Welt, Musik und Klänge entstehen beim Start aus Code |

**Kurzbeschreibung.** Du bist ein Ertrunkener, den die See zurückgegeben hat. Du erwachst in einem Sarg auf einem Piratenfriedhof, findest eine Festung voller Feuer und zerbrochener Throne und erfährst, dass vier Geisterkönige und vier Ungeheuer der See die Welt zwischen sich aufgeteilt haben. Hinter allem steht Malakor, der Ertrunkene König, der am Ende der Welt auf seinem Fliegenden Holländer wartet.

**Säulen des Spiels**
1. **Dark Fantasy**: gedämpftes Licht, Lichtinseln aus Feuern und Laternen, Nebel, Stimmung je Gebiet.
2. **Soulslike-Kampf**: Ausdauer, Rolle, Parieren, schwere Bosse mit zwei Gestalten, Tod mit Perlenfleck.
3. **Seefahrt**: eigenes Schiff mit Crew, Breitseiten, Seeungeheuer und Geisterschiffe.
4. **Handgemachter Look aus Code**: HD-Pixelart mit Licht, Höhe und 3D-Wirkung, vollständig prozedural.

---

## 2. Geschichte und Spielablauf

1. **Erwachen** (Galgenstrand): Klopfen im Dunkeln, der Sargdeckel ruckelt und wird weggeschoben, der Ertrunkene steigt heraus. Um ihn herum Galgen mit schwingenden Gehängten, Käfige, offene Gräber und über 70 Tote. Der Alte Galionsschnitzer erklärt die Lage.
2. **Glutwacht**: Die Festung in der Mitte der Welt ist der Knotenpunkt. Hier brennt das Säbelfeuer, hier stehen vier leere Throne, hier arbeiten Schmied, Händlerin, Laternenmagd und Schiffsmeister.
3. **Vier Könige** auf vier Inseln. Jeder hat zwei Gestalten. Besiegt hinterlässt er seine **Galionsfigur**, die auf seinen Thron in der Glutwacht gesetzt wird.
4. **Vier Seeungeheuer** in vier Seegebieten. Jedes verwandelt sich bei halber Kraft in ein Piratenschiff. Das versenkte Schiff brennt danach in Originalgröße im Ring um den See der Glutwacht.
5. **Crew**: In jedem Königsgebiet wartet jemand, der an Bord kommt (vier Crewmitglieder mit eigenem Lied und eigenem Geschütz).
6. **Der Strudel**: Sitzen alle vier Galionsfiguren auf den Thronen und sind alle vier Ungeheuer versenkt, öffnet sich im See der Glutwacht ein Strudel. Acht Krakenarme wiegen sich darum. Kommt das Schiff nahe, packen sie es und ziehen es hinab.
7. **Ende der Welt**: ein Meer in der Südost-Ecke der Karte, im Süden und Osten stürzt es als Wasserfall ins Nichts.
8. **Endkampf gegen Malakor** (siehe 5.4): Fliegender Holländer, roter Kraken, das Meer läuft leer, Malakor zu Fuß am Meeresgrund, zuletzt sein Gebein als Knochenkraken.
9. **Nach dem Sieg**: Am Grund erscheint das Säbelfeuer „Ende der Welt“. Die Laternenmagd bietet die **neue Fahrt** an (Neues Spiel+).

---

## 3. Die Welt

### 3.1 Karte
- 1776 × 1328 Kacheln à 32 Pixel (56 832 × 42 496 Weltpixel) – zehnmal die Fläche der ersten Karte. Der Archipel der sechs Hauptinseln liegt in der Mitte, das Ende der Welt in der Südost-Ecke, die vier Seegebiete weit draußen in den Himmelsrichtungen, dazwischen 72 wilde Inseln im weiten Ozean.
- Jede Insel hat Höhenstufen (Ebenen 0–4): Inselmitte und ein bis zwei Bergkuppen höher, Strände flach.
- Außerhalb der Festung sind Ebenenkanten **flache, sanfte Naturhänge** (nach Süden 168 px tief, seitlich 64 px, oben 20 px). Die Bodentextur läuft geneigt weiter, in flachen, gewellten Grasstufen, mit seltenen Felsausbissen, offener Erde, Halmen und Steinen; zum Fuß hin nur leicht im Schatten.
- **Treppen** gibt es nur in Siedlungen und in der Festung. In der Natur führen Wege im Gebietsstil über die Hänge (Schneepfad, Bohlen im Moor …).
- Herunterspringen geht genau eine Ebene tief.
- **Wasserfälle** sitzen an echten Hangkanten: Bach oben, Becken unten, Gischt.

### 3.2 Inseln

| Insel (Schlüssel) | Gebiet | Stimmung | Siedlung / Laden | König |
|---|---|---|---|---|
| **Galgenstrand** (`kueste`) | Küste, fahl | Nebel, kalt, Friedhof | keine | – (Start) |
| **Glutwacht** (`hain`) | Festung | Glut, Basalt, Feuer | Händlerin Sabeth, Schmied Harrow | – (Hub) |
| **Aschebucht** (`asche`) | Glut | Asche, Lava, rotes Licht | Glutgrund – Schmiedin Ilka | Admiral Brandvogt |
| **Frostgrotte** (`frost`) | Frost | Schnee, Eis, Säulenbasalt | Reifstein – Pelzhändlerin Wendla | Königin Ysra Frostweh |
| **Hafen Rabenwacht** (`raben`) | Sturm | Fachwerk, Schiefer, Wind | Rabenwacht (Stadt) – Kartenhändler Orrin | Kapitän Rabensturm |
| **Fluchmangroven** (`mangroven`) | Tiefe | Moor, Kanäle, Pfahlbauten | Moorhain – Kräuterhexe Mabel | Morgrim Korallenhand |
| **Ende der Welt** (`ende`) | Ende | Gewitter, tote See | – | Malakor |

Jede Siedlung hat Dorfbewohner, Hühner und Katzen, Rauch aus den Schornsteinen und einen Laden mit **begehbarem Innenraum**. Gegner halten sich von Siedlungen fern.

**Wilde Inseln**: 72 Eilande (klein, mittel, wenige große) mit Namen wie „Reifholm“, „Glutzahn“, „Moderwerder“, „Sturmklippe“ oder „Schädelriff“. Ihr Biom richtet sich nach der Himmelsrichtung (Frost im Norden, Glut im Osten, Sturm im Westen, Moder im Süden, dazwischen Galgeninseln). Jede hat einen Anleger zur Kartenmitte, einen Landeplatz und ein Herzstück:
- **Piratenlager**: Zelte, Tafel, Kanonen, Fässer, Flagge, Truhe.
- **Ruine**: Marmorplatz, Säulenring, Bogen, Wächterstatue, Kerzen, Schwertgrab, Truhe.
- **Friedhof**: Galgen, Gräberreihen, Särge, Käfig, Fackeln, manchmal eine lebende Truhe.
- **Wrackstrand**: gestrandetes Schiff, Fracht, Anker, Netze, Ruderboot, Haiskelette, manchmal ein Mimic.
- **Wilder Hain**: dichter Wald, Leuchtblumen bzw. Kristalle, Seelenblume.
- **Schatzinsel**: zwei Truhen (Rumflasche, Seelenphiole oder Gezeitensplitter) mit starkem Wächter.

Je weiter draußen, desto höher die **Gefahrenstufe** (0–3): Gegner haben mehr Leben (+35 % je Stufe), mehr Schaden (+12 %) und geben mehr Perlen (+40 %). Auf jeder vierten Insel brennt ein Säbelfeuer (Schnellreise).

### 3.3 Glutwacht im Detail
Kompakt wie ein Schrein: in der Mitte ein See mit Steg zum Strudel, darum der Ring der brennenden Bossschiffe. Darüber die Feuerterrasse (Ebene 1) mit Säbelfeuer, vier Thronen, Schmied und Händlerin, dann der Schrein mit Bergfried (Ebene 2) und Wachplattformen (Ebene 3). Im Osten ein Seetor-Kanal, im Süden der Anleger. Dort liegt die **Werft** von Schiffsmeister Brack: Plankenkai, Helling mit Rumpf, Kran, Taue, Fässer, Kanone, Laternen, Flagge und Wegweiser. Die Mauern bestehen aus Quadern, Ebenen sind hier echte Mauern mit Treppen.

### 3.4 Seegebiete
| Gebiet | Element | Seeungeheuer → Schiff |
|---|---|---|
| Glutsee | Feuer | Tiamat, Mutter der Glut → Die Glutkrone |
| Eismeer | Frost | Krallgard, der Eishummer → Die Frostwitwe |
| Sturmsee | Sturm | Sturmzunge, die Seeschlange → Die Rabenbraut |
| Tiefengrund | Tiefe | Der Abgrundangler → Die Tiefenlaterne |

Die Seegebiete liegen weit draußen: Eismeer im Norden, Glutsee im Osten, Sturmsee im Westen, Tiefengrund im Süden (Radius 72 Kacheln, das Kampfrevier des Seebosses 40). Jedes hat eigene Stimmung, eigenes Licht und eigene Requisiten: Lavaspitzen und brennende Wracks, Eisberge, Felsnadeln, Korallen und Tang. Auf hoher See springen Fische, Wale tauchen auf, 19 Geisterschiffe patrouillieren, 60 Knochenhaie rammen den Rumpf.

**Seeungeheuer** (über 400, je Seegebiet andere Arten): Sie lauern als Schatten unter der Oberfläche, tauchen auf, wenn ein Schiff nahe kommt, und greifen nach ihrer Art an.

| Ungeheuer | Angriff | Häufig im |
|---|---|---|
| Seeschlange | taucht in Buckeln, rammt den Bug | überall, Eismeer |
| Muräne (zu dritt) | schnelle Bisse | überall, Tiefengrund |
| Riesenkalmar | acht Fangarme, angesagte Tentakelschläge | Sturmsee, Tiefengrund |
| Geisterqualle (im Schwarm) | leuchtet, sticht bei Berührung | Eismeer, Tiefengrund |
| Glutrochen | gleitet, spuckt Feuerbälle | Glutsee |
| Kanonen-Angler | Lockleuchte, Kanonenkugeln | Tiefengrund |
| Skelett-Kraken | vier Tentakelschläge | Tiefengrund, Eismeer |
| Panzerkröte | Salven aus drei Kugeln | Glutsee |
| Sturm-Wyvern | kreist, Sturzflug aufs Deck | Sturmsee |
| Riesenhai | rammt | überall, Sturmsee |

Ungeheuer zeigen eine Lebensleiste, sobald sie verwundet sind; die Crew zielt mit ihren Geschützen auch auf sie.

---

## 4. Figuren

- **Spieler**: der Ertrunkene. Aussehen im Editor wählbar (Hautton, Haare, Bart, Mantel, Hut …), Herkunft und Geschenk zu Beginn.
- **Glutwacht**: Laternenmagd (Stufenaufstieg, Neues Spiel+), Schmied Harrow (Waffen härten und kaufen), Händlerin Sabeth (Waren), Schiffsmeister Brack (Ausbauten), Crew an der Tafel.
- **Start**: Alter Galionsschnitzer.
- **Crew** (je mit Lied und Geschütz):

| Crew | Element | Lied | Geschütz |
|---|---|---|---|
| Kanonierin Asra | Glut | „Glutwind“ | Mörser |
| Navigatorin Lene | Frost | „Eisige Leine“ | Kettenkugeln |
| Bootsmann Kalo | Sturm | „Rabensturm“ | Flammenwerfer |
| Zimmermann Ode | Tiefe | „Tiefer Grund“ | Kraken-Begleiter (acht grüne Tentakel um das Ziel) |

Die Geschütze werden nach Anzahl der Crew freigeschaltet (1 Crew Mörser, 2 Kettenkugeln, 3 Flammenwerfer, 4 Kraken).

- **Händler und Dorfbewohner**: Schmiedin Ilka, Pelzhändlerin Wendla, Kartenhändler Orrin, Kräuterhexe Mabel, Dorfbewohner A–H mit eigenen Dialogen je Insel.

---

## 5. Gegner und Bosse

### 5.1 Gegner
Alle Gegner haben dieselbe Grafikqualität und dieselben Animationen wie Spieler und NPCs (Stehen, Gehen, Angriff, Treffer, Tod, teils Schießen). Menschenähnliche Gegner zeichnet der Figuren-Maler, Tiere und Monster das 3D-Kreaturen-Rig.

| Gegner | Art |
|---|---|
| Ertrunkener Matrose, Ankerträger, Musketen-Gerippe, Aschegänger | Skelette |
| Tangertrunkener, Frostertrunkener, Sturmmatrose | Ertrunkene |
| Strandkrabbe, Krabbenkrieger, Muschelkriecher | Strand |
| Lebendiger Schatzrechner, Schatz-Mimic, Mimic-Kapitäns wahre Gestalt | Mimics (Truhen) |
| Skelett-Kraken, Kanonen-Angel-Fisch-Dämon, Kanonen-Fresser-Dämon | Monster |
| Küsten-Wyvern, Schild-Kröten-Festungs-Wurm | große Wesen |
| Sturm-Wind-Harpye-Konstrukt, Uhrwerk-Glocken-Matrosen-Konstrukt | Konstrukte |
| Ork-Schlachter-Kapitän, Tiefen-Abgrund-Pirat, Voodoo-Mutanten-Kapitän | Elitekapitäne |
| Haie, Geisterschiffe (Fleisch-Geist-Schiff, Geister-Galeere-Chimäre), See-Kraken | auf See |

Gegner können fernkämpfen, Flächenangriffe machen, anstürmen, brennen oder einfrieren. Elite-Varianten sind stärker. Gegner meiden Bossräume und Siedlungen.

### 5.2 Die vier Könige (je zwei Gestalten)
| König | Insel | Zweite Gestalt | Leben |
|---|---|---|---|
| Admiral Brandvogt, der Glutkönig | Aschebucht | Dämonen-Kapitän-Mantikor (Rig) | 1300 |
| Königin Ysra Frostweh | Frostgrotte | Abgrund-Gorgonen-Königin (Schlangenleib und -haar) | 1250 |
| Kapitän Rabensturm, der Sturmkönig | Rabenwacht | Uhrwerk-Sturm-Monster (Uhrwerkflügel) | 1200 |
| Morgrim Korallenhand, König der Tiefe | Fluchmangroven | Kraken-Basilisk-Chimäre (Rig) | 1450 |

Der Kampf beginnt hinter einem goldgelben **Nebeltor**. Angriffe mit Bodenmarkierungen: Kombo, Schwung, Sturmangriff, Einschlag, Atemkegel, Salve, Regen, Nova, Böe, Stachelreihe, Wirbel. Jeder König lässt 4000 Perlen und seine Galionsfigur fallen.

### 5.3 Die vier Seeungeheuer
Phase 1: das Ungeheuer (Seitenansicht, sehr groß). Bei 50 % Phase 2: ein eigenes Piratenschiff mit eigenem Entwurf. Einschläge (`Slam`) mit Vorwarnung. Wer das Gebiet verlässt, setzt den Kampf zurück. Belohnung: 5000 Perlen, das brennende Wrack in der Glutwacht.

### 5.4 Endkampf: Malakor, der Ertrunkene König
1. **Annäherung**: mit dem Schiff am Ende der Welt. Nach einigen Sekunden: „Unter dir bewegt sich etwas Großes …“
2. **Aufstieg** (Zwischensequenz): der Fliegende Holländer steigt aus dem Wasser.
3. **Seeschlacht**, Breitseite an Breitseite: XXL-Feuer aus dem Galion, XXL-Breitseiten, Donnermörser mit Blitz. Unter 60 % taucht der **rote Kraken** auf. Solange er lebt, treffen Schüsse nur ihn.
4. **Das Meer läuft leer**: Das Wasser stürzt über den Rand, das Ende der Welt wird zum Meeresgrund mit Sand, Seegras, Haiskeletten, Wracks und Korallen.
5. **Malakor zu Fuß** (2400 Leben): vier Schwerter und Tentakelbart. Angriffe: Viererkombo, Wirbel, Peitsche (der Bart zieht heran), Sprung, Geysire.
6. **Malakors Gebein, der Knochenkraken** (2800 Leben): Schädelmantel und acht Wirbelarme. Stacheln, Schwung, Splitter rundum, Sprung.
7. **Ende**: Das Säbelfeuer „Ende der Welt“ erscheint.

---

## 6. Spielsysteme

### 6.1 Charaktererstellung
**Herkunft** (Startwaffe, Nebenhand, Werte):

| Herkunft | Beschreibung | Waffe | Nebenhand |
|---|---|---|---|
| Deckmatrose | ausgewogen, empfohlen | Entermesser | Pistole |
| Kanonier | langsam, harte Treffer | Ankerhammer | – |
| Duellant | Parieren und Gegenschläge | Rapier | – |
| Schmuggler | schnell, mehr Perlen | Dolch | Pistole |
| Sumpfkind | Gezeitenzauber, wenig Leben | Muschelfetisch | Gezeitenstoß |
| Treibgut | Stufe 1, nur Lumpen und Knüppel – für Profis | Morscher Knüppel | – |

**Geschenk**: zusätzliche Rumflasche, Glücksdublone (mehr Perlen), Alte Seekarte (Karte aufgedeckt), Pulverbomben, Schwarze Perle oder nichts.

### 6.2 Werte und Stufen
Fünf Werte: **Vitalität** (Leben), **Ausdauer**, **Kraft** (Hammer, Säbel, Knüppel), **Geschick** (Rapier, Dolch, Pistole), **Tiefenglaube** (Geist und Zauber). Stufen kauft man bei der Laternenmagd mit Perlen (Kosten steigen um 10 % je Stufe).

### 6.3 Kampf
- **Ausdauer** in vier Segmenten. Leichter und schwerer Hieb mit Kombos, Rolle mit Unverwundbarkeit, Parieren mit Gegenschlag, Zielerfassung.
- **Nebenhand**: Steinschlosspistole (Bleikugeln) oder Gezeitenstoß.
- **Zauber**: Kettenblitz (22 Geist), Feuerball (18 Geist, aus dem Folio der Glut).
- **Gürtel**: Bomben und weitere Verbrauchsgüter. **Rum** heilt Leben, **Seelenphiolen** füllen Geist. Beides füllt sich an jedem Feuer auf.

**Waffen**
| Waffe | Schaden | Wert | Tempo | Preis |
|---|---|---|---|---|
| Entermesser | 17 | Kraft | 1,0 | Start |
| Ankerhammer | 30 | Kraft | 0,66 | 2400 |
| Rapier | 13 | Geschick | 1,3 (Parieren ×1,6) | 1800 |
| Dolch | 11 | Geschick | 1,45 | 900 |
| Muschelfetisch | 9 | Tiefenglaube | 1,1 | 1500 |
| Morscher Knüppel | 10 | Kraft | 0,95 | Start |

Der Schmied härtet Waffen mit **Gezeitensplittern** (aus Seelenblumen).

### 6.4 Säbelfeuer, Tod und Perlen
- **Säbelfeuer** sind die Leuchtfeuer der Welt (11 Stück, zwei je Königsinsel). Entzünden, rasten (heilt, füllt Rum und Phiolen auf, belebt Gegner wieder), zwischen entzündeten Feuern reisen. Automatisches Speichern beim Rasten.
- **Tod** („DU BIST ERTRUNKEN“): Alle Perlen bleiben als **Perlenfleck** an der letzten sicheren Stelle zurück. Zurückholen geht nur, wenn man dort ankommt, ohne vorher noch einmal zu sterben.
- **Perlen** sind die einzige Währung: für Stufen, Waffen, Waren und Schiffsausbauten.

### 6.5 Gegenstände
| Gegenstand | Herkunft | Wirkung |
|---|---|---|
| Rumflasche / Glutrum | Truhe / Glutgrund | eine Rumflasche mehr |
| Seelenphiole | Händler | eine Phiole mehr |
| Bleikugeln, Pulverbomben | Händler | Munition |
| Seelenblume, Gezeitensplitter | Welt | Material für den Schmied |
| Folio der Glut | Welt | Zauber Feuerball |
| Sturmglas | Welt | +20 Geist |
| Ring der Ertrunkenen | Welt | 12 % weniger Schaden |
| Mantel des Lotsen | Welt | schnellere Ausdauer |
| Muschelhorn | Rabenwacht | +20 Ausdauer |
| Treibholz-Amulett | Rabenwacht | +15 % Perlen |
| Ascheherz | Glutgrund | +10 % Waffenschaden |
| Frostmantel | Reifstein | +30 Leben |
| Eisglas | Reifstein | +15 Geist, +10 % Zauberkraft |
| Sturmkompass | Rabenwacht | Schiff eine Stufe schneller |
| Rabenfeder | Rabenwacht | Ausdauer 20 % schneller |
| Moortinktur | Moorhain | 8 % weniger Schaden |
| Voodoopuppe | Moorhain | +10 Geist, +10 % Zauberkraft |
| Galionsfiguren (4) | Könige | auf die Throne setzen |

Gegenstände am Boden schweben mit einem Schimmer in ihrer Seltenheitsfarbe. Mit Alt erscheinen alle Namen.

### 6.6 Seefahrt
- Das Schiff steigt an jedem Steg aus der Tiefe, wenn man es ruft. An Bord gehen und anlegen mit E.
- **Steuerung**: links/rechts wenden, hoch/runter Segelstufen (je Stufe 74 px/s). J/L feuern Breitseiten nach Backbord bzw. Steuerbord, C setzt das Crew-Geschütz ein, F stimmt das Shanty der Crew an (das Schiff wird schneller).
- **Seekarte** (M): gezeichnet wie auf vergilbtem Papier, deckt sich beim Erkunden auf (Felder à 16 × 16 Kacheln). Drei Zoomstufen (Ganze Welt, Umgebung, Nah), schieben mit den Richtungstasten. Namen der wilden Inseln erscheinen, sobald man dort war.
- **Ausbauten** beim Schiffsmeister, je bis Stufe 10: Tempo (+6 % Fahrt), Verteidigung (+24 Rumpf, −4 % Schaden), Angriff (+12 % Schaden, mehr Kugeln). Jede Stufe ist am Schiff sichtbar (Segel, Eisenbänder, Kanonen).
- Kapitän und Crew stehen an Deck und drehen sich mit dem Kurs.

### 6.7 Fortschritt, Neues Spiel+, Cheats
- **Fortschritt** (Taste I): Könige, Seeungeheuer, Crew, Strudel, Endkampf, aktuelle Fahrt.
- **Neues Spiel+**: Könige, Seeungeheuer, Throne und Endkampf werden zurückgesetzt. Stufe, Ausrüstung, Schiff, Crew, Feuer und Schätze bleiben. Gegner-Leben × (1 + 0,5 · Fahrt), Schaden × (1 + 0,3 · Fahrt), Bossperlen +25 % je Fahrt.
- **Pause-Menü**: Cheat „Unendlich Perlen“ (umschaltbar, der alte Stand bleibt erhalten). **Debug-Menü**: Trophäen der Könige und Seeungeheuer, ganze Crew, alle Säbelfeuer.

---

## 7. Grafik

Die vollständige Analyse mit allen Formeln, Farbwerten, Shadern und einer Schritt-für-Schritt-Anleitung zum Nachbau steht in [`GRAFIK_ANLEITUNG.md`](GRAFIK_ANLEITUNG.md).

### 7.1 Vorgaben
- **HD-Pixelart** mit dem Detailgrad der Beispielbilder: Blütenbäume mit Glyzinien, Basaltklippen, türkises Wasser, Fachwerkhäuser, eine Rotnebel-Zone.
- **Dark Fantasy**: gedämpftes Licht, Lichtinseln, Nebel, eigene Stimmung je Zone.
- **3D-Wirkung** an Hängen, Abgründen und Küsten (Normalen- und Höhenpuffer).
- Kürzere Bildseite zeigt 540 Weltpixel. FullHD-Modus verdoppelt die interne Auflösung.
- Alle Gegner in derselben Qualität und mit denselben Animationen wie Spieler und NPCs.
- Ausführliche Grafik-Spezifikation: https://claude.ai/artifact/V4yVVaGGr1YaLyLo4UUA19

### 7.2 Stil: HD-2D (wie Final Fantasy Resonance)
- **Diorama-Kippung**: Das Endbild ist leicht gekippt. Unten ist die Welt näher und größer, oben ferner.
- **Tilt-Shift-Tiefenschärfe** um den Fuß des Spielers, Bokeh-Staub.
- **Licht**: 3D-Punktlichter mit Höhe, Schlagschatten der acht nächsten Lichter, Sonnenschatten, Randlicht, Lichtstrahlen je Zone, Umgebungsverdeckung aus dem Höhenpuffer, Bloom, Farbkorrektur, Farbsaum.
- **Wasser** (Fotolook): Fresnel-Himmelsspiegelung, Wellenkämme, Sonnen- bzw. Mondglanz als wandernder Lichtpfad mit funkelnden Glitzerpunkten (statt flächig weißer See bei Gegenlicht), pixelgenauer Uferschaum, großflächige Tiefen und Strömungsbänder auf hoher See, Spiegelbilder von Schiffen und Figuren.
- **Wolkenschatten** ziehen langsam über Land und See.
- **Nebel** in Bänken, Funken, Glühwürmchen, Asche, Schnee, Regen und Blitze je Gebiet.
- Option „Sparsam“ lässt nur die Kippung an (für schwache Geräte).

### 7.3 Prozedurale Kunst (alles aus Code)
Jedes Bild ist eine **Leinwand mit vier Kanälen**: Farbe, Normale, Höhe über Grund und Leuchten/Glanz. So kann der Renderer jedes Pixel beleuchten und Schatten werfen lassen.

- **Figuren-Maler** (`chars.rs`): Figuren 64 × 72 Pixel, drei Richtungen (vorn, Seite gespiegelt, hinten), alle Animationen. Über 40 Vorlagen: Gegner, NPCs, Crew, Dorfbewohner, Könige, Malakor (vier Arme, Tentakelbart, Korallenkrone). Bausteine wie Hüte, Mäntel, Bärte, Flügel, Schlangenhaar, Uhrwerk, Leuchtaugen, Auren.
- **Kreaturen-Rig** (`rig.rs`, `beasts.rs`): Wesen bestehen aus Ellipsoiden, Quadern, Röhren, Häuten und Dreiecken im Raum. Ein Strahlenwerfer rendert sie aus der 3/4-Kamera (Neigung 0,55) mit Mustern (Schuppen, Fell, Nieten …), Innenkonturen und Umriss. Er schreibt die Höhe je Pixel, damit Schatten der Grundfläche folgen.
- **Schiffe** (`ships.rs`): ebenfalls als Rig. 16 Kurse (9 gerendert, der Rest gespiegelt), das eigene Schiff mit sichtbaren Ausbauten, vier Bossschiffe mit eigenem Entwurf und der Fliegende Holländer (Maßstab 6,9).
- **Gelände** (`terrain.rs`): Der Boden wird in 256-Pixel-Stücken gebacken. Materialien wie Gras, Wiese, Sand, Erde, Pflaster, Marmor, Planken, Basalt, Asche, Schnee, Eis, Schlamm, Fels und Moos haben je Gebiet eigene Paletten. Dazu weiche Materialgrenzen, nasse Uferkanten, Ufersteine, Hänge, Festungsmauern aus Quadern und Naturwege.
- **Requisiten** (`props.rs`, `flora.rs`, `buildings.rs`, `structures.rs`, `grave.rs`): Bäume mit Kronen, die durchsichtig werden, wenn man dahinter steht, Büsche, Fachwerkhäuser, Festung, Werft, Galgen mit schwingenden Gehängten, Särge, Wracks, Eisberge, Korallen.
- **Atlas**: alles auf Seiten à 2048² (aktuell 5). Feste Streifen für Spieler, eigenes Schiff und den gerade aktiven Boss werden bei Bedarf neu gezeichnet.
- **Schrift und UI**: eigene Pixelschrift mit Umlauten und Sonderzeichen für Fließtext, dazu eine **Zierschrift** (Antiqua-Versalien wie gemeißelt, aus Breitfeder-Strichen mit Keilserifen) für Titel, Banner, Ortsnamen, Fenstertitel und Knöpfe. Rahmen, Symbole.
- **Boden-Deko** (Gras, Blumen, Klee, Farn, Steinchen, Zweige, Laub, Tang, Treibholz, Wurzeln, Pilze, Kristalle, Knochen, Glut) entsteht je 256-px-Eimer erst beim Zeichnen – mehrfach so dicht wie früher, ohne Speicher und Ladezeit.

### 7.4 Zonenstimmung
Jede Zone hat eine eigene Licht- und Farbstimmung (`env.rs`), die weich eingeblendet wird: Grundlicht, Sonne, Nebelfarbe, Wasserfarben, Belichtung, Lichtstrahlen. Stimmungen gibt es für Galgenstrand, Glutwacht, Aschebucht, Frostgrotte, Rabenwacht, Fluchmangroven, Ende der Welt, Innenräume, die Stille vor dem Endkampf, das Gewitter beim Holländer, den Meeresgrund und jedes Seegebiet.

### 7.5 Titelbild
**„Der Thron der Ertrunkenen“**: In der nächtlichen See ragt ein riesiger Thron aus schwarzem Basalt auf – Spitzbogen-Lehne mit Bronzeband und Rosette, in der ein grüner Seelenstein glüht, Schädelfries, Armlehnen mit Schädeln, drei Nischen mit Seelenlichtern, Korallen, Seepocken, Tang und Wasserspuren. Über dem leeren Sitz schwebt die Korallenkrone, Ketten hängen von den Armlehnen ins Wasser, gebrochene Säulen der versunkenen Halle stehen ringsum, grüne Seelenlichter steigen auf, und hinter dem Thron winden sich Krakenarme aus der Tiefe. Oben der Schriftzug „THRONE of the DROWNED“ in Gold mit Zierlinie und Krone, links das Menü.

---

## 8. Klang

- **Synthesizer** im Code (`synth.rs`): Lieder werden als PCM gerendert (22 050 Hz, mono). Instrumente: Glocke, Harfe (Karplus-Strong), Cello, Flächenklang, Trommel, Tick und weitere.
- **Musik** (alle in Moll, instrumental, melancholische Seemannsmusik):

| Stück | Wo |
|---|---|
| Titel | Hauptmenü |
| „Salz auf der Haut“ | an Land |
| „Hol die Leine“ | auf See |
| „Atem“ | am Feuer |
| „Lied der Laternen“ | Glutwacht |
| „Glutwind“ | Aschebucht / Shanty Asra |
| „Eisige Leine“ | Frostgrotte / Shanty Lene |
| „Rabensturm“ | Rabenwacht / Shanty Kalo |
| „Tiefer Grund“ | Fluchmangroven / Shanty Ode |
| „Weltschlund“ | Ende der Welt |
| Boss | Bosskämpfe |

- **Ambiente**: Wellen, Wind, Dröhnen (Tod), Feuerknistern nahe Feuern.
- **Klangeffekte** (über 30): Hiebe, Treffer, Parieren, Rolle, Trinken, Kanonen, Pistole, Gebrüll, Glocke, Ketten, Feuer entzünden, Truhe, Stufenaufstieg und mehr.
- Wiedergabe über WebAudio mit Hall und Kompressor. Der Ton wird beim ersten Tippen oder Klick freigeschaltet.

---

## 9. Technik

### 9.1 Aufbau
- `crates/art` – Bibliothek für die prozedurale Kunst (Leinwand, Atlas, Gelände, Figuren, Rig, Schiffe, Requisiten, Schrift).
- `src/gfx` – eigener Renderer: Sofortmodus-Zeichenlisten (Boden, Abziehbilder, y-sortierte Welt, Overlays, UI, Lichter), G-Puffer, Licht-Pass, Bloom, Tiefenschärfe, Endbild mit Kippung.
- `src/world` – Weltgenerator (Inseln, Ebenen, Siedlungen, Requisiten, Kollision) und Weltdarstellung (Streaming).
- `src/game` – Spielablauf, Spieler und Gegner, Bosse, Endkampf, Schiff, Crew, Dialoge, Daten, Stimmung, Effekte.
- `src/ui` – HUD, Menüs, Titel, Pixelschrift, Touch-Steuerung, iPhone-Tastatur.
- `src/synth.rs`, `src/audio.rs` – Musik und Klang. `src/save.rs` – Spielstände.

### 9.2 Ablauf eines Bildes
Eingabe → Spiel-Update (nur Gegner und NPCs in Kameranähe) → Welt zeichnen (sichtbare Boden-Chunks, Requisiten und Deko aus 256-Pixel-Eimern) → Figuren, Effekte, UI → Packen (Welt nach y sortiert, Lichter nach Nähe) → GPU.

### 9.3 Leistung und Laden
- **Streaming**: Nur Boden-Chunks im Bild und in einem Ring darum liegen auf der GPU (50 Plätze). Fehlende sichtbare Chunks werden sofort gebacken. Der Ring entsteht vorab in kleinen Schritten mit höchstens 5 ms je Bild, damit Laufen und Segeln nicht ruckeln. Offene See braucht keine Pixelarbeit.
- **Nahbereich**: Gegner, NPCs und Interaktionen werden nur um Kamera bzw. Spieler herum berechnet.
- **Musik** wird nach Zeitbudget vorberechnet und danach an den Browser übergeben.
- **Speicher**: Atlasseiten werden beim Start in den Upload verschoben statt kopiert (Wasm-Speicher etwa 270 MB mit der großen Welt).
- **Große Welt**: Die Wassermaske hat nur eine Kachel je Texel; die feinen Ufer stecken in den Boden-Chunks. Die Boden-Deko entsteht je Eimer beim Zeichnen. Seeungeheuer, Haie und Gegner rechnen nur nahe am Schiff bzw. an der Kamera. Die Welt entsteht in rund 0,6 s.
- **Messung**: `?debug&start=…&prof` gibt jede Sekunde Bildzeit und Anteile in die Konsole aus.
- Startzeit: rund 3–4 Sekunden für Grafik und Welt.

### 9.4 Spielstände
Profile nur mit Namen (kein Passwort), gespeichert im Browser (localStorage) als JSON. Alte, noch verschlüsselte Profile (PBKDF2-SHA256 + AES-256-GCM) öffnet man ein einziges Mal mit dem früheren Passwort; danach werden sie unverschlüsselt gespeichert. Automatisches Speichern an Feuern. Neue Felder bleiben abwärtskompatibel, alte Spielstände laden weiter: Version 5 verschiebt Perlenfleck und aufgehobene Gegenstände in die große Karte.

### 9.5 Steuerung
| Taste | Aktion |
|---|---|
| WASD / Pfeile | Laufen (auf See: lenken, ↑↓ Segel) |
| J · L | Hieb · schwerer Hieb (auf See: Breitseite links · rechts) |
| Leertaste | Rolle |
| Q | Parieren |
| R · 1 · 2 | Rum · Gürtel 1 · Gürtel 2 |
| K | Pistole bzw. Gezeitenstoß |
| C · X | Zauber wirken · wechseln (auf See: Crew-Geschütz) |
| Tab | Gegner fixieren |
| E | Interagieren, rasten, an Bord, anlegen |
| F | Shanty auf See |
| Alt | Namen aller Gegenstände |
| M · H · I · Esc | Seekarte · Steuerung · Fortschritt · Menü |

iPhone: Touch-Stick links, Knöpfe rechts, hochkant und quer. Am Computer Tastatur und Maus (Hover zeigt Umrisse, Klick in Menüs).

### 9.6 Bauen und Veröffentlichen
- Lokal: `cargo build --target wasm32-unknown-unknown --profile fast`, dann `./build.sh` → `dist/`.
- Veröffentlichen: Push auf `main` → GitHub Actions baut das Release (mit `wasm-opt`) und stellt es auf GitHub Pages (ca. 10–15 Minuten).
- Testspiele ohne Profil über URL-Parameter, z. B. `?debug&start=hain&sail&crew`, `&boss=fire`, `&seaboss=ice`, `&endphase=4`, `&intro` (alle Parameter in `CLAUDE.md`).

### 9.7 Wichtige technische Regeln
- Texturfelder nie mit einer durch 6 teilbaren Ebenenzahl anlegen. WebGL-wgpu hält sie für Würfel-Felder, dann fehlen Sprites bzw. der Boden ist schwarz.
- WebGL erlaubt nur einen Sampler je Textur. Weiche Normalen werden deshalb von Hand gemischt.
- Die Atlasstreifen für Spieler (208), eigenes Schiff (300) und Boss (1520) ergeben zusammen genau eine Seite.
- `cargo fmt` nie auf das ganze Repo anwenden.

---

## 10. Offene Punkte und Ideen
- Feinabstimmung auf echter GPU und iPhone (Helligkeit, Bloom, Leistung), Balancing der Bosse, Gegner und Seeungeheuer.
- Eigene NPCs, Geschichten und Nebenaufgaben für die wilden Inseln.
- Startzeit weiter senken (Figuren- und Kreaturen-Erzeugung machen den Großteil aus).
- Alte Profile der früheren JS-Version werden nicht übernommen.
