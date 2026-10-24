// Material library: turns the procedural texture sets into PBR StandardMaterials.
// Albedos are dark and desaturated (the look lives off the lights, not the textures).
import {
    Texture, StandardMaterial, Color, Vec2, PIXELFORMAT_SRGBA8, PIXELFORMAT_RGBA8,
    FILTER_LINEAR_MIPMAP_LINEAR, FILTER_LINEAR, ADDRESS_REPEAT, ADDRESS_CLAMP_TO_EDGE,
    CULLFACE_NONE, CULLFACE_BACK, BLEND_NORMAL, BLEND_ADDITIVE, SHADERLANGUAGE_GLSL
} from 'playcanvas';
import { makeSurface, waterNormal, frondCard, grassCard, leafCard, decalAtlas, softDot, smokePuff, flameSprite, noiseTex } from './textures.js';

export function texture(device, cvs, { srgb = true, repeat = true, mips = true, name = 'tex' } = {}) {
    const t = new Texture(device, {
        name, width: cvs.width, height: cvs.height,
        format: srgb ? PIXELFORMAT_SRGBA8 : PIXELFORMAT_RGBA8,
        mipmaps: mips,
        minFilter: mips ? FILTER_LINEAR_MIPMAP_LINEAR : FILTER_LINEAR,
        magFilter: FILTER_LINEAR,
        addressU: repeat ? ADDRESS_REPEAT : ADDRESS_CLAMP_TO_EDGE,
        addressV: repeat ? ADDRESS_REPEAT : ADDRESS_CLAMP_TO_EDGE,
        anisotropy: 8
    });
    t.setSource(cvs);
    return t;
}

const lin = (hex) => {
    const c = new Color().fromString(hex);
    return c;
};

export class Materials {
    constructor(device, quality = 'high') {
        this.device = device;
        const S = quality === 'low' ? 256 : 512;
        this.sets = {};
        for (const k of ['sand', 'grass', 'dirt', 'stone', 'rock', 'wood', 'bark', 'brick', 'cloth', 'leather', 'skin', 'metal', 'bone']) {
            const big = ['sand', 'grass', 'dirt', 'stone', 'rock'].includes(k);
            const s = makeSurface(k, big ? S : S / 2);
            this.sets[k] = {
                albedo: texture(device, s.albedo, { name: k + '-a' }),
                normal: texture(device, s.normal, { srgb: false, name: k + '-n' }),
                surface: texture(device, s.surface, { srgb: false, name: k + '-s' })
            };
        }
        this.cache = new Map();
        this.waterN = texture(device, waterNormal(256), { srgb: false, name: 'water-n' });
        this.frond = texture(device, frondCard(), { repeat: false, name: 'frond' });
        this.grassTex = texture(device, grassCard(), { repeat: false, name: 'grass-card' });
        this.leafTex = texture(device, leafCard(), { repeat: false, name: 'leaf-card' });
        this.decals = texture(device, decalAtlas(), { repeat: false, name: 'decals' });
        this.dot = texture(device, softDot(), { repeat: false, name: 'dot' });
        this.smoke = texture(device, smokePuff(), { repeat: false, name: 'smoke' });
        this.flame = texture(device, flameSprite(), { repeat: false, name: 'flame' });
        this.noise = texture(device, noiseTex(256, 4), { name: 'noise' });
    }

