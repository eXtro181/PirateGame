# Spielwelt und Inhalte

## Inseln und Zonen

**Inseln/Zonen** (`ZONES`): Galgenstrand `kueste` (Start, Piratenfriedhof mit über 70 Toten, Erwachen im Sarg `Cut::Wake`,
auf dem Steg der Geisterkapitän mit dem Flaschenschiff;
Feuer und Sarg liegen auf ebenem Strand – `World.low_areas` senkt den Bereich auf Meereshöhe, sonst lief die Böschung der Anhöhe darüber;
keine Siedlung), Glutwacht `hain` (Festung/Hub), Aschebucht `asche` (Glut, Siedlung Glutgrund), Frostgrotte `frost` (Siedlung Reifstein),
Hafen Rabenwacht `raben` (Stadt), Fluchmangroven `mangroven` (Pfahldorf Moorhain), Ende der Welt `ende` (Meer in der Südost-Ecke,
nur per Schiff durch den Strudel), **Wrackzwinger** `prolog` (Zone `Prolog`, Nordwest-Ecke, nur zu Beginn; Ortsnamen nach Bereich:
Stillwasser, Galgenweg, Spinnenschlund).

## Prolog

**Prolog** (`save.prolog`, neue Spielstände beginnen hier; neu gestaltet 8.11.): ummauerte Felsinsel aus PixelLab-Grafik, von Norden nach Süden immer düsterer.
Man erwacht am Tresen der Kneipe **„Zur Letzten Flut“** (`Cut::Washed`:
Gläserklirren im Dunkeln, die Kneipe blendet auf, man sitzt zusammengesunken vor der Theke und steht auf, „Wirt Klaas:
Na, endlich wach?“) in **Stillwasser**, einem hellen Dorf voller freundlicher Ertrunkener (Lichtstimmung `prolog_dorf`: heller Morgen, Lichtstrahlen;
in der Kneipe `kneipe`; Musik „leine“), die nicht wissen, dass sie tot sind (Dialoge:
Wirt, zwei Zecher, Fischersfrau Ebba, der alte Hinnerk, Matrose Jesko am Dorfrand, der vor dem Süden warnt).
Nach Süden wird es düster: der **Galgenweg** mit Schädeln, Galgen, Käfigen und Gräbern in der Dämmerung, dann Gebein und der **Spinnenschlund**
– eine dunkle Schlucht aus Felsen, Netzen, Kokons und Eiern, nur Leuchtpilze geben Licht – bis zur Klippe mit dem extrem tiefen Wasserfall,
das Meer an seinem Fuß tiefschwarz.
Licht, Nebel und Musik folgen dem Weg (`game/prolog.rs`).
Je näher man der lauernden Spinne kommt, desto lauter und häufiger hört man ihre Schritte (fern „rustle“, nah „skitter“, Lautstärke über `Game.sfx_vol`),
die Musik verstummt, und drei Vorahnungen erscheinen als leise Zeile (`Ui::whisper`): „… ein leises Rascheln …“ (1500 px), „… was bewegt sich da …?“
(1000 px), „… Schritte.
Viele Beine. Ganz nah …“ (640 px).
Erster Boss: die **Riesige Meeresspinne** (nicht schwer, nicht zwingend; Netzschüsse, Säure-Einschläge;
voller Muscheln, Seepocken, Tang und Moos, zwei Holzbeine).
Sie lauert von Anfang an in der Klippenwand westlich des Sargs und klettert herauf, sobald man sich dem Wasserfall nähert.
Außer ihr gibt es dort keine Gegner.
Das Ende spielt am **Sarg neben dem großen Wasserfall** (`Cut::Coffin`, `coffin.rs`), links und rechts je eine sehr helle Piratenfackel
– Sarg und Fackeln erscheinen erst, wenn der Kampf entschieden ist:
nach dem Sieg (`save.spider`, Banner „Am Wasserfall wartet ein Sarg“, kein Hinweis auf einen Sprung) „In den Sarg steigen“ (`Use::Coffin`;
vorher nur „Ein leerer Sarg“) – man steigt hinein, legt sich hin, der Deckel gleitet zu;
nach einer Niederlage (Tod im Prolog → `start_coffin(true)` statt des normalen Sterbens) wickelt die Spinne den Leichnam in einen weißen Kokon,
zieht ihn bis zum Sarg, legt ihn mit den Beinen hinein, schließt ihn und krabbelt selbst über die Klippe in die Tiefe.
In beiden Fällen schlängeln sich danach zwei riesige Arme aus der schwarzen Tiefe die Klippe herauf, umschlingen den Sarg und ziehen ihn langsam hinab
→ `end_prolog`: Erwachen im ursprünglichen Sarg am Galgenstrand (`Cut::Wake`), das Spiel beginnt.
Im Prolog kein Perlenfleck, keine Seekarte.

