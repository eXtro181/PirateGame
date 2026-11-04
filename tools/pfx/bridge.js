// Brücke zwischen dem Spiel (Rust/Wasm) und PixelFX (https://github.com/patch-gg/pixelfx, MIT).
// tools/pfx/bundle.sh bündelt sie samt PixelFX-Kern und den ausgewählten Effekten zu web/pfx.js.
// Beim Start zeichnet das Spiel jeden Effekt einmal Bild für Bild (pfx_bake) in den Effektatlas und
// spielt ihn danach als Sprite ab – keine Bilddateien, alles aus Code. Paletten und Optionen kommen
// aus dem Spiel (src/pfx.rs), damit die Effekte zur düsteren Piratenwelt passen.

import { createFrame, clearFrame, createFx, composite, fit } from "PIXELFX/src/core/index.js";
import { noiseFor, hotFirst, burn, looped, drift, spark, ember } from "PIXELFX/src/fx/fire/_lib.js";
import * as sword_slash from "PIXELFX/src/fx/melee/sword-slash.js";
import * as double_slash from "PIXELFX/src/fx/melee/double-slash.js";
import * as thrust from "PIXELFX/src/fx/melee/thrust.js";
import * as claw_swipe from "PIXELFX/src/fx/melee/claw-swipe.js";
import * as hit_spark from "PIXELFX/src/fx/impacts/hit-spark.js";
import * as crit_hit from "PIXELFX/src/fx/impacts/crit-hit.js";
import * as parry from "PIXELFX/src/fx/impacts/parry.js";
import * as blunt_hit from "PIXELFX/src/fx/impacts/blunt-hit.js";
import * as heavy_landing from "PIXELFX/src/fx/impacts/heavy-landing.js";
import * as ground_slam from "PIXELFX/src/fx/impacts/ground-slam.js";
import * as bullet_impact from "PIXELFX/src/fx/impacts/bullet-impact.js";
import * as muzzle_pistol from "PIXELFX/src/fx/projectiles/muzzle-pistol.js";
import * as small_explosion from "PIXELFX/src/fx/explosions/small-explosion.js";
import * as torch from "PIXELFX/src/fx/fire/torch.js";
import * as brazier from "PIXELFX/src/fx/fire/brazier.js";
import * as spirit_flame from "PIXELFX/src/fx/fire/spirit-flame.js";
import * as flame_pillar from "PIXELFX/src/fx/fire/flame-pillar.js";
import * as fire_ring from "PIXELFX/src/fx/fire/fire-ring.js";
import * as frost_nova from "PIXELFX/src/fx/ice/frost-nova.js";
import * as ice_shatter from "PIXELFX/src/fx/ice/ice-shatter.js";
import * as lightning_strike from "PIXELFX/src/fx/lightning/lightning-strike.js";
import * as zap_hit from "PIXELFX/src/fx/lightning/zap-hit.js";
import * as earth_spike from "PIXELFX/src/fx/nature/earth-spike.js";
import * as acid_splash from "PIXELFX/src/fx/nature/acid-splash.js";
import * as splash from "PIXELFX/src/fx/water/splash.js";
import * as geyser from "PIXELFX/src/fx/water/geyser.js";
import * as heal from "PIXELFX/src/fx/holy/heal.js";
import * as soul_rise from "PIXELFX/src/fx/spawn/soul-rise.js";
import * as poof from "PIXELFX/src/fx/spawn/poof.js";
import * as shatter_death from "PIXELFX/src/fx/spawn/shatter-death.js";
import * as respawn_beam from "PIXELFX/src/fx/spawn/respawn-beam.js";
import * as collect_burst from "PIXELFX/src/fx/pickups/collect-burst.js";
import * as loot_beam from "PIXELFX/src/fx/pickups/loot-beam.js";
import * as bat from "PIXELFX/src/fx/critters/bat.js";

// Nur die Flamme der PixelFX-Fackel (ohne Wandhalter, Griff und Wickel): sitzt auf den Fackeln des Spiels.
// Zungen und Feuer-Parameter wie in fire/torch.js.
const torch_flame = (() => {
  const meta = { ...torch.meta, name: "Torch flame", size: [64, 96], origin: [32, 84] };
  const D = meta.duration, NL = noiseFor(D);
  const TONGUES = [[-1.9, -1, 0.74, 2.0, 1], [2.1, -1, 0.7, 2.0, 1], [-3.4, 0.5, 0.42, 1.6, 0], [3.6, 0.5, 0.46, 1.6, 0], [0, 0, 1, 2.9, 0]];
  function draw(p, t, o) {
    const H = hotFirst(fit(o.palette, 9));
    burn(p, t, TONGUES, { H, D, NL, S: o.seed, x: 0, y: 0, height: o.height ?? 20, lean: 0.12, spread: 4, puffs: 10, licks: 5, smoke: 7, smokeSize: 2.3, smokeLife: [1.3, 2.1], embers: 7, glowR: 48, glow: 1.25, wind: 2 });
  }
  return { meta, draw };
})();

