// Character creator options (colours and part lists) shared by the UI and the 3D figures.
export const SKIN = ['#f0c8a2', '#dfab82', '#c78c63', '#a6704b', '#875336', '#673e27', '#4c2c1b', '#c9c2b4'];
export const HAIR_COLORS = ['#1c1714', '#3b2617', '#6a4126', '#9a6a3a', '#c9a050', '#d9d0c0', '#7e7e86', '#7a2a1e', '#2a3a5a', '#4a1f3a'];
export const CLOTH = ['#7a2a2a', '#2a3f6a', '#3e5a32', '#6a4a2a', '#4a2a5a', '#2e2e34', '#8a7a5a', '#5a1e3a', '#1e4a4a', '#a0522d', '#3a3a6a', '#bcae8c'];
export const SCARF = ['#a8322e', '#2e5aa8', '#d8b040', '#2e8a5a', '#7a3aa0', '#e0e0d0', '#202024', '#d06a2a'];
export const HAIR_STYLES = ['Kurz', 'Lang', 'Zopf', 'Locken', 'Glatze', 'Dreadlocks'];
export const BEARDS = ['Keiner', 'Stoppeln', 'Vollbart', 'Kinnbart'];
export const HATS = ['Keine', 'Kopftuch', 'Dreispitz', 'Kapitänsmütze'];
export const BODIES = ['Schmal', 'Breit'];

export const DEFAULT_LOOK = { body: 0, skin: 1, hair: 0, hairColor: 1, beard: 1, hat: 1, coat: 0, accent: 5, scarf: 0, patch: false, scar: false };

export function resolveLook(look = {}) {
    const l = { ...DEFAULT_LOOK, ...look };
    return {
        ...l,
        skinHex: SKIN[l.skin] || SKIN[1],
        hairHex: HAIR_COLORS[l.hairColor] || HAIR_COLORS[1],
        coatHex: CLOTH[l.coat] || CLOTH[0],
        accHex: CLOTH[l.accent] || CLOTH[5],
        scarfHex: SCARF[l.scarf] || SCARF[0]
    };
}
