# Bildvergleich

`shots.js` nimmt sieben feste Testszenen auf (Galgenstrand bei Tag und Nacht, Regen in der
Rabenwacht, Schnee, Aschebucht, Stillwasser, alle Gegner) und vergleicht sie mit Referenzbildern.
Er fängt grobe Fehler ab – schwarzer Boden, fehlende Sprites, Absturz des Renderers –, nicht jede
Kleinigkeit: Teilchen, Wellen und Wetter bewegen sich.

```
cd dist && python3 -m http.server 8790 &      # gebautes Spiel ausliefern
node tools/shots/shots.js http://localhost:8790 /tmp/shots --ref   # Referenzen (vor der Änderung)
node tools/shots/shots.js http://localhost:8790 /tmp/shots         # nach der Änderung vergleichen
```

Die Bilder liegen außerhalb des Repos (keine Bilddateien im Projekt). Swiftshader ist langsam:
je Szene etwa eine halbe Minute.