## Weiter Ozean

**Weiter Ozean**: 64 große wilde Inseln zwischen Glutwacht, Königsinseln und Seegebieten (Namen wie „Reifholm“, „Glutzahn“, „Schädelriff“;
Gefahr steigt nach außen), jede vierte mit Säbelfeuer;
**Schädelhorn** (Nr. 25, Stufe 0, mit Säbelfeuer, zwischen Galgenstrand und Glutwacht) ist mit PixelLab-Grafik neu gebaut;
Schatzinseln mit Rumflasche/Seelenphiole/Splitter.
Seeungeheuer überall (Arten je Seegebiet), 60 Knochenhaie, 19 Geisterschiffe.
Die Seekarte deckt sich beim Erkunden auf (`save.chart`, Felder à 16 × 16 Kacheln).

## Glutwacht

**Glutwacht**: kreisrunde Hafenfestung nach Art des Feuerband-Schreins, im Piraten-Stil:
extrem hohe Ringmauern mit einer Brüstung rundherum (72 Segmente à 5°, jedes im echten Winkel des Kreises gerendert – 18 Winkel `ringWall0..17`,
die andere Hälfte gespiegelt –, genau so lang wie die Sehne, Pfeiler `ringPost` an jedem Stoß, die acht Türme sitzen darauf, nur das Seetor bleibt frei),
mehrere Ringterrassen übereinander, durch Treppen verbunden;
See in der Mitte, überspannt von einer **hohen, stützenlosen Steinbrücke** (West–Ost, 150 px über dem Strudel;
der alte Steg ist fort, `pier_end` nur noch für `&pos=whirl`), im Süden das **Seetor** auf zwei Pfeilern im Wasser, am Nordkai die **Sündenstatue**,
Ring der brennenden erbeuteten Bossschiffe (`hub_wrecks`), Feuerterrasse mit Säbelfeuer und 4 Thronen, Schmied Harrow, Händlerin Sabeth, Laternenmagd (Stufen,
NG+), Crew an der Tafel, Schiffsmeister Brack in seiner **Werft** auf dem Südostplatz neben der Helling (Kai aus Planken, Helling mit Rumpf,
Kran an der Kaikante, Taue, Fässer, Laternen, Flagge, Wegweiser; alles außerhalb des Beckens, das bis 20 Kacheln reicht).
Im Becken blitzen ab und zu Krakenaugen auf, unter der Brücke krabbelt die verschonte Spinne (`game/hub.rs`). Mauern als Quader, Treppen.

## Ebenen

**Ebenen** wie in A Link to the Past (8.11.): wenige große, rund geglättete Plateaus (wilde Inseln höchstens 2 Ebenen, Königsinseln 3),
außerhalb der Festung 96 px hohe Erd- und Felswände nach Süden, schmale Felsseiten und eine dünne Nordkante mit Lichtkante, Schlagschatten;
jede Fläche ist vom Ufer aus über Treppen erreichbar (`prune_stairs`, `force_access`), die Kanten bleiben frei von Bewuchs.
Treppen gibt es nur noch, wo sie zu etwas führen (`prune_stairs`);
Kanten, Hänge und Treppen sind mit hellen Oberkanten, Schwellen, Stufennasen und Schlagschatten am Fuß abgesetzt.
Herunterspringen genau eine Ebene (`jump_target`).

## Siedlungen im Dark-Fantasy-Stil

**Siedlungen im Dark-Fantasy-Stil** (7.11.): Schieferdächer mit Moos und fehlenden Schindeln, verrußter Putz, vernagelte Fenster, Schädel über der Tür, Laterne,
Krähe auf dem First; Taverne mit ochsenblutfarbenem Dach und Schädelschild; zerschlissene Markisen, Trockenfisch und Talgkerzen statt bunter Waren;
Lumpen statt bunter Wäsche; Bannpfähle mit Schädeln (`wardPost`) statt Blumentöpfen;
ein Galgen mit schwingendem Käfig (die vorhandenen `gibbet0/1` des Galgenstrands – Achtung:
`view.rs` behandelt alles, was mit „gibbet“/„gallows“ beginnt, als Schwingkäfig) je Ort;
Ebenenkanten in Siedlungen sind gemauerte Stützmauern (`masonry_pixel` ohne Schießscharten) statt Felswänden; Holz allgemein dunkler;
fahle Nachtfalter statt Schmetterlinge, schwarze Katzen, mehr Krähen.
**Feindschaft**: siehe `game/hostile.rs`.

## Läden

