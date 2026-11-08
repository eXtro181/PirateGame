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

0. **Prolog – Stillwasser und der Spinnenschlund** (neu gestaltet 8.11.): Das Spiel beginnt auf einer abgelegenen, von extrem hohen Mauern umschlossenen Felsinsel in der Nordwest-Ecke, mit PixelLab gestaltet. Man erwacht **am Tresen der Kneipe „Zur Letzten Flut“**: Gläserklirren im Dunkeln, dann blendet die warme Schankstube auf – Rumfässer, Kisten, eine Kanone, die Piratenflagge, Kerzen auf dem langen Tresen, dahinter der kahle, bärtige Wirt Klaas, an den Tischen zwei Zecher. „Na, endlich wach?“ Draußen liegt **Stillwasser**, ein helles Dorf im Morgenlicht voller freundlicher **Ertrunkener** (fahle, meergrüne Haut, nasses Haar, Tang an der Kleidung, schwach leuchtende Augen), die nicht wissen, dass sie tot sind: ein Dorfplatz mit Brunnen und Marktständen, Fachwerkhäuser, die Ecke der Fischer mit Netzen und Booten, Obstwiesen jenseits einer Brücke über den Bach. Es ist immer Morgen, das Bier schmeckt nach Salz, die Netze sind immer leer, und alle raten: Geh nicht nach Süden. Am Dorfrand warnt Matrose Jesko. Wer weitergeht, kommt auf den **Galgenweg**: es dämmert, Bannpfähle mit Schädeln, Galgen mit Gehängten, Käfige, ein Gräberfeld, tote und bleiche Bäume. Dann Gebein – Schädelhaufen, sitzende Tote, Knochen – und der **Spinnenschlund**, eine dunkle Schlucht aus Felsblöcken, Netze auf Pfählen und über dem Boden, Kokons, Eier, nur Leuchtpilze geben Licht; im Westen ein Wrackfriedhof mit drei Schiffen im Morast. Licht, Nebel und Musik folgen dem Weg: im Dorf ein Shanty, im Dunkeln die Tiefe, zuletzt Stille. Je näher man der Spinne kommt, desto lauter hört man ihre Schritte – erst ein Rascheln, dann das Klicken vieler Beine auf Fels –, und drei Vorahnungen ziehen durch den Kopf: „… ein leises Rascheln …“, „… was bewegt sich da …?“, „… Schritte. Viele Beine. Ganz nah …“. Im Süden bricht die Insel an einer Klippe ab, über die ein extrem tiefer Wasserfall stürzt; das Meer an seinem Fuß ist völlig schwarz. Hier lauert die **Riesige Meeresspinne**, der erste Boss – nicht schwer, besiegbar, aber nicht zwingend; andere Gegner gibt es dort nicht. Sie hängt von Anfang an in der Klippenwand westlich des Wasserfalls, nur ihre Augen und Beine ragen über die Kante; nähert man sich dem Wasserfall, krabbelt sie die Wand herauf und springt mit einem Aufschlag auf die Platte – der Kampf beginnt. Ist der Kampf entschieden, steigt neben dem großen Wasserfall an der Abbruchkante Nebel auf, und ein offener **Sarg** erscheint, der Deckel lehnt daneben; links und rechts entzündet sich je eine sehr helle Piratenfackel (Eisenkorb auf einem Pfahl mit Schädel). Vorher ist dort nichts als nackter Fels. Wer die Spinne besiegt, steigt hinein, legt sich hin und zieht den Deckel zu. Wer fällt, stirbt nicht einfach: die Spinne wickelt den Leichnam in einen weißen Kokon, zieht ihn an einem Faden hinter sich her bis zum Sarg, hebt ihn mit ihren Beinen hinein, schiebt den Deckel zu und krabbelt dann selbst kopfüber die Klippe hinab, bis die Tiefe sie verschluckt. In beiden Fällen schlängeln sich erst dann langsam zwei riesige Arme aus der schwarzen Tiefe die Klippe herauf, umschlingen den Sarg und ziehen ihn hinab ins Dunkel – und man erwacht im ursprünglichen Sarg am Galgenstrand; das reguläre Spiel beginnt.
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
- 1776 × 1328 Kacheln à 32 Pixel (56 832 × 42 496 Weltpixel) – zehnmal die Fläche der ersten Karte. **Strahlenförmig und ausgewogen**: die Glutwacht in der Mitte, der Galgenstrand gleich südlich davon, die vier Königsinseln in den vier Diagonalen (Frostgrotte im Nordwesten, Aschebucht im Nordosten, Rabenwacht im Südwesten, Fluchmangroven im Südosten), die vier Seegebiete weit draußen in den Himmelsrichtungen, dazwischen 64 große wilde Inseln. Das Ende der Welt liegt in der Südost-Ecke, die Prolog-Insel abseits in der Nordwest-Ecke.
- Jede Insel hat Höhenstufen – wie in A Link to the Past (8.11.): wenige große, klar umrissene Plateaus statt verstreuter Hügel (wilde Inseln höchstens zwei Stufen, Königsinseln drei), Strände flach. Jede Fläche ist vom Ufer aus über Treppen erreichbar; wo ein Aufgang fehlt, wird einer in die Felswand gehauen.
- Außerhalb der Festung sind Ebenenkanten **Erd- und Felswände** (nach Süden 96 px hoch, seitlich 22 px schmale Felswände mit Graskante, oben eine dünne Kante mit Lichtsaum): oben eine überhängende Graslippe mit Lichtkante und tiefem Unterschnitt, ein kurzer bewachsener Hang, darunter unregelmäßige Felsbrocken mit dunklen Fugen, Regenspuren, Moos (im Frost Eiszapfen, in der Aschebucht Glut in den Fugen), nach unten immer dunkler, am Fuß Geröll; die Wand wirft einen Schlagschatten auf die tiefere Ebene. So liest man jede Höhenstufe sofort als Kante (Nutzerwunsch 7.11.: der 3D-Effekt war zu schwach).
- **Treppen** (in der Natur Steinstufen in der Wand) gibt es nur dort, wo man wirklich hinauf oder hinunter will: Aufgänge ohne Ziel (zu leeren Plateaus) wurden entfernt, winzige Hochland-Inseln eingeebnet. In der Natur führen Wege im Gebietsstil über die Hänge (Schneepfad, Bohlen im Moor, Holz- und Steinstufen im Hang). Höhenunterschiede sind klar abgesetzt: helle Oberkanten, Schwellen und Stufennasen, Schlagschatten am Fuß der Hänge und Mauern.
- Herunterspringen geht genau eine Ebene tief.
- **Wasserfälle** sitzen an echten Hangkanten mit genug ebenem Land darüber: Ein Bach entspringt einem kleinen Quellteich mit Felsrand und Schilf, schlängelt sich zur Kante und stürzt erst dort als Kaskade mit Schaumstufen den Hang hinab; unten ein Becken mit Gischt.
- **Kollision**: Durch Büsche, Stauden, Farne und Schilf läuft man hindurch, über Kokons, sitzende Tote, Schädelhaufen und Knochensärge hinweg. Bäume halten nur ihren Stamm.

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
| **Wrackzwinger** (`prolog`) | Prolog | helles Dorf Stillwasser, Galgenweg, Spinnenschlund, schwarzer Abgrund | – | Riesige Meeresspinne |

