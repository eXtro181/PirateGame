#!/usr/bin/env node
// Bildschirmfotos und Konsolenprüfung des gebauten Spiels (Playwright + Swiftshader) – knapp gehalten:
// meldet nur Fehler, gewünschte Konsolenzeilen und die Bildpfade; mehrere Bilder kommen zusätzlich
// verkleinert auf EIN Kontaktblatt (ein Blatt ansehen statt vieler Einzelbilder spart Tokens).
//
//   shot [optionen] "<szene>" ["<szene>" …]
//   szene = "name|parameter" oder nur "parameter"; Parameter ohne "?debug&", z. B. "start=kueste&nobanner"
//           "titel" = Startseite ohne Debug
//   -t 8,20        Aufnahmezeiten in s nach „Welt erzeugt“ (Standard 8)
//   -k <folge>     Eingaben beim ersten Zeitpunkt: KeyD:300 (halten in ms), KeyJ, wait:2000, click:640:360,
//                  move:x:y, text:abc – durch Komma getrennt
//   -r <regex>     Konsolenzeilen, die passen, mit ausgeben
//   -s             Zustand ausgeben (hängt &state an; je Aufnahme die letzte STATE-Zeile)
//   -q             keine Bilder, nur Konsole/Zustand (billigste Prüfung)
//   -o <ordner>    Ausgabe (Standard .shots/)      -p <präfix>   Dateipräfix
//   --crop x,y,w,h Ausschnitt in Bildschirmpixeln  --size 1280x720   Fenstergröße
//   --tile <b>     Kachelbreite im Kontaktblatt    --no-sheet    kein Kontaktblatt
//   -j <n>         Szenen gleichzeitig (Standard 3) --url <basis> (Standard http://localhost:8790)
//   --timeout <s>  Warten auf die Welt (Standard 240)
const { chromium } = require('playwright');
const fs = require('fs');
const path = require('path');
const { makeSheet } = require('./sheet.js');

const ROOT = path.resolve(__dirname, '../..');
const GL = ['--use-gl=angle', '--use-angle=swiftshader', '--enable-unsafe-swiftshader', '--ignore-gpu-blocklist'];
const BAD = /panicked|^ERROR |^WARNUNG |PAGEERR|unreachable|RuntimeError/;
const HARMLOS = /software rendering|GPU stall|Automatic fallback to software/i;

// Bevy schreibt „%cINFO%c pfad:zeile%c Text“ und hängt CSS-Angaben an – übrig bleibt der Text.
// Warnungen aus dem eigenen Code (src/, crates/) zählen als Fehler, die von Bevy nicht.
function clean(raw) {
  let t = raw.replace(/%c/g, '').replace(/\s*color: [^;]+; background: #\w+ color: \w+; font-style: italic color: inherit\s*$/, '');
  const m = t.match(/^\s*(INFO|WARN|ERROR|DEBUG|TRACE)\s+(\S+):\d+\s+/);
  if (!m) return t.trim();
  t = t.slice(m[0].length).trim();
  const own = !m[2].includes('/registry/');
  if (m[1] === 'ERROR') return 'ERROR ' + t;
  if (m[1] === 'WARN') return (own ? 'WARNUNG ' : 'WARN ') + t;
  return t;
}

function parseOpts(argv) {
  const o = { times: [8], out: path.join(ROOT, '.shots'), prefix: '', w: 1280, h: 720, j: 3, url: 'http://localhost:8790', timeout: 240, sheet: true, scenes: [] };
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    const v = () => argv[++i];
    if (a === '-t') o.times = v().split(',').map(Number);
    else if (a === '-k') o.keys = v();
    else if (a === '-r') o.re = new RegExp(v());
    else if (a === '-s') o.state = true;
    else if (a === '-q') o.quiet = true;
    else if (a === '-o') o.out = path.resolve(v());
    else if (a === '-p') o.prefix = v();
    else if (a === '--crop') { const c = v().split(',').map(Number); o.clip = { x: c[0], y: c[1], width: c[2], height: c[3] }; }
    else if (a === '--size') { const s = v().split('x').map(Number); o.w = s[0]; o.h = s[1]; }
    else if (a === '--tile') o.tile = +v();
    else if (a === '--no-sheet') o.sheet = false;
    else if (a === '-j') o.j = +v();
    else if (a === '--url') o.url = v().replace(/\/$/, '');
    else if (a === '--timeout') o.timeout = +v();
    else if (a === '-h' || a === '--help') o.help = true;
    else o.scenes.push(a);
  }
  return o;
}

function sceneUrl(o, q) {
  if (q === 'titel' || q === '') return o.url + '/';
  if (q.startsWith('?')) return o.url + '/' + q;
  return `${o.url}/?debug&${q}${o.state && !/[?&]state\b/.test(q) ? '&state' : ''}`;
}

async function act(p, folge) {
  await p.focus('canvas').catch(() => {});
  for (const k of folge.split(',')) {
    const [key, a, b] = k.split(':');
    if (key === 'wait') await p.waitForTimeout(+a);
    else if (key === 'click') { await p.mouse.click(+a, +b); await p.waitForTimeout(1200); }
    else if (key === 'move') { await p.mouse.move(+a, +b); await p.waitForTimeout(600); }
    else if (key === 'text') { await p.keyboard.type(a); await p.waitForTimeout(600); }
    else {
      await p.keyboard.down(key);
      await p.waitForTimeout(+(a || 250));
      await p.keyboard.up(key);
      await p.waitForTimeout(+(b || 1200));
    }
  }
}