**Läden** (`data::SHOPS`): Glutgrund (Ilka), Reifstein (Wendla), Rabenwacht (Orrin, auch Muschelhorn und Treibholz-Amulett), Moorhain (Mabel);
begehbare Innenräume (`interior`, Tür → `Use::Enter/Leave`;
`Cut::Door` blendet erst auf, wenn am neuen Ort alle sichtbaren Chunks gebacken sind – `WorldView.missing`, höchstens 1,3 s).

## Könige

**Könige** (je 2 Phasen, Galionsfigur als Beute):
Admiral Brandvogt (→ Mantikor), Königin Ysra Frostweh (→ Gorgonen-Königin), Kapitän Rabensturm (→ Uhrwerk-Sturm-Monster),
Morgrim Korallenhand (→ Kraken-Basilisk).

## Seebosse

**Seebosse** (Ungeheuer → bei 50 % Piratenschiff → Wrack, danach brennt es in der Glutwacht):
Tiamat → Die Glutkrone (Glutsee), Krallgard → Die Frostwitwe (Eismeer), Sturmzunge → Die Rabenbraut (Sturmsee), Der Abgrundangler
→ Die Tiefenlaterne (Tiefengrund).

## Strudel

**Strudel**: öffnet sich, wenn alle 4 Galionsfiguren auf den Thronen sitzen und alle 4 Seebosse besiegt sind (`whirl_ready`).
Acht Krakenarme wiegen sich darum; kommt das Schiff auf 250 px heran, packen sie es (`Cut::Whirl`: 1 s Griff mit Wackeln, dann Spirale hinein) → `arrive_end`.

## Geschichte – der Mondraub

**Geschichte – der Mondraub** (9.11.): An seinem Spiegelbild zieht der Mond die Gezeiten.
Malakor wollte nicht hinüber, als er ertrank; er fischte das Spiegelbild des Mondes aus der See und kettete es in den Bauch seines Holländers.
Seitdem steht das Meer still, die **Letzte Flut**, die die Ertrunkenen heimträgt, kommt nie – darum spuckt die See ihre Toten wieder aus (Spieler, Stillwasser,
Glutwacht). Erzählt vom Galionsschnitzer, der Laternenmagd („Über die Letzte Flut“), Wirt Klaas (Kneipe „Zur Letzten Flut“) und dem Geisterkapitän.

## Endkampf

