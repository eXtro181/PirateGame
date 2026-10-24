// Game orchestration: modes (land/sea), combat resolution, fires, death and pearls, camera.
import { Player } from './player.js';
import { Crab, Skeleton, Shark } from './enemies.js';
import { Ship, SHIP_R } from './ship.js';
import { T } from '../world/defs.js';
import { ChestFigure } from './figure.js';
import { MeshBuilder } from '../gfx/mesh.js';
import { MeshInstance, Entity } from 'playcanvas';
import { Vec3 } from 'playcanvas';

const _v = new Vec3(), _o = new Vec3();
import { Fx } from './fx.js';
import { derived, ORIGINS, OFFHAND, levelCost } from './data.js';

export class Game {
    constructor({ stage, M, lib, models, particles, map, terrain, view, input, audio, hud, ui, store }) {
        Object.assign(this, { stage, M, lib, models, particles, map, terrain, view, input, audio, hud, ui, store });
        this.app = stage.app;
        this.time = 0;
        this.mode = 'land';
        this.paused = true;
        this.camX = map.start.x; this.camZ = map.start.z;
        this.shakeT = 0; this.shakeAmp = 0;
        this.hitstop = 0;
        this.wind = { angle: 2.3, strength: 1 };
        this.lockTarget = null;
        this.projectiles = [];
        this.fx = new Fx(this);
        this.explored = new Uint8Array(map.w * map.h);
        this.exploreT = 0;

        this.ship = new Ship(this);
        this.ship.place(map.dock.shipX, map.dock.shipZ, map.dock.heading);
        this.enemies = map.spawns.map(s => s.type === 'crab' ? new Crab(this, s) : s.type === 'skeleton' ? new Skeleton(this, s) : new Shark(this, s));
        this.chests = {};
        for (const c of map.chests) {
            const f = new ChestFigure(this.app, this.M, this.models);
            f.setPos(c.x, terrain.ground(c.x, c.z), c.z);
            f.setYaw(0.3, 1);
            this.chests[c.id] = f;
        }
        // lost pearls: a glowing orb hovering where you died
        const sb = new MeshBuilder(); sb.sphere(0.16, 12, 8);
        this.stain = new Entity('stain');
        this.stain.addComponent('render', { meshInstances: [new MeshInstance(sb.toMesh(this.app.graphicsDevice), this.M.glow('#7fe8ff', 6))] });
        this.app.root.addChild(this.stain);
        this.stain.enabled = false;
        this.stainLight = view.addOmni(0, 0.8, 0, [0.4, 0.9, 1.0], 0, 5);
        this.app.on('update', dt => this.update(dt));
    }

    // ---------------------------------------------------------------- session
    start(profile, save) {
        this.profile = profile;
        this.save = save;
        this.originData = ORIGINS.find(o => o.id === save.origin) || ORIGINS[0];
        this.save.hints = this.save.hints || {};
        if (this.player) this.player.setSave(save);
        else this.player = new Player(this, save);
        if (save.explored === 'all') this.explored.fill(1);
        else if (save.explored) this.explored.set(decodeBits(save.explored, this.explored.length));
        else this.explored.fill(0);
        this.refreshDerived();
        for (const c of this.map.chests) this.chests[c.id].setOpen(save.chests.includes(c.id));
        this.respawn(true);
        this.paused = false;
        const fresh = save.playTime < 1;
        if (fresh) {
            this.ui.banner('Schwarzwasser', 'Du bist ertrunken. Die Flut hat dich zurückgegeben.', 'title', 5.5);
            this.hint('controls', this.touch
                ? 'Daumen links auflegen und ziehen zum Laufen. Rechts: Hieb, Rolle, Parieren, Rum, Blitz. Tippe auf Hinweise wie „Rasten“, um zu interagieren.'
                : 'WASD bewegen · J Angriff · L schwer · Leertaste Rolle · Q Parieren · C Kettenblitz · 1–4 Rum · E Interagieren · Tab Zielen');
        }
    }

    logout() {
        this.persist(null);
        this.save = null;
        this.paused = true;
        this.player.sprite.setVisible(false);
        this.lockTarget = null;
        this.hud.el.classList.add('hidden');
        this.hud.prompt(null);
        this.onLogout?.();
    }

    refreshDerived() {
        this.derived = derived(this.save);
    }

