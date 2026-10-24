// Dialogues (Dark-Souls-like: few words, much left unsaid) and the NPC services.
import { WEAPONS, STAT_NAMES, STAT_DESC, levelCost, upgradeCost, ITEMS } from './data.js';

const FIGS = ['fire', 'ice', 'wind', 'earth'];
export const CREW = {
    fire: { id: 'crewFire', name: 'Kanonierin Asra', song: 'glut', songName: 'Glutwind' },
    ice: { id: 'crewIce', name: 'Navigatorin Lene', song: 'frost', songName: 'Eisige Leine' },
    wind: { id: 'crewWind', name: 'Bootsmann Kalo', song: 'sturm', songName: 'Rabensturm' },
    earth: { id: 'crewEarth', name: 'Zimmermann Ode', song: 'tiefe', songName: 'Tiefer Grund' }
};

// returns { pages: [...], choices: [{ label, act }] }
export function dialogFor(game, npc) {
    const s = game.save, t = s.talked;
    const missing = FIGS.filter(f => !s.mounted[f]).length;
    const first = !t[npc.id];
    t[npc.id] = (t[npc.id] || 0) + 1;
    const bye = { label: 'Lebewohl', act: () => game.endTalk() };
    switch (npc.def.dialog) {
        case 'herald':
            if (first) return { pages: [
                'Ah … noch einer, den die See wieder ausgespuckt hat.',
                'Hör zu, Ertränkter. Nördlich von hier liegt der Schiffshain. Ein Friedhof für Schiffe, die nicht sterben durften.',
                'Ihre Galionsfiguren wurden geraubt. Vier Könige der Geisterflotte tragen sie in sich: Glut, Frost, Sturm und Tiefe.',
                'Bring die Seelen zurück an die Buge. Dann öffnet sich der Strudel, und du kannst bis ans Ende der Welt segeln.',
                'Dein Schiff wartet am Steg. Es steigt aus der Tiefe, wenn du es rufst.'
            ], choices: [bye] };
            if (missing === 0) return { pages: ['Der Strudel ruft dich. Nördlich des Hains, wo das Wasser schwarz wird.', 'Und wenn du am Ende der Welt stehst … grüß den Moloch nicht von mir.'], choices: [bye] };
            return { pages: [`Noch ${missing} Galionsfigur${missing > 1 ? 'en fehlen' : ' fehlt'}. Die Buge im Hain warten.`, 'Die Glut liegt im Osten, der Frost im Nordwesten. Rabenwacht im Westen, die Mangroven im Südosten.'], choices: [bye] };
        case 'maid':
            return { pages: first ? ['Willkommen am Seelenbaum, Ertränkter.', 'Ich hüte die Laternen. Bring mir Perlen, und ich gieße dein Licht stärker.'] : [missing ? 'Der Baum flüstert. Er vermisst seine Kinder.' : 'Der Baum ist voll. Hörst du es? Das Wasser im Norden singt.'],
                choices: [{ label: 'Stufe aufsteigen', act: () => game.ui.levelUp() }, { label: 'Über den Baum', act: () => game.talkPages(['Der Baum wuchs aus dem Kiel des ersten Schiffes, das die See verschlang.', 'Seine Laternen sind die Seelen jener, die nie heimkamen. Auch deine hing einmal hier.']) }, bye] };
        case 'smith': {
            const lvl = s.weaponLevel || 0, c = upgradeCost(lvl);
            return { pages: first ? ['Mein Amboss ist aus einer Kanone gegossen. Er hat mehr Männer erschlagen als ich.', 'Gezeitensplitter und Perlen. Dann härte ich dir die Klinge.'] : ['Was soll es sein?'],
                choices: [
                    { label: lvl >= 5 ? 'Waffe ist vollendet' : `Waffe härten (+${lvl + 1}) · ${c.pearls} Perlen, ${c.shards} Splitter`, disabled: lvl >= 5, act: () => game.upgradeWeapon() },
                    { label: 'Waffen', act: () => game.ui.weaponShop() },
                    bye] };
        }
        case 'innkeep':
            return { pages: first ? ['Rum, Kugeln, Pulver. Und Geschichten, wenn du zuhörst.', 'Die Leute hier trinken nicht, um zu vergessen. Sie trinken, um sich zu erinnern.'] : ['Noch einen?'],
                choices: [{ label: 'Handeln', act: () => game.ui.shop() }, { label: 'Die Crew', act: () => game.talkPages(crewTalk(game)) }, bye] };
    }
    // crew recruits
    const el = npc.def.crew;
    if (el) {
        const C = CREW[el];
        if (s.crew.includes(el)) return { pages: [`Unser Lied heißt „${C.songName}“. Wenn wir auf See sind, drück F, und wir singen.`, game.zoneKey === 'hain' ? 'Der Rum hier schmeckt nach Teer. Ich mag ihn.' : 'Wir sehen uns im Hain.'], choices: [bye] };
        const lines = {
            fire: ['Die Glut hat mein Schiff gefressen. Mit allen, die ich kannte.', 'Wenn du den Glutkönig suchst … nimm mich mit. Ich will sehen, wie er brennt.'],
            ice: ['Ich habe die Sterne über dem Eis gezählt. Neunhundert Nächte lang.', 'Du hast ein Schiff? Dann brauchst du jemanden, der den Weg kennt.'],
            wind: ['Rabenwacht war einmal ein Hafen. Jetzt ist es ein Nest.', 'Ich kann ein Segel flicken, während es reißt. Willst du so jemanden an Bord?'],
            earth: ['Die Wurzeln hier unten flüstern, wenn man still ist.', 'Ich habe Rümpfe aus Treibholz gebaut. Deiner sieht aus, als bräuchte er mich.']
        }[el];
        return { pages: lines, choices: [{ label: 'Komm an Bord', act: () => game.recruit(el) }, { label: 'Später', act: () => game.endTalk() }] };
    }
    return { pages: ['…'], choices: [bye] };
}

