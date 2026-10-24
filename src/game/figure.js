// 3D figures built from code: the pirate (all character-creator options), skeleton pirates,
// beach crabs and the sea monster "Rostkiefer". Animation is procedural: every state sets target
// joint angles, which are blended smoothly each frame. Facing works in any direction.
import { Entity, MeshInstance, CULLFACE_NONE } from 'playcanvas';
import { MeshBuilder } from '../gfx/mesh.js';
import { resolveLook } from './looks.js';

const D = 180 / Math.PI;
const ease = t => t < 0 ? 0 : t > 1 ? 1 : t * t * (3 - 2 * t);
const lerp = (a, b, k) => a + (b - a) * k;
const meshCache = new Map();

// a joint: entity + current/target euler angles for smooth blending
class Joint {
    constructor(name, parent, x, y, z) {
        this.e = new Entity(name);
        this.e.setLocalPosition(x, y, z);
        parent.addChild(this.e);
        this.cur = [0, 0, 0]; this.tgt = [0, 0, 0];
        this.base = [x, y, z];
    }
    set(x = 0, y = 0, z = 0) { this.tgt[0] = x; this.tgt[1] = y; this.tgt[2] = z; }
    blend(k) {
        for (let i = 0; i < 3; i++) this.cur[i] = lerp(this.cur[i], this.tgt[i], k);
        this.e.setLocalEulerAngles(this.cur[0], this.cur[1], this.cur[2]);
    }
}

class FigureBase {
    constructor(app, M) {
        this.app = app; this.M = M;
        this.root = new Entity('figure');
        app.root.addChild(this.root);
        this.joints = {};
        this.time = 0;
        this.rate = 16;
        this.flash = 0;
        this.meshInstances = [];
    }
    J(name, parent, x, y, z) { const j = new Joint(name, parent.e || parent, x, y, z); this.joints[name] = j; return j; }
    // attach geometry (built once per cache key) to a joint
    part(j, key, mat, build, { shadow = true } = {}) {
        let mesh = meshCache.get(key);
        if (!mesh) { const b = new MeshBuilder(); build(b); mesh = b.toMesh(this.app.graphicsDevice); mesh.incRefCount(); meshCache.set(key, mesh); } // cached meshes must survive destroyed figures
        const mi = new MeshInstance(mesh, mat);
        mi.castShadow = shadow; mi.receiveShadow = true;
        const e = new Entity(key);
        e.addComponent('render', { meshInstances: [mi], castShadows: shadow, receiveShadows: true });
        (j.e || j).addChild(e);
        this.meshInstances.push(mi);
        return e;
    }
    setPos(x, y, z) { this.root.setPosition(x, y, z); }
    setYaw(fx, fz) { this.yaw = Math.atan2(fx, fz) * D; this.root.setEulerAngles(0, this.yaw, 0); }
    setVisible(v) { this.root.enabled = v; }
    destroy() { this.root.destroy(); }
    blendAll(dt, rate = this.rate) {
        const k = 1 - Math.exp(-dt * rate);
        for (const j of Object.values(this.joints)) j.blend(k);
    }
}

