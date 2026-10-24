// Pixel font: 5×7 glyphs (+1 descender row), proportional widths, German umlauts.
// Glyph rows are 5-bit masks, bit 4 = leftmost column.
import { Pix } from './pix.js';

const G = {
    '0': '0E 11 13 15 19 11 0E', '1': '04 0C 04 04 04 04 0E', '2': '0E 11 01 02 04 08 1F', '3': '1F 02 04 02 01 11 0E', '4': '02 06 0A 12 1F 02 02',
    '5': '1F 10 1E 01 01 11 0E', '6': '06 08 10 1E 11 11 0E', '7': '1F 01 02 04 08 08 08', '8': '0E 11 11 0E 11 11 0E', '9': '0E 11 11 0F 01 02 0C',
    A: '0E 11 11 11 1F 11 11', B: '1E 11 11 1E 11 11 1E', C: '0E 11 10 10 10 11 0E', D: '1C 12 11 11 11 12 1C', E: '1F 10 10 1E 10 10 1F',
    F: '1F 10 10 1E 10 10 10', G: '0E 11 10 17 11 11 0F', H: '11 11 11 1F 11 11 11', I: '0E 04 04 04 04 04 0E', J: '07 02 02 02 02 12 0C',
    K: '11 12 14 18 14 12 11', L: '10 10 10 10 10 10 1F', M: '11 1B 15 15 11 11 11', N: '11 11 19 15 13 11 11', O: '0E 11 11 11 11 11 0E',
    P: '1E 11 11 1E 10 10 10', Q: '0E 11 11 11 15 12 0D', R: '1E 11 11 1E 14 12 11', S: '0F 10 10 0E 01 01 1E', T: '1F 04 04 04 04 04 04',
    U: '11 11 11 11 11 11 0E', V: '11 11 11 11 11 0A 04', W: '11 11 11 15 15 15 0A', X: '11 11 0A 04 0A 11 11', Y: '11 11 11 0A 04 04 04', Z: '1F 01 02 04 08 10 1F',
    a: '00 00 0E 01 0F 11 0F', b: '10 10 16 19 11 11 1E', c: '00 00 0E 10 10 11 0E', d: '01 01 0D 13 11 11 0F', e: '00 00 0E 11 1F 10 0E',
    f: '06 09 08 1C 08 08 08', g: '00 00 0F 11 11 0F 01 0E', h: '10 10 16 19 11 11 11', i: '04 00 0C 04 04 04 0E', j: '02 00 06 02 02 02 12 0C',
    k: '10 10 12 14 18 14 12', l: '0C 04 04 04 04 04 0E', m: '00 00 1A 15 15 11 11', n: '00 00 16 19 11 11 11', o: '00 00 0E 11 11 11 0E',
    p: '00 00 1E 11 11 1E 10 10', q: '00 00 0F 11 11 0F 01 01', r: '00 00 16 19 10 10 10', s: '00 00 0E 10 0E 01 1E', t: '08 08 1C 08 08 09 06',
    u: '00 00 11 11 11 13 0D', v: '00 00 11 11 11 0A 04', w: '00 00 11 11 15 15 0A', x: '00 00 11 0A 04 0A 11', y: '00 00 11 11 11 0F 01 0E', z: '00 00 1F 02 04 08 1F',
    'ä': '0A 00 0E 01 0F 11 0F', 'ö': '0A 00 0E 11 11 11 0E', 'ü': '0A 00 11 11 11 13 0D', 'Ä': '0A 00 0E 11 1F 11 11', 'Ö': '0A 00 0E 11 11 11 0E', 'Ü': '0A 00 11 11 11 11 0E', 'ß': '0C 12 12 14 12 11 16',
    '.': '00 00 00 00 00 00 10', ',': '00 00 00 00 00 08 08 10', ':': '00 00 10 00 00 10 00', ';': '00 00 08 00 00 08 08 10', '!': '10 10 10 10 10 00 10',
    '?': '0E 11 01 02 04 00 04', '-': '00 00 00 1C 00 00 00', '–': '00 00 00 1F 00 00 00', '+': '00 04 04 1F 04 04 00', '/': '01 01 02 04 08 10 10',
    '(': '08 10 10 10 10 10 08', ')': '10 08 08 08 08 08 10', "'": '10 10 00 00 00 00 00', '"': '14 14 00 00 00 00 00', '„': '00 00 00 00 00 14 14 08',
    '“': '14 14 00 00 00 00 00', '‘': '10 10 00 00 00 00 00', '’': '10 10 00 00 00 00 00', '·': '00 00 00 10 00 00 00', '%': '18 19 02 04 08 13 03', '×': '00 11 0A 04 0A 11 00',
    '=': '00 00 1F 00 1F 00 00', '<': '02 04 08 10 08 04 02', '>': '10 08 04 02 04 08 10', '…': '00 00 00 00 00 00 15', '#': '0A 0A 1F 0A 1F 0A 0A',
    '[': '18 10 10 10 10 10 18', ']': '18 08 08 08 08 08 18', '*': '00 15 0E 1F 0E 15 00', '&': '0C 12 14 08 15 12 0D', '»': '00 14 0A 05 0A 14 00', '«': '00 05 0A 14 0A 05 00',
    '↑': '04 0E 15 04 04 04 04', '↓': '04 04 04 04 15 0E 04', '←': '00 04 08 1F 08 04 00', '→': '00 04 02 1F 02 04 00', '♪': '06 05 04 04 1C 3C 18'
};

