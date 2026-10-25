// The game scene: player, enemies, bosses, NPCs, sailing, fires, death and pearls, saving.
import { Player, Enemy, NPC, GroundItem, Shot, Walker, dist } from './actors.js';
import { Boss } from './boss.js';
import { Ship, GhostShip, Shark, Kraken } from './ship.js';
import { FX, FXCOL } from './fx.js';
import { dialogFor, levelEntries, weaponEntries, shopEntries, CREW } from './dialog.js';
import { ITEMS, KINGS, SPELLS, EL_NAME, levelCost, upgradeCost, WEAPONS, derived } from './data.js';
import { ZONES, MW, MH } from '../world/world.js';
import { gradesFor } from '../art/pal.js';
import { makeCritters, updateCritters, drawCritters } from './critters.js';
import { TS } from '../art/tiles.js';

const lerp = (a, b, t) => a + (b - a) * t;
const lerp3 = (a, b, t) => [lerp(a[0], b[0], t), lerp(a[1], b[1], t), lerp(a[2], b[2], t)];

export class GameScene {
    constructor(app, profile, save) {
        this.app = app; this.mode = 'world';
        this.profile = profile; this.save = save;
        this.world = app.world; this.view = app.view; this.input = app.input; this.audio = app.audio; this.ui = app.cui;
        this.fx = new FX(app);
        migrate(save);
        this.player = new Player(this, save);
        this.cam = [0, 0]; this.camF = [0, 0];
        this.shakeK = 0; this.flashC = [1, 1, 1, 0]; this.slowT = 0; this.shantyT = 0; this.hpGhost = 0; this.bossGhost = 0;
        this.pearlShown = save.pearls; this.pearlGain = 0; this.pearlGainT = 0;
        this.zoneKey = null; this.zoneT = 0; this.gradeCur = { ...gradesFor(app.options.look).coast };
        this.shots = []; this.lock = null; this.focus = null; this.talkTo = null;
        this.npcs = this.world.npcs.map(n => new NPC(this, n));
        this.walkers = this.world.walkers.map(w => new Walker(this, w));
        this.critters = makeCritters(this);
        this.items = this.world.items.filter(i => !save.picked.includes(i.id)).map(i => new GroundItem(this, i));
        for (const c of this.world.chests) { c.opened = save.chests.includes(c.chestId); c.def = this.app.art.props[c.opened ? 'chestopen' : 'chest']; }
        this.bosses = {};
        for (const k of Object.keys(KINGS)) { const z = KINGS[k].zone; const b = new Boss(this, k, this.world.arenas[z]); if (save.kings[k]) b.state = 'dead', b.hidden = true; this.bosses[k] = b; }
        this.bosses.mega = new Boss(this, 'mega', this.world.arenas.ende); if (save.megalodon) { this.bosses.mega.state = 'dead'; this.bosses.mega.hidden = true; }
        this.world.dynamicBlockers = this.world.gates.map(g => ({ gate: g, active: false, x0: g.x - 34, x1: g.x + 34, y0: g.y - 30, y1: g.y + 30 }));
        for (const g of this.world.gates) { const k = Object.keys(KINGS).find(k => KINGS[k].zone === g.key); g.king = k; if (save.kings[k]) g.prop.hidden = true; }
        // hub figurehead slots
        for (const s of this.world.slots) this.applySlot(s);
        this.whirlpoolOpen = ['fire', 'ice', 'wind', 'earth'].every(e => save.mounted[e]);
        // ship at its dock
        const dock = this.world.docks.find(d => d.zone === save.shipDock) || this.world.docks[0];
        this.ship = new Ship(this, dock.x, dock.y, dock.heading);
        this.sailing = false;
        this.spawnAll();
        this.placeAtFire(save.lastFire, true);
        this.t = 0; this.saveT = 0;
        this.ui.buildMap(this.world);
        if (!save.litFires.includes(save.lastFire)) save.litFires.push(save.lastFire);
        this.lightFires();
        if (save.playTime < 1) { const zk = this.world.fires.find(f => f.id === save.lastFire)?.zone || 'kueste'; this.zoneKey = zk; setTimeout(() => this.ui.banner(ZONES[zk].name, 'Die See hat dich zurückgegeben.', 'big', 4), 600); }
        if (save.stain) this.stain = { ...save.stain, t: 0 };
    }
    // ---------------------------------------------------------------- spawning
    spawnAll() {
        this.enemies = this.world.spawns.map(s => new Enemy(this, s));
        this.ghosts = this.world.ghostRoutes.map((r, i) => new GhostShip(this, r, i));
        if (!this.kraken || !this.kraken.dead) this.kraken = new Kraken(this, 214 * TS, 66 * TS);
        this.sharks = [[210, 140], [110, 80], [220, 60], [100, 150], [200, 170], [110, 200]].map(([x, y]) => new Shark(this, x * TS, y * TS));
    }
    placeAtFire(id, initial = false) {
        const f = this.world.fires.find(f => f.id === id) || this.world.fires[0];
        this.player.x = f.x + 14; this.player.y = f.y + 10; this.player.face(0, 1);
        this.sailing = false; this.player.state = 'free';
        this.camF = [this.player.x, this.player.y];
        if (!initial) this.fx.clear();
    }
    lightFires() { for (const f of this.world.fires) f.prop.lit = this.save.litFires.includes(f.id); }
    // ---------------------------------------------------------------- main update
    update(dt, input) {
        this.t += dt;
        this.save.playTime += dt;
        this.view.time += dt;
        const modal = this.ui.update(dt, input);
        this.shakeK = Math.max(0, this.shakeK - dt * 14);
        this.flashC[3] = Math.max(0, this.flashC[3] - dt * 2.5);
        this.slowT = Math.max(0, this.slowT - dt); this.shantyT = Math.max(0, this.shantyT - dt);
        this.pearlGainT -= dt; if (this.pearlGainT <= 0) this.pearlGain = 0;
        this.pearlShown += (this.save.pearls - this.pearlShown) * Math.min(1, dt * 6);
        if (Math.abs(this.pearlShown - this.save.pearls) < 1) this.pearlShown = this.save.pearls;
        const p = this.player;
        this.hpGhost = this.hpGhost > p.hp ? Math.max(p.hp, this.hpGhost - p.d.maxHp * dt * 0.5) : p.hp;
        if (this.boss) this.bossGhost = this.bossGhost > this.boss.hp ? Math.max(this.boss.hp, this.bossGhost - this.boss.maxHp * dt * 0.3) : this.boss.hp;
        if (this.deathT !== undefined) { this.deathT += dt; this.ui.fade = Math.max(0, Math.min(1, (this.deathT - 2.6) * 1.5)); if (this.deathT > 3.6) this.respawn(); }
        if (!modal && !this.cutscene) {
            if (input.pressed('menu')) { this.pauseMenu(); return; }
            if (input.pressed('map')) { this.ui.mapOpen = true; this.audio.sfx('click'); }
            if (input.pressed('legend')) this.ui.legendOpen = true;
            if (input.pressed('lock')) this.toggleLock();
            if (this.sailing) this.updateSailing(dt, input);
            else { p.update(dt, input); if (input.pressed('interact')) this.interact(); if (p.state === 'free' && this.world.walkable(p.x, p.y)) this.lastSafe = { x: p.x, y: p.y }; }
        } else if (!this.sailing) { p.at += dt; }
        // world simulation keeps running behind dialogues (but not behind menus)
        const frozen = modal && !this.ui.dialog;
        if (!frozen) {
            for (const e of this.enemies) e.update(dt);
            for (const b of Object.values(this.bosses)) b.update(dt);
            for (const n of this.npcs) n.update(dt);
            for (const w of this.walkers) w.update(dt);
            updateCritters(this, dt);
            for (const it of this.items) it.update(dt);
            for (const s of this.shots) s.update(dt);
            this.shots = this.shots.filter(s => s.life > 0);
            this.ship.update(dt, input, this.sailing && !modal);
            for (const g of this.ghosts) g.update(dt);
            for (const s of this.sharks) s.update(dt);
            this.kraken.update(dt);
            this.fx.update(dt);
        }
        if (this.stain) this.stain.t += dt;
        this.updateFocus();
        this.updateZone(dt);
        this.updateBossArena();
        this.updateCamera(dt);
        this.fx.ambient(dt, this.cam, this.app.renderer.W, this.app.renderer.H, ZONES[this.zoneKey]?.biome, this.sailing);
        if (this.whirlpoolOpen && this.sailing && dist(this.ship, this.world.whirlpool) < 36 && !this.cutscene) this.enterWhirlpool();
        this.saveT -= dt;
    }
    updateSailing(dt, input) {
        if (input.pressed('interact')) {
            const d = this.world.docks.find(d => Math.hypot(d.x - this.ship.x, d.y - this.ship.y) < 60);
            if (d) this.disembark(d);
            else this.ui.toast('Hier gibt es keinen Steg zum Anlegen.');
        }
        if (input.pressed('shanty')) this.singShanty();
    }
    // ---------------------------------------------------------------- camera / zones
    updateCamera(dt) {
        const r = this.app.renderer, t = this.sailing ? this.ship : this.player;
        let tx = t.x, ty = t.y - (this.sailing ? 10 : 12);
        if (!this.sailing) { tx += this.player.fx * 16; ty += this.player.fy * 10; }
        if (this.boss && !this.boss.dead && !this.boss.hidden) { tx = lerp(tx, this.boss.x, 0.3); ty = lerp(ty, this.boss.y, 0.3); }
        const k = Math.min(1, dt * (this.sailing ? 3 : 5));
        this.camF[0] = lerp(this.camF[0], tx, k); this.camF[1] = lerp(this.camF[1], ty, k);
        const sx = this.shakeK ? (Math.random() - 0.5) * this.shakeK * 2 : 0, sy = this.shakeK ? (Math.random() - 0.5) * this.shakeK * 2 : 0;
        this.cam = [Math.round(this.camF[0] - r.W / 2 + sx), Math.round(this.camF[1] - r.H / 2 + sy)];
    }
    updateZone(dt) {
        const t = this.sailing ? this.ship : this.player;
        const z = this.world.zoneAt(t.x, t.y);
        const ti = this.world.idx(Math.max(0, Math.min(MW - 1, Math.floor(t.x / TS))), Math.max(0, Math.min(MH - 1, Math.floor(t.y / TS))));
        const tile = this.world.zone[ti];
        const onIsland = !this.sailing || (tile && this.world.shoreDist[ti] < 8);
        this.view.setWater(this.sailing && !onIsland ? 'sea' : ZONES[z].biome);
        if (z !== this.zoneKey) {
            this.zoneKey = z;
            if (!this.sailing || onIsland) this.zoneT = 4;
            if (!this.save.explored.includes(z)) this.save.explored.push(z);
        }
        this.zoneT = Math.max(0, this.zoneT - dt);
        // grading towards the zone (or the sea)
        const GR = gradesFor(this.app.options.look);
        const G = this.sailing && !onIsland ? GR.sea : GR[ZONES[z].grade];
        this.lightK = (this.lightK ?? G.light) + (G.light - (this.lightK ?? G.light)) * Math.min(1, dt * 1.5);
        const k = Math.min(1, dt * 1.5), c = this.gradeCur;
        c.lift = lerp3(c.lift || G.lift, G.lift, k); c.gain = lerp3(c.gain || G.gain, G.gain, k); c.ambient = lerp3(c.ambient || G.ambient, G.ambient, k);
        c.sat = lerp(c.sat ?? G.sat, G.sat, k); c.contrast = lerp(c.contrast ?? G.contrast, G.contrast, k);
        // music
        let mood = this.sailing ? 'sea' : ZONES[z].music;
        if (this.boss && !this.boss.dead && this.boss.state !== 'dormant') mood = 'boss';
        if (this.player.state === 'sit') mood = 'fire';
        if (this.deathT !== undefined) mood = 'death';
        this.audio.setMood(mood);
    }
    updateBossArena() {
        const mega = this.bosses.mega;
        if (mega.state === 'dormant' && !this.save.megalodon && this.zoneKey === 'ende' && !this.sailing) {
            const a = this.world.arenas.ende;
            if (Math.hypot((this.player.x - a.x) / a.r, (this.player.y - a.y) / a.ry) < 0.55) mega.activate();
        }
    }
    shake(k) { this.shakeK = Math.max(this.shakeK, k); }
    flash(col, a) { this.flashC = [col[0], col[1], col[2], a]; }
    // ---------------------------------------------------------------- focus + interaction
    interactables() {
        const out = [], p = this.player, s = this.save;
        for (const f of this.world.fires) out.push({ x: f.x, y: f.y, h: 36, label: s.litFires.includes(f.id) ? 'Rasten' : 'Feuer entzünden', act: () => this.useFire(f), obj: f.prop });
        for (const n of this.npcs) out.push({ x: n.x, y: n.y, h: 36, label: 'Sprechen', act: () => this.talk(n), obj: n });
        for (const it of this.items) out.push({ x: it.x, y: it.y, h: 24, label: 'Aufheben', act: () => this.pickUp(it), obj: it });
        for (const c of this.world.chests) if (!c.opened) out.push({ x: c.x, y: c.y, h: 22, label: 'Öffnen', act: () => this.openChest(c), obj: c });
        for (const g of this.world.gates) if (!g.prop.hidden && !this.save.kings[g.king] && this.bosses[g.king].state === 'dormant') out.push({ x: g.x, y: g.y + 8, h: 40, label: 'Durch den Nebel schreiten', act: () => this.enterGate(g), obj: g.prop });
        for (const sl of this.world.slots) if (!s.mounted[sl.el]) out.push({ x: sl.x, y: sl.y, h: 30, label: s.figureheads[sl.el] ? 'Galionsfigur anbringen' : 'Ein leerer Bug', act: () => this.useSlot(sl), obj: sl.prop });
        for (const d of this.world.docks) {
            const here = Math.hypot(this.ship.x - d.x, this.ship.y - d.y) < 24;
            out.push({ x: d.px, y: d.py, h: 30, label: here ? 'An Bord gehen' : 'Schiff rufen', act: () => here ? this.board(d) : this.callShip(d) });
        }
        if (this.stain) out.push({ x: this.stain.x, y: this.stain.y, h: 18, label: 'Perlen zurückholen', act: () => this.takeStain() });
        return out;
    }
    updateFocus() {
        const p = this.player;
        let best = null, bd = 30;
        if (!this.sailing && p.state !== 'dead') for (const it of this.interactables()) { const d = Math.hypot(it.x - p.x, (it.y - p.y) * 1.2); if (d < bd) { bd = d; best = it; } }
        // mouse hover outline (enemies, items, NPCs, props with interactions)
        let hover = best?.obj || null;
        const m = this.input.mouse;
        if (m.x >= 0 && m.moved) {
            const wx = m.x + this.cam[0], wy = m.y + this.cam[1];
            const cands = [...this.enemies.filter(e => !e.dead), ...this.npcs, ...this.items, ...Object.values(this.bosses).filter(b => !b.hidden && !b.dead)];
            for (const c of cands) { const h = c.sheet?.h || 20, w = (c.sheet?.w || 16) * 0.5; if (wx > c.x - w && wx < c.x + w && wy > c.y - h && wy < c.y + 2) { hover = c; break; } }
        }
        this.focus = best ? { ...best, hover } : (hover ? { hover } : null);
    }
    interact() { if (this.focus?.act) this.focus.act(); }
    toggleLock() {
        if (this.lock) { this.lock = null; return; }
        const p = this.player;
        let best = null, bd = 160;
        for (const e of this.hittables()) { const d = dist(e, p); if (d < bd) { bd = d; best = e; } }
        this.lock = best; if (best) this.audio.sfx('lock');
    }
    hittables() {
        const out = this.enemies.filter(e => !e.dead);
        for (const b of Object.values(this.bosses)) if (!b.hidden && !b.dead) out.push(b);
        if (this.sailing) { for (const g of this.ghosts) if (!g.dead) out.push(g); for (const s of this.sharks) if (!s.dead) out.push(s); if (!this.kraken.dead && !this.kraken.hidden) out.push(this.kraken); }
        return out;
    }
    // ---------------------------------------------------------------- combat hooks
    playerStrike(p, heavy) {
        const w = p.w, reach = w.range * 14 + 6;
        let dmg = p.d.dmg * (heavy ? w.heavyMul : 1) * (1 + p.combo * 0.1);
        const ang = Math.atan2(p.fy, p.fx);
        this.fx.slash(p.x + p.fx * 6, p.y - 10 + p.fy * 4, ang, reach * 0.75, heavy ? [1, 0.85, 0.6, 1] : [0.9, 0.95, 1, 1], heavy ? 2 : 1.4);
        let hit = false;
        for (const t of this.hittables()) {
            if (!p.inArc(t, reach + (t.isBoss ? t.hurtR * 0.5 : 0), heavy ? 1.3 : 1.15)) continue;
            t.hurt(Math.round(dmg * (0.9 + Math.random() * 0.2)), p, { push: [p.fx * 6, p.fy * 6], poise: heavy ? dmg * 2 : dmg });
            hit = true;
            this.fx.burst('spark', t.x, t.y - (t.bodyY || 10), 5, { speed: 70 });
        }
        if (hit) this.shake(heavy ? 3 : 1.5);
    }
    onParry(from) {
        from.stagger?.();
        this.audio.sfx('parried'); this.flash([1, 1, 1], 0.25); this.shake(2);
        this.fx.burst('spark', from.x, from.y - 14, 18, { speed: 110 });
        this.ui.toast('Pariert! Jetzt zustechen.');
    }
    castSpell(p, sp) {
        const dmg = p.d.spellDmg;
        const t = this.lock && !this.lock.dead ? this.lock : null;
        const ang = t ? Math.atan2(t.y - p.y, t.x - p.x) : Math.atan2(p.fy, p.fx);
        if (sp === 'fireball') {
            this.shots.push(new Shot(this, { owner: 'player', kind: 'fireball', x: p.x + Math.cos(ang) * 8, y: p.y + Math.sin(ang) * 6, z: 12, vx: Math.cos(ang) * 170, vy: Math.sin(ang) * 140, dmg: Math.round(dmg * 1.3), r: 5, life: 1.4, glow: [1.6, 0.7, 0.25, 1], col: [1, 0.55, 0.2, 1], light: [1, 0.5, 0.2],
                onHit: (s) => this.explode(s.x, s.y, 26, s.dmg, 'ember'), onExpire: (s) => this.explode(s.x, s.y, 26, s.dmg, 'ember'), onHitWall: (s) => this.explode(s.x, s.y, 26, s.dmg, 'ember') }));
            this.audio.sfx('cannon');
        } else {
            // chain lightning: first target in front, then hops
            const cands = this.hittables();
            let cur = null, bd = 140;
            for (const e of cands) { const dx = e.x - p.x, dy = e.y - p.y, d = Math.hypot(dx, dy); const a = Math.acos(Math.max(-1, Math.min(1, (dx * Math.cos(ang) + dy * Math.sin(ang)) / (d || 1)))); if (d < bd && a < 0.9) { bd = d; cur = e; } }
            const pts = [[p.x + p.fx * 4, p.y - 22]];
            const hitSet = new Set();
            let k = 1;
            while (cur && hitSet.size < 4) {
                hitSet.add(cur); pts.push([cur.x, cur.y - (cur.bodyY || 10)]);
                cur.hurt(Math.round(dmg * k), p, { poise: dmg });
                this.fx.burst('soulLight', cur.x, cur.y - 10, 8, { speed: 60 });
                k *= 0.8;
                let nx = null, nd = 80;
                for (const e of cands) if (!hitSet.has(e) && !e.dead) { const d = dist(e, cur); if (d < nd) { nd = d; nx = e; } }
                cur = nx;
            }
            if (pts.length === 1) pts.push([p.x + Math.cos(ang) * 90, p.y + Math.sin(ang) * 70]);
            this.fx.bolt(pts); this.flash([0.7, 1, 0.95], 0.18); this.audio.sfx('zap');
        }
    }
    explode(x, y, r, dmg, kind) {
        this.fx.burst(kind, x, y, 26, { speed: 100 }); this.fx.burst('smoke', x, y, 8, { speed: 40, up: 20, g: -20, life: 0.8 });
        this.fx.ring(x, y, 3, r, 0.3, [1, 0.6, 0.3]); this.shake(3); this.audio.sfx('cannon');
        this.flashLight = { x, y, r: 90, col: [1, 0.55, 0.25], i: 2.5, t: 0.25 };
        for (const t of this.hittables()) if (Math.hypot(t.x - x, (t.y - y) * 1.2) < r + (t.hurtR || 6)) t.hurt(dmg, this.player, { poise: dmg });
    }
    playerShoot(p) {
        const t = this.lock && !this.lock.dead ? this.lock : null;
        const ang = t ? Math.atan2(t.y - p.y, t.x - p.x) : Math.atan2(p.fy, p.fx);
        if (this.save.off === 'tide') {
            this.shots.push(new Shot(this, { owner: 'player', kind: 'tide', x: p.x, y: p.y, z: 8, vx: Math.cos(ang) * 150, vy: Math.sin(ang) * 120, dmg: p.d.offDmg, r: 6, life: 1.1, pierce: true, glow: [0.4, 1.1, 0.9, 1], col: [0.4, 0.8, 0.9, 1] }));
            this.audio.sfx('tide');
        } else {
            this.shots.push(new Shot(this, { owner: 'player', kind: 'bullet', x: p.x + Math.cos(ang) * 10, y: p.y + Math.sin(ang) * 8, z: 14, vx: Math.cos(ang) * 300, vy: Math.sin(ang) * 260, dmg: p.d.offDmg, r: 3, life: 0.7 }));
            this.fx.burst('smoke', p.x + Math.cos(ang) * 14, p.y + Math.sin(ang) * 10, 5, { speed: 20, up: 10, g: -10, life: 0.6 });
            this.fx.burst('spark', p.x + Math.cos(ang) * 14, p.y + Math.sin(ang) * 10 - 12, 4, { speed: 60 });
            this.audio.sfx('pistol'); this.shake(1.5);
        }
    }
    throwBomb(p) {
        const tx = p.x + p.fx * 70, ty = p.y + p.fy * 55;
        this.shots.push(new Shot(this, { owner: 'none', kind: 'bomb', x: p.x, y: p.y, z: 16, vz: 110, grav: 300, vx: (tx - p.x) / 0.75, vy: (ty - p.y) / 0.75, dmg: 0, life: 3, onLand: (s) => this.explode(s.x, s.y, 32, 70, 'ember') }));
    }
    spawnShot(e, kind, tx, ty, dmg) {
        const a = Math.atan2(ty - e.y, tx - e.x);
        const sp = kind === 'cannon' ? 140 : kind === 'soul' ? 110 : 190;
        this.shots.push(new Shot(this, { owner: 'enemy', kind, x: e.x + Math.cos(a) * 10, y: e.y + Math.sin(a) * 8, z: Math.max(10, (e.bodyY || 10)), vx: Math.cos(a) * sp, vy: Math.sin(a) * sp * 0.9, dmg, r: kind === 'cannon' ? 5 : 3, life: kind === 'bullet' ? 1.2 : 1.8, src: e,
            glow: kind === 'soul' ? [0.4, 1.4, 1.0, 1] : undefined, col: kind === 'soul' ? [0.4, 0.95, 0.8, 1] : undefined, light: kind === 'soul' ? [0.35, 1, 0.8] : undefined,
            onHit: (s, t) => t.hurt(s.dmg, { parryable: false }) }));
        this.fx.burst('smoke', e.x + Math.cos(a) * 12, e.y + Math.sin(a) * 9, 4, { speed: 20, up: 10, g: -10 });
    }
    spawnBossShot(b, kind, a, sp, dmg) {
        const C = { fireball: [[1.5, 0.6, 0.2, 1], [1, 0.5, 0.2, 1]], shard: [[0.6, 1, 1.3, 1], [0.7, 0.9, 1, 1]], feather: [[1, 1, 1.2, 1], [0.8, 0.85, 0.9, 1]], rock: [null, null] }[kind];
        this.shots.push(new Shot(this, { owner: 'enemy', kind: kind === 'rock' ? 'rock' : kind, x: b.x + Math.cos(a) * 14, y: b.y + Math.sin(a) * 10, z: b.human ? 18 : 24, vx: Math.cos(a) * sp, vy: Math.sin(a) * sp * 0.8, dmg, r: 4, life: 2.2, glow: C[0], col: C[1],
            onHit: (s, t) => { t.hurt(s.dmg, { parryable: false }, { unparryable: true }); if (kind === 'shard') this.slowT = 1.5; } }));
    }
    spawnShotRaw(o) { this.shots.push(new Shot(this, { ...o, onHit: (s, t) => { t.hurt(s.dmg); this.fx.burst('splash', s.x, s.y, 6); } })); }
    pushPlayer(dx, dy) { const p = this.player; for (let i = 0; i < 6; i++) p.move(dx / 6, dy / 6); }
    gainPearls(n, src) {
        this.save.pearls += n; this.pearlGain += n; this.pearlGainT = 2;
        if (src) for (let i = 0; i < 10; i++) this.fx.p(src.x + (Math.random() - 0.5) * 10, src.y - 8, { z: 6, vz: 30 + Math.random() * 20, vx: (this.player.x - src.x) * 0.8, vy: (this.player.y - src.y) * 0.8, life: 1, col: FXCOL.soul, glow: 1.5, drag: 0.3 });
        this.audio.sfx('pearls');
    }
    // ---------------------------------------------------------------- death
    playerDied() {
        const p = this.player, s = this.save;
        p.state = 'dead'; p.anim = 'dead'; p.frame = 0;
        this.audio.sfx('death');
        this.ui.banner('DU BIST ERTRUNKEN', '', 'death', 3.4);
        s.deaths = (s.deaths || 0) + 1;
        // pearls stay where you fell (on land)
        const safe = this.world.walkable(p.x, p.y) ? p : (this.lastSafe || p);
        s.stain = s.pearls > 0 ? { x: safe.x, y: safe.y, pearls: s.pearls } : null;
        s.pearls = 0; this.pearlShown = 0;
        this.stain = s.stain ? { ...s.stain, t: 0 } : null;
        this.deathT = 0; this.lock = null;
    }
    shipSunk() { this.sailing = false; this.player.x = this.ship.x; this.player.y = this.ship.y; this.player.hp = 0; this.playerDied(); this.player.hidden = true; }
    respawn() {
        this.deathT = undefined; this.ui.fade = 0;
        const p = this.player;
        p.hidden = false; p.refresh(true); p.state = 'free';
        for (const b of Object.values(this.bosses)) if (!b.dead) b.reset();
        this.boss = null;
        for (const g of this.world.dynamicBlockers) g.active = false;
        this.spawnAll();
        this.ship.hull = this.ship.maxHull;
        const dock = this.world.docks.find(d => d.zone === this.save.shipDock) || this.world.docks[0];
        this.ship.x = dock.x; this.ship.y = dock.y; this.ship.h = dock.heading; this.ship.v = 0; this.sailing = false;
        this.placeAtFire(this.save.lastFire);
        this.persist();
    }
    takeStain() {
        this.gainPearls(this.stain.pearls, this.stain);
        this.ui.banner('Perlen zurückgeholt', '', 'small', 2);
        this.stain = null; this.save.stain = null; this.persist();
    }
    // ---------------------------------------------------------------- fires
    useFire(f) {
        const s = this.save, p = this.player;
        if (this.boss && !this.boss.dead && this.boss.state !== 'dormant') { this.ui.toast('Nicht, während ein König wacht.'); return; }
        if (!s.litFires.includes(f.id)) {
            s.litFires.push(f.id); f.prop.lit = true;
            this.audio.sfx('kindle'); this.ui.banner('SÄBELFEUER ENTZÜNDET', f.name, 'big', 3);
            this.fx.rise('soul', f.x, f.y, 30, { w: 14 });
        }
        s.lastFire = f.id;
        p.state = 'sit'; p.face(f.x - p.x, f.y - p.y);
        p.refresh(true);
        this.spawnAll();
        this.ship.hull = this.ship.maxHull;
        this.persist();
        this.restMenu(f);
    }
    restMenu(f) {
        this.ui.openList(f.name, [
            { label: 'Reisen', desc: 'Zu einem anderen entzündeten Säbelfeuer.', act: () => this.travelMenu(f) },
            { label: 'Steuerung', desc: 'Alle Tasten auf einen Blick.', act: () => { this.ui.closeList(); this.ui.legendOpen = true; this.player.state = 'free'; } },
            { label: 'Aufstehen', desc: 'Weiter.', act: () => { this.ui.closeList(); this.player.state = 'free'; } }
        ], { onBack: () => { this.player.state = 'free'; } });
    }
    travelMenu(from) {
        const items = this.world.fires.filter(f => this.save.litFires.includes(f.id)).map(f => ({ label: f.name, right: f === from ? 'hier' : ZONES[f.zone]?.name.split(' ')[0], desc: ZONES[f.zone]?.name, disabled: f === from, act: () => this.travelTo(f) }));
        this.ui.openList('Reisen', items, { onBack: () => this.restMenu(from) });
    }
    travelTo(f) {
        this.ui.closeList();
        this.cutscene = true; let t = 0;
        const step = () => {
            t += 1 / 30; this.ui.fade = Math.min(1, t * 2);
            if (t < 0.6) return setTimeout(step, 33);
            this.save.lastFire = f.id; this.placeAtFire(f.id); this.player.state = 'sit';
            this.spawnAll(); this.persist();
            const back = () => { this.ui.fade = Math.max(0, this.ui.fade - 0.06); if (this.ui.fade > 0) setTimeout(back, 33); else { this.cutscene = false; this.restMenu(f); } };
            back();
        };
        step();
    }
    // ---------------------------------------------------------------- NPCs
    talk(n) {
        this.talkTo = n; this.player.state = 'talk'; this.player.face(n.x - this.player.x, n.y - this.player.y);
        this.ui.openDialog(n.name, dialogFor(this, n));
    }
    talkPages(pages) { this.ui.openDialog(this.talkTo?.name || '', { pages, choices: [{ label: 'Weiter', act: () => this.talk(this.talkTo) }, { label: 'Lebewohl', act: () => this.endTalk() }] }); }
    endTalk() { this.talkTo = null; this.ui.dialog = null; this.ui.closeList(); if (this.player.state === 'talk') this.player.state = 'free'; }
    recruit(el) {
        const s = this.save;
        if (!s.crew.includes(el)) s.crew.push(el);
        const n = this.npcs.find(n => n.def.crew === el);
        this.ui.banner('Neue Crew', `${CREW[el].name} · Shanty „${CREW[el].songName}“`, 'big', 3);
        this.audio.sfx('level');
        this.moveCrewToHub();
        this.endTalk(); this.persist();
        void n;
    }
    moveCrewToHub() { // recruited crew sit by the tavern in the hub
        const spots = this.world.crewSpots || [];
        this.save.crew.forEach((el, i) => { const n = this.npcs.find(n => n.def.crew === el); if (n && spots[i]) { n.x = spots[i].x; n.y = spots[i].y; n.def = { ...n.def, sit: true }; } });
    }
    levelStat(k) {
        const s = this.save, cost = levelCost(s.level);
        if (s.pearls < cost) return;
        s.pearls -= cost; s.level++; s.stats[k]++;
        this.player.refresh(); this.player.hp = this.player.d.maxHp;
        this.audio.sfx('level'); this.fx.rise('soul', this.player.x, this.player.y, 20);
        this.ui.levelUp(); this.persist();
    }
    upgradeWeapon() {
        const s = this.save, lvl = s.weaponLevel || 0, c = upgradeCost(lvl);
        if (lvl >= 5) return;
        if (s.pearls < c.pearls || s.shards < c.shards) { this.ui.toast(`Du brauchst ${c.pearls} Perlen und ${c.shards} Gezeitensplitter.`); this.endTalk(); return; }
        s.pearls -= c.pearls; s.shards -= c.shards; s.weaponLevel = lvl + 1;
        this.player.refresh(); this.audio.sfx('chest'); this.shake(2);
        this.ui.banner(`${WEAPONS[s.weapon].name} +${s.weaponLevel}`, 'Gehärtet im Feuer des Ambosses', 'big', 2.5);
        this.endTalk(); this.persist();
    }
    buyWeapon(k) {
        const s = this.save, w = WEAPONS[k];
        if (!s.owned.includes(k)) { if (s.pearls < w.price) return; s.pearls -= w.price; s.owned.push(k); this.audio.sfx('pearls'); }
        s.weapon = k; this.player.refresh(); this.ui.weaponShop(); this.persist();
    }
    buy(what, price) {
        const s = this.save;
        if (s.pearls < price) return;
        s.pearls -= price;
        if (what === 'bullets') s.bullets += 6; else if (what === 'bombs') s.bombs += 3; else if (what === 'phial') { s.phialsMax++; this.player.phials++; }
        this.audio.sfx('pearls'); this.ui.shop(); this.persist();
    }
    // ---------------------------------------------------------------- items, chests, slots
    pickUp(it) {
        this.items.splice(this.items.indexOf(it), 1);
        this.save.picked.push(it.def.id);
        this.giveItem(it.def.item);
        this.persist();
    }
    giveItem(id) {
        const s = this.save, I = ITEMS[id];
        this.audio.sfx('chest');
        switch (id) {
            case 'pearls': { const n = 300 + Math.floor(Math.random() * 400); this.gainPearls(n); this.ui.banner(I.name, `${n} Perlen`, 'small', 2.2); return; }
            case 'rum': s.flasksMax++; this.player.flasks++; break;
            case 'phial': s.phialsMax++; this.player.phials++; break;
            case 'bullets': s.bullets += 6; break;
            case 'bombs': s.bombs += 3; break;
            case 'soulflower': case 'shard': s.shards++; break;
            case 'fireball': if (!s.spells.includes('fireball')) s.spells.push('fireball'); s.spell = s.spells.indexOf('fireball'); break;
            default: s.items[id] = true;
        }
        this.player.refresh();
        this.ui.banner(I.name, I.desc, 'big', 3);
    }
    openChest(c) {
        c.opened = true; c.def = this.app.art.props.chestopen; this.save.chests.push(c.chestId);
        this.fx.burst('spark', c.x, c.y - 8, 10, { speed: 50 });
        this.giveItem(c.loot.item); this.persist();
    }
    applySlot(sl) {
        if (this.save.mounted[sl.el]) {
            sl.prop.def = this.app.art.props['figurehead_' + sl.el]; sl.prop.kind = 'figurehead_' + sl.el;
            if (!sl.light) { const L = sl.prop.def.light; sl.light = { x: sl.x, y: sl.y + L.dy, r: L.r, col: L.col, i: L.i, pulse: 2, prop: sl.prop }; this.world.lightsStatic.push(sl.light); }
        }
    }
    useSlot(sl) {
        const s = this.save;
        if (!s.figureheads[sl.el]) { this.ui.toast(`Hier fehlt die Galionsfigur ${EL_NAME[sl.el] === 'Tiefe' ? 'der Tiefe' : 'des ' + EL_NAME[sl.el] + 's'}.`); return; }
        s.mounted[sl.el] = true; this.applySlot(sl);
        this.fx.rise('soul', sl.x, sl.y, 40, { w: 20 }); this.audio.sfx('kindle'); this.flash([0.4, 1, 0.85], 0.4);
        const n = ['fire', 'ice', 'wind', 'earth'].filter(e => s.mounted[e]).length;
        this.ui.banner('Galionsfigur angebracht', `${n} von 4`, 'big', 3);
        if (n === 4) { this.whirlpoolOpen = true; setTimeout(() => { this.ui.banner('DER STRUDEL ÖFFNET SICH', 'Nördlich des Schiffshains', 'boss', 4); this.audio.sfx('rise'); this.shake(6); }, 3200); }
        this.persist();
    }
    // ---------------------------------------------------------------- bosses
    enterGate(g) {
        const p = this.player;
        p.x = g.x - g.dir[0] * 50; p.y = g.y - g.dir[1] * 50;
        this.world.dynamicBlockers.find(b => b.gate === g).active = true;
        g.prop.open = false;
        this.bosses[g.king].activate();
    }
    bossStarted(b) { this.boss = b; this.bossGhost = b.hp; this.ui.banner(b.name, b.mega ? '' : 'Geisterkönig', 'boss', 3); }
    bossDefeated(b) {
        const s = this.save;
        this.ui.banner(b.mega ? 'DAS ENDE DER WELT' : 'SEELE ERBEUTET', b.mega ? 'Moloch ist gefallen. Die See wird still.' : b.def.name.split(',')[0], 'big', 5);
        this.gainPearls(b.def.pearls, b);
        if (b.mega) { s.megalodon = true; setTimeout(() => this.ending(), 6000); }
        else {
            s.kings[b.key] = true; s.figureheads[b.key] = true; s.shards += 2;
            const g = this.world.gates.find(g => g.king === b.key);
            if (g) { g.prop.hidden = true; this.world.dynamicBlockers.find(x => x.gate === g).active = false; }
            setTimeout(() => this.ui.banner(ITEMS[b.def.figurehead].name, 'Bring sie zum Schiffshain.', 'big', 3.5), 5200);
        }
        this.boss = null;
        this.persist();
    }
    ending() {
        this.ui.openDialog('Die Laternenmagd', { pages: [
            'Du hast es getan. Das Wasser am Ende der Welt ist still.',
            'Die Schiffe im Hain haben ihre Gesichter wieder. Sie werden heimfahren, eines nach dem anderen.',
            'Und du? Die See hat dich zurückgegeben. Vielleicht, damit du ihr dies hier zurückgibst.',
            'Danke fürs Spielen von Schwarzwasser.'
        ], choices: [{ label: 'Weitersegeln', act: () => this.endTalk() }] });
    }
    // ---------------------------------------------------------------- sailing
    callShip(d) {
        this.ship.x = d.x; this.ship.y = d.y; this.ship.h = d.heading; this.ship.v = 0; this.ship.rise = 0; this.ship.alpha = 0;
        this.audio.sfx('rise'); this.shake(3);
        this.save.shipDock = d.zone;
    }
    board(d) {
        this.sailing = true; this.player.state = 'ship'; this.player.hidden = true;
        this.ship.sail = 1; this.audio.sfx('board'); this.lock = null;
        if (!this.boardedOnce) { this.boardedOnce = true; this.ui.toast('←→ steuern · ↑↓ Segel · J/L Breitseiten · E anlegen'); }
    }
    disembark(d) {
        this.sailing = false; this.player.state = 'free'; this.player.hidden = false;
        this.player.x = d.px; this.player.y = d.py;
        this.ship.x = d.x; this.ship.y = d.y; this.ship.h = d.heading; this.ship.v = 0; this.ship.sail = 0;
        this.save.shipDock = d.zone; this.audio.sfx('board');
        this.persist();
    }
    singShanty() {
        const s = this.save;
        if (!s.crew.length) { this.ui.toast('Ohne Crew singt niemand.'); return; }
        const el = s.crew[(this.shantyIdx = ((this.shantyIdx ?? -1) + 1) % s.crew.length)];
        this.shantyT = 40; this.shantyName = CREW[el].songName;
        this.audio.playSong?.(CREW[el].song, 40);
        this.ui.toast(`Die Crew singt „${this.shantyName}“. Der Wind dreht sich.`);
    }
    enterWhirlpool() {
        this.cutscene = true; let t = 0;
        this.audio.sfx('rise'); this.ui.banner('Der Strudel', 'Die See nimmt dich mit', 'boss', 3);
        const step = () => {
            t += 1 / 30;
            this.ship.h += 0.15; this.ship.x += (this.world.whirlpool.x - this.ship.x) * 0.05; this.ship.y += (this.world.whirlpool.y - this.ship.y) * 0.05;
            this.ui.fade = Math.min(1, Math.max(0, (t - 1.2) * 1.2));
            if (t < 2.4) return setTimeout(step, 33);
            const dock = this.world.docks.find(d => d.zone === 'hain');
            this.ship.x = dock.x; this.ship.y = dock.y; this.ship.h = dock.heading; this.save.shipDock = 'hain';
            this.sailing = false; this.player.hidden = false; this.player.state = 'free';
            this.player.x = this.world.endArrive.x; this.player.y = this.world.endArrive.y;
            if (!this.save.litFires.includes('ende')) this.save.litFires.push('ende');
            this.lightFires(); this.camF = [this.player.x, this.player.y];
            const back = () => { this.ui.fade = Math.max(0, this.ui.fade - 0.04); if (this.ui.fade > 0) setTimeout(back, 33); else this.cutscene = false; };
            back(); this.persist();
        };
        step();
    }
    // ---------------------------------------------------------------- menus
    pauseMenu() {
        this.audio.sfx('click');
        this.ui.openList('Pause', [
            { label: 'Weiter', act: () => this.ui.closeList() },
            { label: 'Seekarte', act: () => { this.ui.closeList(); this.ui.mapOpen = true; } },
            { label: 'Steuerung', act: () => { this.ui.closeList(); this.ui.legendOpen = true; } },
            { label: 'Werte', act: () => this.statsView() },
            { label: 'Optionen', act: () => { this.ui.closeList(); this.app.ui.options(); } },
            { label: 'Speichern und zum Titel', act: async () => { this.ui.closeList(); await this.persist(true); this.app.toTitle(); } }
        ], { desc: false });
    }
    statsView() {
        const s = this.save, d = derived(s);
        const items = [
            { label: 'Stufe', right: String(s.level) }, { label: 'Leben', right: String(d.maxHp) }, { label: 'Ausdauer', right: String(d.maxSt) }, { label: 'Geist', right: String(d.maxMp) },
            { label: 'Waffe', right: `${WEAPONS[s.weapon].name}${s.weaponLevel ? ' +' + s.weaponLevel : ''}` }, { label: 'Schaden', right: String(d.dmg) }, { label: 'Zauberkraft', right: String(d.spellDmg) },
            { label: 'Zauber', right: s.spells.map(k => SPELLS[k].name).join(', ') }, { label: 'Gezeitensplitter', right: String(s.shards) },
            { label: 'Galionsfiguren', right: `${Object.keys(s.mounted).length} angebracht, ${Object.keys(s.figureheads).length} erbeutet` }, { label: 'Crew', right: `${s.crew.length} von 4` },
            { label: 'Ertrunken', right: `${s.deaths}×` }
        ].map(i => ({ ...i, act: () => {} }));
        this.ui.openList(`${s.name}`, items, { onBack: () => this.pauseMenu(), desc: false, wide: true });
    }
    async persist(force = false) {
        if (!this.profile || !this.app.store) return;
        if (!force && this.saveT > 0) { this.saveDirty = true; return; }
        this.saveT = 1;
        try { await this.app.store.save(this.profile, this.save); } catch (e) { console.warn('Speichern fehlgeschlagen', e); }
    }
    // ---------------------------------------------------------------- rendering
    entities() {
        const out = [];
        const add = (e, sortY) => { if (e && !e.hidden) { if (sortY !== undefined) e.sortY = sortY; out.push(e); } };
        add(this.player.hidden ? null : this.player);
        for (const e of this.enemies) add(e, e.dead ? e.y - 6 : undefined);
        for (const b of Object.values(this.bosses)) if (!b.hidden && b.state !== 'dormant') add(b);
        for (const n of this.npcs) add(n);
        for (const w of this.walkers) add(w);
        for (const it of this.items) add(it);
        for (const s of this.shots) { s.shadow = s.kind === 'bomb' ? 4 : 0; add(s); }
        add(this.ship);
        for (const g of this.ghosts) if (!g.dead || g.alpha > 0) add(g);
        for (const s of this.sharks) add(s);
        if (!this.kraken.hidden) add(this.kraken);
        if (this.stain) add(this.stainEntity());
        return out;
    }
    stainEntity() {
        const S = this.stain, game = this;
        return { x: S.x, y: S.y, sortY: S.y - 20, draw() {}, drawGlow(r, cam) { const w = game.app.art.atlas.white; const k = 0.6 + 0.4 * Math.sin(S.t * 3); for (let i = 0; i < 9; i++) { const a = i / 9 * Math.PI * 2 + S.t; r.rect(w.tex, Math.round(S.x + Math.cos(a) * 5 - cam[0]), Math.round(S.y + Math.sin(a) * 3 - cam[1]), 1, 1, [0.37 * k * 2, 0.95 * k * 2, 0.78 * k * 2, 1]); } r.rect(w.tex, Math.round(S.x - cam[0]), Math.round(S.y - 2 - cam[1]), 1, 2, [0.7, 1.6, 1.4, 1]); } };
    }
    lights() {
        const out = [];
        const p = this.player, cam = this.cam;
        if (!this.sailing && !p.hidden) out.push({ x: p.x, y: p.y - 10, r: 70, col: [1, 0.85, 0.65], i: 0.32 });
        if (this.sailing) out.push({ x: this.ship.x, y: this.ship.y - 10, r: 80, col: [1, 0.75, 0.45], i: 0.35 });
        this.view.lights(cam, out);
        for (const s of this.shots) if (s.light) out.push({ x: s.x, y: s.y - s.z, r: 50, col: s.light, i: 1.3 });
        for (const b of Object.values(this.bosses)) if (!b.hidden && !b.dead && b.state !== 'dormant') out.push({ x: b.x, y: b.y - 20, r: b.human ? 70 : 110, col: [...KINGS[b.key]?.col || [1, 0.4, 0.2]], i: 0.6 });
        for (const g of this.ghosts) if (!g.dead) out.push({ x: g.x, y: g.y - 16, r: 70, col: [0.35, 1, 0.82], i: 0.6 });
        if (this.flashLight) { const f = this.flashLight; f.t -= 1 / 60; if (f.t > 0) out.push(f); else this.flashLight = null; }
        if (this.whirlpoolOpen) out.push({ x: this.world.whirlpool.x, y: this.world.whirlpool.y, r: 100, col: [0.35, 1, 0.82], i: 0.8 + Math.sin(this.t * 2) * 0.2 });
        if (this.stain) out.push({ x: this.stain.x, y: this.stain.y, r: 30, col: [0.35, 1, 0.82], i: 0.6 });
        this.fx.lights(out);
        return out;
    }
    drawWhirlpool(r, cam, glow) {
        if (!this.whirlpoolOpen) return;
        const w = this.app.art.atlas.white, W = this.world.whirlpool;
        if (W.x < cam[0] - 120 || W.x > cam[0] + r.W + 120 || W.y < cam[1] - 120 || W.y > cam[1] + r.H + 120) return;
        r.spriteBegin('alpha');
        for (let arm = 0; arm < 3; arm++) for (let i = 0; i < 70; i++) {
            const t = i / 70, a = arm * 2.09 + t * 9 - this.t * 2.2, rr = 6 + t * 70;
            const x = W.x + Math.cos(a) * rr - cam[0], y = W.y + Math.sin(a) * rr * 0.6 - cam[1];
            const c = glow ? [0.37 * (1 - t) * 1.6, 0.95 * (1 - t) * 1.6, 0.78 * (1 - t) * 1.6, 1] : [0.66, 0.78, 0.9, 0.9 - t * 0.6];
            r.rect(w.tex, Math.round(x), Math.round(y), 2, 1, c);
        }
        if (!glow) r.rect(w.tex, Math.round(W.x - 5 - cam[0]), Math.round(W.y - 3 - cam[1]), 10, 6, [0.02, 0.03, 0.05, 1]);
        r.end();
    }
    frame() {
        const r = this.app.renderer, cam = this.cam, v = this.view;
        r.grade = this.gradeCur;
        r.flash = this.flashC;
        const ents = this.entities();
        const focus = this.focus ? { ...this.focus, x: this.player.x, y: this.player.y } : { x: this.player.x, y: this.player.y };
        return {
            cam, ambient: this.gradeCur.ambient, lightmap: v.lightmap, lights: this.lights(), lightK: this.lightK ?? 1,
            drawScene: (r) => {
                v.drawScene(r, cam, ents, focus, { under: (r) => { this.drawWhirlpool(r, cam, false); this.fx.drawUnder(r, cam); } });
                drawCritters(this, r, cam, false);
                this.fx.drawParticles(r, cam, false);
                v.drawFog(r, cam, this.zoneKey || 'kueste', this.sailing ? 0.6 : 1);
            },
            drawGlow: (r) => { v.drawGlow(r, cam, ents); this.drawWhirlpool(r, cam, true); drawCritters(this, r, cam, true); this.fx.drawParticles(r, cam, true); },
            drawUI: (r) => { this.fx.drawNumbers(r, cam, this.app.font); this.ui.draw(r, this); this.ui.drawHudGlow(r, this); }
        };
    }
}