// ============================================================================ humanoid
export class Humanoid extends FigureBase {
    // spec: { look } for a pirate, { skeleton: true } for a skeleton pirate
    constructor(app, M, spec = {}) {
        super(app, M);
        this.skeleton = !!spec.skeleton;
        this.look = resolveLook(spec.look);
        this.weaponId = spec.weapon || 'cutlass';
        this.offhand = spec.off || 'none';
        this.walkPh = 0;
        this.build();
    }
    mats() {
        const M = this.M, L = this.look;
        if (this.skeleton) {
            const bone = M.pbr('bone', { tiling: 2 });
            return { skin: bone, coat: M.pbr('cloth', { tint: '#2a2630', tiling: 2, cull: CULLFACE_NONE }), shirt: bone, trousers: M.pbr('cloth', { tint: '#24222a', tiling: 2 }), boots: M.pbr('leather', { tint: '#2a2018' }), belt: M.pbr('leather', { tint: '#2a1a12' }), scarf: M.pbr('cloth', { tint: '#3a1c1c' }), hair: bone, hat: M.pbr('leather', { tint: '#1c1a1e' }), metal: M.pbr('metal', { metal: 0.6, tint: '#8a6a50' }), eye: M.plain('#071210', { emissive: '#56f0c0', emissiveIntensity: 8 }) };
        }
        return {
            skin: M.pbr('skin', { tint: L.skinHex }),
            coat: M.pbr('cloth', { tint: L.coatHex, tiling: 2, cull: CULLFACE_NONE }),
            shirt: M.pbr('cloth', { tint: L.accHex, tiling: 2 }),
            trousers: M.pbr('cloth', { tint: '#3a3430', tiling: 2 }),
            boots: M.pbr('leather', { tint: '#4a3020' }),
            belt: M.pbr('leather', { tint: '#3a2416' }),
            scarf: M.pbr('cloth', { tint: L.scarfHex, tiling: 2, cull: CULLFACE_NONE }),
            hair: M.pbr('cloth', { tint: L.hairHex, tiling: 4, gloss: 0.6 }),
            hat: M.pbr('leather', { tint: '#2a2220' }),
            metal: M.pbr('metal', { metal: 0.9, gloss: 1 }),
            eye: M.plain('#0b0907', { gloss: 0.8 })
        };
    }
    build() {
        const sk = this.skeleton, L = this.look;
        const m = this.mats();
        this.m = m;
        const wide = !sk && L.body === 1 ? 1.15 : 1;
        const K = sk ? 'sk' : `hu${wide}`;
        // pivot: hips at 0.92
        this.body = this.J('body', this.root, 0, 0.5, 0); // pivot at the belly (rolls, falls)
        const hips = this.J('hips', this.body, 0, 0.42, 0);
        const chest = this.J('chest', hips, 0, 0.08, 0);
        const neck = this.J('neck', chest, 0, 0.5, 0);
        const head = this.J('head', neck, 0, 0.07, 0);
        if (sk) {
            this.part(hips, 'sk-pelvis', m.skin, b => { b.at(0, -0.04, 0, 0, 0, 0, 1, 0.6, 0.8).lathe([[0.06, -0.08], [0.16, 0], [0.12, 0.08]], 10); });
            this.part(chest, 'sk-spine', m.skin, b => { for (let i = 0; i < 6; i++) b.at(0, 0.03 + i * 0.075, -0.04).sphere(0.035, 6, 4); });
            this.part(chest, 'sk-ribs', m.skin, b => {
                for (let i = 0; i < 5; i++) { const y = 0.18 + i * 0.06, r = 0.15 - Math.abs(i - 2) * 0.012; b.at(0, y, 0.0, 0, 0, 0, 1, 1, 0.75).lathe([[r, -0.012], [r, 0.012]], 14, { caps: false }); }
                b.at(0, 0.44, 0, 0, 0, 90).cylinder(0.025, 0.025, 0.38, 6); // collar bones
                b.at(-0.19, 0.44, 0, 0, 0, -90).cylinder(0.02, 0.02, 0.38, 6);
            });
            // tattered coat remnant
            this.part(chest, 'sk-coat', m.coat, b => { b.at(0, -0.02, 0, 0, 0, 0, 1, 1, 0.75).lathe([[0.2, -0.45], [0.19, -0.2], [0.18, 0.1], [0.2, 0.42], [0.12, 0.5]], 12, { caps: false, uvU: 2 }); });
            this.part(head, 'sk-skull', m.skin, b => {
                b.at(0, 0.13, 0.0, 0, 0, 0, 0.95, 1.05, 1.05).sphere(0.12, 12, 9);
                b.at(0, 0.03, 0.05).box(0.13, 0.06, 0.12, 2); // jaw
            });
            this.part(head, 'sk-eyes', m.eye, b => { for (const s of [-1, 1]) b.at(s * 0.045, 0.13, 0.1).sphere(0.028, 6, 4); }, { shadow: false });
            this.part(head, 'sk-hat', m.hat, b => this.tricorn(b, 0.98));
        } else {
            this.part(hips, `${K}-hips`, m.trousers, b => { b.at(0, -0.06, 0, 0, 0, 0, 1, 1, 0.8).lathe([[0.15 * wide, -0.1], [0.17 * wide, 0.02], [0.16 * wide, 0.1]], 12); });
            this.part(chest, `${K}-torso`, m.shirt, b => { b.at(0, 0, 0, 0, 0, 0, 1, 1, 0.72).lathe([[0.16 * wide, 0], [0.19 * wide, 0.2], [0.21 * wide, 0.36], [0.17 * wide, 0.47], [0.07, 0.52]], 14); });
            // long coat: back and sides of the torso plus flaring tails down to the knees
            this.part(chest, `${K}-coat`, m.coat, b => {
                b.at(0, 0, 0, 0, 0, 0, 1, 1, 0.76).lathe([[0.24 * wide, -0.5], [0.205 * wide, -0.25], [0.19 * wide, -0.02], [0.215 * wide, 0.2], [0.228 * wide, 0.37], [0.18 * wide, 0.48]], 16, { caps: false, uvU: 2, phase: 0 });
                b.at(0, 0.44, 0, 0, 0, 0, 1, 1, 0.8).lathe([[0.18 * wide, 0], [0.12, 0.08]], 12, { caps: false }); // collar
            });
            this.part(chest, `${K}-belt`, m.belt, b => { b.at(0, 0.0, 0, 0, 0, 0, 1, 1, 0.75).lathe([[0.2 * wide, -0.035], [0.2 * wide, 0.035]], 14, { caps: false }); });
            this.part(chest, `${K}-buckle`, m.metal, b => { b.at(0, 0.0, 0.16).box(0.08, 0.07, 0.03, 4); });
            this.part(chest, `${K}-sash`, m.scarf, b => {
                b.at(0, 0.05, 0, 0, 0, 0, 1, 1, 0.75).lathe([[0.205 * wide, -0.03], [0.205 * wide, 0.03]], 14, { caps: false });
                b.at(0.14 * wide, -0.12, 0.08, 0, 30, 8).box(0.06, 0.24, 0.02, 2);
                b.at(0, 0.43, 0.06, 0, 0, 0, 1, 1, 0.7).lathe([[0.12, -0.03], [0.1, 0.04]], 10); // neck scarf
            });
            // head
            this.part(head, `hu-head`, m.skin, b => {
                b.at(0, 0.12, 0, 0, 0, 0, 0.92, 1.1, 1).sphere(0.12, 14, 10);
                b.at(0, 0.11, 0.115, 10).box(0.04, 0.06, 0.05, 4); // nose
                for (const s of [-1, 1]) b.at(s * 0.115, 0.12, 0).sphere(0.03, 6, 4); // ears
            });
            this.part(head, 'hu-eyes', m.eye, b => { for (const s of [-1, 1]) b.at(s * 0.043, 0.145, 0.1).sphere(0.022, 8, 6); });
            this.part(head, 'hu-brows', m.hair, b => { for (const s of [-1, 1]) b.at(s * 0.045, 0.178, 0.104, 0, 0, -s * 8).box(0.05, 0.014, 0.02, 4); });
            this.part(head, 'hu-mouth', this.M.plain('#3a1612', { gloss: 0.3 }), b => { b.at(0, 0.075, 0.108).box(0.045, 0.01, 0.01, 4); });
            this.hair(head, m.hair, L.hair);
            if (L.beard) this.part(head, `beard${L.beard}`, L.beard === 1 ? this.M.pbr('skin', { tint: shade(L.skinHex, 0.55) }) : m.hair, b => this.beard(b, L.beard));
            if (L.hat) this.part(head, `hat${L.hat}`, L.hat === 1 ? m.scarf : m.hat, b => this.hat(b, L.hat));
            if (L.hat === 3) this.part(head, 'hat3-trim', this.M.pbr('metal', { tint: '#c9a060', metal: 1 }), b => { b.at(0, 0.25, 0).lathe([[0.15, 0], [0.15, 0.025]], 14, { caps: false }); });
            if (L.patch) this.part(head, 'patch', this.M.plain('#0c0b0b', { gloss: 0.4 }), b => { b.at(0.042, 0.145, 0.108).box(0.06, 0.05, 0.015, 4); b.at(0, 0.16, 0, 0, 0, -12, 1, 1, 1).lathe([[0.124, -0.007], [0.124, 0.007]], 14, { caps: false }); });
            if (L.scar) this.part(head, 'scar', this.M.plain('#6a2a22', { gloss: 0.5 }), b => { b.at(-0.05, 0.1, 0.105, 0, 0, 20).box(0.012, 0.09, 0.01, 4); });
        }
        // arms
        for (const s of [1, -1]) { // +1 = left (+x), -1 = right
            const side = s > 0 ? 'L' : 'R';
            const sh = this.J('sh' + side, chest, s * 0.2 * wide, 0.42, 0);
            const fore = this.J('fore' + side, sh, 0, -0.29, 0);
            const hand = this.J('hand' + side, fore, 0, -0.26, 0);
            if (sk) {
                this.part(sh, 'sk-upper', m.skin, b => { b.at().tube([[0, 0, 0], [0, -0.29, 0]], [0.025, 0.022], 6); b.at(0, -0.29, 0).sphere(0.032, 6, 4); });
                this.part(fore, 'sk-fore', m.skin, b => { b.at().tube([[0, 0, 0], [0, -0.25, 0]], [0.022, 0.02], 6); });
                this.part(hand, 'sk-hand', m.skin, b => { b.at(0, -0.04, 0, 0, 0, 0, 1, 1.4, 0.7).sphere(0.035, 6, 4); });
            } else {
                this.part(sh, `${K}-upper`, m.coat, b => { b.at().tube([[0, 0.02, 0], [0, -0.15, 0], [0, -0.3, 0]], [0.07 * wide, 0.06 * wide, 0.055 * wide], 8); });
                this.part(fore, `${K}-fore`, m.coat, b => { b.at().tube([[0, 0, 0], [0, -0.2, 0], [0, -0.24, 0]], [0.055 * wide, 0.05, 0.065], 8); });
                this.part(hand, 'hu-hand', m.skin, b => { b.at(0, -0.045, 0, 0, 0, 0, 0.9, 1.2, 0.7).sphere(0.045, 8, 6); });
            }
        }
        // legs
        for (const s of [1, -1]) {
            const side = s > 0 ? 'L' : 'R';
            const th = this.J('thigh' + side, hips, s * 0.09 * wide, -0.04, 0);
            const shin = this.J('shin' + side, th, 0, -0.42, 0);
            const foot = this.J('foot' + side, shin, 0, -0.42, 0);
            if (sk) {
                this.part(th, 'sk-thigh', m.skin, b => { b.at().tube([[0, 0, 0], [0, -0.42, 0]], [0.03, 0.026], 6); b.at(0, -0.42, 0).sphere(0.038, 6, 4); });
                this.part(shin, 'sk-shin', m.skin, b => { b.at().tube([[0, 0, 0], [0, -0.41, 0]], [0.026, 0.022], 6); });
                this.part(foot, 'sk-foot', m.skin, b => { b.at(0, -0.02, 0.05).box(0.07, 0.04, 0.17, 3); });
            } else {
                this.part(th, `${K}-thigh`, m.trousers, b => { b.at().tube([[0, 0, 0], [0, -0.22, 0], [0, -0.42, 0]], [0.085 * wide, 0.075 * wide, 0.06], 8); });
                this.part(shin, 'hu-shin', m.boots, b => { b.at().tube([[0, 0.02, 0], [0, -0.2, 0], [0, -0.4, 0]], [0.072, 0.064, 0.058], 8); b.at(0, 0.02, 0).lathe([[0.08, -0.06], [0.085, 0.02]], 10); });
                this.part(foot, 'hu-foot', m.boots, b => { b.at(0, -0.03, 0.05).box(0.1, 0.07, 0.24, 3); });
            }
        }
        this.buildWeapon();
        this.buildOffhand();
        // a bottle shown only while drinking
        this.bottle = this.part(this.joints.handL, 'bottle', this.M.plain('#2a3a20', { gloss: 0.9, metal: 0 }), b => { b.at(0, -0.12, 0.02).lathe([[0.04, -0.08], [0.045, 0.02], [0.02, 0.07], [0.015, 0.12]], 8); });
        this.bottle.enabled = false;
    }
    tricorn(b, s = 1) {
        // brim folded up on three sides, crown on top
        b.at(0, 0.21 * s, 0, 0, 0, 0, s, s, s).lathe([[0.13, 0], [0.12, 0.09], [0.0, 0.11]], 12);
        for (let i = 0; i < 3; i++) {
            const a = i * 120 + 180;
            b.at(Math.sin(a / D) * 0.11 * s, 0.2 * s, Math.cos(a / D) * 0.11 * s, -60, a, 0, s).box(0.24, 0.13, 0.02, 3, 0);
        }
    }
    hair(head, mat, style) {
        if (style === 4) return; // bald
        this.part(head, `hair${style}`, mat, b => {
            b.at(0, 0.14, -0.01, 0, 0, 0, 0.98, 1.06, 1.04).sphere(0.128, 12, 8); // cap (head mostly hides the inner part)
            if (style === 1) b.at(0, 0.05, -0.07, 8, 0, 0, 1, 1, 0.6).lathe([[0.12, -0.18], [0.13, 0.05]], 12); // long
            if (style === 2) b.at(0, 0.06, -0.12, 30).tube([[0, 0, 0], [0, -0.1, -0.04], [0, -0.22, -0.05]], [0.04, 0.035, 0.02], 6); // ponytail
            if (style === 3) for (let i = 0; i < 10; i++) { const a = i / 10 * Math.PI * 2; b.at(Math.cos(a) * 0.11, 0.17 + Math.sin(i * 1.7) * 0.03, Math.sin(a) * 0.1 - 0.02).sphere(0.05, 6, 4); }
            if (style === 5) for (let i = 0; i < 9; i++) { const a = (i / 8 - 0.5) * 3.4 + Math.PI; b.at(Math.sin(a) * 0.11, 0.14, Math.cos(a) * 0.11).tube([[0, 0, 0], [Math.sin(a) * 0.03, -0.12, Math.cos(a) * 0.03], [Math.sin(a) * 0.04, -0.26, Math.cos(a) * 0.04]], [0.018, 0.016, 0.012], 5); }
        });
    }
    beard(b, kind) {
        if (kind === 1) { b.at(0, 0.07, 0.03, 0, 0, 0, 0.93, 0.7, 1).sphere(0.115, 10, 6); return; }
        if (kind === 2) { b.at(0, 0.05, 0.07, 10, 0, 0, 1, 1.1, 0.8).sphere(0.1, 10, 6); return; }
        b.at(0, 0.0, 0.1, 20).lathe([[0.03, -0.08], [0.04, 0.02]], 8);
    }
    hat(b, kind) {
        if (kind === 1) { // bandana with knot tails
            b.at(0, 0.15, -0.01, 0, 0, 0, 1, 1.05, 1.06).sphere(0.133, 12, 8);
            b.at(0.0, 0.14, -0.14, -40, 0, 20).box(0.03, 0.12, 0.02, 3, -0.12);
            return;
        }
        if (kind === 2) { this.tricorn(b, 1.0); return; }
        // captain's hat: tall crown, wide brim, feather
        b.at(0, 0.2, 0).lathe([[0.2, 0], [0.2, 0.02], [0.13, 0.03], [0.13, 0.17], [0.11, 0.19], [0.0, 0.2]], 16);
        b.at(-0.1, 0.3, -0.04, 10, 0, 50).tube([[0, 0, 0], [0.05, 0.12, -0.04], [0.04, 0.24, -0.12]], [0.02, 0.025, 0.004], 5);
    }
    buildWeapon() {
        const hand = this.joints.handR, M = this.M;
        const steel = M.pbr('metal', { metal: 0.95, gloss: 1.1, cull: CULLFACE_NONE, tint: this.skeleton ? '#9a7a60' : '#ffffff' });
        const wood = M.pbr('wood', { tiling: 3 });
        const brass = M.pbr('metal', { metal: 1, tint: '#c9a060' });
        const id = this.skeleton ? 'cutlass' : this.weaponId;
        const g = new Entity('weapon');
        g.setLocalPosition(0, -0.06, 0);
        g.setLocalEulerAngles(90, 0, 0); // blade points forward (+z) out of the fist
        hand.e.addChild(g);
        this.weapon = g;
        const blade = (len, w, curve) => b => {
            const pts = [], wid = [];
            for (let i = 0; i <= 8; i++) { const t = i / 8; pts.push([0, 0.08 + t * len, -Math.sin(t * 1.4) * curve]); wid.push(w * (1 - t * 0.35) - (i === 8 ? w * 0.6 : 0)); }
            b.at().ribbon(pts, wid, [1, 0, 0]);
        };
        const wj = { e: g };
        switch (id) {
            case 'hammer':
                this.part(wj, 'w-hammer-h', wood, b => { b.at(0, -0.15, 0).cylinder(0.025, 0.025, 0.95, 6); });
                this.part(wj, 'w-hammer-head', M.pbr('metal', { metal: 0.8 }), b => { b.at(0, 0.72, 0, 0, 0, 90).cylinder(0.08, 0.08, 0.32, 8); b.at(-0.16, 0.72, 0, 0, 0, 90).lathe([[0.1, 0], [0.02, 0.12]], 8); });
                break;
            case 'rapier':
                this.part(wj, 'w-rapier', steel, blade(0.95, 0.025, 0.0));
                this.part(wj, 'w-rapier-guard', brass, b => { b.at(0, 0.06, 0).lathe([[0.0, 0], [0.07, 0.03], [0.065, 0.05]], 10); });
                break;
            case 'dagger':
                this.part(wj, 'w-dagger', steel, blade(0.34, 0.05, 0.0));
                this.part(wj, 'w-guard-s', brass, b => { b.at(0, 0.06, 0).box(0.12, 0.02, 0.025, 4); });
                break;
            case 'fetish':
                this.part(wj, 'w-fetish', wood, b => { b.at(0, -0.1, 0).cylinder(0.022, 0.018, 0.65, 6); });
                this.part(wj, 'w-shell', M.plain('#c8c0b0', { emissive: '#4fd0c0', emissiveIntensity: 1.5 }), b => { b.at(0, 0.58, 0, 0, 0, 0, 1, 1.3, 0.6).sphere(0.07, 8, 6); });
                break;
            case 'club':
                this.part(wj, 'w-club', wood, b => { b.at(0, -0.08, 0).lathe([[0.025, 0], [0.03, 0.3], [0.06, 0.62], [0.04, 0.7]], 8); });
                break;
            default: // cutlass
                this.part(wj, 'w-cutlass', steel, blade(0.72, 0.065, 0.12));
                this.part(wj, 'w-cutlass-guard', this.skeleton ? steel : brass, b => { b.at().tube([[0, 0.06, 0.05], [0, 0.03, -0.03], [0, -0.02, -0.06], [0, -0.09, -0.05]], [0.012, 0.015, 0.015, 0.012], 5); b.at(0, 0.07, 0).box(0.05, 0.02, 0.1, 4); });
        }
    }
    buildOffhand() {
        if (this.skeleton || this.offhand === 'none') return;
        const hand = this.joints.handL, M = this.M;
        const g = new Entity('offhand'); g.setLocalPosition(0, -0.06, 0); g.setLocalEulerAngles(90, 0, 0); hand.e.addChild(g);
        const wj = { e: g };
        if (this.offhand === 'pistol') {
            this.part(wj, 'o-pistol-b', M.pbr('metal', { metal: 0.9 }), b => { b.at(0, 0.05, 0.02).cylinder(0.018, 0.016, 0.32, 8); });
            this.part(wj, 'o-pistol-g', M.pbr('wood', { tiling: 3 }), b => { b.at(0, -0.02, -0.02, 30).box(0.04, 0.14, 0.06, 4); });
        } else {
            this.part(wj, 'o-tide', M.plain('#b8d0d0', { emissive: '#5fd8ff', emissiveIntensity: 2.5 }), b => { b.at(0, 0.05, 0, 0, 0, 0, 1, 1.4, 0.7).sphere(0.06, 8, 6); });
        }
    }

