// Game data: origins, gifts, weapons, spells, items, enemies, kings and the save format.

export const WEAPONS = {
    cutlass: { name: 'Entermesser', dmg: 17, scale: 'str', speed: 1.0, stam: 14, range: 1.6, heavyMul: 2.0, icon: 'saber', price: 0 },
    hammer: { name: 'Ankerhammer', dmg: 30, scale: 'str', speed: 0.66, stam: 24, range: 1.9, heavyMul: 1.9, icon: 'hammer', price: 2400 },
    rapier: { name: 'Rapier', dmg: 13, scale: 'dex', speed: 1.3, stam: 10, range: 1.9, heavyMul: 1.8, parry: 1.6, icon: 'rapier', price: 1800 },
    dagger: { name: 'Dolch', dmg: 11, scale: 'dex', speed: 1.45, stam: 9, range: 1.3, heavyMul: 1.7, icon: 'dagger', price: 900 },
    fetish: { name: 'Muschelfetisch', dmg: 9, scale: 'fai', speed: 1.1, stam: 12, range: 1.4, heavyMul: 1.6, icon: 'fetish', price: 1500 },
    club: { name: 'Morscher Knüppel', dmg: 10, scale: 'str', speed: 0.95, stam: 13, range: 1.5, heavyMul: 1.8, icon: 'club', price: 0 }
};
export const OFFHAND = {
    pistol: { name: 'Steinschlosspistole', dmg: 34, stam: 10, icon: 'pistol' },
    tide: { name: 'Gezeitenstoß', dmg: 24, stam: 18, icon: 'tide' },
    none: null
};
export const SPELLS = {
    bolt: { name: 'Kettenblitz', mp: 22, icon: 'bolt', desc: 'Ein Blitz springt von Feind zu Feind.' },
    fireball: { name: 'Feuerball', mp: 18, icon: 'fireball', desc: 'Eine Kugel aus Glut, die beim Aufprall birst.' }
};
export const ORIGINS = [
    { id: 'matrose', name: 'Deckmatrose', desc: 'Ausgewogen. Empfohlen für den Einstieg.', weapon: 'cutlass', off: 'pistol', stats: { vit: 6, end: 6, str: 6, dex: 6, fai: 4 }, level: 6 },
    { id: 'kanonier', name: 'Kanonier', desc: 'Langsam, harte Treffer, starke Breitseiten.', weapon: 'hammer', off: 'none', stats: { vit: 7, end: 5, str: 9, dex: 3, fai: 3 }, level: 6, broadside: 1.3 },
    { id: 'duellant', name: 'Duellant', desc: 'Parieren und Gegenschläge.', weapon: 'rapier', off: 'none', stats: { vit: 5, end: 7, str: 4, dex: 9, fai: 3 }, level: 6 },
    { id: 'schmuggler', name: 'Schmuggler', desc: 'Schnell, Pistole, mehr Perlen von Gegnern.', weapon: 'dagger', off: 'pistol', stats: { vit: 5, end: 7, str: 4, dex: 8, fai: 4 }, level: 6, greed: 1.2 },
    { id: 'sumpfkind', name: 'Sumpfkind', desc: 'Gezeitenzauber auf Distanz, wenig Leben.', weapon: 'fetish', off: 'tide', stats: { vit: 4, end: 6, str: 3, dex: 5, fai: 10 }, level: 6 },
    { id: 'treibgut', name: 'Treibgut', desc: 'Stufe 1, nur Lumpen und ein Knüppel. Für Profis.', weapon: 'club', off: 'none', stats: { vit: 3, end: 3, str: 3, dex: 3, fai: 3 }, level: 1 }
];
export const GIFTS = [
    { id: 'rum', name: 'Zusätzliche Rumflasche', desc: 'Eine Heilung mehr von Anfang an.' },
    { id: 'coin', name: 'Glücksdublone', desc: 'Gegner lassen mehr Perlen fallen.' },
    { id: 'chart', name: 'Alte Seekarte', desc: 'Die Karte (M) ist von Anfang an aufgedeckt.' },
    { id: 'bombs', name: 'Pulverbomben', desc: 'Fünf Bomben für den Anfang.' },
    { id: 'pearl', name: 'Schwarze Perle', desc: 'Wertlos – oder doch nicht?' },
    { id: 'none', name: 'Nichts', desc: 'Für echte Seeleute.' }
];
export const STAT_NAMES = { vit: 'Vitalität', end: 'Ausdauer', str: 'Kraft', dex: 'Geschick', fai: 'Tiefenglaube' };
export const STAT_DESC = { vit: 'Mehr Leben.', end: 'Mehr Ausdauer.', str: 'Schaden mit Hammer, Säbel, Knüppel.', dex: 'Schaden mit Rapier, Dolch und Pistole.', fai: 'Mehr Geist und stärkere Zauber.' };