    // textured PBR material, optionally tinted. tiling = repeats per UV unit
    pbr(kind, { tint = '#ffffff', tiling = 1, gloss = 1, bump = 1, emissive = null, emissiveIntensity = 1, vertexColor = false, cull = CULLFACE_BACK, metal = 0 } = {}) {
        const key = `${kind}|${tint}|${tiling}|${gloss}|${bump}|${emissive}|${emissiveIntensity}|${vertexColor}|${cull}|${metal}`;
        if (this.cache.has(key)) return this.cache.get(key);
        const s = this.sets[kind];
        const m = new StandardMaterial();
        m.name = key;
        m.diffuseMap = s.albedo;
        m.diffuse = lin(tint);
        m.normalMap = s.normal;
        m.bumpiness = bump;
        m.glossMap = s.surface;
        m.glossMapChannel = 'r';
        m.gloss = gloss;
        m.useMetalness = true;
        m.metalness = metal;
        if (vertexColor) { m.diffuseVertexColor = true; }
        const t = new Vec2(tiling, tiling);
        for (const p of ['diffuseMapTiling', 'normalMapTiling', 'glossMapTiling']) m[p] = t;
        if (emissive) { m.emissive = lin(emissive); m.emissiveIntensity = emissiveIntensity; }
        m.cull = cull;
        m.useSkybox = false;
        m.update();
        this.cache.set(key, m);
        return m;
    }
    // plain colour material (for small parts: eyes, teeth, ropes)
    plain(hex, { gloss = 0.3, metal = 0, emissive = null, emissiveIntensity = 1, cull = CULLFACE_BACK, vertexColor = false } = {}) {
        const key = `plain|${hex}|${gloss}|${metal}|${emissive}|${emissiveIntensity}|${cull}|${vertexColor}`;
        if (this.cache.has(key)) return this.cache.get(key);
        const m = new StandardMaterial();
        m.diffuse = lin(hex);
        m.gloss = gloss; m.useMetalness = true; m.metalness = metal;
        if (emissive) { m.emissive = lin(emissive); m.emissiveIntensity = emissiveIntensity; }
        if (vertexColor) m.diffuseVertexColor = true;
        m.cull = cull; m.useSkybox = false;
        m.update();
        this.cache.set(key, m);
        return m;
    }
    // alpha-tested foliage card
    card(tex, tint = '#ffffff', { gloss = 0.25 } = {}) {
        const key = `card|${tex.name}|${tint}`;
        if (this.cache.has(key)) return this.cache.get(key);
        const m = new StandardMaterial();
        m.diffuseMap = tex; m.diffuse = lin(tint);
        m.opacityMap = tex; m.opacityMapChannel = 'a'; m.alphaTest = 0.45;
        m.gloss = gloss; m.useMetalness = true; m.metalness = 0;
        m.cull = CULLFACE_NONE; m.twoSidedLighting = true; m.useSkybox = false;
        m.update();
        this.cache.set(key, m);
        return m;
    }
    // glowing, additive (fire, lightning, slashes) – HDR colour so bloom picks it up
    glow(hex, intensity = 4, tex = null) {
        const key = `glow|${hex}|${intensity}|${tex?.name}`;
        if (this.cache.has(key)) return this.cache.get(key);
        const m = new StandardMaterial();
        m.diffuse = new Color(0, 0, 0);
        m.emissive = lin(hex); m.emissiveIntensity = intensity;
        if (tex) { m.emissiveMap = tex; m.opacityMap = tex; m.opacityMapChannel = 'a'; }
        m.blendType = BLEND_ADDITIVE; m.depthWrite = false;
        m.cull = CULLFACE_NONE; m.useLighting = false; m.useSkybox = false; m.useFog = false;
        m.update();
        this.cache.set(key, m);
        return m;
    }
    // ground decals (blood, puddles, burns, moss) from the 2×2 atlas; transparent, receives light
    decal() {
        if (this._decal) return this._decal;
        const m = new StandardMaterial();
        m.diffuseMap = this.decals; m.opacityMap = this.decals; m.opacityMapChannel = 'a';
        m.blendType = BLEND_NORMAL; m.depthWrite = false;
        m.gloss = 0.75; m.useMetalness = true; m.metalness = 0;
        m.depthBias = -1; m.slopeDepthBias = -1;
        m.useSkybox = false;
        m.update();
        this._decal = m;
        return m;
    }