// Nur das Feuer der PixelFX-Feuerschale (ohne Ständer, Schale und Glut): für Feuerschalen und brennende Wracks.
// Zungen, Feuer und Funkenflug wie in fire/brazier.js.
const brazier_flame = (() => {
  const meta = { ...brazier.meta, name: "Brazier flame", size: [104, 132], origin: [52, 116] };
  const D = meta.duration, NL = noiseFor(D);
  const TONGUES = [[-7, -1.5, 0.5, 2.0, 1], [7.2, -1.5, 0.55, 2.0, 1], [-3, -2.5, 0.84, 2.2, 1], [3.3, -2.5, 0.86, 2.2, 1], [-11.5, 0.5, 0.2, 1.5, 0], [11.5, 0.5, 0.22, 1.5, 0], [-8.3, 0, 0.34, 2.0, 0], [8.5, 0, 0.38, 2.0, 0], [-4, 0.5, 0.6, 2.4, 0], [4.4, 0.5, 0.62, 2.4, 0], [0, 0, 1, 3.0, 0]];
  function draw(p, t, o) {
    const H = hotFirst(fit(o.palette, 9)), S = o.seed;
    const SPARKS = [H[1], H[2], H[4], H[5]], EMB = [H[1], H[2], H[4], H[6]];
    burn(p, t, TONGUES, { H, D, NL, S, x: 0, y: 0, height: o.height ?? 32, lean: 0.1, spread: 7, puffs: 8, licks: 4, smoke: 14, smokeSize: 3, smokeLife: [1.6, 2.4], embers: 10, glowR: 58, glow: 1.4, wind: 3 });
    looped(t, D, 60, S + 600, (r) => {
      const side = r.range(-1, 1);
      return { x: side * 8, y: -r.range(4, 18), vx: side * r.range(10, 40), vy: -r.range(70, 150), life: r.range(0.7, 1.7), a: r.range(0, 6.3), w: r.range(1.8, 3.2), m: r.range(3, 8), big: r.chance(0.12) };
    }, (q, age) => {
      const [x, y, vx, vy] = drift(q.x, q.y, q.vx, q.vy, age, 0, -22, 1.9, 1.9), k = 1 - age / q.life;
      const wob = q.m * (Math.sin(age * q.w + q.a) - Math.sin(q.a)) * Math.min(1, age * 1.5), wv = q.m * q.w * Math.cos(age * q.w + q.a) * Math.min(1, age * 1.5);
      if (Math.hypot(vx + wv, vy) > 40) spark(p, x + wob, y, vx + wv, vy, k, SPARKS);
      else ember(p, x + wob, y, k, q.big ? 2 : 1, EMB, age * 40 + q.a);
    });
  }
  return { meta, draw };
})();

const FX = {
  "melee/sword-slash": sword_slash,
  "melee/double-slash": double_slash,
  "melee/thrust": thrust,
  "melee/claw-swipe": claw_swipe,
  "impacts/hit-spark": hit_spark,
  "impacts/crit-hit": crit_hit,
  "impacts/parry": parry,
  "impacts/blunt-hit": blunt_hit,
  "impacts/heavy-landing": heavy_landing,
  "impacts/ground-slam": ground_slam,
  "impacts/bullet-impact": bullet_impact,
  "projectiles/muzzle-pistol": muzzle_pistol,
  "explosions/small-explosion": small_explosion,
  "game/torch-flame": torch_flame,
  "game/brazier-flame": brazier_flame,
  "fire/spirit-flame": spirit_flame,
  "fire/flame-pillar": flame_pillar,
  "fire/fire-ring": fire_ring,
  "ice/frost-nova": frost_nova,
  "ice/ice-shatter": ice_shatter,
  "lightning/lightning-strike": lightning_strike,
  "lightning/zap-hit": zap_hit,
  "nature/earth-spike": earth_spike,
  "nature/acid-splash": acid_splash,
  "water/splash": splash,
  "water/geyser": geyser,
  "holy/heal": heal,
  "spawn/soul-rise": soul_rise,
  "spawn/poof": poof,
  "spawn/shatter-death": shatter_death,
  "spawn/respawn-beam": respawn_beam,
  "pickups/collect-burst": collect_burst,
  "pickups/loot-beam": loot_beam,
  "critters/bat": bat,
};

// Figuren des Spiels als PixelFX-Darsteller: Effekte wie „Seele steigt auf“ oder „Zersplittern“ arbeiten
// mit der Gestalt der Figur (Maske + eigene Farben), so zerbricht ein Skelett in seiner eigenen Form.
const ACTORS = {};