// rarity colours for item names on the ground
export const RARITY = { common: [0.85, 0.81, 0.72, 1], uncommon: [0.37, 0.95, 0.78, 1], rare: [0.91, 0.64, 0.26, 1], unique: [0.86, 0.32, 0.25, 1] };
export const ITEMS = {
    pearls: { name: 'Perlenbeutel', rarity: 'common', icon: 'pearl', desc: 'Ein nasser Beutel voller Perlen.' },
    rum: { name: 'Rumflasche', rarity: 'rare', icon: 'rum', desc: 'Eine Rumflasche mehr an jedem Feuer.' },
    phial: { name: 'Seelenphiole', rarity: 'uncommon', icon: 'phial', desc: 'Füllt den Geist. Eine Phiole mehr an jedem Feuer.' },
    bullets: { name: 'Bleikugeln', rarity: 'common', icon: 'bullets', desc: 'Sechs Kugeln für die Pistole.' },
    bombs: { name: 'Pulverbomben', rarity: 'common', icon: 'bomb', desc: 'Drei Bomben. Mit 1 werfen.' },
    soulflower: { name: 'Seelenblume', rarity: 'uncommon', icon: 'shard', desc: 'Gezeitensplitter für den Schmied.' },
    shard: { name: 'Gezeitensplitter', rarity: 'uncommon', icon: 'shard', desc: 'Der Schmied härtet damit deine Waffe.' },
    fireball: { name: 'Folio der Glut', rarity: 'unique', icon: 'fireball', desc: 'Lehrt den Zauber Feuerball.' },
    bolt: { name: 'Sturmglas', rarity: 'rare', icon: 'bolt', desc: 'Dein Geist wächst um 20.' },
    ring: { name: 'Ring der Ertrunkenen', rarity: 'rare', icon: 'ring', desc: 'Du nimmst 12 % weniger Schaden.' },
    coat: { name: 'Mantel des Lotsen', rarity: 'rare', icon: 'coat', desc: 'Ausdauer erholt sich schneller.' },
    figurehead_fire: { name: 'Galionsfigur der Glut', rarity: 'unique', icon: 'figurehead', desc: 'Die Seele des Glutkönigs.' },
    figurehead_ice: { name: 'Galionsfigur des Frostes', rarity: 'unique', icon: 'figurehead', desc: 'Die Seele der Frostkönigin.' },
    figurehead_wind: { name: 'Galionsfigur des Sturms', rarity: 'unique', icon: 'figurehead', desc: 'Die Seele des Sturmkönigs.' },
    figurehead_earth: { name: 'Galionsfigur der Tiefe', rarity: 'unique', icon: 'figurehead', desc: 'Die Seele des Königs der Tiefe.' }
};