    respawn(initial = false) {
        const s = this.save;
        const fire = this.map.fires.find(f => f.id === s.lastFire && s.litFires.includes(f.id));
        this.refillAll();
        for (const e of this.enemies) e.reset();
        this.view.clearBlood();
        this.ship.hull = this.ship.maxHull;
        this.lockTarget = null;
        if (fire && fire.kind === 'sea') {
            this.ship.place(fire.x - 3.5, fire.z + 2, 0);
            this.mode = 'sea';
            this.player.place(this.ship.x, this.ship.z);
            this.player.sprite.setVisible(false);
        } else {
            const p = fire ? { x: fire.x - 0.2, z: fire.z + 1.3 } : this.map.start;
            if (this.ship.hull <= 0 || !initial) this.ship.place(this.map.dock.shipX, this.map.dock.shipZ, this.map.dock.heading);
            this.mode = 'land';
            this.player.place(p.x, p.z);
            this.player.face(0, 1);
        }
        this.summonCd = 0;
        this.camX = this.player.x; this.camZ = this.player.z;
        this.updateStain();
        this.audio.setMood(this.mode === 'sea' ? 'sea' : 'land');
    }

    refillAll() {
        const p = this.player, s = this.save;
        p.hp = this.derived.maxHp; p.st = this.derived.maxSt;
        p.flasks = s.flasksMax;
        p.mp = this.derived.maxMp;
        p.ammo = s.off !== 'none' ? OFFHAND[s.off].ammo : 0;
        p.state = 'idle';
    }

    persist(msg = 'Gespeichert') {
        this.save.explored = this.explored.every(v => v) ? 'all' : encodeBits(this.explored);
        return this.store.save(this.profile, this.save).then(() => { if (msg) this.ui.saveIcon(); }).catch(e => { console.error(e); this.toast('Speichern fehlgeschlagen'); });
    }

    get touch() { return this.stage.device.touch; }

    // ---------------------------------------------------------------- feedback
    sfx(name) { this.audio.sfx(name); }
    shake(t, amp) { this.shakeT = Math.max(this.shakeT, t); this.shakeAmp = Math.max(this.shakeAmp, amp); }
    toast(text) { this.ui.toast(text); }
    hint(id, text) {
        if (!this.save || this.save.hints[id]) return;
        this.save.hints[id] = 1;
        this.ui.hint(text);
    }
    gy(x, z) { return this.terrain.ground(x, z); }
    slashFx(p, heavy) {
        this.fx.slash(p.x, this.gy(p.x, p.z), p.z, Math.atan2(p.fx, p.fz), heavy, (p.combo || 0) % 2 === 1);
    }
    sparkFx(x, z, y = 1.0) { this.fx.burst('spark', x, this.gy(x, z) + y, z); this.fx.flash(x, this.gy(x, z) + y, z, [1, 0.8, 0.5], 6, 0.1, 5); }
    bloodFx(x, z) { this.fx.burst('blood', x, this.gy(x, z) + 0.9, z); this.view.addBlood(x + (Math.random() - 0.5) * 0.4, z + (Math.random() - 0.5) * 0.4, 0.5 + Math.random() * 0.6); }
    healFx(p) { this.fx.burst('heal', p.x, this.gy(p.x, p.z) + 0.3, p.z); this.fx.flash(p.x, this.gy(p.x, p.z) + 1.5, p.z, [1, 0.75, 0.35], 6, 0.6, 6); this.sfx('heal'); }
    smokeFx(x, z) { this.fx.burst('smoke', x, Math.max(0, this.gy(x, z)) + 1.0, z); this.fx.flash(x, Math.max(0, this.gy(x, z)) + 1.2, z, [1, 0.6, 0.25], 14, 0.12, 8); }
    splashFx(x, z) { this.fx.burst('splash', x, 0.05, z); }