    // ------------------------------------------------------------------ animation
    // s: { state, t, speed, atk: {wind, act, rec, heavy}, combo }
    animate(dt, s) {
        this.time += dt;
        const j = this.joints;
        const t = s.t || 0;
        let rate = 16;
        // defaults: relaxed stance, breathing
        const br = Math.sin(this.time * 2.1);
        let body = [0, 0, 0], bodyY = 0, hipsY = 0.92;
        const P = {
            hips: [0, 0, 0], chest: [br * 1.5, 0, 0], neck: [0, 0, 0], head: [0, 0, 0],
            shL: [4, 0, 8], foreL: [-12, 0, 0], handL: [0, 0, 0], shR: [-8, 0, -10], foreR: [-30, 0, 0], handR: [10, 0, 0],
            thighL: [0, 0, 2], shinL: [0, 0, 0], footL: [0, 0, 0], thighR: [0, 0, -2], shinR: [0, 0, 0], footR: [0, 0, 0]
        };
        if (this.skeleton) { P.chest[0] += 8; P.neck[0] = 10; P.shL[2] = 14; P.shR[2] = -14; }
        const walk = (speed, k = 1) => {
            this.walkPh += dt * (3.2 + speed * 1.55);
            const ph = this.walkPh, a = Math.min(1, speed / 3.5) * k;
            P.thighL[0] = -Math.sin(ph) * 34 * a; P.thighR[0] = Math.sin(ph) * 34 * a;
            P.shinL[0] = Math.max(0, Math.sin(ph - 1.3)) * 50 * a + 4; P.shinR[0] = Math.max(0, Math.sin(ph + Math.PI - 1.3)) * 50 * a + 4;
            P.footL[0] = -P.shinL[0] * 0.3; P.footR[0] = -P.shinR[0] * 0.3;
            P.shL[0] = Math.sin(ph) * 28 * a; P.shR[0] = -Math.sin(ph) * 22 * a - 10;
            P.chest[1] = Math.sin(ph) * 6 * a; P.hips[1] = -Math.sin(ph) * 6 * a;
            P.chest[0] += 5 * a;
            hipsY = 0.92 - Math.abs(Math.cos(ph)) * 0.04 * a;
        };
        switch (s.state) {
            case 'walk': case 'chase': case 'return': walk(s.speed ?? 3.5); break;
            case 'drink':
                walk((s.speed ?? 0) * 0.5, 0.4);
                P.shL = [-128, 0, 22]; P.foreL = [-80, 0, 0]; P.head = [-18, 0, 0];
                break;
            case 'roll': {
                const k = Math.min(1, t / 0.44);
                body = [k * 360, 0, 0]; bodyY = -Math.sin(k * Math.PI) * 0.12;
                P.thighL = P.thighR = [-110, 0, 0]; P.shinL = P.shinR = [125, 0, 0];
                P.shL = P.shR = [-70, 0, 0]; P.foreL = P.foreR = [-90, 0, 0]; P.chest = [40, 0, 0]; P.head = [30, 0, 0];
                rate = 40;
                break;
            }
            case 'windup': case 'attack': case 'strike': {
                const a = s.atk || { wind: 0.4, act: 0.12, rec: 0.3, heavy: false };
                const heavy = a.heavy;
                const pre = s.state === 'windup' || (s.state === 'attack' && t < a.wind);
                const k = pre ? ease(t / Math.max(0.01, a.wind)) : ease((t - (s.state === 'attack' ? a.wind : 0)) / Math.max(0.01, a.act + 0.05));
                const flip = (s.combo || 0) % 2 === 1; // alternate swing direction in combos
                if (heavy) {
                    if (pre) { P.shR = [lerp(-20, -175, k), 0, -8]; P.foreR = [lerp(-30, -40, k), 0, 0]; P.chest = [lerp(0, -14, k), 10, 0]; }
                    else { P.shR = [lerp(-175, -35, k), 0, -10]; P.foreR = [-10, 0, 0]; P.chest = [lerp(-14, 22, k), -8, 0]; P.thighL[0] = -30; P.shinL[0] = 20; P.thighR[0] = 18; }
                    P.shL = [-40, 0, 25];
                } else if (!flip) {
                    if (pre) { P.shR = [-60, 0, lerp(-30, -105, k)]; P.foreR = [-40, 0, 0]; P.chest = [4, lerp(0, -40, k), 0]; P.handR = [0, 0, 30]; }
                    else { P.shR = [lerp(-60, -95, k), 0, lerp(-105, 10, k)]; P.foreR = [-8, 0, 0]; P.chest = [8, lerp(-40, 38, k), 0]; P.handR = [0, 0, -20]; }
                    P.thighL[0] = -22; P.thighR[0] = 14; P.shinL[0] = 12;
                } else {
                    if (pre) { P.shR = [-70, 0, lerp(-10, 25, k)]; P.foreR = [-50, 0, 0]; P.chest = [4, lerp(0, 38, k), 0]; P.handR = [0, 0, -40]; }
                    else { P.shR = [lerp(-70, -90, k), 0, lerp(25, -100, k)]; P.foreR = [-10, 0, 0]; P.chest = [8, lerp(38, -35, k), 0]; P.handR = [0, 0, 20]; }
                    P.thighL[0] = 14; P.thighR[0] = -22; P.shinR[0] = 12;
                }
                hipsY = 0.86;
                rate = pre ? 22 : 45;
                break;
            }
            case 'parry':
                P.shR = [-85, 0, -5]; P.foreR = [-75, 0, 0]; P.handR = [0, 0, 60]; P.chest = [0, 25, 0]; P.shL = [-30, 0, 30];
                P.thighL[0] = -20; P.thighR[0] = 15; hipsY = 0.87; rate = 40;
                break;
            case 'cast':
                P.shL = [-92, 0, 6]; P.foreL = [-10, 0, 0]; P.handL = [0, 0, 0]; P.chest = [0, -25, 0]; P.shR = [-15, 0, -20]; rate = 30;
                break;
            case 'hurt': case 'stagger':
                P.chest = [-22, 0, 8]; P.head = [-15, 0, 0]; P.shL = [-30, 0, 45]; P.shR = [-20, 0, -45]; hipsY = 0.88; rate = 30;
                break;
            case 'dead': {
                const k = ease(t / 0.6);
                body = [lerp(0, -88, k), 0, 0]; bodyY = lerp(0, -0.36, k);
                P.shL = [-150 * k, 0, 40]; P.shR = [-140 * k, 0, -50]; P.head = [-20 * k, 25 * k, 0];
                P.thighL = [-8 * k, 0, 12]; P.thighR = [5 * k, 0, -8];
                rate = 10;
                break;
            }
            case 'pile': case 'gone': {
                const k = s.state === 'gone' ? 1 : ease(t / 0.35);
                body = [lerp(0, -90, k), 0, lerp(0, 25, k)]; bodyY = lerp(0, -0.4, k);
                P.chest = [30 * k, 30 * k, 20 * k]; P.head = [40 * k, -60 * k, 30 * k];
                P.shL = [-60 * k, 40 * k, 120 * k]; P.shR = [-100 * k, 0, -130 * k]; P.foreL = [-90 * k, 0, 0];
                P.thighL = [-90 * k, 30 * k, 40 * k]; P.shinL = [120 * k, 0, 0]; P.thighR = [-20 * k, 0, -50 * k];
                rate = 22;
                break;
            }
            case 'rise': {
                const k = 1 - ease(t / 0.7);
                body = [-90 * k, 0, 25 * k]; bodyY = -0.4 * k;
                P.shL = [-60 * k, 0, 100 * k]; P.shR = [-60 * k, 0, -100 * k];
                rate = 30;
                break;
            }
            case 'sit':
                hipsY = 0.36; P.thighL = [-82, 0, 12]; P.thighR = [-85, 0, -10]; P.shinL = [84, 0, 0]; P.shinR = [88, 0, 0];
                P.chest = [16, 0, 0]; P.shL = [-42, 0, 10]; P.foreL = [-40, 0, 0]; P.shR = [-38, 0, -10]; P.foreR = [-45, 0, 0]; P.head = [8, 0, 0];
                rate = 6;
                break;
            case 'frozen': break;
            default: break; // idle
        }
        if (s.hipsOffset) hipsY += s.hipsOffset;
        for (const [name, v] of Object.entries(P)) j[name].set(v[0], v[1], v[2]);
        j.body.set(body[0], body[1], body[2]);
        this.blendAll(dt, rate);
        j.body.e.setLocalPosition(0, 0.5 + bodyY, 0);
        j.hips.e.setLocalPosition(0, lerp(j.hips.e.getLocalPosition().y, hipsY - 0.5, 1 - Math.exp(-dt * 20)), 0);
        this.bottle.enabled = s.state === 'drink';
    }
}

