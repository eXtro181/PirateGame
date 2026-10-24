// Game orchestration: modes (land/sea), combat resolution, fires, death and pearls, camera.
import { Sprite } from '../core/sprite.js';
import { buildPlayerSheet } from '../core/assets.js';
import { Player } from './player.js';
import { Crab, Skeleton, Shark } from './enemies.js';
import { Ship } from './ship.js';
import { Fx } from './fx.js';
import { derived, ORIGINS, OFFHAND, levelCost } from './data.js';

export class Game {
    constructor({ stage, art, map, view, input, audio, hud, ui, store }) {
        Object.assign(this, { stage, art, map, view, input, audio, hud, ui, store });
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
        this.chestSprites = {};
        const chestSheet = { tex: art.props.tex, mat: art.props.mat, casterMat: art.props.casterMat, frames: { closed: [art.props.rects['chest:0']], open: [art.props.rects['chestOpen:0']] } };
        for (const c of map.chests) {
            const s = new Sprite(this.app, chestSheet, { y: 0.03 });
            s.setPos(c.x, c.z);
            this.chestSprites[c.id] = s;
        }
        this.stainSprite = new Sprite(this.app, art.fx, { y: 0.07, caster: false, anchor: 0.5 });
        this.stainSprite.set('pearl'); this.stainSprite.setVisible(false);
        this.stainLight = view.addOmni(0, 0.8, 0, [0.4, 0.9, 1.0], 0, 4);
        this.app.on('update', dt => this.update(dt));
    }

    // ---------------------------------------------------------------- session
    start(profile, save) {
        this.profile = profile;
        this.save = save;
        this.originData = ORIGINS.find(o => o.id === save.origin) || ORIGINS[0];
        this.save.hints = this.save.hints || {};
        if (this.player) this.player.setSheet(buildPlayerSheet(this.app, save.look));
        else this.player = new Player(this, buildPlayerSheet(this.app, save.look));
        if (save.explored === 'all') this.explored.fill(1);
        else if (save.explored) this.explored.set(decodeBits(save.explored, this.explored.length));
        else this.explored.fill(0);
        this.refreshDerived();
        for (const c of this.map.chests) this.chestSprites[c.id].set(save.chests.includes(c.id) ? 'open' : 'closed');
        this.respawn(true);
        this.paused = false;
        const fresh = save.playTime < 1;
        if (fresh) {
            this.ui.banner('Schwarzwasser', 'Du bist ertrunken. Die Flut hat dich zurückgegeben.', 'title', 5.5);
            this.hint('controls', this.touch
                ? 'Daumen links auflegen und ziehen zum Laufen. Rechts: Hieb, Rolle, Parieren, Rum. Tippe auf Hinweise wie „Rasten“, um zu interagieren.'
                : 'WASD bewegen · J Angriff · L schwerer Angriff · Leertaste Rolle · Q Parieren · R Rum · E Interagieren · Tab Zielen · M Karte · Esc Menü');
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
        this.ship.sprite.setVisible(true);
        this.camX = this.player.x; this.camZ = this.player.z;
        this.updateStain();
        this.audio.setMood(this.mode === 'sea' ? 'sea' : 'land');
    }

    refillAll() {
        const p = this.player, s = this.save;
        p.hp = this.derived.maxHp; p.st = this.derived.maxSt;
        p.flasks = s.flasksMax;
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
    slashFx(p, heavy) {
        const a = Math.atan2(p.fx, -p.fz);
        this.fx.play(this.art.fx, 'slash', p.x + p.fx * 0.75, p.z + p.fz * 0.75 - 0.4, { fps: heavy ? 11 : 15, angle: a, scale: heavy ? 1.3 : 1 });
    }
    sparkFx(x, z) { this.fx.play(this.art.fx, 'spark', x, z, { fps: 20 }); this.fx.flash(x, z, [1, 0.85, 0.55], 2.5, 0.1, 4); }
    bloodFx(x, z) { this.fx.play(this.art.fx, 'blood', x, z - 0.2, { dur: 2.5, fade: true }); }
    healFx(p) { this.fx.flash(p.x, p.z, [1, 0.8, 0.4], 3, 0.6, 5); this.sfx('heal'); }
    smokeFx(x, z) { this.fx.play(this.art.fx, 'smoke', x, z, { fps: 6, scale: 1.4 }); this.fx.flash(x, z, [1, 0.6, 0.25], 6, 0.15, 7); }
    splashFx(x, z) { this.fx.play(this.art.fx, 'splash', x, z, { fps: 12 }); }

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
                this.sparkFx(e.x - dx / (l || 1) * 0.3, e.z - dz / (l || 1) * 0.3 - 0.5);
                this.sfx(atk.riposte ? 'crit' : 'hit');
                this.hitstop = atk.riposte ? 0.16 : (atk.heavy ? 0.08 : 0.05);
                this.shake(0.1, atk.heavy ? 0.12 : 0.06);
                if (atk.riposte) this.ui.floatText('KRITISCH', e.x, e.z - 1.6, 'crit');
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
            this.sparkFx((p.x + e.x) / 2, (p.z + e.z) / 2 - 0.6);
            this.hitstop = 0.12;
            this.ui.floatText('PARIERT', e.x, e.z - 1.6, 'parry');
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
        const s = this.fx.get(this.art.fx);
        s.sprite.set(tide ? 'pearl' : 'ball');
        const light = tide ? this.fx.flash(p.x, p.z, [0.3, 0.9, 1], 2, 0.3, 4) : this.fx.flash(p.x + p.fx, p.z + p.fz, [1, 0.7, 0.3], 5, 0.12, 6);
        this.projectiles.push({ s, x: p.x + p.fx * 0.5, z: p.z + p.fz * 0.5 - 0.5, vx: p.fx * sp, vz: p.fz * sp, life: tide ? 0.9 : 0.55, dmg: this.derived.offDmg, land: true });
        this.sfx(tide ? 'tide' : 'pistol');
        if (!tide) this.smokeFx(p.x + p.fx * 0.8, p.z + p.fz * 0.8 - 0.5);
    }

    spawnBall(x, z, vx, vz, dmg, delay) {
        const s = this.fx.get(this.art.fx);
        s.sprite.set('ball');
        this.projectiles.push({ s, x, z, vx, vz, life: 0.85 + delay, dmg, sea: true });
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
            b.s.sprite.setPos(b.x, b.z);
            let done = b.life <= 0;
            for (const e of this.enemies) {
                if (done || !e.alive) continue;
                if (b.sea !== !!e.sea && !(b.land && !e.sea)) continue;
                const hz = e.sea ? 0 : 0.4;
                if (Math.hypot(e.x - b.x, e.z - hz - b.z) < e.radius + 0.25) {
                    e.damage(b.dmg, 1, { x: b.x - b.vx, z: b.z - b.vz });
                    this.sparkFx(b.x, b.z); this.sfx('hit');
                    done = true;
                }
            }
            if (!done && b.land && this.map.blockedWalk(b.x, b.z + 0.4, 0.05)) { done = true; this.sparkFx(b.x, b.z); }
            if (done) {
                if (b.sea) this.splashFx(b.x, b.z);
                b.s.busy = false; b.s.sprite.setVisible(false);
                this.projectiles.splice(i, 1);
            }
        }
    }