Jede Siedlung hat Dorfbewohner, Hühner und Katzen, Rauch aus den Schornsteinen und einen Laden mit **begehbarem Innenraum**. Gegner halten sich von Siedlungen fern.

**Wilde Inseln**: 64 große Eilande (mittel, groß und einige sehr große; zu kleine gibt es nicht mehr), gleichmäßig zwischen Glutwacht, Königsinseln und Seegebieten verteilt, mit Namen wie „Reifholm“, „Glutzahn“, „Moderwerder“, „Sturmklippe“ oder „Schädelriff“. Ihr Biom richtet sich nach der nächsten Königsinsel bzw. dem nächsten Seegebiet. Jede hat einen Anleger zur Kartenmitte, einen Landeplatz und ein Herzstück:
- **Piratenlager**: Zelte, Tafel, Kanonen, Fässer, Flagge, Truhe.
- **Ruine**: Marmorplatz, Säulenring, Bogen, Wächterstatue, Kerzen, Schwertgrab, Truhe.
- **Friedhof**: Galgen, Gräberreihen, Särge, Käfig, Fackeln, manchmal eine lebende Truhe.
- **Wrackstrand**: gestrandetes Schiff, Fracht, Anker, Netze, Ruderboot, Haiskelette, manchmal ein Mimic.
- **Wilder Hain**: dichter Wald, Leuchtblumen bzw. Kristalle, Seelenblume.
- **Schatzinsel**: zwei Truhen (Rumflasche, Seelenphiole oder Gezeitensplitter) mit starkem Wächter.