// enemies: hp, dmg, speed (px/s), aggro radius, reach (px), windup/active/recover (s), pearls, sheet
export const ENEMIES = {
    skeleton: { name: 'Ertrunkener Matrose', hp: 60, dmg: 16, speed: 38, aggro: 110, reach: 22, windup: 0.5, recover: 0.6, pearls: 60, sheet: 'skeleton', poise: 20 },
    skelBrute: { name: 'Ankerträger', hp: 150, dmg: 32, speed: 28, aggro: 100, reach: 28, windup: 0.95, recover: 1.0, pearls: 180, sheet: 'skelBrute', poise: 60, aoe: 26 },
    skelGunner: { name: 'Musketen-Gerippe', hp: 45, dmg: 20, speed: 30, aggro: 150, reach: 130, windup: 0.9, recover: 1.2, pearls: 80, sheet: 'skelGunner', ranged: true, poise: 10 },
    ashSkeleton: { name: 'Aschegänger', hp: 80, dmg: 20, speed: 42, aggro: 120, reach: 22, windup: 0.45, recover: 0.5, pearls: 110, sheet: 'ashSkeleton', poise: 25, burn: true },
    drowned: { name: 'Tangertrunkener', hp: 95, dmg: 22, speed: 34, aggro: 110, reach: 30, windup: 0.6, recover: 0.7, pearls: 120, sheet: 'drowned', poise: 30 },
    drownedIce: { name: 'Frostertrunkener', hp: 95, dmg: 22, speed: 34, aggro: 110, reach: 30, windup: 0.6, recover: 0.7, pearls: 120, sheet: 'drownedIce', poise: 30, chill: true },
    stormSailor: { name: 'Sturmmatrose', hp: 85, dmg: 19, speed: 46, aggro: 120, reach: 26, windup: 0.4, recover: 0.5, pearls: 120, sheet: 'stormSailor', poise: 20 },
    crab: { name: 'Strandkrabbe', hp: 35, dmg: 10, speed: 52, aggro: 80, reach: 16, windup: 0.35, recover: 0.5, pearls: 30, beast: 'crab', poise: 5 },
    crabWarrior: { name: 'Krabbenkrieger', hp: 170, dmg: 28, speed: 32, aggro: 100, reach: 26, windup: 0.75, recover: 0.8, pearls: 220, beast: 'crabWarrior', poise: 70 },
    shell: { name: 'Muschelkriecher', hp: 70, dmg: 18, speed: 26, aggro: 90, reach: 18, windup: 0.6, recover: 0.6, pearls: 90, beast: 'shell', poise: 40, charge: true },
    // monsters from the template sheet (static pictures, animated in code)
    livingChest: { name: 'Lebendiger Schatzrechner', hp: 90, dmg: 22, speed: 44, aggro: 120, reach: 22, windup: 0.4, recover: 0.6, pearls: 260, monster: 'livingChest', poise: 40, mimic: true },
    mimicChest: { name: 'Schatz-Mimic-Rechner', hp: 120, dmg: 26, speed: 40, aggro: 120, reach: 24, windup: 0.45, recover: 0.6, pearls: 320, monster: 'mimicChest', poise: 50, mimic: true },
    skelkraken: { name: 'Skelett-Kraken · Leviathans Zorn', hp: 420, dmg: 30, speed: 30, aggro: 120, reach: 36, windup: 0.9, recover: 1.0, pearls: 900, monster: 'skelkraken', poise: 120, aoe: 34 },
    cannonAngler: { name: 'Kanonen-Angel-Fisch-Dämon', hp: 120, dmg: 26, speed: 24, aggro: 170, reach: 150, windup: 1.1, recover: 1.4, pearls: 240, monster: 'cannonAngler', poise: 60, ranged: true, shot: 'cannon' },
    cannonEater: { name: 'Kanonen-Fresser-Dämon', hp: 140, dmg: 28, speed: 30, aggro: 110, reach: 26, windup: 0.7, recover: 0.8, pearls: 260, monster: 'cannonEater', poise: 60, charge: true },
    stormHarpy: { name: 'Sturm-Wind-Harpye-Konstrukt', hp: 70, dmg: 18, speed: 62, aggro: 140, reach: 22, windup: 0.4, recover: 0.6, pearls: 160, monster: 'stormHarpy', poise: 15 },
    wyvern: { name: 'Küsten-Wyvern · Schiffszerschmetterer', hp: 520, dmg: 34, speed: 50, aggro: 150, reach: 34, windup: 0.8, recover: 1.0, pearls: 1400, monster: 'wyvern', poise: 140, charge: true },
    orcCaptain: { name: 'Ork-Schlachter-Kapitän', hp: 560, dmg: 36, speed: 30, aggro: 130, reach: 34, windup: 0.9, recover: 1.0, pearls: 1400, monster: 'orcCaptain', poise: 160, aoe: 36 },
    bellConstruct: { name: 'Uhrwerk-Glocken-Matrosen-Konstrukt', hp: 600, dmg: 34, speed: 24, aggro: 130, reach: 40, windup: 1.1, recover: 1.1, pearls: 1500, monster: 'bellConstruct', poise: 200, aoe: 44 },
    abyssPirate: { name: 'Tiefen-Abgrund-Pirat', hp: 100, dmg: 22, speed: 36, aggro: 120, reach: 26, windup: 0.55, recover: 0.6, pearls: 180, monster: 'abyssPirate', poise: 35 },
    voodoo: { name: 'Voodoo-Mutanten-Kapitän', hp: 160, dmg: 22, speed: 30, aggro: 160, reach: 140, windup: 0.9, recover: 1.0, pearls: 420, monster: 'voodoo', poise: 60, ranged: true, shot: 'soul', volley: 3 },
    turtleWorm: { name: 'Schild-Kröten-Festungs-Wurm', hp: 720, dmg: 30, speed: 18, aggro: 170, reach: 160, windup: 1.2, recover: 1.3, pearls: 1800, monster: 'turtleWorm', poise: 260, ranged: true, shot: 'cannon', volley: 3 },
    trueMimic: { name: 'Mimic-Kapitäns wahre Gestalt', hp: 650, dmg: 38, speed: 34, aggro: 130, reach: 40, windup: 0.9, recover: 0.9, pearls: 1800, monster: 'trueMimic', poise: 200, aoe: 40 },
    shark: { name: 'Knochenhai', hp: 60, dmg: 0, speed: 60, aggro: 140, reach: 20, windup: 0.6, recover: 1, pearls: 90, beast: 'shark', sea: true }
};

