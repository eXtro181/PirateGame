// Builds all runtime-painted textures and materials once.
import { pixTextures, spriteMaterial } from './gfx.js';
import { buildTileAtlas } from '../art/tiles.js';
import { buildPropAtlas } from '../art/props.js';
import { crabSheet, shipSheet, sharkSheet, fxSheet, whirlSheet } from '../art/sheets.js';
import { SKELETON_LOOK } from '../art/character.js';
import { paintCharacterSheetHi, saberFireHi, flameSheetHi } from '../art/hires.js';

export function makeSheet(app, painted, name, { emissive = 0, caster = true, normal = 2.2, gloss = 0.15 } = {}) {
    const tex = pixTextures(app.graphicsDevice, painted.pix, name, normal);
    return {
        tex,
        frames: painted.frames,
        mat: spriteMaterial(tex, { emissive, gloss }),
        casterMat: caster ? spriteMaterial(tex, { shadowCaster: true }) : null
    };
}

export function buildArt(app) {
    const dev = app.graphicsDevice;
    const tiles = buildTileAtlas();
    const tilesTex = pixTextures(dev, tiles.pix, 'tiles', 1.3, 16);
    const props = buildPropAtlas();
    const propsTex = pixTextures(dev, props.img, 'props', 2.2);
    return {
        tiles: { tex: tilesTex, uv: tiles.uv, mat: spriteMaterial(tilesTex, { emissive: 1.2 }), casterMat: spriteMaterial(tilesTex, { shadowCaster: true }) },
        props: { tex: propsTex, rects: props.rects, mat: spriteMaterial(propsTex, { emissive: 2.5 }), casterMat: spriteMaterial(propsTex, { shadowCaster: true }) },
        flameTeal: makeSheet(app, flameSheetHi('teal', 14, 22), 'flame-teal', { emissive: 3.2, caster: false }),
        flameOrange: makeSheet(app, flameSheetHi('orange', 18, 26), 'flame-orange', { emissive: 3.2, caster: false }),
        flameSmall: makeSheet(app, flameSheetHi('orange', 8, 12), 'flame-small', { emissive: 3, caster: false }),
        crab: makeSheet(app, crabSheet(), 'crab', { emissive: 4 }),
        skeleton: makeSheet(app, paintCharacterSheetHi(SKELETON_LOOK), 'skeleton', { emissive: 5 }),
        saberFire: makeSheet(app, { pix: saberFireHi(), frames: { idle: [{ x: 0, y: 0, w: 18, h: 30 }] } }, 'saber-fire', { emissive: 4, caster: false }),
        ship: makeSheet(app, shipSheet(), 'ship', { emissive: 3 }),
        shark: makeSheet(app, sharkSheet(), 'shark', { emissive: 3 }),
        fx: makeSheet(app, fxSheet(), 'fx', { emissive: 4, caster: false }),
        whirl: makeSheet(app, whirlSheet(), 'whirl', { emissive: 0.8, caster: false, gloss: 0.3 })
    };
}

export function buildPlayerSheet(app, look) {
    return makeSheet(app, paintCharacterSheetHi(look), 'player', { emissive: 3 });
}