    nearestEnemy(x, z, r) {
        let best = null, bd = r;
        for (const e of this.enemies) {
            if (!e.alive || e.sea || e.state === 'pile') continue;
            const d = Math.hypot(e.x - x, e.z - z);
            if (d < bd) { bd = d; best = e; }
        }
        return best;
    }
    // Kettenblitz: jumps from the caster to up to four enemies, each jump weaker
    chainLightning(p) {
        const gy = this.gy(p.x, p.z);
        const pts = [[p.x + p.fx * 0.5, gy + 1.35, p.z + p.fz * 0.5]];
        const hit = new Set();
        let cx = p.x, cz = p.z, dmg = this.derived.spellDmg;
        let first = this.lockTarget && this.lockTarget.alive ? this.lockTarget : null;
        for (let k = 0; k < 4; k++) {
            let tgt = first && k === 0 ? first : null;
            if (!tgt) {
                let bd = k === 0 ? 8 : 5;
                for (const e of this.enemies) {
                    if (!e.alive || e.sea || hit.has(e) || e.state === 'pile') continue;
                    const d = Math.hypot(e.x - cx, e.z - cz);
                    if (k === 0 && (e.x - p.x) * p.fx + (e.z - p.z) * p.fz < -0.5) continue; // roughly in front
                    if (d < bd) { bd = d; tgt = e; }
                }
            }
            if (!tgt) break;
            hit.add(tgt);
            pts.push([tgt.x, this.gy(tgt.x, tgt.z) + 0.9, tgt.z]);
            tgt.damage(Math.round(dmg), 2, { x: cx, z: cz });
            cx = tgt.x; cz = tgt.z; dmg *= 0.78;
        }
        if (pts.length === 1) { // nothing in reach: the bolt strikes the ground ahead
            pts.push([p.x + p.fx * 4, gy + 0.1, p.z + p.fz * 4]);
        }
        this.fx.lightning(pts);
        this.sfx('zap');
        this.shake(0.2, 0.12);
        this.hitstop = 0.05;
    }

    // ---------------------------------------------------------------- combat
    meleeHit(p, atk) {
        for (const e of this.enemies) {
            if (!e.alive || e.sea || atk.hit.has(e)) continue;
            const dx = e.x - p.x, dz = e.z - p.z, l = Math.hypot(dx, dz);
            if (l > atk.range + e.radius) continue;
            const ang = Math.acos(Math.max(-1, Math.min(1, (dx * p.fx + dz * p.fz) / (l || 1))));
            if (l > 0.45 && ang > atk.arc) continue;
            atk.hit.add(e);
            if (e.damage(atk.dmg, atk.poise, p)) {
                this.sparkFx(e.x - dx / (l || 1) * 0.3, e.z - dz / (l || 1) * 0.3);
                if (!(e instanceof Skeleton)) this.bloodFx(e.x, e.z);
                this.sfx(atk.riposte ? 'crit' : 'hit');
                this.hitstop = atk.riposte ? 0.16 : (atk.heavy ? 0.08 : 0.05);
                this.shake(0.1, atk.heavy ? 0.12 : 0.06);
                if (atk.riposte) this.ui.floatText('KRITISCH', e.x, e.z, 'crit');
            }
        }
    }

    enemyHit(e, range, arc, dmg) {
        const p = this.player;
        if (e.attackLanded === e.attackId) return;
        if (p.state === 'dead' || this.mode !== 'land') return;
        const dx = p.x - e.x, dz = p.z - e.z, l = Math.hypot(dx, dz);
        if (l > range + p.radius) return;
        const ang = Math.acos(Math.max(-1, Math.min(1, (dx * e.fx + dz * e.fz) / (l || 1))));
        if (ang > arc) return;
        e.attackLanded = e.attackId;
        if (p.isParrying()) {
            e.parried();
            this.sfx('parried');
            this.sparkFx((p.x + e.x) / 2, (p.z + e.z) / 2, 1.2);
            this.hitstop = 0.12;
            this.ui.floatText('PARIERT', e.x, e.z, 'parry');
            this.hint('riposte', 'Pariert! Greife jetzt an für einen kritischen Gegenschlag.');
            return;
        }
        if (p.isInvulnerable()) return;
        p.hurt(dmg, e.x, e.z);
    }

    // at most two enemies may wind up or strike at the same time
    attackToken(e) {
        let n = 0;
        for (const o of this.enemies) if (o !== e && o.alive && (o.state === 'windup' || o.state === 'attack')) n++;
        return n < 2;
    }

