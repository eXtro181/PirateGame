// The game palette (graphics spec, 32 colours) plus hue-shifted ramps:
// lights warmer (towards yellow), shadows cooler (towards blue/violet).

export const P = {
    abyss: '#0B1016', stormDark: '#16232E', storm: '#24394A', mistBlue: '#4A6170', mist: '#8FA3A8',
    deadwood: '#2A1E1A', tarwood: '#4A3326', oldwood: '#7A5A3E',
    mossDark: '#1E2B22', swamp: '#34473A', pale: '#5C7454',
    basaltDark: '#1F2226', basalt: '#3A3F45',
    bone: '#D8CFB8', soul: '#5FF2C8', soulLight: '#B8FFE8', lantern: '#E8A443', ember: '#C2502E',
    blood: '#7A1A22', coral: '#5A2E5C',
    slate: '#2B3A40', kelp: '#1F4A4C',
    // water
    waterDeep: '#0E2140', waterMid: '#173763', water: '#22508A', glint: '#4F86C0', foam: '#A9C8E6',
    // extra working colours (all derived from the palette's families)
    skin: '#B08A6E', skinDark: '#7A5A4A', ice: '#7E9EAC', iceDark: '#45697A', snow: '#A4B4BA',
    rust: '#8A4A2E', ash: '#5A5050', lava: '#E8743A', hexVein: '#9A2A2E', sand: '#6E6450', sandDark: '#4A4436',
    leaf: '#2E4A36', leafLight: '#4E6E4A', violet: '#7A4A7E', petrol: '#2E6A6A', gold: '#C9A04A'
};

export function rgb(h) {
    const v = parseInt(h.slice(1), 16);
    return [(v >> 16) & 255, (v >> 8) & 255, v & 255];
}
export const hexOf = (c) => '#' + c.map(v => Math.max(0, Math.min(255, Math.round(v))).toString(16).padStart(2, '0')).join('');
export const f3 = (h) => rgb(h).map(v => v / 255); // for shaders

function toHsl([r, g, b]) {
    r /= 255; g /= 255; b /= 255;
    const mx = Math.max(r, g, b), mn = Math.min(r, g, b), l = (mx + mn) / 2;
    if (mx === mn) return [0, 0, l];
    const d = mx - mn, s = l > 0.5 ? d / (2 - mx - mn) : d / (mx + mn);
    let h = mx === r ? (g - b) / d + (g < b ? 6 : 0) : mx === g ? (b - r) / d + 2 : (r - g) / d + 4;
    return [h * 60, s, l];
}
function toRgb([h, s, l]) {
    h = ((h % 360) + 360) % 360;
    const c = (1 - Math.abs(2 * l - 1)) * s, x = c * (1 - Math.abs((h / 60) % 2 - 1)), m = l - c / 2;
    const [r, g, b] = h < 60 ? [c, x, 0] : h < 120 ? [x, c, 0] : h < 180 ? [0, c, x] : h < 240 ? [0, x, c] : h < 300 ? [x, 0, c] : [c, 0, x];
    return [(r + m) * 255, (g + m) * 255, (b + m) * 255];
}
// move a hue towards a target hue by amount (degrees, shortest way)
function towards(h, t, amt) { let d = ((t - h + 540) % 360) - 180; return h + Math.sign(d) * Math.min(Math.abs(d), amt); }

// 5-step ramp around a base colour: [darkest, dark, base, light, lightest]
// shadows shift to blue/violet (240°), lights to yellow (55°)
export function ramp(hex, spread = 1) {
    const [h, s, l] = toHsl(rgb(hex));
    const out = [];
    for (const k of [-2, -1, 0, 1, 2]) {
        const dl = k * 0.085 * spread;
        const hh = k < 0 ? towards(h, 245, -k * 12) : k > 0 ? towards(h, 55, k * 10) : h;
        const ss = Math.min(1, s * (k < 0 ? 1 + -k * 0.05 : 1 - k * 0.06));
        out.push(toRgb([hh, ss, Math.max(0.02, Math.min(0.96, l + dl))]).map(Math.round));
    }
    return out;
}
// selective outline colour of a material: ~35 % darker, hue shifted to blue/violet
export function outlineOf(c) {
    const [h, s, l] = toHsl(c);
    return toRgb([towards(h, 250, 25), Math.min(1, s * 1.1), Math.max(0.02, l * 0.62)]).map(Math.round);
}
export function shade(c, f) { return [c[0] * f, c[1] * f, c[2] * f].map(Math.round); }
export function mix(a, b, t) { return [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t].map(Math.round); }