// the four ghost kings (phase 1 humanoid, phase 2 monster) and the megalodon
export const KINGS = {
    fire: { zone: 'asche', name: 'Admiral Brandvogt, der Glutkönig', monster: 'Brandvogt, Dämonen-Kapitän-Mantikor', sheet: 'kingFire', beast: 'manticore', hp: 1300, dmg: 30, col: [1, 0.45, 0.15], el: 'fire', figurehead: 'figurehead_fire', pearls: 4000 },
    ice: { zone: 'frost', name: 'Königin Ysra Frostweh', monster: 'Ysra, Abgrund-Gorgonen-Königin', sheet: 'kingIce', beast: 'gorgonQueen', hp: 1250, dmg: 28, col: [0.55, 0.85, 1], el: 'ice', figurehead: 'figurehead_ice', pearls: 4000 },
    wind: { zone: 'raben', name: 'Kapitän Rabensturm, der Sturmkönig', monster: 'Rabensturm, Uhrwerk-Sturm-Monster', sheet: 'kingWind', beast: 'clockStorm', hp: 1200, dmg: 27, col: [0.8, 0.9, 1], el: 'wind', figurehead: 'figurehead_wind', pearls: 4000 },
    earth: { zone: 'mangroven', name: 'Morgrim Korallenhand, König der Tiefe', monster: 'Morgrim, Kraken-Basilisk-Chimäre', sheet: 'kingEarth', beast: 'krakenBasilisk', hp: 1450, dmg: 32, col: [0.55, 0.9, 0.4], el: 'earth', figurehead: 'figurehead_earth', pearls: 4000 }
};
export const MEGALODON = { name: 'Moloch, der Weltschlinger', hp: 3200, dmg: 36, pearls: 20000 };
export const EL_COL = { fire: [1, 0.45, 0.15], ice: [0.55, 0.85, 1], wind: [0.85, 0.92, 1], earth: [0.55, 0.9, 0.4] };
export const EL_NAME = { fire: 'Glut', ice: 'Frost', wind: 'Sturm', earth: 'Tiefe' };

export function derived(save) {
    const s = save.stats, w = WEAPONS[save.weapon];
    const scaleStat = s[w.scale] || 5;
    const lvl = save.weaponLevel || 0;
    return {
        maxHp: Math.round(60 + s.vit * 9),
        maxSt: Math.round(55 + s.end * 7),
        maxMp: Math.round(40 + s.fai * 6 + (save.items?.bolt ? 20 : 0)),
        spellDmg: Math.round(18 + s.fai * 4.5),
        dmg: Math.round(w.dmg * (0.7 + scaleStat * 0.06) * (1 + lvl * 0.12)),
        offDmg: save.off !== 'none' ? Math.round(OFFHAND[save.off].dmg * (0.75 + (save.off === 'tide' ? s.fai : s.dex) * 0.05)) : 0,
        armor: save.items?.ring ? 0.88 : 1,
        stRegen: save.items?.coat ? 1.3 : 1
    };
}
export const levelCost = level => Math.round(120 * Math.pow(1.1, level));
export const upgradeCost = lvl => ({ pearls: Math.round(600 * Math.pow(1.6, lvl)), shards: lvl + 1 });

export function newSave(name, look, originId, giftId) {
    const o = ORIGINS.find(x => x.id === originId) || ORIGINS[0];
    return {
        version: 2, name, look, origin: o.id, gift: giftId,
        weapon: o.weapon, off: o.off, weaponLevel: 0, owned: [o.weapon],
        stats: { ...o.stats }, level: o.level,
        pearls: giftId === 'pearl' ? 1 : 0,
        flasksMax: 3 + (giftId === 'rum' ? 1 : 0), phialsMax: 1, bullets: o.off === 'pistol' ? 8 : 0, bombs: giftId === 'bombs' ? 5 : 0, shards: 0,
        spells: ['bolt'], spell: 0,
        items: {}, picked: [], chests: [],
        lastFire: 'kueste', litFires: [], stain: null,
        kings: {}, figureheads: {}, mounted: {}, megalodon: false,
        crew: [], talked: {}, shipDock: 'kueste',
        explored: giftId === 'chart' ? ['kueste', 'hain', 'asche', 'frost', 'raben', 'mangroven'] : ['kueste'],
        deaths: 0, playTime: 0, createdAt: Date.now()
    };
}
