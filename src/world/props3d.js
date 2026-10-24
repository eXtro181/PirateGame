// 3D models for everything placed in the world, built from code (script-generated geometry).
// Each model is a list of parts { mesh, mat }; the view instances them per type and variant.
import { MeshBuilder } from '../gfx/mesh.js';
import { Rng } from './defs.js';
import { CULLFACE_NONE } from 'playcanvas';

// small helper: one builder per material key
class Parts {
    constructor(M) { this.M = M; this.list = new Map(); }
    b(key, matFn) {
        if (!this.list.has(key)) this.list.set(key, { b: new MeshBuilder(), mat: matFn() });
        return this.list.get(key).b;
    }
    build(device) {
        return [...this.list.values()].filter(p => p.b.idx.length).map(p => ({ mesh: p.b.toMesh(device), mat: p.mat }));
    }
}

const bez = (p0, p1, p2, t) => [0, 1, 2].map(k => (1 - t) * (1 - t) * p0[k] + 2 * (1 - t) * t * p1[k] + t * t * p2[k]);

export function buildModels(device, M) {
    const W = (k, o = {}) => () => M.pbr(k, o);
    const models = {};
    const make = (name, fn) => { const P = new Parts(M); fn(P); models[name] = P.build(device); };

    // ---------------------------------------------------------------- palms
    for (let v = 0; v < 3; v++) make(`palm:${v}`, P => {
        const rng = new Rng(100 + v);
        const h = 2.7 + v * 0.35, lean = 0.55 + v * 0.3;
        const top = [lean, h, lean * 0.3];
        const pts = [], rad = [];
        for (let i = 0; i <= 10; i++) {
            const t = i / 10;
            pts.push(bez([0, -0.1, 0], [0, h * 0.55, 0], top, t));
            rad.push(0.17 - t * 0.07 + (i % 2) * 0.008);
        }
        P.b('bark', W('bark', { tiling: 1 })).at().tube(pts, rad, 8, { uvU: 1, uvV: 0.9 });
        // fronds: arching strips with the frond texture
        const fb = P.b('frond', () => M.card(M.frond, '#9fb08a'));
        fb.at();
        const n = 8;
        for (let i = 0; i < n; i++) {
            const a = i / n * Math.PI * 2 + rng.range(-0.2, 0.2);
            const len = rng.range(1.25, 1.65), droop = rng.range(0.8, 1.2);
            const dx = Math.cos(a), dz = Math.sin(a);
            const ptsF = [], wid = [];
            for (let k = 0; k <= 6; k++) {
                const t = k / 6;
                ptsF.push([top[0] + dx * len * t, top[1] + Math.sin(t * Math.PI * 0.6) * 0.5 - t * t * droop, top[2] + dz * len * t]);
                wid.push(0.2 + Math.sin(t * Math.PI) * 0.42);
            }
            fb.ribbon(ptsF, wid, [-dz * 0.3, 1, dx * 0.3]);
        }
        const nut = P.b('nut', () => M.plain('#2a2016', { gloss: 0.4 }));
        for (let i = 0; i < 3; i++) nut.at(top[0] + Math.cos(i * 2.1) * 0.16, top[1] - 0.18, top[2] + Math.sin(i * 2.1) * 0.16).sphere(0.11, 8, 6);
    });

    // ---------------------------------------------------------------- bushes, grass, flowers
    for (let v = 0; v < 3; v++) make(`bush:${v}`, P => {
        const rng = new Rng(200 + v);
        const lb = P.b('leaf', () => M.card(M.leafTex, '#b4c4a4'));
        for (let i = 0; i < 9; i++) {
            const s = rng.range(0.7, 1.15);
            lb.at(rng.range(-0.25, 0.25), rng.range(0.05, 0.35), rng.range(-0.25, 0.25), rng.range(-30, 30), rng.range(0, 180), rng.range(-30, 30), s);
            lb.quad([-0.5, 0, 0], [0.5, 0, 0], [0.5, 0.95, 0], [-0.5, 0.95, 0]);
        }
    });
    for (let v = 0; v < 3; v++) make(`tuft:${v}`, P => {
        const gb = P.b('grass', () => M.card(M.grassTex, '#c4d4a8'));
        for (let i = 0; i < 3; i++) {
            gb.at(0, 0, 0, 0, i * 60 + v * 20, 0, 0.75 + v * 0.1);
            gb.quad([-0.4, 0, 0], [0.4, 0, 0], [0.4, 0.55, 0], [-0.4, 0.55, 0]);
        }
    });
    for (let v = 0; v < 3; v++) make(`flowers:${v}`, P => {
        const rng = new Rng(300 + v);
        const gb = P.b('grass', () => M.card(M.grassTex, '#8e9a7e'));
        for (let i = 0; i < 2; i++) { gb.at(0, 0, 0, 0, i * 90, 0, 0.6); gb.quad([-0.4, 0, 0], [0.4, 0, 0], [0.4, 0.45, 0], [-0.4, 0.45, 0]); }
        // pale ghost lilies glowing faintly
        const pb = P.b('petal', () => M.plain('#b8c8c0', { emissive: '#7fb0a8', emissiveIntensity: 0.35, gloss: 0.4 }));
        for (let i = 0; i < 5; i++) pb.at(rng.range(-0.3, 0.3), rng.range(0.25, 0.42), rng.range(-0.3, 0.3)).sphere(0.05, 6, 4);
    });

    // ---------------------------------------------------------------- rocks
    for (let v = 0; v < 2; v++) {
        make(`boulder:${v}`, P => { P.b('rock', W('rock', { tiling: 1.2 })).at(0, 0, 0, 0, v * 70).rock(0.55, 3 + v, 0.4); });
        make(`boulderBig:${v}`, P => {
            const b = P.b('rock', W('rock', { tiling: 1 }));
            b.at(0, 0, 0, 0, v * 50, 0, 1, 0.9, 1).rock(0.9, 11 + v, 0.45);
            b.at(0.6, 0, 0.35, 0, 20).rock(0.4, 17 + v, 0.4);
        });
    }
    // pebbles for scattering on beaches
    make('pebble', P => { const b = P.b('rock', W('rock', { tiling: 3 })); b.at(0, 0, 0).rock(0.13, 5, 0.5, 8, 6); b.at(0.22, 0, 0.1).rock(0.08, 8, 0.5, 8, 6); });
    // driftwood
    make('drift', P => { P.b('wood', W('bark', { tint: '#8a8070' })).at().tube([[-0.7, 0.06, 0], [0, 0.1, 0.12], [0.6, 0.07, -0.05]], [0.06, 0.08, 0.04], 6); });

    // ---------------------------------------------------------------- ruins: pillars and modular walls
    for (let v = 0; v < 2; v++) make(`pillar:${v}`, P => {
        const b = P.b('brick', W('brick', { tiling: 1 }));
        b.at().box(0.8, 0.3, 0.8, 1);
        const H = v ? 1.25 : 2.6;
        const prof = [[0.3, 0.3], [0.27, 0.45]];
        for (let i = 1; i <= 6; i++) prof.push([0.27 - i * 0.004, 0.45 + (H - 0.6) * i / 6]);
        b.at().lathe(prof, 10, { uvU: 2, uvV: 1, caps: !v });
        if (!v) b.at(0, H - 0.15, 0).box(0.75, 0.25, 0.75, 1);
        else b.at(0.5, 0, 0.3, 15, 30, 70).box(0.5, 0.5, 0.5, 1); // fallen drum
    });
    const wall = (P, v, rot) => {
        const b = P.b('brick', W('brick', { tiling: 0.55 }));
        const ry = rot ? 90 : 0;
        if (v === 0) {
            b.at(0, 0, 0, 0, ry).box(2.1, 1.7, 0.5, 0.55);
            for (let i = 0; i < 4; i++) b.at(rot ? 0 : -0.82 + i * 0.55, 1.7, rot ? -0.82 + i * 0.55 : 0, 0, ry).box(0.32, 0.32, 0.52, 0.55); // merlons (Zinnen)
        } else {
            b.at(rot ? 0 : -0.55, 0, rot ? -0.55 : 0, 0, ry).box(1.0, 1.4, 0.5, 0.55);
            b.at(rot ? 0 : 0.45, 0, rot ? 0.45 : 0, 0, ry).box(1.0, 0.75, 0.5, 0.55);
            b.at(rot ? 0 : 0.9, 0, rot ? 0.9 : 0, 0, ry).box(0.4, 0.45, 0.48, 0.55);
            const r = P.b('rock', W('rock', { tiling: 1.5 }));
            r.at(rot ? 0.5 : 0.6, 0, rot ? 0.6 : 0.5).rock(0.25, 4, 0.5, 8, 6);
            r.at(rot ? -0.45 : 0.2, 0, rot ? 0.2 : -0.45).rock(0.18, 9, 0.5, 8, 6);
        }
    };
    for (let v = 0; v < 2; v++) { make(`wall:${v}`, P => wall(P, v, false)); make(`wallV:${v}`, P => wall(P, v, true)); }
    make('palisade:0', P => {
        const b = P.b('bark', W('bark', { tint: '#9a8a78' }));
        for (let i = 0; i < 7; i++) {
            const x = -0.95 + i * 0.32, h = 1.5 + ((i * 7) % 3) * 0.12;
            b.at(x, 0, 0, (i % 3 - 1) * 3, i * 40).lathe([[0.12, -0.2], [0.12, h - 0.25], [0.0, h]], 7, { uvV: 1 });
        }
        P.b('wood', W('wood', { tiling: 1 })).at(0, 0.9, 0.12).box(2.2, 0.14, 0.08, 1);
    });

    // ---------------------------------------------------------------- crates, barrels, skulls, graves, posts, torches
    make('crate:0', P => {
        const b = P.b('wood', W('wood', { tiling: 1.4 }));
        b.at(0, 0, 0, 0, 12).box(0.72, 0.72, 0.72, 1.3);
        const f = P.b('frame', W('wood', { tint: '#6a5a4a', tiling: 2 }));
        for (const [x, z] of [[-0.34, -0.34], [0.34, -0.34], [-0.34, 0.34], [0.34, 0.34]]) f.at(x * Math.cos(0.21) - z * Math.sin(0.21), 0, z * Math.cos(0.21) + x * Math.sin(0.21), 0, 12).box(0.07, 0.74, 0.07, 2);
    });
    make('barrel:0', P => {
        const b = P.b('wood', W('wood', { tiling: 1 }));
        const prof = [];
        for (let i = 0; i <= 8; i++) { const t = i / 8; prof.push([0.28 + Math.sin(t * Math.PI) * 0.06, t * 0.9]); }
        b.at().lathe(prof, 14, { uvU: 3, uvV: 1.1 });
        const m = P.b('metal', W('metal', { metal: 0.8 }));
        for (const y of [0.12, 0.45, 0.78]) { const r = 0.285 + Math.sin(y / 0.9 * Math.PI) * 0.06; m.at(0, y, 0).lathe([[r + 0.012, -0.03], [r + 0.012, 0.03]], 16, { caps: false }); }
    });
    make('skulls:0', P => {
        const b = P.b('bone', W('bone', { tiling: 2 }));
        const eye = P.b('eye', () => M.plain('#050404', { gloss: 0.1 }));
        [[0, 0.13, 0, 0], [0.24, 0.1, 0.1, 40], [-0.2, 0.1, 0.12, -30], [0.05, 0.32, 0.05, 10]].forEach(([x, y, z, r]) => {
            b.at(x, y, z, 0, r, 0, 1, 0.9, 1.1).sphere(0.13, 10, 8);
            b.at(x + Math.sin(r * Math.PI / 180) * 0.08, y - 0.08, z + Math.cos(r * Math.PI / 180) * 0.08, 0, r).box(0.14, 0.07, 0.1, 2);
            for (const s of [-1, 1]) eye.at(x + Math.sin((r + 90) * Math.PI / 180) * 0.05 * s + Math.sin(r * Math.PI / 180) * 0.11, y + 0.02, z + Math.cos((r + 90) * Math.PI / 180) * 0.05 * s + Math.cos(r * Math.PI / 180) * 0.11).sphere(0.035, 6, 4);
        });
    });
    make('grave:0', P => {
        P.b('stone', W('brick', { tiling: 0.8 })).at(0, -0.05, 0, -6, 0, 4).box(0.55, 0.85, 0.14, 1);
        P.b('rock', W('dirt', { tiling: 2 })).at(0, -0.05, 0.6, 0, 0, 0, 1, 0.35, 1.6).rock(0.4, 6, 0.3, 10, 6);
    });
    make('post:0', P => { P.b('wood', W('bark', { tint: '#8a7a68' })).at(0, -0.6, 0).cylinder(0.12, 0.11, 2.0, 8, { uvU: 1, uvV: 1 }); });
    make('torch:0', P => {
        P.b('wood', W('bark', { tint: '#8a7a68' })).at().cylinder(0.06, 0.05, 1.55, 7);
        const m = P.b('metal', W('metal', { metal: 0.8 }));
        m.at(0, 1.45, 0).lathe([[0.06, 0], [0.16, 0.22], [0.17, 0.24]], 8, { caps: false });
    });

    // ---------------------------------------------------------------- the burning saber (Säbelfeuer)
    make('saberFire:0', P => {
        const rng = new Rng(77);
        const r = P.b('rock', W('rock', { tiling: 2 }));
        for (let i = 0; i < 8; i++) { const a = i / 8 * Math.PI * 2; r.at(Math.cos(a) * 0.62, 0, Math.sin(a) * 0.62, 0, i * 40).rock(0.17, 20 + i, 0.4, 8, 6); }
        P.b('ash', () => M.plain('#0d0b0a', { gloss: 0.1 })).at(0, 0.02, 0, 0, 0, 0, 1, 0.12, 1).sphere(0.55, 12, 4);
        // logs
        const lb = P.b('log', W('bark', { tint: '#5a4a3a' }));
        for (let i = 0; i < 4; i++) lb.at(0, 0.08, 0, 70, i * 90 + 20, 0).cylinder(0.06, 0.05, 0.75, 6);
        // the saber stuck in the ground, slightly tilted
        const steel = P.b('steel', W('metal', { metal: 0.9, gloss: 1.1, cull: CULLFACE_NONE }));
        steel.at(0.05, 0, -0.05, 8, 25, -6);
        const blade = [], bw = [];
        for (let i = 0; i <= 8; i++) { const t = i / 8; blade.push([Math.sin(t * 1.2) * 0.12, -0.15 + t * 1.45, 0]); bw.push(0.11 - t * 0.03 - (i === 0 ? 0.08 : 0)); }
        steel.ribbon(blade, bw, [0, 0, 1]);
        const guard = P.b('brass', W('metal', { tint: '#c9a060', metal: 1, gloss: 1 }));
        guard.at(0.05, 0, -0.05, 8, 25, -6).tube([[-0.22, 1.32, 0], [-0.05, 1.36, 0], [0.12, 1.32, 0], [0.2, 1.45, 0], [0.16, 1.62, 0]], [0.025, 0.035, 0.035, 0.03, 0.02], 6);
        P.b('grip', W('leather', { tint: '#3a2418' })).at(0.05, 0, -0.05, 8, 25, -6).tube([[0, 1.36, 0], [-0.02, 1.62, 0]], [0.035, 0.03], 7);
        // skull pommel with teal eyes
        const sk = P.b('bone', W('bone', { tiling: 2 }));
        sk.at(0.05, 0, -0.05, 8, 25, -6);
        sk.at(-0.04, 1.72, 0.0, 8, 25, -6, 1, 0.95, 1.05).sphere(0.11, 10, 8);
        const eye = P.b('eye', () => M.plain('#0a1412', { emissive: '#5ff2c8', emissiveIntensity: 6 }));
        for (const s of [-1, 1]) eye.at(-0.02 + s * 0.04, 1.73, 0.09).sphere(0.026, 6, 4);
        void rng;
    });

    // ---------------------------------------------------------------- the sea wreck (Wrackfeuer)
    make('wreck:0', P => {
        const w = P.b('wood', W('wood', { tiling: 0.8, tint: '#7a6a5a' }));
        for (let i = 0; i < 7; i++) { // ribs
            const z = -2 + i * 0.65, h = 1.6 - Math.abs(i - 3) * 0.12;
            for (const s of [-1, 1]) w.at().tube([[0, -0.6, z], [s * 0.9, -0.2, z], [s * 1.25, 0.6, z], [s * 1.15 + (i % 2) * 0.1 * s, h * 0.7, z]], [0.09, 0.08, 0.07, 0.05], 6);
        }
        w.at(0, -0.55, 0).box(0.25, 0.2, 4.6, 1); // keel
        w.at(-0.9, 0.0, -0.6, 0, 0, -25).box(0.12, 0.9, 2.6, 1); // planking
        w.at(0.4, 0.1, 1.2, 70, 20, 0).cylinder(0.12, 0.09, 3.2, 8); // fallen mast
    });

    // ---------------------------------------------------------------- chest (base + lid)
    make('chestBase', P => {
        P.b('wood', W('wood', { tiling: 1.5 })).at().box(0.82, 0.48, 0.55, 1.5);
        const m = P.b('metal', W('metal', { metal: 0.9, tint: '#b09060' }));
        for (const x of [-0.3, 0.3]) m.at(x, -0.01, 0).box(0.06, 0.5, 0.57, 2);
        m.at(0, 0.3, 0.28).box(0.1, 0.14, 0.03, 2);
    });
    make('chestLid', P => {
        const b = P.b('wood', W('wood', { tiling: 1.5 }));
        b.at(0, 0, 0.275, 0, 0, 90).lathe([[0.275, -0.41], [0.275, 0.41]], 12, { caps: true });
    });
    return models;
}