    shoot(p, kind) {
        const tide = kind === 'tide';
        const sp = tide ? 9 : 16;
        const b = this.fx.ball(tide ? 'tide' : 'pistol');
        const y = this.gy(p.x, p.z) + 1.15;
        if (tide) this.fx.flash(p.x, y, p.z, [0.3, 0.9, 1], 6, 0.3, 5); else this.fx.flash(p.x + p.fx, y, p.z + p.fz, [1, 0.7, 0.3], 14, 0.1, 7);
        this.projectiles.push({ b, x: p.x + p.fx * 0.6, y, z: p.z + p.fz * 0.6, vx: p.fx * sp, vz: p.fz * sp, life: tide ? 0.9 : 0.55, dmg: this.derived.offDmg, land: true });
        this.sfx(tide ? 'tide' : 'pistol');
        if (!tide) this.smokeFx(p.x + p.fx * 0.8, p.z + p.fz * 0.8);
    }

    spawnBall(x, z, vx, vz, dmg, delay) {
        const b = this.fx.ball('cannon');
        this.projectiles.push({ b, x, y: 1.3, z, vx, vz, life: 0.85 + delay, dmg, sea: true });
    }

    ramShip(shark, dmg) {
        this.ship.damage(dmg);
        const f = this.ship.fwd;
        this.ship.dx += (this.ship.x - shark.x) * 1.5; this.ship.dz += (this.ship.z - shark.z) * 1.5;
        this.sfx('crash');
        this.splashFx(this.ship.x, this.ship.z);
        this.hint('ram', this.touch ? 'Rostkiefer rammt dein Schiff! Dreh ihm die Breitseite zu und feuere.' : 'Rostkiefer rammt dein Schiff! Breitseiten: J links, K rechts. W/S Segel, A/D steuern.');
    }

    updateProjectiles(dt) {
        for (let i = this.projectiles.length - 1; i >= 0; i--) {
            const b = this.projectiles[i];
            b.life -= dt;
            b.x += b.vx * dt; b.z += b.vz * dt;
            if (b.sea) b.y = Math.max(0.1, b.y - dt * 0.9);
            b.b.e.setPosition(b.x, b.y, b.z);
            let done = b.life <= 0;
            for (const e of this.enemies) {
                if (done || !e.alive) continue;
                if (b.sea !== !!e.sea && !(b.land && !e.sea)) continue;
                if (Math.hypot(e.x - b.x, e.z - b.z) < e.radius + 0.25) {
                    e.damage(b.dmg, 1, { x: b.x - b.vx, z: b.z - b.vz });
                    this.sparkFx(b.x, b.z); this.sfx('hit');
                    done = true;
                }
            }
            if (!done && b.land && this.map.blockedWalk(b.x, b.z, 0.05)) { done = true; this.sparkFx(b.x, b.z); }
            if (done) {
                if (b.sea) this.splashFx(b.x, b.z);
                this.fx.free(b.b);
                this.projectiles.splice(i, 1);
            }
        }
    }

    onEnemyKilled(e) {
        const greed = (this.originData.greed || 1) * (this.save.gift === 'coin' ? 1.2 : 1);
        const n = Math.round(e.pearls * greed);
        this.save.pearls += n;
        this.ui.pearlGain(n);
        this.fx.burst('pearl', e.x, this.gy(e.x, e.z) + 0.8, e.z);
        if (this.lockTarget === e) this.lockTarget = null;
    }

    // ---------------------------------------------------------------- death
    onPlayerDeath() {
        this.dying = 0;
        this.sfx('death');
        this.audio.setMood('death');
        this.lockTarget = null;
    }
    onShipSunk() {
        this.dying = 0;
        this.sinking = true;
        this.sfx('death');
        this.audio.setMood('death');
    }
    updateDeath(dt) {
        if (this.dying === undefined) return;
        this.dying += dt;
        if (this.sinking && this.ship.state === 'surfaced') { this.ship.state = 'sinking'; this.ship.riseT = 1; }
        if (this.dying > 1.2 && !this.deathShown) { this.deathShown = true; this.ui.death(); }
        if (this.dying > 5.2) {
            const s = this.save;
            const sea = this.mode === 'sea';
            s.stain = s.pearls > 0 ? { x: sea ? this.ship.x : this.player.x, z: sea ? this.ship.z : this.player.z, amount: s.pearls, sea } : null;
            s.pearls = 0;
            s.deaths = (s.deaths || 0) + 1;
            this.dying = undefined; this.deathShown = false; this.sinking = false;
            this.respawn();
            this.persist(null);
            this.ui.hideDeath();
            if (s.stain) this.hint('stain', 'Deine Perlen liegen dort, wo du gestorben bist. Hol sie zurück – stirbst du vorher, sind sie verloren.');
        }
    }
    updateStain() {
        const st = this.save?.stain;
        this.stain.enabled = !!st;
        this.stainLight.light.intensity = st ? 5 : 0;
        if (st) { const y = Math.max(0, this.gy(st.x, st.z)) + 0.7; this.stain.setPosition(st.x, y, st.z); this.stainLight.setLocalPosition(st.x, y + 0.3, st.z); }
    }
    checkStain() {
        const st = this.save.stain;
        if (!st) return;
        const sea = this.mode === 'sea';
        if (st.sea !== sea) return;
        const px = sea ? this.ship.x : this.player.x, pz = sea ? this.ship.z : this.player.z;
        if (Math.hypot(px - st.x, pz - st.z) < (sea ? 2 : 0.9)) {
            this.save.pearls += st.amount;
            this.ui.banner('Perlen zurückgewonnen', `+${st.amount}`, 'gold', 2.5);
            this.sfx('pearls');
            this.save.stain = null;
            this.updateStain();
        }
    }