**Endkampf** (`endboss.rs`, Ablauf `EP`, neu 9.11.):
Approach (tote See, leise Gedanken „kein Mond im Wasser“, grünes Licht unter dem Schiff) → Rise (11 s, Zeitplan `R_*`:
Strudel mit grünem Licht und Seelenlichtern, die Masten durchstoßen zuerst die Oberfläche – Wasser stürzt von den Segeln –, Durchbruch mit Fontänen,
Sprung über die Wasserlinie, Aufschlag, zwei Wellenringe werfen das eigene Schiff zurück (`update_rings`), Pforten entzünden sich von Heck bis Bug,
Malakor tritt aufs Achterdeck; das eigene Schiff wird in Rise/Drain nicht gesteuert) → Ship (der PixelLab-Holländer schwimmt:
Wasserlinie (`SprOpt.cut_bottom`), Gischtsaum, Bugwelle, Spiegelung, schaukelt; Malakor auf dem Achterdeck;
XXL-Feuer aus dem Galion, Breitseite aus den zehn Pforten, Donnermörser;
unter 60 % roter Kraken, der beim Tod in einem Blasenstrudel versinkt) → Drain (15 s, `D_*`:
der Holländer zerbricht, grünes Feuer, silbernes Licht durch die Planken, der Spiegelmond fliegt heraus und taucht zum Grund, Malakor springt nach;
bei 3 s „Der Bann bricht“: `drain_end` unter voller Wasserdecke, dann weicht die Wasserkante im Shader von Nordwest nach Südost,
die Strömung reißt das Schiff zum Kampfplatz, bis es aufläuft (`end_strand`, der Spieler springt in den Schlick;
das Schiff liegt schief, im Spielstand an der Glutwacht), Fische zappeln, Wracks tauchen auf, der Weltrand donnert (`edge_flow` 2,8))
→ Seabed (in der Mitte schwebt der **Spiegelmond** an vier Ankerketten (`draw_moon`, `draw_chain`, `moon_anchors`), darunter kniet Malakor betend;
nähert man sich, Kampf zu Fuß (`mega`, „Der den Mond stahl“); bei 0 Leben **flieht er in den Mond** (`BS::Transform`:
schwebt hinauf, Banner) → **Malakor, der Mondfresser** (zweite Gestalt, `MEGA_NAME2`, `draw_moon_eater` in `draw.rs`:
der verdunkelte rote Mond mit Malakors Schatten und glühenden Augen, acht Knochenarme brechen aus dem Schlick;
Angriffe wie der alte Knochenkraken – Stacheln, Feger, Splitter, beim Sprung steigt der Mond hoch und stürzt herab) → Sieg:
`start_tide` (der Mondfresser bleibt nicht liegen) → **Die Letzte Flut** (`tide.rs`) → Done.
`save.end_won` (danach Meer mit Mondfels, Lichtstimmung `mondnacht`, Musik „salz“), `save.end_naval` (trockengefallen, nur bis zum Sieg).

## Die Letzte Flut

**Die Letzte Flut** (`game/tide.rs`, `Cut::Tide`, 65,5 s; Zeitplan `T_*`): Zerfall (Arme zerbröckeln, Malakor stürzt aus dem Mond und zerfließt:
„Ohne den Mond … holt mich die Flut …“) · die vier Ketten reißen (`EndBoss.chains`) · „Der Spiegelmond ist frei“ · die **Meeresspinne** vom Anfang (verschont:
sie selbst, erschlagen: ihr bleicher Geist, `save.spider`) krabbelt heran, schießt Fäden in den Himmel,
spinnt ein Netz um den Mond (`silk`) und zieht ihn hinauf, Tautropfen fliegen als Sterne davon · „Der Mond kehrt zurück“ · die Flut:
Weltrand läuft rückwärts (`edge_flow` −2,5), die Wasserkante rollt zurück, Quellen springen auf, Wracks und das aufgelaufene Schiff versinken
· **die Ertrunkenen ziehen heim** (Klaas, vier Stillwasser-Leute, Geisterkapitän, die vier Geisterkönige stehen im Wasser, nicken, lösen sich in Licht auf;
„Auf die Letzte Flut, Fremder!“) · das Wasser trägt den Spieler, das eigene Schiff steigt unter ihm auf, an Bord · Morgengrauen (`morgenflut`),
Banner „Die Letzte Flut“ · Abblende, drei Epilogseiten (`Ui.story`, über dem Schwarz), im Schwarz `tide_swap` (`flood_end`, `raise_end_isle`,
Schiff vor dem Anleger) · „Am Mondfels brennt ein Säbelfeuer“.
Lichtstimmung `tide_env` (meeresgrund → mondnacht → morgenflut), Musik `tide_music` (Stille, „tiefe“, „leine“, „titel“).
Der Nebel am Ende der Welt richtet sich nach dem Abschnitt (`end_stille/sturm/grund/mond/morgen`; früher lag immer blutroter Nebel darüber).

## NG+

**NG+** (`save.ng`): nach dem Sieg bei der Laternenmagd.
Könige, Seebosse, Throne, Endkampf zurückgesetzt; Stufe, Ausrüstung, Schiff, Crew bleiben.
Gegner-Leben × (1 + 0,5·ng), Schaden × (1 + 0,3·ng).

## Schiff

**Schiff**: schwarze Galeone; erst nach dem Flaschenschiff des Geisterkapitäns verfügbar, danach an jedem Steg „Rufe dein Schiff“ (steigt aus der Tiefe,
versinkt beim Anlegen).
Ausbauten Tempo/Rumpf/Geschütze bis 20 beim Schiffsmeister (bis 10 wie früher, darüber halbe Wirkung, Kosten ×1,16 je Stufe).
Crew-Geschütze (Taste C auf See): 1 Mörser, 2 Kettenkugeln, 3 Flammenwerfer, 4 Kraken-Begleiter (acht grüne Tentakel um das Ziel).
F = Shanty der Crew. Crew und Kapitän stehen per `deck_point` an Deck und drehen sich mit dem Kurs.

## Leben

**Leben**: Tiere (`critters.rs`), Fußspuren/Staub, Rauch aus Schornsteinen (`World::chimneys`), Dorfbewohner, Glühwürmchen.

## Cheats und Debug-Menü

**Cheats** (Pause-Menü): „Unendlich Perlen“ schaltet `save.inf_pearls` um (Some = alter Stand, wird beim Abschalten zurückgegeben;
Perlen bleiben auf `INF_PEARLS`, kein Perlenfleck beim Tod, Anzeige „unendlich“ über `pearl_str`).
Debug-Menü: Trophäen der Könige/Seeungeheuer, ganze Crew, „Alle Säbelfeuer“ (alle sichtbaren Feuer entzündet, das Feuer am Ende der Welt – auf dem Mondfels
– erst nach Malakor).

## Namen

Namen Crew: Kanonierin Asra, Navigatorin Lene, Bootsmann Kalo, Zimmermann Ode. Start-NPC: Alter Galionsschnitzer.
