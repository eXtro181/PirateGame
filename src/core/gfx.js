// Graphics helpers: textures from painted pixels, lit sprite materials and quad meshes.
import {
    Texture, StandardMaterial, Mesh, MeshInstance, Color,
    PIXELFORMAT_SRGBA8, PIXELFORMAT_RGBA8, FILTER_NEAREST, FILTER_LINEAR, ADDRESS_CLAMP_TO_EDGE, ADDRESS_REPEAT,
    CULLFACE_NONE, SEMANTIC_TANGENT, calculateTangents, PRIMITIVE_TRIANGLES
} from 'playcanvas';

export function makeTexture(device, canvas, { srgb = true, repeat = false, name = 'tex', linear = false } = {}) {
    const t = new Texture(device, {
        name,
        width: canvas.width,
        height: canvas.height,
        format: srgb ? PIXELFORMAT_SRGBA8 : PIXELFORMAT_RGBA8,
        mipmaps: false,
        minFilter: linear ? FILTER_LINEAR : FILTER_NEAREST,
        magFilter: linear ? FILTER_LINEAR : FILTER_NEAREST,
        addressU: repeat ? ADDRESS_REPEAT : ADDRESS_CLAMP_TO_EDGE,
        addressV: repeat ? ADDRESS_REPEAT : ADDRESS_CLAMP_TO_EDGE
    });
    t.setSource(canvas);
    return t;
}

// Camera tilt (A Link Between Worlds style): the camera looks down PITCH degrees from the horizon.
// Standing sprites lean back by LEAN so they face the camera squarely.
export const PITCH = 56;
export const LEAN = (90 - PITCH) * Math.PI / 180;
// layer ids filled in by the stage: 'shadow' holds shadow-only casters the camera never draws
export const LAYERS = { shadow: -1 };

// Build the three maps of a painted image.
// DETAIL = texture pixels per painted pixel ("128-bit" mode: 4x the pixel density of the art)
export const DETAIL = 4;
export function pixTextures(device, pix, name, normalStrength = 2.2, cell = 0) {
    // art painted natively at high resolution (pix.native = pixels per logical pixel) is used as is
    const native = pix.native || 1;
    const hi = native > 1 ? pix : (DETAIL > 1 ? pix.upscale(DETAIL, cell) : pix);
    if (native > 1) hi._detail(0);
    const k = native > 1 ? native : DETAIL;
    return {
        diffuse: makeTexture(device, hi.toCanvas(), { name: name + '-d' }),
        normal: makeTexture(device, hi.toCanvas(hi.normalData(normalStrength * k * 0.7, cell * k)), { srgb: false, name: name + '-n' }),
        emissive: makeTexture(device, hi.toCanvas(hi.emi), { name: name + '-e' }),
        // rects stay in logical-pixel units; UVs are normalised, so the logical size is kept
        w: pix.w / native,
        h: pix.h / native
    };
}

export function spriteMaterial(tex, { emissive = 0, gloss = 0.15, specular = 0.04, shadowCaster = false } = {}) {
    const m = new StandardMaterial();
    m.diffuseMap = tex.diffuse;
    m.opacityMap = tex.diffuse;
    m.opacityMapChannel = 'a';
    m.alphaTest = 0.5;
    if (!shadowCaster) {
        m.normalMap = tex.normal;
        m.bumpiness = 1;
        if (emissive > 0) {
            m.emissiveMap = tex.emissive;
            m.emissive = new Color(1, 1, 1);
            m.emissiveIntensity = emissive;
        }
        m.specular = new Color(specular, specular, specular);
        m.gloss = gloss;
    }
    m.cull = CULLFACE_NONE;
    m.useSkybox = false;
    m.update();
    return m;
}