    // ---------------------------------------------------------------- interaction
    nearestInteraction() {
        const m = this.map, p = this.player, s = this.save;
        if (this.mode === 'land') {
            for (const f of m.fires) if (f.kind === 'land' && Math.hypot(p.x - f.x, p.z - f.z - 0.3) < 1.6) return { label: s.litFires.includes(f.id) ? 'Rasten' : 'Feuer entzünden', act: () => this.useFire(f) };
            for (const c of m.chests) if (!s.chests.includes(c.id) && Math.hypot(p.x - c.x, p.z - c.z) < 1.3) return { label: 'Truhe öffnen', act: () => this.openChest(c) };
            if (this.ship.ready && this.shipDist(p.x, p.z) < 4.6) return { label: 'An Bord gehen', act: () => this.board() };
        } else {
            for (const f of m.fires) if (f.kind === 'sea' && Math.hypot(this.ship.x - f.x - 1, this.ship.z - f.z) < 4.2) return { label: s.litFires.includes(f.id) ? 'Am Wrack rasten' : 'Wrackfeuer entzünden', act: () => this.useFire(f) };
            const land = this.landingSpot();
            if (land) return { label: 'An Land gehen', act: () => this.disembark(land) };
        }
        return null;
    }
    landingSpot() {
        const sx = this.ship.x, sz = this.ship.z;
        let best = null, bd = 4.4;
        for (let j = -5; j <= 5; j++) for (let i = -5; i <= 5; i++) {
            const tx = Math.floor(sx) + i, tz = Math.floor(sz) + j;
            const cx = tx + 0.5, cz = tz + 0.5;
            if (this.map.blockedWalk(cx, cz, 0.3)) continue;
            const d = this.shipDist(cx, cz);
            if (d < bd) { bd = d; best = { x: cx, z: cz }; }
        }
        return best;
    }
    // distance from a point to the ship's keel line (the hull is long, not round)
    shipDist(x, z) {
        const sh = this.ship, f = sh.fwd, half = sh.len / 2 - 0.3;
        const t = Math.max(-half, Math.min(half, (x - sh.x) * f.x + (z - sh.z) * f.z));
        return Math.hypot(x - (sh.x + f.x * t), z - (sh.z + f.z * t));
    }
    // the Möwenschrei answers its captain: at any shore it rises from the deep
    updateSummon(dt) {
        const sh = this.ship, p = this.player, m = this.map;
        this.summonCd = Math.max(0, (this.summonCd || 0) - dt);
        if (sh.hull <= 0) return;
        const d = Math.hypot(p.x - sh.x, p.z - sh.z);
        // left behind: it sinks back into the deep
        if (sh.state === 'surfaced' && d > 22) { sh.state = 'submerged'; sh.riseT = 0; sh.sync(); }
        if (this.summonCd > 0 || sh.state === 'rising' || sh.state === 'sinking') return;
        // at the shore?
        let shore = false;
        for (let j = -2; j <= 2 && !shore; j++) for (let i = -2; i <= 2; i++) {
            const tx = Math.floor(p.x + i * 0.8), tz = Math.floor(p.z + j * 0.8);
            if (m.get(tx, tz) === T.WATER) { shore = true; break; }
        }
        if (!shore) return;
        if (sh.state === 'surfaced') {
            if (d < 9) return;
            sh.dive(); // far away: it goes down and comes up here
            this.summonCd = 2.8;
            return;
        }
        const spot = this.riseSpot(p.x, p.z);
        if (!spot) return;
        sh.rise(spot.x, spot.z, spot.heading);
        this.summonCd = 3;
        this.sfx('rise');
        this.shake(0.25, 1.6);
        for (let k = 0; k < 5; k++) setTimeout(() => this.splashFx(sh.x + (Math.random() - 0.5) * 3, sh.z + (Math.random() - 0.5) * 2), k * 260);
        this.hint('rise', 'Die Möwenschrei steigt aus der Tiefe. Wo immer du ans Ufer trittst, folgt sie dir.');
    }
    riseSpot(px, pz) {
        let best = null, bd = 1e9;
        for (let r = 2.6; r <= 6.5; r += 0.6) {
            for (let a = 0; a < 16; a++) {
                const ang = a / 16 * Math.PI * 2, cx = px + Math.cos(ang) * r, cz = pz + Math.sin(ang) * r;
                if (this.map.get(Math.floor(cx), Math.floor(cz)) !== T.WATER) continue;
                // broadside to the shore looks best; try both directions along it
                for (const heading of [ang, ang + Math.PI, ang + Math.PI / 2]) {
                    if (this.map.blockedSail(cx, cz, SHIP_R, heading)) continue;
                    const d = this.shipDistFrom(px, pz, cx, cz, heading);
                    if (d < 1.5) continue; // never right on top of the captain
                    if (d < bd) { bd = d; best = { x: cx, z: cz, heading }; }
                }
            }
            if (best && bd < 2.2) break;
        }
        return best;
    }
    shipDistFrom(x, z, sx, sz, heading) {
        const fx = Math.sin(heading), fz = -Math.cos(heading), half = this.ship.len / 2 - 0.3;
        const t = Math.max(-half, Math.min(half, (x - sx) * fx + (z - sz) * fz));
        return Math.hypot(x - (sx + fx * t), z - (sz + fz * t));
    }
    board() {
        this.mode = 'sea';
        this.player.sprite.setVisible(false);
        this.lockTarget = null;
        this.ship.sail = 0;
        this.sfx('board');
        this.audio.setMood('sea');
        this.hint('sail', this.touch
            ? 'Stick links/rechts: steuern · Segel + / −: Fahrt · Breitseiten feuern zur Seite · Tippe „An Land gehen“, um anzulegen.'
            : 'W/S: Segel setzen · A/D: steuern · J/K: Breitseite links/rechts · E: an Land gehen · F: Shanty');
    }
    disembark(spot) {
        this.mode = 'land';
        this.ship.sail = 0; this.ship.speed = 0;
        this.player.place(spot.x, spot.z);
        this.player.sprite.setVisible(true);
        this.sfx('board');
        this.audio.setMood('land');
    }
    openChest(c) {
        const s = this.save;
        s.chests.push(c.id);
        this.chests[c.id].setOpen(true);
        this.fx.flash(c.x, this.gy(c.x, c.z) + 0.8, c.z, [1, 0.8, 0.45], 10, 1.2, 6);
        s.pearls += c.pearls;
        this.sfx('chest');
        if (c.item === 'flask') { s.flasksMax++; this.player.flasks++; this.ui.banner('Rumflaschen-Splitter', `Eine Rumflasche mehr · +${c.pearls} Perlen`, 'gold', 3.5); }
        else { if (!s.songs.includes('leine')) s.songs.push('leine'); this.ui.banner('Vergilbtes Liedblatt', `„Hol die Leine“ – die Crew kennt jetzt den Shanty · +${c.pearls} Perlen`, 'gold', 4); }
    }
    useFire(f) {
        const s = this.save;
        const first = !s.litFires.includes(f.id);
        if (first) {
            s.litFires.push(f.id);
            this.sfx('kindle');
            this.ui.banner(f.kind === 'sea' ? 'Wrackfeuer entzündet' : 'Feuer entzündet', f.name, 'gold', 3);
        }
        s.lastFire = f.id;
        this.refillAll();
        this.ship.hull = this.ship.maxHull;
        for (const e of this.enemies) e.reset();
        this.lockTarget = null;
        this.paused = true;
        if (this.mode === 'land') { this.player.state = 'sit'; this.player.t = 0; this.player.face(f.x - this.player.x, f.z - this.player.z); }
        this.audio.setMood('fire');
        this.persist();
        setTimeout(() => this.ui.restMenu(f), first ? 1600 : 200);
    }
    leaveFire() {
        this.paused = false;
        if (this.mode === 'land') this.player.toIdle();
        this.audio.setMood(this.mode === 'sea' ? 'sea' : 'land');
    }
    levelUp(stat) {
        const s = this.save;
        const cost = levelCost(s.level);
        if (s.pearls < cost) return false;
        s.pearls -= cost;
        s.level++;
        s.stats[stat]++;
        this.refreshDerived();
        this.refillAll();
        this.player.state = 'frozen';
        this.sfx('level');
        this.persist();
        return true;
    }
    travel(f) {
        this.save.lastFire = f.id;
        this.respawn();
        this.persist(null);
        this.paused = true;
        this.audio.setMood('fire');
        if (this.mode === 'land') { this.player.state = 'sit'; this.player.t = 0; }
    }

