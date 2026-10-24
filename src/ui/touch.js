// On-screen touch controls: a floating stick on the left, action buttons on the right.
import { h } from './ui.js';

const BTN = [
    ['attack', 'Hieb', 'b-attack'], ['heavy', 'Schwer', 'b-heavy'], ['roll', 'Rolle', 'b-roll'], ['parry', 'Parade', 'b-parry'],
    ['drink', 'Rum', 'b-drink'], ['spell', 'Zauber', 'b-spell'], ['item', 'Schuss', 'b-item'], ['interact', 'E', 'b-interact'],
    ['belt1', 'Bombe', 'b-bomb'], ['belt2', 'Phiole', 'b-phial'], ['lock', 'Ziel', 'b-lock'], ['shanty', 'Shanty', 'b-shanty']
];
const TOP = [['menu', 'Menü'], ['map', 'Karte'], ['swap', 'Zauber ↻']];

export class Touch {
    constructor(input, root) {
        this.input = input;
        this.el = h('div', { class: 'touch hidden' });
        this.stickZone = h('div', { class: 'stick-zone' });
        this.knob = h('div', { class: 'stick-knob' }); this.base = h('div', { class: 'stick-base' }, this.knob);
        this.stickZone.append(this.base);
        const pad = h('div', { class: 'pad' }, BTN.map(([a, label, cls]) => this.btn(a, label, 'tbtn ' + cls)));
        const top = h('div', { class: 'tops' }, TOP.map(([a, label]) => this.btn(a, label, 'tbtn small')));
        this.el.append(this.stickZone, pad, top);
        root.append(this.el);
        let id = null, ox = 0, oy = 0;
        this.stickZone.addEventListener('pointerdown', e => { id = e.pointerId; ox = e.clientX; oy = e.clientY; this.base.style.left = ox + 'px'; this.base.style.top = oy + 'px'; this.base.classList.add('on'); this.stickZone.setPointerCapture(id); e.preventDefault(); input.any = true; });
        this.stickZone.addEventListener('pointermove', e => { if (e.pointerId !== id) return; let dx = e.clientX - ox, dy = e.clientY - oy; const l = Math.hypot(dx, dy), m = 46; if (l > m) { dx *= m / l; dy *= m / l; } this.knob.style.transform = `translate(${dx}px, ${dy}px)`; input.touchStick = { x: dx / m, y: dy / m }; });
        const up = e => { if (e.pointerId !== id) return; id = null; input.touchStick = { x: 0, y: 0 }; this.knob.style.transform = ''; this.base.classList.remove('on'); };
        this.stickZone.addEventListener('pointerup', up); this.stickZone.addEventListener('pointercancel', up);
    }
    btn(action, label, cls) {
        const b = h('button', { class: cls }, label);
        b.addEventListener('pointerdown', e => { e.preventDefault(); e.stopPropagation(); this.input.vPress(action); b.classList.add('on'); });
        const rel = () => { this.input.vRelease(action); b.classList.remove('on'); };
        b.addEventListener('pointerup', rel); b.addEventListener('pointercancel', rel); b.addEventListener('pointerleave', rel);
        return b;
    }
    show(v, sailing = false) { this.el.classList.toggle('hidden', !v); this.el.classList.toggle('sailing', sailing); }
}