// Eine Szene laden und aufnehmen. Ergebnis: { name, ok, world, lines[], shots[{file,label}], log[] }
async function runScene(sc, o) {
  const [name, q] = sc.includes('|') ? [sc.split('|')[0], sc.split('|').slice(1).join('|')] : [sc.name || sc, sc];
  const res = { name, q, ok: true, world: null, lines: [], shots: [], log: [] };
  const b = await chromium.launch({ args: GL });
  try {
    const p = await b.newPage({ viewport: { width: o.w, height: o.h } });
    let t0 = 0;
    let state = '';
    const errs = new Map();
    const sec = () => ((Date.now() - (t0 || Date.now())) / 1000).toFixed(1);
    const bad = (t) => { const k = t.slice(0, 200); errs.set(k, (errs.get(k) || 0) + 1); };
    p.on('console', (m) => {
      const t = clean(m.text());
      res.log.push(t);
      if (/Welt erzeugt/.test(t) && !t0) t0 = Date.now();
      if (t.startsWith('STATE ')) { state = t; return; }
      if (BAD.test(t) && !HARMLOS.test(t)) bad(t);
      else if (o.re && o.re.test(t) && res.lines.length < 40) res.lines.push(`  ${sec()} s ${t.slice(0, 220)}`);
    });
    p.on('pageerror', (e) => bad('PAGEERR ' + e.message.split('\n')[0]));
    const start = Date.now();
    await p.goto(sceneUrl(o, q));
    while (!t0 && Date.now() - start < o.timeout * 1000) await p.waitForTimeout(200);
    if (!t0) {
      res.ok = false;
      res.lines.push(`  ! keine Welt nach ${o.timeout} s`);
    } else {
      res.world = (t0 - start) / 1000;
      for (let k = 0; k < o.times.length; k++) {
        const w = o.times[k] * 1000 - (Date.now() - t0);
        if (w > 0) await p.waitForTimeout(w);
        if (k === 0 && o.keys) await act(p, o.keys);
        const at = sec();
        if (!o.quiet) {
          const file = path.join(o.out, `${o.prefix}${name}_${k}.png`);
          await p.screenshot({ path: file, timeout: 180000, clip: o.clip });
          res.shots.push({ file, label: `${name} ${at} s` });
          res.lines.push(`  ${at} s → ${path.relative(process.cwd(), file)}`);
        }
        if (o.state) res.lines.push(`  ${at} s ${state || 'STATE fehlt (Spiel ohne &state gebaut?)'}`);
      }
    }
    for (const [t, n] of [...errs].slice(0, 8)) res.lines.push(`  ! ${t}${n > 1 ? ` (×${n})` : ''}`);
    if (errs.size > 8) res.lines.push(`  ! … ${errs.size - 8} weitere Fehlerarten`);
    if (errs.size) res.ok = false;
  } catch (e) {
    res.ok = false;
    res.lines.push('  ! ' + e.message.split('\n')[0]);
  }
  await b.close();
  return res;
}

async function pool(items, n, fn) {
  const out = new Array(items.length);
  let next = 0;
  await Promise.all(Array.from({ length: Math.min(n, items.length) }, async () => {
    while (next < items.length) { const i = next++; out[i] = await fn(items[i], i); }
  }));
  return out;
}

async function serverUp(url) {
  try { const r = await fetch(url + '/'); return r.ok; } catch (e) { return false; }
}

module.exports = { runScene, parseOpts, pool, serverUp, ROOT };

if (require.main === module) {
  (async () => {
    const o = parseOpts(process.argv.slice(2));
    if (o.help || !o.scenes.length) {
      console.log(fs.readFileSync(__filename, 'utf8').split('\n').slice(1, 20).map((l) => l.replace(/^\/\/ ?/, '')).join('\n'));
      process.exit(o.help ? 0 : 2);
    }
    if (!(await serverUp(o.url))) { console.log(`Server ${o.url} antwortet nicht – erst tools/dev/build (oder tools/dev/serve)`); process.exit(1); }
    fs.mkdirSync(o.out, { recursive: true });
    const names = 'abcdefghijklmnopqrstuvwxyz';
    const scenes = o.scenes.map((s, i) => (s.includes('|') ? s : `${o.scenes.length > 1 ? names[i % 26] : 's'}|${s}`));
    const all = await pool(scenes, o.j, (s) => runScene(s, o));
    let ok = true;
    const shots = [];
    for (const r of all) {
      ok = ok && r.ok;
      console.log(`${r.name}: ${r.q || 'titel'}${r.world ? ` · Welt nach ${r.world.toFixed(0)} s` : ''}${r.ok ? '' : ' · FEHLER'}`);
      r.lines.forEach((l) => console.log(l));
      shots.push(...r.shots);
    }
    if (o.sheet && shots.length > 1) {
      const out = path.join(o.out, `${o.prefix}blatt.png`);
      const [w, h] = await makeSheet(shots, out, { tile: o.tile });
      console.log(`Blatt: ${path.relative(process.cwd(), out)} (${shots.length} Bilder, ${w}×${h})`);
    }
    process.exit(ok ? 0 : 1);
  })().catch((e) => { console.log('shot: ' + e.message.split('\n')[0]); process.exit(1); });
}