function shade(hex, k) {
    const c = [1, 3, 5].map(i => Math.round(parseInt(hex.slice(i, i + 2), 16) * k));
    return '#' + c.map(v => v.toString(16).padStart(2, '0')).join('');
}

// ============================================================================ crab
export class CrabFigure extends FigureBase {
    constructor(app, M) {
        super(app, M);
        const shell = M.pbr('bone', { tint: '#8a3a22', tiling: 3, gloss: 1.2 });
        const dark = M.pbr('bone', { tint: '#4a1e14', tiling: 3 });
        this.body = this.J('body', this.root, 0, 0.26, 0);
        this.part(this.body, 'crab-shell', shell, b => { b.at(0, -0.06, 0, 0, 0, 0, 1, 0.55, 0.78).sphere(0.34, 14, 8); b.at(0, 0.0, 0.18, 0, 0, 0, 1, 0.4, 0.4).sphere(0.2, 10, 6); });
        this.part(this.body, 'crab-eyes', M.plain('#140a04', { emissive: '#d0a030', emissiveIntensity: 1.6 }), b => { for (const s of [-1, 1]) { b.at(s * 0.08, 0.05, 0.25).cylinder(0.012, 0.01, 0.1, 5); b.at(s * 0.08, 0.16, 0.25).sphere(0.025, 6, 4); } });
        this.legs = [];
        for (const s of [-1, 1]) for (let i = 0; i < 3; i++) {
            const leg = this.J(`leg${s}${i}`, this.body, s * 0.26, -0.04, 0.1 - i * 0.12);
            leg.e.setLocalEulerAngles(0, s * (80 + (i - 1) * 25), 0);
            const knee = this.J(`knee${s}${i}`, leg, 0, 0, 0.22);
            this.part(leg, 'crab-leg1', dark, b => { b.at().tube([[0, 0, 0], [0, 0.08, 0.12], [0, 0.04, 0.22]], [0.03, 0.028, 0.024], 5); });
            this.part(knee, 'crab-leg2', dark, b => { b.at().tube([[0, 0.04, 0], [0, -0.1, 0.1], [0, -0.26, 0.12]], [0.024, 0.02, 0.006], 5); });
            this.legs.push({ leg, knee, s, i });
        }
        this.claws = [];
        for (const s of [-1, 1]) {
            const arm = this.J(`arm${s}`, this.body, s * 0.2, 0.0, 0.22);
            const pin = this.J(`pin${s}`, arm, 0, 0.0, 0.26);
            this.part(arm, 'crab-arm', dark, b => { b.at().tube([[0, 0, 0], [s * 0.05, 0.05, 0.12], [0, 0.0, 0.26]], [0.04, 0.035, 0.035], 6); });
            this.part(pin, 'crab-claw', shell, b => { b.at(0, 0, 0.1, 0, 0, 0, 0.8, 0.6, 1.4).sphere(0.1, 10, 6); });
            const jaw = this.J(`jaw${s}`, pin, 0, -0.02, 0.15);
            this.part(jaw, 'crab-jaw', shell, b => { b.at(0, 0, 0.08, 0, 0, 0, 0.5, 0.4, 1.4).sphere(0.07, 8, 5); });
            this.claws.push({ arm, pin, jaw, s });
        }
    }
    animate(dt, s) {
        this.time += dt;
        const moving = s.state === 'chase' || s.state === 'walk' || (s.speed || 0) > 0.2;
        const ph = this.time * (moving ? 18 : 2);
        for (const L of this.legs) {
            const off = (L.i + (L.s > 0 ? 0.5 : 0)) * 2.1;
            L.leg.set(moving ? Math.sin(ph + off) * 18 : Math.sin(ph + off) * 2, L.s * (80 + (L.i - 1) * 25) + (moving ? Math.cos(ph + off) * 14 : 0), 0);
            L.knee.set(moving ? Math.max(0, Math.sin(ph + off)) * -25 : 0, 0, 0);
        }
        let open = 10, armX = 0, bodyRoll = 0, bodyY = 0.26;
        if (s.state === 'windup') { open = 40; armX = -35; }
        if (s.state === 'attack') { open = s.t < 0.12 ? 0 : 25; armX = s.t < 0.15 ? 25 : 0; }
        if (s.state === 'stagger') bodyRoll = 15;
        if (s.state === 'dead') { bodyRoll = 180 * ease(s.t / 0.4); bodyY = 0.16; }
        for (const c of this.claws) { c.arm.set(armX, c.s * -20, 0); c.jaw.set(open, 0, 0); c.pin.set(0, -c.s * 15, 0); }
        this.joints.body.set(moving ? Math.sin(ph * 0.5) * 3 : 0, 0, bodyRoll);
        this.blendAll(dt, s.state === 'attack' ? 40 : 14);
        this.joints.body.e.setLocalPosition(0, lerp(this.joints.body.e.getLocalPosition().y, bodyY, 0.2) + (moving ? Math.abs(Math.sin(ph)) * 0.015 : 0), 0);
    }
}