function crewTalk(game) {
    const s = game.save;
    if (!s.crew.length) return ['Deine Crew? Du hast keine. Such in den Ländern der Könige, da sitzen Verlorene an jedem Feuer.'];
    return [`Du hast ${s.crew.length} von 4 an Bord. ` + s.crew.map(e => CREW[e].name).join(', ') + '.', 'Eine volle Crew singt lauter. Und lautes Singen hält die Geister fern.'];
}

export function levelEntries(game) {
    const s = game.save, cost = levelCost(s.level);
    return Object.keys(STAT_NAMES).map(k => ({ label: `${STAT_NAMES[k]}`, right: String(s.stats[k]), desc: `${STAT_DESC[k]} Kosten: ${cost} Perlen`, disabled: s.pearls < cost, act: () => game.levelStat(k) }));
}
export function weaponEntries(game) {
    const s = game.save;
    return Object.entries(WEAPONS).map(([k, w]) => {
        const own = s.owned.includes(k);
        return { label: w.name + (s.weapon === k ? ' (geführt)' : ''), right: own ? 'besessen' : `${w.price} Perlen`, desc: `Schaden ${w.dmg} · skaliert mit ${STAT_NAMES[w.scale]}`, disabled: !own && (w.price === 0 || s.pearls < w.price), act: () => game.buyWeapon(k) };
    });
}
export function shopEntries(game) {
    const s = game.save;
    return [
        { label: 'Bleikugeln ×6', right: '150', desc: ITEMS.bullets.desc, disabled: s.pearls < 150, act: () => game.buy('bullets', 150) },
        { label: 'Pulverbomben ×3', right: '300', desc: ITEMS.bombs.desc, disabled: s.pearls < 300, act: () => game.buy('bombs', 300) },
        { label: 'Seelenphiole', right: s.phialsMax >= 4 ? 'ausverkauft' : '2500', desc: ITEMS.phial.desc, disabled: s.pearls < 2500 || s.phialsMax >= 4, act: () => game.buy('phial', 2500) }
    ];
}