Je weiter draußen, desto höher die **Gefahrenstufe** (0–3): Gegner haben mehr Leben (+35 % je Stufe), mehr Schaden (+12 %) und geben mehr Perlen (+40 %). Auf jeder vierten Insel brennt ein Säbelfeuer (Schnellreise).

**Schädelhorn, die PixelLab-Insel**: eine der wilden Inseln (Nr. 25, Gefahrenstufe 0, mit Säbelfeuer, zwischen Galgenstrand und Glutwacht) ist mit Grafik aus PixelLab neu gebaut – ein Versuch, wie sich KI-Pixelkunst ins Spiel fügt. Sie ist flach, ihr Boden besteht aus fertigen Kacheln (grauer, nasser Kiesstrand am Ufer, dunkles Moosgras, im Inneren Waldboden mit Wurzeln und Laub), dazu Zypressen, tote Bäume, Palmen am Strandsaum, Farne, Dornbüsche mit roten Beeren, Findlinge, ein Wrackbug und ein Ruderboot am Strand, Fässer und Kisten am Landeplatz und im Herzen ein überwucherter Ruinenbogen mit glimmender Feuerstelle, Truhe und Schädelpfählen mit roten Kopftüchern.

### 3.3 Glutwacht im Detail
Eine **kreisrunde Hafenfestung** nach dem Vorbild des Feuerband-Schreins, ganz im düsteren Piraten-Stil: extrem hohe Ringmauern mit einer Zinnenbrüstung, die lückenlos dem Kreis folgt (jedes Segment im echten Winkel, Pfeiler an den Stößen, Türme darauf), Fackeln und Bannern umschließen mehrere Ringterrassen, die übereinander liegen und durch Treppen verbunden sind. In der Mitte ein See, über den sich eine **hohe, stützenlose Steinbrücke** in einem einzigen Bogen von West nach Ost spannt – 150 px über dem Strudel, mit Brüstungen, Laternen, hängenden Ketten und Ranken, an beiden Enden Brückenköpfe mit Feuerschalen; Schiffe fahren darunter durch, Strudel und Krakenarme liegen darunter. Darum der Ring der brennenden Bossschiffe. Am Nordkai kniet die Sündenstatue. Darüber die Feuerterrasse mit Säbelfeuer, vier Thronen, Schmied und Händlerin, höher der Schrein mit Bergfried und Wachgängen auf den Mauern. Im Süden schneidet der Seetor-Kanal durch die Ringmauer; das **Seetor** steht in der Ebene der äußeren Mauerfront auf zwei Pfeilern im Wasser und reicht bis zur Mauerkrone, ein Spitzbogen über die ganze Einfahrt mit hochgezogenem Fallgitter, Zinnen, Bannern und Fackeln. Im Osten der Anleger. Dort liegt die **Werft** von Schiffsmeister Brack: Plankenkai, Helling mit Rumpf, Kran, Taue, Fässer, Kanone, Laternen, Flagge und Wegweiser. Die Mauern bestehen aus Quadern, Ebenen sind hier echte Mauern mit Treppen.

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

- **Spieler**: der Ertrunkene oder die Ertrunkene. Im Editor Geschlecht (Frau/Mann), Statur, Hautton, Haare, Bart (nur Männer), Kopfbedeckung, Mantel, Augenklappe; der Knopf „Zufall“ würfelt Aussehen und einen Namen (Vorname nach Geschlecht, zusammengesetzter Nachname wie „Brida Rosthaken“). Herkunft und Geschenk zu Beginn.
- **Glutwacht**: Laternenmagd (Stufenaufstieg, Neues Spiel+), Schmied Harrow (Waffen härten und kaufen), Händlerin Sabeth (Waren), Schiffsmeister Brack (Ausbauten), Crew an der Tafel.
- **Start**: Alter Galionsschnitzer; auf dem Steg der **Geisterkapitän** mit dem Flaschenschiff.
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