    // ---------------------------------------------------------------- loop
    update(rawDt) {
        const inp = this.input;
        inp.poll();
        if (!this.save) { // attract mode behind the title screen
            this.time += rawDt;
            this.view.update(rawDt);
            this.fx.update(rawDt);
            this.onAttract?.(rawDt);
            inp.endFrame();
            return;
        }
        if (inp.pressed('menu') && !this.ui.modalOpen() && this.dying === undefined) this.ui.pauseMenu();
        if (inp.pressed('map') && !this.ui.modalOpen()) this.ui.toggleMap();
        if (this.paused || this.ui.modalOpen()) {
            this.view.update(rawDt, { x: this.camX, z: this.camZ }); this.fx.update(rawDt);
            for (const c of Object.values(this.chests)) c.animate(rawDt);
            if (this.mode === 'land' && this.player) this.player.animate(rawDt);
            this.updateCamera(rawDt); this.hud.update(); inp.endFrame(); return;
        }
        let dt = rawDt;
        if (this.hitstop > 0) { this.hitstop -= rawDt; dt = rawDt * 0.05; }
        this.time += dt;
        this.save.playTime += rawDt;
        this.wind.angle = 2.3 + Math.sin(this.time * 0.015) * 0.9;
        this.wind.strength = 0.85 + Math.sin(this.time * 0.05) * 0.15;

        if (this.mode === 'land') {
            if (inp.pressed('lock')) this.toggleLock();
            this.player.update(dt);
            if (this.dying === undefined) this.updateSummon(dt);
        } else {
            this.ship.update(dt, this.dying === undefined);
            this.player.x = this.ship.x; this.player.z = this.ship.z;
            if (inp.pressed('shanty')) this.requestShanty();
        }
        if (this.mode !== 'sea') this.ship.update(dt, false);
        for (const e of this.enemies) {
            if (!e.alive && e.state !== 'dead' && e.state !== 'gone') continue;
            // a fresh attack id each time an attack begins
            if (e.state === 'attack' && e._prev !== 'attack') e.attackId = (e.attackId || 0) + 1;
            e._prev = e.state;
            if (Math.hypot(e.x - this.camX, e.z - this.camZ) < 22 || e.sea) e.update(dt);
        }
        this.updateProjectiles(dt);
        this.fx.update(dt);
        this.view.update(dt, { x: this.camX, z: this.camZ });
        for (const c of Object.values(this.chests)) c.animate(dt);
        this.updateDeath(rawDt);
        if (this.dying === undefined) {
            this.checkStain();
            const it = this.nearestInteraction();
            this.hud.prompt(it ? it.label : null);
            if (it && inp.pressed('interact')) it.act();
        } else this.hud.prompt(null);
        if (this.stain.enabled) { const st = this.save.stain; this.stain.setPosition(st.x, Math.max(0, this.gy(st.x, st.z)) + 0.7 + Math.sin(this.time * 2) * 0.08, st.z); }
        this.audio.update(dt, this);
        this.updateExplored(dt);
        this.updateCamera(rawDt);
        this.hud.update();
        inp.endFrame();
    }