    // terrain: four layers blended by vertex colour (r sand, g grass, b dirt, a stone) with
    // height-based transitions, plus rock on steep slopes (triplanar). Custom diffuse/normal/gloss chunks.
    terrain() {
        const m = new StandardMaterial();
        m.diffuseVertexColor = true;
        m.normalMap = this.sets.sand.normal; // makes the shader build a TBN and uv0
        m.glossMap = this.sets.sand.surface; m.glossMapChannel = 'r';
        m.useMetalness = true; m.metalness = 0; m.gloss = 1;
        m.useSkybox = false;
        m.shaderChunksVersion = '2.23';
        const common = `
#ifndef TERRAIN_COMMON
#define TERRAIN_COMMON
uniform sampler2D tA0; uniform sampler2D tA1; uniform sampler2D tA2; uniform sampler2D tA3; uniform sampler2D tA4;
uniform sampler2D tN0; uniform sampler2D tN1; uniform sampler2D tN2; uniform sampler2D tN3; uniform sampler2D tN4;
uniform sampler2D tS0; uniform sampler2D tS1; uniform sampler2D tS2; uniform sampler2D tS3; uniform sampler2D tS4;
uniform float tTile;
vec4 tWeights;
float tRock;
vec2 tUv;
vec2 tUvR;
void terrainPrep() {
    tUv = vPositionW.xz * tTile;
    vec4 w = clamp(vVertexColor, 0.0, 1.0);
    float h0 = texture2D(tS0, tUv).g, h1 = texture2D(tS1, tUv).g, h2 = texture2D(tS2, tUv).g, h3 = texture2D(tS3, tUv).g;
    vec4 hw = w + vec4(h0, h1, h2, h3) * 0.6 * step(0.001, w);
    float mx = max(max(hw.x, hw.y), max(hw.z, hw.w));
    vec4 b = max(hw - (mx - 0.25), 0.0);
    tWeights = b / max(dot(b, vec4(1.0)), 1e-4);
    vec3 vn = normalize(vNormalW);
    tRock = smoothstep(0.86, 0.62, vn.y);
    vec3 n = abs(vn);
    tUvR = (n.x > n.z ? vPositionW.zy : vPositionW.xy) * tTile * vec2(1.0, 1.4);
}
#endif
`;
        const chunks = m.getShaderChunks(SHADERLANGUAGE_GLSL);
        chunks.set('normalMapPS', common + `
void getNormal() {
    vec3 n = texture2D(tN0, tUv).xyz * tWeights.x + texture2D(tN1, tUv).xyz * tWeights.y + texture2D(tN2, tUv).xyz * tWeights.z + texture2D(tN3, tUv).xyz * tWeights.w;
    n = mix(n, texture2D(tN4, tUvR).xyz, tRock);
    n = normalize(n * 2.0 - 1.0);
    dNormalW = normalize(dTBN * n);
}
`);
        chunks.set('diffusePS', common + `
void getAlbedo() {
    terrainPrep();
    vec3 a = texture2D(tA0, tUv).rgb * tWeights.x + texture2D(tA1, tUv).rgb * tWeights.y + texture2D(tA2, tUv).rgb * tWeights.z + texture2D(tA3, tUv).rgb * tWeights.w;
    a = mix(a, texture2D(tA4, tUvR).rgb, tRock);
    dAlbedo = a;
}
`);
        chunks.set('glossPS', common + `
void getGlossiness() {
    float g = texture2D(tS0, tUv).r * tWeights.x + texture2D(tS1, tUv).r * tWeights.y + texture2D(tS2, tUv).r * tWeights.z + texture2D(tS3, tUv).r * tWeights.w;
    g = mix(g, texture2D(tS4, tUvR).r, tRock);
    dGlossiness = g + 0.0000001;
}
`);
        const layers = ['sand', 'grass', 'dirt', 'stone', 'rock'];
        layers.forEach((k, i) => {
            m.setParameter(`tA${i}`, this.sets[k].albedo);
            m.setParameter(`tN${i}`, this.sets[k].normal);
            m.setParameter(`tS${i}`, this.sets[k].surface);
        });
        m.setParameter('tTile', 0.32);
        m.update();
        return m;
    }

    water() {
        const m = new StandardMaterial();
        m.diffuse = new Color(0.006, 0.02, 0.026);
        m.normalMap = this.waterN;
        m.normalMapTiling = new Vec2(18, 18);
        m.bumpiness = 0.55;
        m.useMetalness = true; m.metalness = 0.0;
        m.gloss = 0.93;
        m.opacity = 0.86;
        m.blendType = BLEND_NORMAL;
        m.depthWrite = true;
        m.useSkybox = false;
        m.update();
        return m;
    }
}