// colour grading per zone (lift = shadow tint, gain = highlight tint)
export const GRADES = {
    menu: { lift: [0.02, 0.03, 0.06], gain: [0.98, 1.0, 1.05], sat: 0.85, contrast: 1.06, ambient: [1, 1, 1] },
    coast: { lift: [0.02, 0.035, 0.05], gain: [1.0, 1.0, 1.02], sat: 0.8, contrast: 1.05, ambient: [0.48, 0.54, 0.58] },
    hub: { lift: [0.015, 0.04, 0.05], gain: [0.96, 1.02, 1.04], sat: 0.78, contrast: 1.06, ambient: [0.4, 0.5, 0.52] },
    fire: { lift: [0.04, 0.015, 0.025], gain: [1.03, 0.97, 0.95], sat: 0.82, contrast: 1.08, ambient: [0.5, 0.4, 0.42] },
    ice: { lift: [0.02, 0.04, 0.07], gain: [0.94, 1.0, 1.08], sat: 0.7, contrast: 1.04, ambient: [0.52, 0.58, 0.66] },
    wind: { lift: [0.03, 0.035, 0.05], gain: [0.98, 0.98, 1.02], sat: 0.7, contrast: 1.08, ambient: [0.42, 0.46, 0.52] },
    earth: { lift: [0.02, 0.04, 0.03], gain: [0.98, 1.03, 0.96], sat: 0.8, contrast: 1.06, ambient: [0.36, 0.44, 0.38] },
    sea: { lift: [0.01, 0.03, 0.06], gain: [0.96, 1.0, 1.06], sat: 0.8, contrast: 1.04, ambient: [0.45, 0.52, 0.6] },
    end: { lift: [0.07, 0.01, 0.03], gain: [1.05, 0.92, 0.95], sat: 0.9, contrast: 1.12, ambient: [0.5, 0.28, 0.3] }
};

// "Leuchtend": the look of the reference shots (default). "Düster" (above) follows the spec:
// the same art with saturation −40 % and brightness −30 %.
export const GRADES_BRIGHT = {
    menu: GRADES.menu,
    coast: { lift: [0.01, 0.015, 0.02], gain: [1.02, 1.01, 0.98], sat: 1.05, contrast: 1.03, ambient: [1.0, 0.98, 0.94], light: 0.45 },
    hub: { lift: [0.01, 0.02, 0.025], gain: [1.02, 1.01, 1.0], sat: 1.06, contrast: 1.03, ambient: [0.98, 0.98, 0.96], light: 0.45 },
    fire: { lift: [0.04, 0.01, 0.02], gain: [1.04, 0.96, 0.96], sat: 1.0, contrast: 1.05, ambient: [0.92, 0.72, 0.74], light: 0.6 },
    ice: { lift: [0.01, 0.02, 0.04], gain: [0.98, 1.0, 1.04], sat: 0.95, contrast: 1.03, ambient: [0.88, 0.93, 1.0], light: 0.5 },
    wind: { lift: [0.01, 0.015, 0.02], gain: [1.0, 1.0, 1.0], sat: 1.0, contrast: 1.03, ambient: [0.9, 0.92, 0.94], light: 0.5 },
    earth: { lift: [0.01, 0.025, 0.015], gain: [0.98, 1.02, 0.96], sat: 1.0, contrast: 1.04, ambient: [0.66, 0.78, 0.66], light: 0.7 },
    sea: { lift: [0.01, 0.02, 0.03], gain: [1.0, 1.0, 1.02], sat: 1.04, contrast: 1.03, ambient: [0.96, 0.98, 1.0], light: 0.45 },
    end: { lift: [0.05, 0.01, 0.03], gain: [1.04, 0.92, 0.95], sat: 0.95, contrast: 1.08, ambient: [0.62, 0.42, 0.48], light: 0.8 }
};
export const GRADES_DARK = Object.fromEntries(Object.entries(GRADES_BRIGHT).map(([k, g]) => [k, k === 'menu' ? g : { ...g, sat: g.sat * 0.6, gain: g.gain.map(v => v * 0.82), ambient: g.ambient.map(v => v * 0.62), light: 1 }]));
export const gradesFor = (look) => look === 'dark' ? GRADES_DARK : GRADES_BRIGHT;
