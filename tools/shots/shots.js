// Automatischer Bildvergleich (Runde 13): nimmt feste Testszenen über die Debug-Parameter auf
// und vergleicht sie mit Referenzbildern. Fängt grobe Fehler ab (schwarzer Boden, fehlende
// Sprites, abgestürzter Renderer), nicht jede Kleinigkeit – Teilchen und Wellen bewegen sich.
//
//   node tools/shots/shots.js http://localhost:8790 <ordner>          vergleichen
//   node tools/shots/shots.js http://localhost:8790 <ordner> --ref    Referenzen neu aufnehmen
//
// Braucht Playwright mit Chromium (in der Cloud-Umgebung vorinstalliert). Gibt je Szene den
// Anteil abweichender Pixel aus und endet mit Code 1, wenn eine Szene über der Schwelle liegt.
const { chromium } = require('playwright');
const fs = require('fs');
const path = require('path');

const SCENES = [
  ['kueste', 'start=kueste&nobanner'],
  ['nacht', 'start=kueste&day=0.5&nobanner'],
  ['regen', 'start=raben&pos=townraben&weather=1&nobanner'],
  ['schnee', 'start=frost&weather=1&nobanner'],
  ['asche', 'start=asche&nobanner'],
  ['dorf', 'start=kueste&prolog&pos=dorf&nobanner'],
  ['zoo', 'start=kueste&zoo&nobanner'],
];
const WAIT = 8000; // nach „Welt erzeugt“
const LIMIT = 0.12; // höchstens 12 % deutlich abweichende Pixel

(async () => {
  const [base, dir, flag] = process.argv.slice(2);
  if (!base || !dir) {
    console.log('Aufruf: node tools/shots/shots.js <url-basis> <ordner> [--ref]');
    process.exit(2);
  }
  fs.mkdirSync(dir, { recursive: true });
  const ref = flag === '--ref';
  const b = await chromium.launch({ args: ['--use-gl=angle', '--use-angle=swiftshader', '--enable-unsafe-swiftshader', '--ignore-gpu-blocklist'] });
  let bad = 0;
  for (const [name, q] of SCENES) {
    const p = await b.newPage({ viewport: { width: 1280, height: 720 } });
    let t0 = 0;
    const errs = [];
    p.on('console', (m) => {
      const t = m.text();
      if (/Welt erzeugt/.test(t)) t0 = Date.now();
      // Fehler des Spiels (Panik, Bevy-ERROR); der Hinweis auf Software-Rendering in Swiftshader zählt nicht
      if ((/panicked|\bERROR\b/.test(t)) && !/software rendering/.test(t)) errs.push(t.replace(/%c/g, '').slice(0, 200));
    });
    p.on('pageerror', (e) => errs.push('PAGEERR ' + e.message.slice(0, 200)));
    await p.goto(`${base}/?debug&${q}`);
    const start = Date.now();
    while (!t0 && Date.now() - start < 240000) await p.waitForTimeout(250);
    await p.waitForTimeout(WAIT);
    const file = path.join(dir, `${name}${ref ? '' : '_neu'}.png`);
    await p.screenshot({ path: file, timeout: 180000 });
    if (ref) {
      console.log(`${name}: Referenz aufgenommen${errs.length ? ' (Fehler: ' + errs[0] + ')' : ''}`);
    } else {
      const refFile = path.join(dir, `${name}.png`);
      if (!fs.existsSync(refFile)) {
        console.log(`${name}: keine Referenz (erst mit --ref aufnehmen)`);
      } else {
        // Vergleich im Browser: beide Bilder auf eine Leinwand, Kanäle vergleichen
        const a64 = fs.readFileSync(refFile).toString('base64');
        const b64 = fs.readFileSync(file).toString('base64');
        const frac = await p.evaluate(async ([a, c]) => {
          const load = (s) => new Promise((res) => { const i = new Image(); i.onload = () => res(i); i.src = 'data:image/png;base64,' + s; });
          const [ia, ib] = await Promise.all([load(a), load(c)]);
          const w = Math.min(ia.width, ib.width), h = Math.min(ia.height, ib.height);
          const cv = document.createElement('canvas'); cv.width = w; cv.height = h;
          const g = cv.getContext('2d');
          g.drawImage(ia, 0, 0); const da = g.getImageData(0, 0, w, h).data;
          g.drawImage(ib, 0, 0); const db = g.getImageData(0, 0, w, h).data;
          let n = 0;
          for (let k = 0; k < da.length; k += 4) {
            if (Math.abs(da[k] - db[k]) > 40 || Math.abs(da[k + 1] - db[k + 1]) > 40 || Math.abs(da[k + 2] - db[k + 2]) > 40) n++;
          }
          return n / (w * h);
        }, [a64, b64]);
        const ok = frac <= LIMIT && errs.length === 0;
        if (!ok) bad++;
        console.log(`${name}: ${(frac * 100).toFixed(1)} % abweichend ${ok ? 'ok' : 'AUFFÄLLIG'}${errs.length ? ' – ' + errs[0] : ''}`);
      }
    }
    await p.close();
  }
  await b.close();
  process.exit(bad ? 1 : 0);
})();
