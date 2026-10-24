// Game data: origins, gifts, weapons, stats.

export const WEAPONS = {
    cutlass: { name: 'Entermesser', dmg: 17, scale: 'str', speed: 1.0, stam: 14, range: 1.15, heavyMul: 2.0 },
    hammer: { name: 'Ankerhammer', dmg: 30, scale: 'str', speed: 0.66, stam: 24, range: 1.45, heavyMul: 1.9 },
    rapier: { name: 'Rapier', dmg: 13, scale: 'dex', speed: 1.3, stam: 10, range: 1.25, heavyMul: 1.8, parry: 1.5 },
    dagger: { name: 'Dolch', dmg: 11, scale: 'dex', speed: 1.45, stam: 9, range: 0.95, heavyMul: 1.7 },
    fetish: { name: 'Muschelfetisch', dmg: 9, scale: 'fai', speed: 1.1, stam: 12, range: 1.0, heavyMul: 1.6 },
    club: { name: 'Morscher Knüppel', dmg: 10, scale: 'str', speed: 0.95, stam: 13, range: 1.05, heavyMul: 1.8 }
};

// Second hand / item on K: pistol shots or tide bolts
export const OFFHAND = {
    pistol: { name: 'Steinschlosspistole', dmg: 34, ammo: 4, stam: 10 },
    tide: { name: 'Gezeitenstoß', dmg: 24, ammo: 6, stam: 18 },
    none: null
};

export const ORIGINS = [
    { id: 'matrose', name: 'Deckmatrose', desc: 'Ausgewogen. Empfohlen für den Einstieg.', weapon: 'cutlass', off: 'none', stats: { vit: 6, end: 6, str: 6, dex: 6, fai: 4 }, level: 6 },
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
    { id: 'whistle', name: 'Bootsmannspfeife', desc: 'Die Crew kennt „Hol die Leine“ von Beginn an.' },
    { id: 'pearl', name: 'Schwarze Perle', desc: 'Wertlos – oder doch nicht?' },
    { id: 'none', name: 'Nichts', desc: 'Für echte Seeleute.' }
];

export const STAT_NAMES = { vit: 'Vitalität', end: 'Ausdauer', str: 'Kraft', dex: 'Geschick', fai: 'Tiefenglaube' };

export function derived(save) {
    const s = save.stats;
    const w = WEAPONS[save.weapon];
    const scaleStat = s[w.scale] || 5;
    return {
        maxHp: Math.round(60 + s.vit * 9),
        maxSt: Math.round(55 + s.end * 7),
        maxMp: Math.round(40 + s.fai * 6),
        spellDmg: Math.round(18 + s.fai * 4.5),
        dmg: Math.round(w.dmg * (0.7 + scaleStat * 0.06)),
        offDmg: save.off !== 'none' ? Math.round(OFFHAND[save.off].dmg * (0.75 + (save.off === 'tide' ? s.fai : s.dex) * 0.05)) : 0
    };
}

export function levelCost(level) { return Math.round(120 * Math.pow(1.1, level)); }

export function newSave(name, look, originId, giftId) {
    const o = ORIGINS.find(x => x.id === originId) || ORIGINS[0];
    const save = {
        version: 1,
        name,
        look,
        origin: o.id,
        gift: giftId,
        weapon: o.weapon,
        off: o.off,
        stats: { ...o.stats },
        level: o.level,
        pearls: giftId === 'pearl' ? 1 : 0,
        flasksMax: 3 + (giftId === 'rum' ? 1 : 0),
        lastFire: 'kueste',
        litFires: [],
        chests: [],
        stain: null,
        explored: giftId === 'chart' ? 'all' : '',
        songs: giftId === 'whistle' ? ['leine'] : [],
        deaths: 0,
        playTime: 0,
        createdAt: Date.now()
    };
    return save;
}