/**
 * Einen Darsteller aus einem RGBA-Bild (w × h) mit Fußpunkt (fx, fy) anlegen.
 * Deckende Pixel bilden die Maske; Kopf und Brust werden aus der Gestalt bestimmt.
 */
export function pfx_actor(name, rgba, w, h, fx, fy) {
  let x0 = w, y0 = h, x1 = -1, y1 = -1;
  for (let y = 0; y < h; y++)
    for (let x = 0; x < w; x++)
      if (rgba[(y * w + x) * 4 + 3] > 110) {
        if (x < x0) x0 = x;
        if (x > x1) x1 = x;
        if (y < y0) y0 = y;
        if (y > y1) y1 = y;
      }
  if (x1 < 0) return;
  const mw = x1 - x0 + 1, mh = y1 - y0 + 1, data = new Uint8Array(mw * mh), pixels = new Uint32Array(mw * mh);
  for (let y = 0; y < mh; y++)
    for (let x = 0; x < mw; x++) {
      const s = ((y + y0) * w + x + x0) * 4;
      if (rgba[s + 3] > 110) {
        data[y * mw + x] = 1;
        pixels[y * mw + x] = ((255 << 24) | (rgba[s + 2] << 16) | (rgba[s + 1] << 8) | rgba[s]) >>> 0;
      }
    }
  const top = y0 - fy;
  ACTORS[name] = { mask: { w: mw, h: mh, data }, pixels, x: x0 - fx, y: top, w: mw, h: mh, k: 2, head: [0, top], chest: [0, Math.round(top + mh * 0.32)] };
}

// Ränder weich ausblenden (wie PixelFX beim Export von Sprite-Sheets), damit Licht nicht hart abbricht
function edgeFade(f, E = 8) {
  const { w, h, light: L } = f;
  for (let y = 0; y < h; y++) {
    const ey = Math.min(y, h - 1 - y);
    for (let x = 0; x < w; x++) {
      const e = Math.min(ey, x, w - 1 - x);
      if (e >= E) {
        x = w - 1 - E;
        continue;
      }
      const i = y * w + x, l = L[i];
      if (!(l & 0xffffff)) continue;
      const u = (e + 0.5) / E, k = u * u * (3 - 2 * u);
      L[i] = (l & 0xff000000) | Math.round((l & 255) * k) | (Math.round(((l >>> 8) & 255) * k) << 8) | (Math.round(((l >>> 16) & 255) * k) << 16);
    }
  }
}

/**
 * Einen Effekt in `n` Bildern zeichnen (gleichmäßig über seine Dauer; Schleifen nahtlos; 0 = automatisch).
 * Rückgabe: 7 Werte Int32 (Breite, Höhe, Ursprung x, y, Bilder, Schleife, Dauer in ms), dann die
 * Bilder als RGBA hintereinander (Licht über Transparenz als Farbe mit Deckkraft gemischt, wie PixelFX
 * es für Sprite-Sheets exportiert). `opts` ist JSON mit Effekt-Optionen (Paletten, Stärken; "actor"
 * nennt einen mit pfx_actor angelegten Darsteller), `seed` 0 = Standard. Unbekannt: leeres Feld.
 */
export function pfx_bake(id, n, seed, opts) {
  const mod = FX[id];
  if (!mod) return new Uint8Array(0);
  const m = mod.meta;
  const [W, H] = m.size, [OX, OY] = m.origin;
  // n = 0: nach Dauer und Größe (Schleifen 8–12 Bilder, sonst rund 14 je Sekunde, große Effekte weniger)
  if (n < 1) n = m.loop ? Math.max(8, Math.min(12, Math.round(m.duration * 8))) : Math.max(6, Math.min(W * H > 40000 ? 12 : W * H > 30000 ? 14 : 16, Math.round(m.duration * 14)));
  const o = { x: OX, y: OY, ...(opts ? JSON.parse(opts) : {}) };
  if (typeof o.actor === "string") {
    if (ACTORS[o.actor]) o.actor = ACTORS[o.actor];
    else delete o.actor;
  }
  if (seed) o.seed = seed;
  const f = createFrame(W, H, 0, 0);
  const inst = createFx(mod, o);
  const dt = m.duration / n;
  const head = 7 * 4, size = W * H * 4;
  const out = new Uint8Array(head + n * size);
  new Int32Array(out.buffer, 0, 7).set([W, H, OX, OY, n, m.loop ? 1 : 0, Math.round(m.duration * 1000)]);
  for (let i = 0; i < n; i++) {
    if (i > 0) inst.update(dt);
    clearFrame(f);
    inst.draw(f);
    edgeFade(f);
    composite(f, new Uint8ClampedArray(out.buffer, head + i * size, size), null);
  }
  return out;
}

/** Kennungen aller eingebundenen Effekte, durch Kommas getrennt. */
export function pfx_ids() {
  return Object.keys(FX).join(",");
}

