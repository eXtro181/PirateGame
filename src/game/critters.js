// Small life around the player: butterflies over meadows, gulls over coasts and the sea,
// fish in ponds and shallows, moths and fireflies at night lights, frogs in the swamp.
import { G, TS } from '../art/tiles.js';
import { ZONES } from '../world/world.js';

const COLS = { butterfly: [[0.95, 0.75, 0.85], [0.6, 0.8, 1], [1, 0.9, 0.5], [0.85, 0.6, 1]], fish: [[0.1, 0.2, 0.25]], gull: [[0.95, 0.95, 0.92]] };

export function makeCritters(game) { return { list: [], t: 0 }; }

function spawnAround(game, C, cam, W, H) {
    const w = game.world;
    for (let tries = 0; tries < 6; tries++) {
        const x = cam[0] - 40 + Math.random() * (W + 80), y = cam[1] - 40 + Math.random() * (H + 80);
        const tx = Math.floor(x / TS), ty = Math.floor(y / TS);
        if (!w.inb(tx, ty)) continue;
        const g = w.g(tx, ty), biome = ZONES[w.zone[w.idx(tx, ty)] || 'kueste']?.biome;
        if (g === G.WATER) {
            if (w.shoreDist[w.idx(tx, ty)] < 6 && Math.random() < 0.6) C.list.push({ kind: 'fish', x, y, a: Math.random() * 6.28, t: 0, life: 20 + Math.random() * 20 });
            else if (Math.random() < 0.3) C.list.push({ kind: 'gull', x, y: y - 60, vx: (Math.random() < 0.5 ? -1 : 1) * (30 + Math.random() * 20), vy: (Math.random() - 0.5) * 10, t: 0, life: 30, h: 40 + Math.random() * 30 });
            return;
        }
        if (biome === 'coast' || biome === 'hub' || biome === 'wind') {
            if (Math.random() < 0.7) C.list.push({ kind: 'butterfly', x, y, hx: x, hy: y, t: Math.random() * 10, col: COLS.butterfly[(Math.random() * 4) | 0], life: 30, h: 6 + Math.random() * 10 });
            else C.list.push({ kind: 'gull', x, y: y - 40, vx: (Math.random() < 0.5 ? -1 : 1) * (28 + Math.random() * 20), vy: (Math.random() - 0.5) * 8, t: 0, life: 30, h: 50 });
        } else if (biome === 'earth') C.list.push({ kind: 'frog', x, y, t: Math.random() * 5, life: 40, hop: 0 });
        else if (biome === 'ice' || biome === 'fire' || biome === 'end') C.list.push({ kind: 'moth', x, y, hx: x, hy: y, t: Math.random() * 10, life: 25, h: 10, col: biome === 'ice' ? [0.7, 0.9, 1] : [1, 0.5, 0.3] });
        return;
    }
}

export function updateCritters(game, dt) {
    const C = game.critters, r = game.app.renderer, cam = game.cam;
    C.t += dt;
    if (C.list.length < 26 && Math.random() < dt * 4) spawnAround(game, C, cam, r.W, r.H);
    const p = game.sailing ? game.ship : game.player;
    for (const c of C.list) {
        c.t += dt; c.life -= dt;
        const near = Math.hypot(p.x - c.x, p.y - c.y) < 30;
        switch (c.kind) {
            case 'butterfly': case 'moth': {
                c.x = c.hx + Math.sin(c.t * 0.9) * 18 + Math.sin(c.t * 2.3) * 6; c.y = c.hy + Math.cos(c.t * 0.7) * 10;
                if (near) { c.hx += (c.x - p.x) * dt * 2; c.hy += (c.y - p.y) * dt * 2; c.h = Math.min(30, c.h + dt * 20); }
                break;
            }
            case 'gull': c.x += c.vx * dt; c.y += c.vy * dt; break;
            case 'fish': {
                c.a += Math.sin(c.t * 0.7) * dt; const nx = c.x + Math.cos(c.a) * 14 * dt, ny = c.y + Math.sin(c.a) * 10 * dt;
                const tx = Math.floor(nx / TS), ty = Math.floor(ny / TS);
                if (game.world.g(tx, ty) === G.WATER) { c.x = nx; c.y = ny; } else c.a += Math.PI;
                if (near) c.a = Math.atan2(c.y - p.y, c.x - p.x);
                break;
            }
            case 'frog': if (near && c.hop <= 0) c.hop = 0.5; if (c.hop > 0) { c.hop -= dt; c.x += (c.x - p.x > 0 ? 1 : -1) * 40 * dt; } break;
        }
        if (Math.abs(c.x - cam[0] - r.W / 2) > r.W + 120 || Math.abs(c.y - cam[1] - r.H / 2) > r.H + 120) c.life = 0;
    }
    C.list = C.list.filter(c => c.life > 0);
}

export function drawCritters(game, r, cam, glow) {
    const w = game.app.art.atlas.white.tex, C = game.critters;
    r.spriteBegin('alpha');
    const px = (x, y, c, a = 1, ww = 1, hh = 1) => r.rect(w, Math.round(x - cam[0]), Math.round(y - cam[1]), ww, hh, [c[0], c[1], c[2], a]);
    for (const c of C.list) {
        const fade = Math.min(1, c.life, c.t);
        switch (c.kind) {
            case 'butterfly': case 'moth': {
                const flap = Math.sin(c.t * 22) > 0, y = c.y - c.h;
                if (glow) { if (c.kind === 'moth') px(c.x, y, [c.col[0] * 1.2, c.col[1] * 1.2, c.col[2] * 1.2]); break; }
                px(c.x, c.y, [0, 0, 0], 0.15 * fade, 2, 1);
                if (flap) { px(c.x - 2, y - 1, c.col, fade, 2, 2); px(c.x + 1, y - 1, c.col, fade, 2, 2); } else { px(c.x - 1, y - 1, c.col, fade, 1, 2); px(c.x + 1, y - 1, c.col, fade, 1, 2); }
                px(c.x, y, [0.15, 0.1, 0.1], fade);
                break;
            }
            case 'gull': {
                if (glow) break;
                const flap = Math.sin(c.t * 9) > 0, y = c.y - c.h;
                px(c.x - 2, c.y + 20, [0, 0, 0], 0.12, 5, 1);
                const W = COLS.gull[0];
                px(c.x, y, W); px(c.x - 1, y + (flap ? -1 : 0), W); px(c.x + 1, y + (flap ? -1 : 0), W); px(c.x - 2, y + (flap ? -2 : 1), W); px(c.x + 2, y + (flap ? -2 : 1), W);
                px(c.x + (c.vx > 0 ? 1 : -1), y + 1, [0.3, 0.3, 0.35]);
                break;
            }
            case 'fish': {
                if (glow) break;
                const dx = Math.cos(c.a), dy = Math.sin(c.a);
                for (let k = -3; k <= 3; k++) px(c.x + dx * k, c.y + dy * k * 0.7, [0.08, 0.16, 0.22], 0.45 * fade, k === 0 ? 2 : 1, 1);
                break;
            }
            case 'frog': {
                if (glow) break;
                const y = c.y - (c.hop > 0 ? Math.sin(c.hop / 0.5 * Math.PI) * 5 : 0);
                px(c.x - 2, y - 2, [0.3, 0.55, 0.25], 1, 4, 2); px(c.x - 2, y - 3, [0.5, 0.75, 0.35]); px(c.x + 1, y - 3, [0.5, 0.75, 0.35]); px(c.x - 3, y - 1, [0.2, 0.4, 0.2], 1, 6, 1);
                break;
            }
        }
    }
    r.end();
}
