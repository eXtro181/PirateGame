# Debug-URL-Parameter

## Debug-URL-Parameter (`?debug&start=<feuer-id>`)

Feuer-IDs: `kueste hain asche asche2 frost frost2 raben raben2 mangroven mangroven2 ende`. Zusätze:

- `&all` alle Galionsfiguren + 50 000 Perlen
- `&open` Throne voll + alle Seebosse besiegt (Strudel offen)
- `&seakings` alle Seebosse besiegt
- `&crew` volle Crew
- `&sail` auf dem Schiff starten (`&hdg=<grad>` Kurs)
- `&pos=x,y` bzw. `&pos=whirl` (mit `&sail`: Schiff kurz vor dem Strudel)
- `&pos=isle<n>` am Anleger der n-ten wilden Insel (0–71), `&pos=mid<n>` mitten auf ihr
- `&origin=<herkunft>`
- `&chart` ganze Seekarte aufgedeckt
- `&zoo` jeder Gegnertyp schlafend um den Spieler (`&sail&zoo`: Haie, Geisterschiffe, Kraken und je ein Seeungeheuer jeder Art)
- `&boss=<fire|ice|wind|earth|mega>` (+`&phase2`)
- `&seaboss=<el>` (+`&phase2`)
- `&endsea` mit dem Schiff am Ende der Welt
- `&endphase=1..8` (1 Aufstieg (ab 4,6 s: die Masten durchstoßen die Oberfläche), 2 Seeschlacht, 3 roter Kraken, 4 Malakor am Meeresgrund, 5 Mondfresser,
  6 Holländer zerbricht + Meer läuft leer, 7 Sieg-Sequenz „Die Letzte Flut“ (mit `&tide=<s>` ab Sekunde s), 8 danach: Mondfels mit Säbelfeuer)
- `&tidefront` (mit `endphase=4`: die Wasserkante steht am Spieler – Shader prüfen)
- `&tscale=<k>` größere Zeitschritte (bis ×12; Swiftshader zeichnet ~1 Bild/s, lange Sequenzen laufen so schneller ab)
- `&hp=<n>` mit n Lebenspunkten beginnen (`&prolog&hp=1&pos=4100,3880`: die Spinne klettert herauf und tötet mit dem ersten Hieb – echte Niederlage)
- `&intro` erstes Erwachen im Sarg (`&wake=<sekunde>`)
- `&prolog` im Prolog (Stillwasser, vor dem Tresen) starten (`&prolog&intro`: mit dem Erwachen am Tresen; `&prolog&spider`: Spinne schon erschlagen;
  `&pos=dorf|tuer|tor|galgen|gebein|schlund`: Plätze im Prolog)
- `&die` sofort sterben (im Prolog: Niederlagen-Sequenz)
- `&prolog&coffin=<sekunde>` Prolog-Ende am Sarg ab Sekunde n (mit `&defeat` die Niederlage; mit `&pos=4336,3990` steht man gleich am Sarg)
- `&prolog&gallows=<sekunde>` Galgen-Sequenz ab Sekunde n (Stillwasser feindlich, von den Ertrunkenen erschlagen;
  2,4 s hängt man, 7,2 s Nacht, 10,6 s Spinne, 15,6 s Strick durch, ab 16,9 s die Sarg-Niederlage mit Sprung nah an den Sarg;
  mit `&spider` blendet die Nacht nur ab)
- `&eyes` die Krakenaugen im Becken der Glutwacht sofort zeigen (z. B. `start=hain&pos=28350,20496&eyes`: auf der Brücke, mit der verschonten Spinne darunter;
  `&spider` = Spinne im Prolog erschlagen, dann fehlt sie)
- `&fxzoo` alle PixelFX-Effekte alle 3 s im Raster um den Spieler (Sichtprüfung)
- `&day=<0..1>` Tageszeit (0,5 = tiefe Nacht)
- `&weather=<0..1>` Wetterfront fest (1 = voller Regen/Schnee/Asche je nach Gebiet, 10 min)
- `&dyn` dynamische Auflösung auch im Testlauf (sonst aus, damit Bilder vergleichbar bleiben)
- `&near` Kamera nah
- `&webs` Netze der Meeresspinne (zwei am Boden, drei im Flug)
- `&sink=<s>` das Schiff versinkt am nächsten Steg (ab Sekunde s)
- `&hostile=<schlüssel,..>` Siedlungen feindlich (`asche frost raben mangroven hain`; im Prolog `stillwasser`)
- `&pos=town<schlüssel>` mitten in eine Siedlung (mit Versatz in Kacheln: `&pos=townasche:0,-6`)
- `&pos=sin` vor die Sündenstatue
- `&pos=surf<n>` an die n-te Brandungsstelle an Klippe oder Mauer, `&pos=rock<n>` an ein Felsufer (je zwei Kacheln landwärts)
- Schiff rufen: `&summon` (am nächsten Steg „Rufe dein Schiff“) bzw. `&summon=first` (Flaschenschiff werfen), `&summon=first,2.5` ab Sekunde 2,5;
  `&bottle` Flaschenschiff schon erhalten, `&raised` Schiff schon einmal gerufen; `&pos=ghost` neben dem Geisterkapitän;
  für Bildschirmfotos `&nobanner` (keine Banner) und `&freeze` (Sequenz steht, Blasen/Tropfen laufen weiter)
- `&floramap=<gebiet>` (z. B. `raben`) bzw. `w<n>` (n-te wilde Insel):
  Draufsicht des Bewuchses als PNG in Base64 in der Konsolenzeile `FLORAMAP` (Boden je Material/Ebene, Bäume nach Art, Unterholz und Boden-Deko als Punkte);
  Auslesen per Playwright, Skript siehe Sitzung (Seite laden, Konsolentext nach `FLORAMAP ` dekodieren)
- PixelLab-Insel Schädelhorn: `&pos=mid25` (Lager in der Mitte) bzw. `&pos=isle25` (Anleger)
- `&state` einmal je Sekunde eine Zeile `STATE …` in die Konsole (Ort, Leben, Zwischensequenz, Boss, Endkampf, Flut) – Prüfen ohne Bild:
  `tools/dev/shot -q -s "<parameter>"` (Aufbau der Zeile in `docs/ai/BAUEN.md`, Abschnitt Zustandszeile)
- `&prof` Leistungsmessung je Sekunde (siehe `docs/ai/BAUEN.md`)
