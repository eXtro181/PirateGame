// Animated pixel sprite. Upright sprites stand at their feet and lean back to face the tilted
// camera (A Link Between Worlds style); flat sprites lie on the ground (decals, ships, whirlpools).
import { Entity } from 'playcanvas';
import { QuadBatch, meshInstance, LEAN, LAYERS } from './gfx.js';

const PX = 1 / 16; // world units per pixel
const N = [0, Math.sin(LEAN), Math.cos(LEAN)];

export class Sprite {
    // sheet: { tex, mat, casterMat, frames: {name: [rect,...]} }
    constructor(app, sheet, { y = 0.05, caster = true, parent = null, pivotY = 0, anchor = 0, upright = true } = {}) {
        this.app = app;
        this.sheet = sheet;
        this.anchor = anchor; // flat sprites: 0 = feet at bottom edge, 0.5 = centred
        this.upright = upright && anchor === 0;
        this.y = y;
        this.pivotY = pivotY; // pixels from the bottom of the frame to the feet
        this.entity = new Entity('sprite');
        this.flipX = false;
        this.anim = null;
        this.frameIdx = -1;
        const r = Object.values(sheet.frames)[0][0];
        this.w = 0; this.h = 0;
        this.qb = new QuadBatch();
        this.qb.flat(0, 0, 1, 1, 0, r, sheet.tex.w, sheet.tex.h);
        this.mesh = this.qb.toMesh(app.graphicsDevice);
        this.entity.addComponent('render', { meshInstances: [meshInstance(this.mesh, sheet.mat, { receiveShadow: false })], castShadows: false, receiveShadows: false });
        if (caster && sheet.casterMat) {
            const cb = new QuadBatch();
            cb.caster(0, 0, r.w * PX * 0.8, r.h * PX, r, sheet.tex.w, sheet.tex.h);
            this.casterMesh = cb.toMesh(app.graphicsDevice, false);
            const ce = new Entity('caster');
            ce.addComponent('render', { meshInstances: [meshInstance(this.casterMesh, sheet.casterMat, { castShadow: true, receiveShadow: false })], castShadows: true, receiveShadows: false, layers: [LAYERS.shadow] });
            this.entity.addChild(ce);
        }
        (parent || app.root).addChild(this.entity);
        this.time = 0;
        this.set(Object.keys(sheet.frames)[0]);
    }
    set(name, restart = false) {
        if (this.anim === name && !restart) return;
        this.anim = name;
        this.time = 0;
        this.frameIdx = -1;
        this._apply(0);
    }
    update(dt, fps = 8, loop = true) {
        this.time += dt;
        const frames = this.sheet.frames[this.anim];
        let i = Math.floor(this.time * fps);
        i = loop ? i % frames.length : Math.min(i, frames.length - 1);
        this._apply(i);
    }
    get done() { return this.frameIdx >= this.sheet.frames[this.anim].length - 1; }
    setFlip(f) {
        if (f === this.flipX) return;
        this.flipX = f;
        const i = Math.max(0, this.frameIdx);
        this.frameIdx = -1;
        this._apply(i);
    }
    _geometry(w, h) {
        if (this.upright) {
            // stand at the feet and lean back so the full height faces the camera
            const c = Math.cos(LEAN), s = Math.sin(LEAN), p = this.pivotY * PX;
            const y0 = -p * c, z0 = p * s, y1 = (h - p) * c, z1 = -(h - p) * s;
            return [-w / 2, y1 + this.y, z1, w / 2, y1 + this.y, z1, w / 2, y0 + this.y, z0, -w / 2, y0 + this.y, z0];
        }
        const z0 = -h * (1 - this.anchor), z1 = h * this.anchor;
        return [-w / 2, this.y, z0, w / 2, this.y, z0, w / 2, this.y, z1, -w / 2, this.y, z1];
    }
    _apply(i) {
        if (i === this.frameIdx) return;
        this.frameIdx = i;
        const r = this.sheet.frames[this.anim][i];
        const tw = this.sheet.tex.w, th = this.sheet.tex.h;
        let u0 = r.x / tw, u1 = (r.x + r.w) / tw;
        const v0 = r.y / th, v1 = (r.y + r.h) / th;
        if (this.flipX) [u0, u1] = [u1, u0];
        if (r.w * PX !== this.w || r.h * PX !== this.h) {
            this.w = r.w * PX; this.h = r.h * PX;
            this.mesh.setPositions(this._geometry(this.w, this.h));
            if (this.upright) this.mesh.setNormals([...N, ...N, ...N, ...N]);
        }
        this.mesh.setUvs(0, [u0, v0, u1, v0, u1, v1, u0, v1]);
        this.mesh.update();
        if (this.casterMesh) {
            const uv = [u0, v0, u1, v0, u1, v1, u0, v1];
            this.casterMesh.setUvs(0, uv.concat(uv));
            this.casterMesh.update();
        }
    }
    setPos(x, z) { this.entity.setPosition(x, 0, z); }
    setVisible(v) { this.entity.enabled = v; }
    destroy() { this.entity.destroy(); }
}