    toggleLock() {
        if (this.lockTarget) { this.lockTarget = null; return; }
        let best = null, bd = 9;
        for (const e of this.enemies) {
            if (!e.alive || e.sea || e.state === 'pile') continue;
            const d = Math.hypot(e.x - this.player.x, e.z - this.player.z);
            if (d < bd) { bd = d; best = e; }
        }
        this.lockTarget = best;
        if (best) this.sfx('lock');
    }

    requestShanty() {
        if (!this.save.songs.includes('leine')) { this.toast('Die Crew kennt noch keinen Shanty. Finde ein Liedblatt.'); return; }
        this.audio.crewSing();
        this.toast('„Hol die Leine!“ – die Crew stimmt an.');
    }

    updateExplored(dt) {
        this.exploreT -= dt;
        if (this.exploreT > 0) return;
        this.exploreT = 0.3;
        const r = this.mode === 'sea' ? 11 : 8;
        const cx = Math.floor(this.player.x), cz = Math.floor(this.player.z);
        for (let j = -r; j <= r; j++) for (let i = -r; i <= r; i++) {
            if (i * i + j * j > r * r) continue;
            const x = cx + i, z = cz + j;
            if (x >= 0 && z >= 0 && x < this.map.w && z < this.map.h) this.explored[z * this.map.w + x] = 1;
        }
    }