// older saves → current format
function migrate(s) {
    s.version = 2;
    s.owned ||= [s.weapon]; s.weaponLevel ||= 0; s.phialsMax ||= 1; s.bullets ??= 0; s.bombs ??= 0; s.shards ??= 0;
    s.spells ||= ['bolt']; s.spell ||= 0; s.items ||= {}; s.picked ||= []; s.chests = Array.isArray(s.chests) ? s.chests : [];
    s.kings ||= {}; s.figureheads ||= {}; s.mounted ||= {}; s.crew ||= []; s.talked ||= {}; s.shipDock ||= 'kueste';
    if (!Array.isArray(s.explored)) s.explored = s.explored === 'all' ? Object.keys(ZONES).filter(k => k !== 'ende') : ['kueste'];
    s.litFires ||= []; s.lastFire ||= 'kueste';
    if (!['kueste', 'hain', 'asche', 'asche2', 'frost', 'frost2', 'raben', 'raben2', 'mangroven', 'mangroven2', 'ende'].includes(s.lastFire)) s.lastFire = 'kueste';
    s.litFires = s.litFires.filter(f => ['kueste', 'hain', 'asche', 'asche2', 'frost', 'frost2', 'raben', 'raben2', 'mangroven', 'mangroven2', 'ende'].includes(f));
    s.look = { ...{ body: 0, skin: 1, hair: 1, hairCol: 1, beard: 1, hat: 1, coat: 0, patch: 0 }, ...(s.look || {}) };
    for (const k of ['body', 'skin', 'hair', 'hairCol', 'beard', 'hat', 'coat', 'patch']) if (typeof s.look[k] !== 'number' || s.look[k] < 0) s.look[k] = 0;
    s.look.coat %= 6; s.look.skin %= 4; s.look.hair %= 5; s.look.hairCol %= 5; s.look.beard %= 4; s.look.hat %= 5; s.look.body %= 2; s.look.patch = s.look.patch ? 1 : 0;
}