// ============================================================================ sea monster "Rostkiefer"
export class SharkFigure extends FigureBase {
    constructor(app, M) {
        super(app, M);
        const hide = M.pbr('leather', { tint: '#3a4446', tiling: 3, gloss: 1.4 });
        const rust = M.pbr('metal', { tint: '#8a5a3a', metal: 0.7, tiling: 2 });
        const tooth = M.pbr('bone', { tint: '#d8d0b8', tiling: 4 });
        this.body = this.J('body', this.root, 0, 0, 0);
        const prof = [[0.0, -2.2], [0.25, -1.9], [0.55, -1.2], [0.75, -0.2], [0.7, 0.6], [0.45, 1.4], [0.18, 2.0], [0.0, 2.3]];
        this.part(this.body, 'shark-body', hide, b => { b.at(0, 0, 0, -90, 0, 0, 1, 1, 0.75).lathe(prof, 16, { caps: false, uvU: 2, uvV: 0.5 }); });
        this.part(this.body, 'shark-fins', hide, b => {
            b.at(0, 0.5, 0.0).quad([0, 0, 0.6], [0, 0, -0.3], [0, 1.1, -0.65], [0, 0.95, -0.4]);
            b.at(0, 0.5, 0.0).quad([0, 0, -0.3], [0, 0, 0.6], [0, 0.95, -0.4], [0, 1.1, -0.65]);
            for (const s of [-1, 1]) b.at(s * 0.6, -0.25, 0.7).quad([0, 0, 0], [s * 0.9, -0.2, -0.6], [s * 0.8, -0.2, -0.85], [0, 0, -0.45]);
        });
        this.tail = this.J('tail', this.body, 0, 0, -2.1);
        this.part(this.tail, 'shark-tail', hide, b => { b.at().quad([0, 0, 0], [0, 1.0, -0.8], [0, 0.85, -0.95], [0, -0.1, -0.25]); b.at().quad([0, 0, 0], [0, -0.6, -0.65], [0, -0.5, -0.8], [0, -0.05, -0.2]); });
        this.part(this.body, 'shark-plates', rust, b => { for (let i = 0; i < 4; i++) b.at(0, 0.62 - i * 0.03, 1.2 - i * 0.45, -10, 0, 0, 1, 0.3, 1).sphere(0.32 - i * 0.04, 10, 5); });
        this.jaw = this.J('jaw', this.body, 0, -0.18, 1.3);
        this.part(this.jaw, 'shark-jaw', rust, b => { b.at(0, -0.1, 0.45, 0, 0, 0, 1, 0.45, 1.3).sphere(0.42, 12, 6); });
        this.part(this.jaw, 'shark-teeth', tooth, b => { for (let i = 0; i < 9; i++) { const a = (i / 8 - 0.5) * 2.2; b.at(Math.sin(a) * 0.36, 0.02, 0.45 + Math.cos(a) * 0.45, 0, 0, 0).lathe([[0.035, 0], [0.0, 0.12]], 5); } });
        this.part(this.body, 'shark-eyes', M.plain('#100404', { emissive: '#ff5a20', emissiveIntensity: 7 }), b => { for (const s of [-1, 1]) b.at(s * 0.42, 0.18, 1.45).sphere(0.07, 8, 5); }, { shadow: false });
    }
    animate(dt, s) {
        this.time += dt;
        const charge = s.state === 'charge';
        this.tail.set(0, Math.sin(this.time * (charge ? 9 : 4)) * 22, 0);
        this.joints.jaw.set(charge ? 38 : 6 + Math.sin(this.time * 1.3) * 4, 0, 0);
        this.joints.body.set(0, Math.sin(this.time * 2) * 4, Math.sin(this.time * 1.1) * 5);
        this.blendAll(dt, 8);
    }
}

// ============================================================================ chest
export class ChestFigure extends FigureBase {
    constructor(app, M, models) {
        super(app, M);
        const add = (j, key) => models[key].forEach(p => { const mi = new MeshInstance(p.mesh, p.mat); mi.castShadow = true; const e = new Entity(); e.addComponent('render', { meshInstances: [mi] }); j.e.addChild(e); });
        this.base = this.J('base', this.root, 0, 0, 0);
        add(this.base, 'chestBase');
        this.lid = this.J('lid', this.root, 0, 0.48, -0.275);
        add(this.lid, 'chestLid');
        this.glow = null;
    }
    setOpen(open) { this.open = open; }
    animate(dt) { this.lid.set(this.open ? -105 : 0, 0, 0); this.blendAll(dt, 4); }
}