    updateCamera(dt) {
        const sea = this.mode === 'sea';
        let tx = this.player.x, tz = this.player.z;
        if (sea) { const f = this.ship.fwd; tx = this.ship.x + f.x * this.ship.speed * 0.45; tz = this.ship.z + f.z * this.ship.speed * 0.45; }
        if (this.lockTarget && !sea) { tx = (tx * 2 + this.lockTarget.x) / 3; tz = (tz * 2 + this.lockTarget.z) / 3; }
        // smooth follow (critically damped like SmoothDamp)
        const k = 1 - Math.pow(0.0025, dt);
        this.camX += (tx - this.camX) * k;
        this.camZ += (tz - this.camZ) * k;
        const ty = sea ? 0 : Math.max(0, this.gy(this.camX, this.camZ));
        this.camY = (this.camY ?? ty) + (ty - (this.camY ?? ty)) * k;
        let sx = 0, sz = 0;
        if (this.shakeT > 0) {
            this.shakeT -= dt;
            const fall = Math.max(0, this.shakeT) * 4;
            sx = (Math.random() - 0.5) * this.shakeAmp * 2 * Math.min(1, fall); sz = (Math.random() - 0.5) * this.shakeAmp * 2 * Math.min(1, fall);
            if (this.shakeT <= 0) this.shakeAmp = 0;
        }
        // a little further out at sea; the wheel zooms a bit (in tight limits)
        const want = (sea ? 1.35 : 1) * (this.userZoom || 1);
        this.camZoom = (this.camZoom || 1) + (want - (this.camZoom || 1)) * (1 - Math.pow(0.05, dt));
        this.stage.lookAt(this.camX + sx, this.camY + 0.6, this.camZ + sz, this.camZoom);
    }

    // world -> screen (CSS pixels relative to the canvas); h = height above the ground
    toScreen(x, z, h = 0) {
        const c = this.stage.app.graphicsDevice.canvas;
        const r = c.getBoundingClientRect();
        _v.set(x, (this.mode === 'sea' && h < 1.5 ? 0 : Math.max(0, this.gy(x, z))) + h, z);
        this.stage.camera.camera.worldToScreen(_v, _o);
        const cr = this.stage.app.graphicsDevice.clientRect;
        // relative to the canvas: the HUD layer covers exactly the canvas
        return { x: _o.x * r.width / (cr.width || r.width), y: _o.y * r.height / (cr.height || r.height) };
    }
}

function encodeBits(arr) {
    const bytes = new Uint8Array(Math.ceil(arr.length / 8));
    for (let i = 0; i < arr.length; i++) if (arr[i]) bytes[i >> 3] |= 1 << (i & 7);
    let s = '';
    for (const b of bytes) s += String.fromCharCode(b);
    return btoa(s);
}
function decodeBits(str, n) {
    const out = new Uint8Array(n);
    try {
        const s = atob(str);
        for (let i = 0; i < n; i++) out[i] = (s.charCodeAt(i >> 3) >> (i & 7)) & 1;
    } catch { /* ignore */ }
    return out;
}