export class Font {
    constructor(atlas) {
        this.glyphs = {};
        this.lineH = 10;
        for (const [ch, rows] of Object.entries(G)) {
            const r = rows.split(' ').map(v => parseInt(v, 16));
            let minC = 5, maxC = -1;
            r.forEach(m => { for (let c = 0; c < 5; c++) if (m & (16 >> c)) { minC = Math.min(minC, c); maxC = Math.max(maxC, c); } });
            const w = maxC - minC + 1;
            const p = new Pix(w, 8);
            r.forEach((m, y) => { for (let c = minC; c <= maxC; c++) if (m & (16 >> c)) p.px(c - minC, y, [255, 255, 255]); });
            this.glyphs[ch] = { rect: atlas.add('f:' + ch.charCodeAt(0), p), w };
        }
        this.glyphs[' '] = { rect: null, w: 3 };
    }
    width(str) { let w = 0; for (const ch of str) w += (this.glyphs[ch] || this.glyphs['?']).w + 1; return Math.max(0, w - 1); }
    // word wrap to a pixel width
    wrap(str, maxW) {
        const lines = [];
        for (const para of String(str).split('\n')) {
            let line = '';
            for (const word of para.split(' ')) {
                const t = line ? line + ' ' + word : word;
                if (this.width(t) > maxW && line) { lines.push(line); line = word; } else line = t;
            }
            lines.push(line);
        }
        return lines;
    }
    // draw into the low-res target; col = [r,g,b,a]; shadow: 1 px dark drop shadow
    draw(r, str, x, y, col = [0.85, 0.81, 0.72, 1], { align = 'left', shadow = true, shadowCol = [0.04, 0.06, 0.09, 0.9] } = {}) {
        str = String(str);
        if (align === 'center') x -= Math.floor(this.width(str) / 2);
        else if (align === 'right') x -= this.width(str);
        x = Math.round(x); y = Math.round(y);
        const pass = (ox, oy, c) => {
            let cx = x;
            for (const ch of str) {
                const g = this.glyphs[ch] || this.glyphs['?'];
                if (g.rect) r.sprite(g.rect, cx + ox, y + oy, { color: c });
                cx += g.w + 1;
            }
        };
        if (shadow) pass(1, 1, shadowCol);
        pass(0, 0, col);
        return this.width(str);
    }
}