// A growable batch of quads. Flat quads lie on the ground (XZ), caster quads stand upright.
export class QuadBatch {
    constructor() { this.pos = []; this.nrm = []; this.uv = []; this.idx = []; }
    get count() { return this.pos.length / 3; }
    // flat sprite: feet at (x, z), extends to -z (screen up). uv rect in texels of a tw*th texture
    flat(x, z, w, h, y, r, tw, th, flipX = false) {
        const b = this.count;
        const x0 = x - w / 2, x1 = x + w / 2, z0 = z - h, z1 = z;
        let u0 = r.x / tw, u1 = (r.x + r.w) / tw;
        const v0 = r.y / th, v1 = (r.y + r.h) / th;
        if (flipX) [u0, u1] = [u1, u0];
        this.pos.push(x0, y, z0, x1, y, z0, x1, y, z1, x0, y, z1);
        this.nrm.push(0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0);
        this.uv.push(u0, v0, u1, v0, u1, v1, u0, v1);
        this.idx.push(b, b + 2, b + 1, b, b + 3, b + 2);
    }
    // upright quad standing on (x, z), facing direction ax (0 = facing +z, 1 = facing +x)
    upright(x, z, w, h, r, tw, th, axis) {
        const b = this.count;
        const u0 = r.x / tw, u1 = (r.x + r.w) / tw, v0 = r.y / th, v1 = (r.y + r.h) / th;
        if (axis === 0) {
            this.pos.push(x - w / 2, h, z, x + w / 2, h, z, x + w / 2, 0, z, x - w / 2, 0, z);
            this.nrm.push(0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1);
        } else {
            this.pos.push(x, h, z - w / 2, x, h, z + w / 2, x, 0, z + w / 2, x, 0, z - w / 2);
            this.nrm.push(1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0);
        }
        this.uv.push(u0, v0, u1, v0, u1, v1, u0, v1);
        this.idx.push(b, b + 2, b + 1, b, b + 3, b + 2);
    }
    // standing sprite leaning back towards the camera, feet at (x, z), bottom at height y
    lean(x, z, w, h, y, r, tw, th, flipX = false) {
        const b = this.count;
        const c = Math.cos(LEAN), sn = Math.sin(LEAN);
        let u0 = r.x / tw, u1 = (r.x + r.w) / tw;
        const v0 = r.y / th, v1 = (r.y + r.h) / th;
        if (flipX) [u0, u1] = [u1, u0];
        const yt = y + h * c, zt = z - h * sn;
        this.pos.push(x - w / 2, yt, zt, x + w / 2, yt, zt, x + w / 2, y, z, x - w / 2, y, z);
        for (let i = 0; i < 4; i++) this.nrm.push(0, sn, c);
        this.uv.push(u0, v0, u1, v0, u1, v1, u0, v1);
        this.idx.push(b, b + 2, b + 1, b, b + 3, b + 2);
    }
    // generic quad from four corners (clockwise seen from the front) with one normal
    quad(p, n, r, tw, th) {
        const b = this.count;
        const u0 = r.x / tw, u1 = (r.x + r.w) / tw, v0 = r.y / th, v1 = (r.y + r.h) / th;
        this.pos.push(...p);
        for (let i = 0; i < 4; i++) this.nrm.push(n[0], n[1], n[2]);
        this.uv.push(u0, v0, u1, v0, u1, v1, u0, v1);
        this.idx.push(b, b + 2, b + 1, b, b + 3, b + 2);
    }
    // two crossed upright quads so omni lights from any side find a silhouette
    caster(x, z, w, h, r, tw, th) {
        this.upright(x, z, w, h, r, tw, th, 0);
        this.upright(x, z, w * 0.8, h, r, tw, th, 1);
    }
    toMesh(device, tangents = true) {
        const mesh = new Mesh(device);
        mesh.setPositions(this.pos);
        mesh.setNormals(this.nrm);
        mesh.setUvs(0, this.uv);
        mesh.setIndices(this.idx);
        if (tangents && this.idx.length) {
            const t = calculateTangents(this.pos, this.nrm, this.uv, this.idx);
            mesh.setVertexStream(SEMANTIC_TANGENT, t, 4);
        }
        mesh.update(PRIMITIVE_TRIANGLES);
        return mesh;
    }
}

export function meshInstance(mesh, material, { castShadow = false, receiveShadow = true } = {}) {
    const mi = new MeshInstance(mesh, material);
    mi.castShadow = castShadow;
    mi.receiveShadow = receiveShadow;
    return mi;
}
