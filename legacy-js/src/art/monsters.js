// Monsters from the user's template sheet (assets/monster-vorlage.jpg), cut out and reduced to game
// pixel size by tools/extract_monsters.py into public/monsters.png + monsters.json.
// Each becomes a "static" beast sheet: one picture, animated in code (bob, hover, lunge, flash).

// native facing of each picture (-1 = looks left, 1 = right, 0 = symmetric) and whether it flies
export const MONSTER_META = {
    kraken: { facing: 0, sea: true }, skelkraken: { facing: 0 }, abyssPirate: { facing: -1 }, voodoo: { facing: 0 },
    ghostGalley: { facing: -1, ship: true }, livingChest: { facing: 0 }, stormHarpy: { facing: 0, fly: true }, cannonAngler: { facing: -1 },
    fleshShip: { facing: -1, ship: true }, mimicChest: { facing: 0 }, clockStorm: { facing: 0, fly: true }, cannonEater: { facing: -1 },
    gorgonQueen: { facing: -1 }, krakenBasilisk: { facing: -1 }, manticore: { facing: -1 }, orcCaptain: { facing: -1 },
    wyvern: { facing: -1, fly: true }, bellConstruct: { facing: 0 }, turtleWorm: { facing: -1 }, trueMimic: { facing: 0 }
};

const loadImage = (src) => new Promise((res, rej) => { const i = new Image(); i.onload = () => res(i); i.onerror = () => rej(new Error('Bild fehlt: ' + src)); i.src = src; });

export async function loadMonsters(renderer) {
    const [img, meta] = await Promise.all([loadImage('monsters.png'), fetch('monsters.json').then(r => r.json())]);
    const tex = renderer.texture(img);
    const out = {};
    for (const [name, [x, y, w, h]] of Object.entries(meta)) {
        const m = MONSTER_META[name] || { facing: 0 };
        const rect = { x, y, w, h, tex, gtex: tex, glow: false };
        const one = [rect];
        out[name] = {
            static: true, beast: true, name, w, h, ax: Math.floor(w / 2), ay: h - 2, facing: m.facing, fly: !!m.fly, sea: !!m.sea, ship: !!m.ship,
            shadow: [Math.max(6, Math.min(60, Math.round(w * 0.32))), 3],
            anims: { idle: one, walk: one, atk: [rect, rect], cast: one, hurt: one, dead: one }
        };
    }
    return out;
}
