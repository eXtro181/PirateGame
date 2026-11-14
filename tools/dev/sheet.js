#!/usr/bin/env node
// Kontaktblatt: mehrere Bilder verkleinert auf EIN Bild (spart Tokens beim Ansehen), mit Beschriftung.
//   sheet <aus.png> [--tile <breite>] [--cols <n>] [--crop x,y,w,h] <bild>…
// --crop schneidet aus jedem Bild denselben Ausschnitt (in Bildpixeln), --tile ist die Breite je Kachel
// (ohne Angabe so, dass das Blatt höchstens 1560 px breit wird). Ein einzelnes Bild mit --crop/--tile
// ist die Lupe: Ausschnitt in gewünschter Größe.
const { chromium } = require('playwright');
const fs = require('fs');
const path = require('path');

const esc = (s) => String(s).replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));

// items: [{ file, label }] → schreibt out, gibt [breite, höhe] zurück
async function makeSheet(items, out, opt = {}) {
  const n = items.length;
  const cols = opt.cols || (n <= 1 ? 1 : n <= 4 ? 2 : n <= 9 ? 3 : 4);
  const b = await chromium.launch();
  const p = await b.newPage({ viewport: { width: 800, height: 200 } });
  // natürliche Größe des ersten Bilds bestimmt Seitenverhältnis und Ausschnitt
  const first = 'file://' + path.resolve(items[0].file);
  await p.goto('about:blank');
  const tmp = out + '.html';
  fs.writeFileSync(tmp, `<!doctype html><img id="i" src="${first}">`);
  await p.goto('file://' + path.resolve(tmp));
  const [nw, nh] = await p.evaluate(() => { const i = document.getElementById('i'); return [i.naturalWidth, i.naturalHeight]; });
  const crop = opt.crop || [0, 0, nw, nh];
  const tw = opt.tile || Math.min(n <= 1 ? 1280 : 640, Math.floor(1560 / cols), crop[2] * (opt.crop ? 4 : 1));
  const s = tw / crop[2];
  const th = Math.round(crop[3] * s);
  const cell = items.map((i) => `<figure style="margin:0 2px 2px 0;width:${tw}px">` +
    `<div style="width:${tw}px;height:${th}px;overflow:hidden"><img src="file://${path.resolve(i.file)}" ` +
    `style="display:block;transform-origin:0 0;transform:translate(${-crop[0] * s}px,${-crop[1] * s}px) scale(${s});image-rendering:${s > 1.5 ? 'pixelated' : 'auto'}"></div>` +
    (i.label ? `<figcaption style="font:11px monospace;color:#ddd;height:14px;overflow:hidden;white-space:nowrap">${esc(i.label)}</figcaption>` : '') +
    '</figure>').join('');
  const width = cols * (tw + 2);
  fs.writeFileSync(tmp, `<!doctype html><meta charset="utf-8"><body style="margin:0;background:#101014;width:${width}px;display:flex;flex-wrap:wrap">${cell}`);
  await p.setViewportSize({ width, height: 200 });
  await p.goto('file://' + path.resolve(tmp));
  const size = await p.evaluate(() => [document.documentElement.scrollWidth, document.documentElement.scrollHeight]);
  await p.screenshot({ path: out, fullPage: true });
  await b.close();
  fs.unlinkSync(tmp);
  return size;
}

module.exports = { makeSheet };

if (require.main === module) {
  (async () => {
    const a = process.argv.slice(2);
    const opt = {};
    const files = [];
    let out = null;
    for (let i = 0; i < a.length; i++) {
      if (a[i] === '--tile') opt.tile = +a[++i];
      else if (a[i] === '--cols') opt.cols = +a[++i];
      else if (a[i] === '--crop') opt.crop = a[++i].split(',').map(Number);
      else if (!out) out = a[i];
      else files.push(a[i]);
    }
    if (!out || !files.length) {
      console.log('Aufruf: sheet <aus.png> [--tile <breite>] [--cols <n>] [--crop x,y,w,h] <bild>…');
      process.exit(2);
    }
    const items = files.map((f) => ({ file: f, label: files.length > 1 ? path.basename(f, '.png') : '' }));
    const [w, h] = await makeSheet(items, out, opt);
    console.log(`${out} (${files.length} Bild${files.length > 1 ? 'er' : ''}, ${w}×${h})`);
  })().catch((e) => { console.log('sheet: ' + e.message.split('\n')[0]); process.exit(1); });
}