### 5.1b Die Riesige Meeresspinne (Prolog)
Der erste Boss im Spinnenschlund des Prologs: eine gewaltige (gut 30 % größer als zuerst), rostig gebänderte Meeresspinne mit borstig behaarten Beinen, Stacheln und Tang, einem aufgeblähten Hinterleib voller dunkler Adern, nässender und geplatzter Eiterbeulen, Pilzbefall, sich hineinbohrender Würmer und einem Eiersack, offenen Wunden am Kopfbruststück, einer Traube aus fünfzehn leuchtenden und einem milchig-blinden Auge und Geifer, der von den Klauen tropft (3D-Rig). Sie ist über und über mit Miesmuscheln und Seepocken bewachsen, Tangsträhnen hängen von Leib und Beinen, Moospolster sitzen auf dem Rücken, und zwei ihrer acht Beine sind Holzbeine – Pflöcke aus Schiffsplanken mit Eisenringen und einem eisernen Schuh. Sie lauert in der Klippenwand (eigene Kletterbilder: Rücken zur Kamera, Kopf oben bzw. kopfüber), krabbelt herauf, wenn man sich dem Wasserfall nähert („Eingesponnen“ statt einer Kälte-Anzeige), schleudert **Netze**: Sie bäumt sich auf und speit drei Netzklumpen im hohen Bogen – einen auf den Spieler, zwei daneben –, ein Faden reicht noch zurück zu ihrem Maul, im Flug entfalten sie sich drehend zu runden Netzen, ihr Schatten zeigt, wo sie aufschlagen. Ein Volltreffer spinnt den Spieler ein (festgehalten, Angreifen oder Ausweichen reißt die Fäden), sonst bleibt das Netz einige Sekunden am Boden kleben, verlangsamt jeden, der hindurchgeht, und zerreißt, wenn man hindurchrollt. Dazu lässt sie Säure einschlagen. Nicht schwer und nicht zwingend: Sieg und Niederlage führen beide zum Sarg am Wasserfall (siehe Prolog).

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

**Kampfgefühl** (Runde 13): Jeder Treffer hat Gewicht – ein kurzer **Trefferstopp** (leicht 45 ms, schwer 85 ms), ein **gerichteter Kamerastoß** in Hiebrichtung, **Rückstoß** (die Gegner gleiten aus), Blutspritzer bzw. Knochenstaub am Boden und **Stofflichkeit** (Knochen splittern, Panzer und Stein sprühen Funken, Geister zerfließen). **Paraden** werfen Funken, frieren das Bild für einen Augenblick ein (Aufprallbild) und gehen in eine kurze **Zeitlupe** über; ein Treffer im ersten Augenblick der Rolle ist ein **perfektes Ausweichen** (Zeitlupe, Nachbilder, Ausdauer zurück). Bosse haben eine **Haltung**, die sich mit Treffern füllt – voll, taumeln sie und der **Gegenstoß** wird zur **Hinrichtung** (Kamera nah heran, Zeitlupe). Eingaben werden kurz **gepuffert**, der Nachschwung eines Hiebs lässt sich mit einer Rolle abbrechen, die Figur läuft mit kurzem **Schwung** an. Gegner kündigen Hiebe mit einem **Ausholglanz** an und werden mit sinkendem Leben sichtbar verwundet. Fässer und Kisten zerbrechen unter Hieben. Bossauftritte und -tode haben Kinobalken, Kamerazoom und einen musikalischen Stinger; in der zweiten Phase verwandelt sich die Arena mit dem Element des Bosses (Hitze und Asche, Schneesturm, Gewitter, Regen). Wenig Leben färbt die Welt grau und lässt den Bildrand im Herzschlag rot pochen; die Laterne der Figur wird mit sinkendem Leben kleiner und rötlicher.

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
- **Das Flaschenschiff**: Am Galgenstrand wartet auf dem Steg ein Geisterkapitän (durchscheinend, schwebend, mit Geisterlaterne). Er übergibt sein Schiff, das er beim Untergang in eine Flasche gebannt hat, und löst sich danach in Nebel auf. Beim ersten Ablegen wirft man die Flasche vom Steg ins Meer: Sie fliegt im Bogen, das Glas zerspringt in einem Lichtblitz mit Scherben und Gischt, das Wasser brodelt, und das Schiff steigt zum ersten Mal vom Grund auf.
- **Rufe dein Schiff**: Danach ruht das Schiff in der Tiefe und kommt an jedem Steg, wenn man es ruft – als Schauspiel: Das Wasser rumpelt und beginnt zu kreisen, ein dunkler Schatten wächst, grünes Licht glimmt aus der Tiefe, Wellenringe laufen aus. Dann schießt eine Fontäne hoch, die Masten durchstoßen die Oberfläche, das Schiff springt über die Wasserlinie, krängt und kracht zurück ins Meer (Gischt rundum, drei Wellenringe, Gischtnebel, ganze Wasserfälle stürzen von Deck), schaukelt aus, und man geht an Bord. Beim Anlegen (E) ächzt es, legt sich zur Seite, ein Sog mit Licht aus der Tiefe zieht es hinab, und zuletzt schießt über den verschwindenden Mastspitzen eine Fontäne auf.
- **Das Schiff**: eine schwarze Galeone – schwarzer Lack mit Goldleisten und ochsenblutrotem Band, geflickte, zerfetzte schwarze Segel, rote Wimpel, Totenkopfflagge, Heckgalerie mit leuchtenden Fenstern, Hecklaternen, Wanten mit Webeleinen, Anker, ein geflügelter Totenengel als Galion.
- **Steuerung**: links/rechts wenden, hoch/runter Segelstufen (je Stufe 74 px/s). J/L feuern Breitseiten nach Backbord bzw. Steuerbord, C setzt das Crew-Geschütz ein, F stimmt das Shanty der Crew an (das Schiff wird schneller).
- **Seekarte** (M): gezeichnet wie auf vergilbtem Papier, deckt sich beim Erkunden auf (Felder à 16 × 16 Kacheln). Drei Zoomstufen (Ganze Welt, Umgebung, Nah), schieben mit den Richtungstasten. Namen der wilden Inseln erscheinen, sobald man dort war.
- **Ausbauten** beim Schiffsmeister, je bis Stufe 20: Tempo (bis 10 je +6 % Fahrt, darüber je +3 %), Verteidigung (bis 10 je +24 Rumpf und −4 % Schaden, darüber je +16 und −2 %), Angriff (bis 10 je +12 % Schaden, darüber je +8 %, mehr Kugeln je Breitseite). Kosten bis 10 wie früher (×1,45 je Stufe), darüber ×1,16 je Stufe. Die Stufen sind am Schiff sichtbar: mehr und größere Segel, Leesegel, Bramsegel, Besantoppsegel, weitere Klüver, auf 20 geisterhaft schimmernde Säume; Eisenbänder, Panzerplatten bis zur Wasserlinie, Goldkanten, Rammsporn, Eisenkappe am Bug, Dornen auf der Reling, gepanzertes Heck, Ketten, auf 20 leuchtende Runen; mehr Stückpforten, zweite Batterie, Drehbassen, Mörser, Jagd- und Heckgeschütze, Karronaden, zweiter Mörser, goldbeschlagene Rohre.
- Kapitän und Crew stehen an Deck und drehen sich mit dem Kurs.