    onEnemyKilled(e) {
        const greed = (this.originData.greed || 1) * (this.save.gift === 'coin' ? 1.2 : 1);
        const n = Math.round(e.pearls * greed);
        this.save.pearls += n;
        this.ui.pearlGain(n);
        this.fx.play(this.art.fx, 'pearl', e.x, e.z - 0.5, { dur: 0.6, vx: 0, vz: -2, fps: 8 });
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
        if (this.sinking) {
            const k = Math.max(0.01, 1 - this.dying * 0.6);
            this.ship.sprite.entity.setLocalScale(k, 1, k);
        }
        if (this.dying > 1.2 && !this.deathShown) { this.deathShown = true; this.ui.death(); }
        if (this.dying > 5.2) {
            const s = this.save;
            const sea = this.mode === 'sea';
            s.stain = s.pearls > 0 ? { x: sea ? this.ship.x : this.player.x, z: sea ? this.ship.z : this.player.z, amount: s.pearls, sea } : null;
            s.pearls = 0;
            s.deaths = (s.deaths || 0) + 1;
            this.ship.sprite.entity.setLocalScale(1, 1, 1);
            this.dying = undefined; this.deathShown = false; this.sinking = false;
            this.respawn();
            this.persist(null);
            this.ui.hideDeath();
            if (s.stain) this.hint('stain', 'Deine Perlen liegen dort, wo du gestorben bist. Hol sie zurück – stirbst du vorher, sind sie verloren.');
        }
    }
    updateStain() {
        const st = this.save?.stain;
        this.stainSprite.setVisible(!!st);
        this.stainLight.light.intensity = st ? 1.6 : 0;
        if (st) { this.stainSprite.setPos(st.x, st.z); this.stainLight.setLocalPosition(st.x, 0.8, st.z); }
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
            if (this.ship.hull > 0 && Math.hypot(p.x - this.ship.x, p.z - this.ship.z) < 4.2) return { label: 'An Bord gehen', act: () => this.board() };
        } else {
            for (const f of m.fires) if (f.kind === 'sea' && Math.hypot(this.ship.x - f.x - 1, this.ship.z - f.z) < 4.2) return { label: s.litFires.includes(f.id) ? 'Am Wrack rasten' : 'Wrackfeuer entzünden', act: () => this.useFire(f) };
            const land = this.landingSpot();
            if (land) return { label: 'An Land gehen', act: () => this.disembark(land) };
        }
        return null;
    }
    landingSpot() {
        const sx = this.ship.x, sz = this.ship.z;
        let best = null, bd = 3.2;
        for (let j = -4; j <= 4; j++) for (let i = -4; i <= 4; i++) {
            const tx = Math.floor(sx) + i, tz = Math.floor(sz) + j;
            const cx = tx + 0.5, cz = tz + 0.5;
            if (this.map.blockedWalk(cx, cz, 0.3)) continue;
            const d = Math.hypot(cx - sx, cz - sz);
            if (d < bd) { bd = d; best = { x: cx, z: cz }; }
        }
        return best;
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
        this.chestSprites[c.id].set('open');
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
        if (this.mode === 'land') { this.player.state = 'frozen'; this.player.face(f.x - this.player.x, f.z - this.player.z); }
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
        if (this.mode === 'land') this.player.state = 'frozen';
    }

    // ---------------------------------------------------------------- loop
    update(rawDt) {
        const inp = this.input;
        inp.poll();
        if (!this.save) { // attract mode behind the title screen
            this.time += rawDt;
            const f = this.map.fires[0];
            this.camX = f.x + Math.sin(this.time * 0.06) * 5 + 2;
            this.camZ = f.z + Math.cos(this.time * 0.045) * 3 - 1;
            this.stage.camera.setPosition(Math.round(this.camX * 16) / 16, 40, Math.round(this.camZ * 16) / 16);
            this.view.update(rawDt);
            inp.endFrame();
            return;
        }
        if (inp.pressed('menu') && !this.ui.modalOpen() && this.dying === undefined) this.ui.pauseMenu();
        if (inp.pressed('map') && !this.ui.modalOpen()) this.ui.toggleMap();
        if (this.paused || this.ui.modalOpen()) { this.view.update(rawDt); this.fx.update(rawDt); this.updateCamera(rawDt); this.hud.update(); inp.endFrame(); return; }
        let dt = rawDt;
        if (this.hitstop > 0) { this.hitstop -= rawDt; dt = rawDt * 0.05; }
        this.time += dt;
        this.save.playTime += rawDt;
        this.wind.angle = 2.3 + Math.sin(this.time * 0.015) * 0.9;
        this.wind.strength = 0.85 + Math.sin(this.time * 0.05) * 0.15;

        if (this.mode === 'land') {
            if (inp.pressed('lock')) this.toggleLock();
            this.player.update(dt);
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
        this.view.update(dt);
        this.updateDeath(rawDt);
        if (this.dying === undefined) {
            this.checkStain();
            const it = this.nearestInteraction();
            this.hud.prompt(it ? it.label : null);
            if (it && inp.pressed('interact')) it.act();
        } else this.hud.prompt(null);
        this.stainSprite.update(dt, 3);
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
        let tx = this.player.x, tz = this.player.z - 0.6;
        if (sea) { const f = this.ship.fwd; tx = this.ship.x + f.x * this.ship.speed * 0.5; tz = this.ship.z + f.z * this.ship.speed * 0.5; }
        if (this.lockTarget && !sea) { tx = (tx * 2 + this.lockTarget.x) / 3; tz = (tz * 2 + this.lockTarget.z) / 3; }
        const k = 1 - Math.pow(0.0015, dt);
        this.camX += (tx - this.camX) * k;
        this.camZ += (tz - this.camZ) * k;
        let sx = 0, sz = 0;
        if (this.shakeT > 0) {
            this.shakeT -= dt;
            sx = (Math.random() - 0.5) * this.shakeAmp * 2; sz = (Math.random() - 0.5) * this.shakeAmp * 2;
            if (this.shakeT <= 0) this.shakeAmp = 0;
        }
        const x = Math.round((this.camX + sx) * 16) / 16, z = Math.round((this.camZ + sz) * 16) / 16;
        this.stage.camera.setPosition(x, 40, z);
    }

    // world -> screen (CSS pixels relative to the canvas)
    toScreen(x, z) {
        const c = this.stage.app.graphicsDevice.canvas;
        const r = c.getBoundingClientRect();
        const pxPerUnit = r.height / (this.stage.ortho * 2);
        const cam = this.stage.camera.getPosition();
        return { x: r.left + r.width / 2 + (x - cam.x) * pxPerUnit, y: r.top + r.height / 2 + (z - cam.z) * pxPerUnit, s: pxPerUnit };
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
