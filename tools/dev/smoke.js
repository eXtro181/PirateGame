#!/usr/bin/env node
// Startprobe des gebauten Spiels ohne Bilder: lädt Spiel und Titel, wartet auf „Welt erzeugt“ und meldet
// je Szene EINE Zeile (Atlasseiten, Fehler). Endet mit Code 1 bei Fehlern.
//   smoke                 Spiel (start=kueste) und Titel
//   smoke --fx            wartet zusätzlich, bis die PixelFX-Effekte gebacken sind (bis 120 s)
//   smoke "<parameter>"   eigene Szene(n) statt der beiden Standardszenen
const { runScene, parseOpts, pool, serverUp } = require('./shot.js');

(async () => {
  const argv = process.argv.slice(2);
  const fx = argv.includes('--fx');
  const o = parseOpts(argv.filter((a) => a !== '--fx'));
  o.quiet = true;
  o.times = [fx ? 120 : 6];
  if (!(await serverUp(o.url))) { console.log(`Server ${o.url} antwortet nicht – erst tools/dev/build`); process.exit(1); }
  const scenes = o.scenes.length ? o.scenes.map((s, i) => (s.includes('|') ? s : `szene${i + 1}|${s}`)) : ['spiel|start=kueste&nobanner', 'titel|titel'];
  const all = await pool(scenes, o.j, async (s) => {
    const r = await runScene(s, o);
    const find = (re) => { const l = r.log.find((x) => re.test(x)); return l ? l.match(re) : null; };
    const parts = [];
    if (r.world) parts.push(`Welt nach ${r.world.toFixed(0)} s`);
    const pages = find(/(\d+) Atlasseiten/);
    const used = find(/Atlas belegt: ([\d.,]+) Seiten/);
    if (pages) parts.push(`Atlas ${used ? used[1] + ' von ' : ''}${pages[1]} Seiten`);
    const pfx = find(/PixelFX: (\d+) Effekte.*Effektatlas ([\d.,]+) %/);
    if (pfx) parts.push(`PixelFX ${pfx[1]} Effekte, Effektseite ${pfx[2]} %`);
    else if (fx) parts.push('PixelFX-Meldung fehlt');
    return { r, parts };
  });
  let ok = true;
  for (const { r, parts } of all) {
    ok = ok && r.ok;
    console.log(`${r.name}: ${r.ok ? 'läuft' : 'FEHLER'} · ${parts.join(' · ')}`);
    r.lines.filter((l) => l.startsWith('  !')).forEach((l) => console.log(l));
  }
  process.exit(ok ? 0 : 1);
})().catch((e) => { console.log('smoke: ' + e.message.split('\n')[0]); process.exit(1); });