### 6.6b Feindschaft und die Sündenstatue
Alle Figuren lassen sich angreifen, sind aber friedlich. Wer in einer Siedlung (Glutgrund, Reifstein, Rabenwacht, Moorhain – und auch in der Glutwacht) eine Figur schlägt, bringt den ganzen Ort gegen sich auf: Die Dorfbewohner greifen als bewaffneter Mob mit Knüppeln, Mistgabeln und Messern an, Händler, Schmied, Schiffsmeister und Laternenmagd verweigern Gespräch und Handel („Mörder! Mit dir rede ich nicht.“), in der Glutwacht stehen Wachen bei ihnen. Die Feindschaft bleibt gespeichert, der Mob kehrt nach jeder Rast zurück. Außerhalb der Siedlungen zuckt eine geschlagene Figur nur zusammen. Am Nordkai der Glutwacht kniet die **Sündenstatue** – ein riesiger Büßer in Ketten mit glühenden Augen und einer Opferschale voller Glut. Dort kauft man für 1 500 Perlen (× 1,5 je Neues-Spiel+-Fahrt) den Frieden mit einem Ort: Der Mob verschwindet, alle kehren an ihre Plätze zurück.

### 6.7 Fortschritt, Neues Spiel+, Cheats
- **Fortschritt** (Taste I): Könige, Seeungeheuer, Crew, Strudel, Endkampf, aktuelle Fahrt.
- **Neues Spiel+**: Könige, Seeungeheuer, Throne und Endkampf werden zurückgesetzt. Stufe, Ausrüstung, Schiff, Crew, Feuer und Schätze bleiben. Gegner-Leben × (1 + 0,5 · Fahrt), Schaden × (1 + 0,3 · Fahrt), Bossperlen +25 % je Fahrt.
- **Pause-Menü**: Cheat „Unendlich Perlen“ (umschaltbar, der alte Stand bleibt erhalten). **Debug-Menü**: Trophäen der Könige und Seeungeheuer, ganze Crew, alle Säbelfeuer.

---

## 7. Grafik

Die vollständige Analyse mit allen Formeln, Farbwerten, Shadern und einer Schritt-für-Schritt-Anleitung zum Nachbau steht in [`GRAFIK_ANLEITUNG.md`](GRAFIK_ANLEITUNG.md).

### 7.1 Vorgaben
- **HD-Pixelart** mit dem Detailgrad der Beispielbilder: Blütenbäume mit Glyzinien, Basaltklippen, türkises Wasser, Fachwerkhäuser, eine Rotnebel-Zone.
- **PixelLab-Stil überall** (Nutzerwunsch 7.11.): was nicht aus PixelLab stammt, wird nachgebildet – alle Sprites mit gestuften Tönen, geglätteten Pixelinseln und einem Umriss in einer Farbe, die Bodentexturen mit ruhigem Grund und kleiner Tonleiter wie die PixelLab-Kacheln.
- **Natürliche Vegetation**: Pflanzen wachsen dort, wo sie auch auf der Erde wüchsen – keine bunten Fantasiebäume und keine wahllos verstreute Deko.
- **Düster und abwechslungsreich** (Nutzerwunsch 7.11.): gedämpftes Nachtgrün, verdorrte und vereinzelt blutrote Kronen, knochenbleiche und bereifte Totbäume, Trauerweiden an stillen Ufern, Leuchtpilze als kalte Lichtinseln im Wald, Blut- und Geisterblumen; Blumen der Boden-Deko in knochenweiß, ocker, violett, geisterblau und blutrot.
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
- **Effekte aus PixelFX** (<https://github.com/patch-gg/pixelfx>, MIT): Säbelhiebe in Stahl, Doppelhieb in Messing, Stöße, Klauenhiebe, Trefferfunken, kritische Treffer, Paraden, dumpfe Treffer mit Staub, Landungen, Bodenschläge, Mündungsfeuer, Kugeleinschläge, Explosionen, Flammensäulen, Feuerringe, Frostnovas, Eissplitter, Blitzschläge, Blitztreffer, Erdstacheln, Säure, Platschen, Geysire, Heilung, aufsteigende Seelen in der Gestalt der Figur, zersplitternde Skelette, die Wiedergeburt am Säbelfeuer, Aufsammeln, Beutestrahlen über Fundstücken, Fackel- und Feuerschalenflammen, brennende Wracks und die Geisterflamme der Säbelfeuer. Sie ersetzen die früheren eigenen Hiebbögen, Druckringe und Flammenbilder, werfen Licht und werden von Figuren und Mauern davor verdeckt.
- Option „Sparsam“ lässt nur die Kippung an (für schwache Geräte).

### 7.3 Prozedurale Kunst (alles aus Code)
Jedes Bild ist eine **Leinwand mit vier Kanälen**: Farbe, Normale, Höhe über Grund und Leuchten/Glanz. So kann der Renderer jedes Pixel beleuchten und Schatten werfen lassen.

- **Figuren-Maler** (`chars.rs`): Figuren 64 × 72 Pixel, drei Richtungen (vorn, Seite gespiegelt, hinten), alle Animationen. Über 40 Vorlagen: Gegner, NPCs, Crew, Dorfbewohner, Könige, Malakor (vier Arme, Tentakelbart, Korallenkrone). Bausteine wie Hüte, Mäntel, Bärte, Flügel, Schlangenhaar, Uhrwerk, Leuchtaugen, Auren. Für alle Menschen gleich: Paspeln an der Mantelöffnung, Saum mit Umschlag, Taschenklappen mit Knopf, Bandelier mit Messingschnalle, Gürteltasche, helle Manschetten, Stiefelstulpen mit Schnalle, Glanz in den Augen; ein Feinschliff je Bild setzt Formlicht an die Kanten, Gewebe- und Faltenstruktur, Haarsträhnen, Metallglanz und Mikrorelief für die Beleuchtung.
- **Kreaturen-Rig** (`rig.rs`, `beasts.rs`): Wesen bestehen aus Ellipsoiden, Quadern, Röhren, Häuten und Dreiecken im Raum. Ein Strahlenwerfer rendert sie aus der 3/4-Kamera (Neigung 0,55) mit Mustern (Schuppen, Fell, Nieten …), Innenkonturen, Höhlenschatten in Ansätzen und Kehlen, Glanzlichtern auf Metall und Umriss. Er schreibt die Höhe je Pixel, damit Schatten der Grundfläche folgen.
- **Schiffe** (`ships.rs`): ebenfalls als Rig. 16 Kurse (9 gerendert, der Rest gespiegelt), das eigene Schiff als detailreiche schwarze Galeone im Pixelstil (`build_black`) mit sichtbaren Ausbauten bis Stufe 20, vier Bossschiffe mit eigenem Entwurf und der Fliegende Holländer (Maßstab 6,9).
- **Gelände** (`terrain.rs`): Der Boden wird in 256-Pixel-Stücken gebacken. Materialien wie Gras, Wiese, Sand, Erde, Pflaster, Marmor, Planken, Basalt, Asche, Schnee, Eis, Schlamm, Fels und Moos haben je Gebiet eigene Paletten. Dazu weiche Materialgrenzen, nasse Uferkanten, Ufersteine, Hänge, Festungsmauern aus Quadern und Naturwege.
- **Requisiten** (`props.rs`, `flora.rs`, `buildings.rs`, `structures.rs`, `grave.rs`): Bäume mit Kronen, die durchsichtig werden, wenn man dahinter steht (in der Natur die PixelLab-Arten: Eichen, Laubbäume, Kiefern, Schneefichten, Zypressen, tote und verkohlte Bäume, Palmen, dazu Mangroven), Büsche, düstere Fachwerkhäuser (Schieferdächer mit Moos, verrußter Putz, vernagelte Fenster, Schädel über der Tür), Bannpfähle mit Schädeln, Galgen mit Käfig, Festung, Werft, Galgen mit schwingenden Gehängten, Särge, Wracks, Eisberge, Korallen, hohe Mauern mit Zinnen, Spinnennetze und Kokons. Die **Truhe** ist ein 3D-Modell (Plankenkorpus, gewölbter Deckel, Eisenbänder mit Nieten, Eckbeschläge, Messingschloss; offen mit glimmenden Münzen und Edelsteinen).
- **Atlas**: alles auf Seiten à 2048² (aktuell gut 5) plus eine Effektseite für die PixelFX-Effekte, die beim Start in wenigen Sekunden Bild für Bild gezeichnet werden. Feste Streifen für Spieler, eigenes Schiff und den gerade aktiven Boss werden bei Bedarf neu gezeichnet.
- **Schrift und UI**: eigene Pixelschrift mit Umlauten und Sonderzeichen für Fließtext, dazu eine **Zierschrift** (Antiqua-Versalien wie gemeißelt, aus Breitfeder-Strichen mit Keilserifen) für Titel, Banner, Ortsnamen, Fenstertitel und Knöpfe. Rahmen, Symbole.
- **Natürlicher Bewuchs** (`world/nature.rs`): Wald wächst in Flecken und wird zum Rand hin lichter, mit Lichtungen und einzelnen Solitärbäumen im Offenland. Die Arten folgen dem Standort: Nadelbäume auf den Höhen, Auwald und Mangroven am Wasser, Palmen nur am Sandstrand, in der Frostgrotte eine Schneefichten-Taiga, in der Aschebucht verkohlte Bäume mit Glut in den Rissen. Unter den Kronen Farne, Pilze und umgestürzte Stämme, am Waldrand Sträucher, im Offenland Gras in Horsten und Blumenkolonien einer Art auf Wiesenflecken, Schilf am stillen Ufer, Strandgras auf der Düne, Treibholz am Flutsaum, Geröll am Hangfuß.
- **Boden-Deko** (Gras, Blumen, Klee, Farnsprossen, Steinchen, Zweige, Laub, Tang, Muscheln, Treibholz, Wurzeln, Pilze, Glut, Seerosen, Schilf) entsteht je 256-px-Eimer erst beim Zeichnen, ohne Speicher und Ladezeit – nach denselben Standortregeln: Laub, Zweige und Pilze nur im Schatten der Bäume, Tang und Muscheln am Flutsaum, Seerosen nur in Teichen und im Sumpf, Rumflaschen, Taue und Münzen nur auf Planken und in Siedlungen; keine verstreuten Schädel und Knochen mehr.

### 7.4 Zonenstimmung
Jede Zone hat eine eigene Licht- und Farbstimmung (`env.rs`), die weich eingeblendet wird: Grundlicht, Sonne, Nebelfarbe, Wasserfarben, Belichtung, Lichtstrahlen und ein **Farbskript** (Farbtrennung in Schatten und Lichter, z. B. violette Schatten und Glut-Lichter in der Glutwacht, grüne Schatten und fahles Gelb in den Fluchmangroven). Stimmungen gibt es für Galgenstrand, Glutwacht, Aschebucht, Frostgrotte, Rabenwacht, Fluchmangroven, Ende der Welt, Innenräume, die Stille vor dem Endkampf, das Gewitter beim Holländer, den Meeresgrund und jedes Seegebiet.

### 7.4b Wetter, Tageszeit und Leben (Runde 13)
- **Tageszeit**: ein Tag dauert eine halbe Stunde; in der zweiten Hälfte sinkt die **tiefe Nacht** über die Zonenstimmung – Mondlicht, dunklerer Nebel, hellere Laternen, Sterne spiegeln sich in der See. Prolog, Innenräume und der Endkampf behalten ihre feste Stimmung.
- **Wetterfronten** je Gebiet: Regen (Galgenstrand, Rabenwacht fast immer, Fluchmangroven, wilde Inseln, offene See), Schnee (Frostgrotte, Eismeer), Asche (Aschebucht, Glutwacht, Ende der Welt), dazu Wind. Fronten ziehen in 40–110 s auf und ab; mit ihnen zieht die Wolkendecke zu, Lichtstrahlen fallen nur durch Wolkenlücken. Regen fällt schräg im Wind und spritzt auf, die Welt wird **nass** (dunkler, glänzend, Pfützen mit Regenringen, Ringe auf dem Wasser) und trocknet langsam; Schnee und Asche legen sich auf Dächer, Felsen und Kronen; in der Kälte steht der Atem als Wolke vor dem Mund.
- **Wind** bewegt Laub, Halme, Nebel und Teilchen in seiner Richtung; Gras, Farne und Schilf biegen sich vor der Figur weg.
- **Wasser**: Wolken spiegeln sich, Laternen werfen zitternde Lichtpfade, das Schiff rollt im Seegang und krängt in Kurven, hinter dem Heck läuft ein Kielwasser auseinander, bei voller Fahrt sprüht Gischt am Bug. Fischschwärme kreisen in Ufernähe und stieben vor dem Schiff auseinander, auf hoher See treibt Wrackgut. Am Fuß der Wasserfälle steht bei Sonne ein Regenbogen im Sprühnebel, der Strudel zieht drei Schaumarme in seinen Schlund.
- **Feuer**: Säbelfeuer knistern mit aufsteigender Glut und Funkenregen, über Feuer und Lava flimmert die Luft, leuchtende Flächen (Lava, Glut, Leuchtpilze) erhellen ihre Umgebung.
- **Lesbarkeit**: Die Spielfigur hat im Dunkeln einen kühlen Rand und bleibt hinter Bäumen, Mauern und Dächern als Silhouette sichtbar. Außerhalb von Kämpfen blendet sich die Anzeige nach einigen Sekunden aus; Ortsnamen erscheinen Buchstabe für Buchstabe.

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

- **Ambiente**: Wellen, Wind (stärker mit dem Wetter), Regen (eigene Lage, unter Dach gedämpft), Dröhnen (Tod) und eine **dunkle Spannungslage**, die mit Gegnern in der Nähe und im Bosskampf anschwillt, Feuerknistern nahe Feuern.
- **Bewusste Stille**: beim Bossauftritt schweigt die Musik einen Atemzug lang; im Prolog verstummt sie, sobald man die Spinne hört. **Stinger** beim Sieg über einen Boss, leise Glocken beim ersten Betreten eines Ortes.
- **Raumklang**: Treffer, Tode und Explosionen erklingen links/rechts nach ihrer Lage zur Kamera und leiser mit der Entfernung. Schwere Hiebe haben eine zweite, dumpfe Klanglage, die Figur stößt beim schweren Hieb einen Kampflaut aus (kein Wort).
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
- **Große Welt**: Die Wassermaske hat nur eine Kachel je Texel; die feinen Ufer stecken in den Boden-Chunks. Die Boden-Deko entsteht je Eimer beim Zeichnen. Seeungeheuer, Haie und Gegner rechnen nur nahe am Schiff bzw. an der Kamera. Die Welt entsteht in rund 0,65 s.
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
