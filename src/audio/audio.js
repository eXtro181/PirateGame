// Synthesized "64-bit" shanty music, crew voices, ambience and sound effects (WebAudio, no files).

const NOTE = { C: 0, 'C#': 1, Db: 1, D: 2, 'D#': 3, Eb: 3, E: 4, F: 5, 'F#': 6, Gb: 6, G: 7, 'G#': 8, Ab: 8, A: 9, 'A#': 10, Bb: 10, B: 11 };
const freq = (n) => { const m = /^([A-G][#b]?)(\d)$/.exec(n); return 440 * Math.pow(2, (NOTE[m[1]] + (+m[2] + 1) * 12 - 69) / 12); };

// "G4:2 B4:1 | ..." -> [{n, len}] (len in units; "R" = rest)
function parse(str) {
    return str.split(/\s+/).filter(t => t && t !== '|').map(t => { const [n, l] = t.split(':'); return { n, len: +l }; });
}
function seq(str, inst, vel = 1, start = 0) {
    const out = [];
    let t = start;
    for (const { n, len } of parse(str)) {
        if (n !== 'R') for (const nn of n.split('+')) out.push({ t, dur: len, inst, f: freq(nn), vel });
        t += len;
    }
    return { events: out, len: t - start };
}

// ----------------------------------------------------------------------------- songs
function songSalz() { // "Salz auf der Haut" – Smaragdküste, G major 6/8
    const A = 'G4:2 B4:1 D5:2 D5:1 | E5:2 D5:1 B4:2 G4:1 | A4:2 B4:1 C5:2 A4:1 | B4:3 G4:3 | G4:2 B4:1 D5:2 D5:1 | E5:2 G5:1 F#5:2 E5:1 | D5:2 B4:1 A4:2 F#4:1 | G4:6';
    const B = 'D5:2 E5:1 F#5:2 G5:1 | A5:2 G5:1 F#5:2 D5:1 | E5:2 D5:1 C5:2 B4:1 | A4:3 D5:3 | B4:2 D5:1 G5:2 F#5:1 | E5:2 C5:1 A4:2 F#4:1 | G4:2 A4:1 B4:2 A4:1 | G4:6';
    const mel = seq(`${A} | ${B}`, 'whistle', 0.55);
    const chords = ['G3+B3+D4', 'C4+E4+G3', 'A3+C4+E4', 'G3+B3+D4', 'G3+B3+D4', 'C4+E4+G3', 'D4+F#4+A3', 'G3+B3+D4', 'D4+F#4+A3', 'D4+F#4+A3', 'C4+E4+G3', 'D4+F#4+A3', 'G3+B3+D4', 'A3+C4+E4', 'D4+F#4+A3', 'G3+B3+D4'];
    const roots = ['G2', 'C3', 'A2', 'G2', 'G2', 'C3', 'D3', 'G2', 'D3', 'D3', 'C3', 'D3', 'G2', 'A2', 'D3', 'G2'];
    const fifths = ['D3', 'G3', 'E3', 'D3', 'D3', 'G3', 'A2', 'D3', 'A2', 'A2', 'G3', 'A2', 'D3', 'E3', 'A2', 'D3'];
    const ev = [...mel.events];
    chords.forEach((c, i) => {
        ev.push(...seq(`${c}:3 ${c}:3`, 'accordion', 0.16, i * 6).events);
        ev.push(...seq(`${roots[i]}:3 ${fifths[i]}:3`, 'bass', 0.5, i * 6).events);
        ev.push({ t: i * 6, dur: 1, inst: 'drum', f: 70, vel: 0.55 }, { t: i * 6 + 3, dur: 1, inst: 'drum', f: 60, vel: 0.35 }, { t: i * 6 + 5, dur: 1, inst: 'tick', f: 0, vel: 0.12 });
    });
    return { name: 'salz', unit: 60 / 76 / 3, len: 96, events: ev, gain: 0.5 };
}

function songLeine() { // "Hol die Leine" – Offene See, D major 4/4, call & response with the crew
    const call = 'D5:2 D5:2 F#5:2 A5:2 | B5:2 A5:2 F#5:4 | R:8 | E5:2 F#5:2 E5:2 D5:2 | D5:2 D5:2 F#5:2 A5:2 | B5:2 D6:2 A5:4 | R:8 | R:8';
    const crew = 'R:16 | A4:2 A4:1 F#4:1 D4:4 | R:8 | R:16 | G4:2 F#4:2 E4:2 A4:2 | D4:8';
    const ev = [...seq(call, 'fiddle', 0.42).events, ...seq(crew, 'crewLead', 0.9).events];
    const chords = ['D3+F#3+A3', 'G3+B3+D4', 'D3+F#3+A3', 'A3+C#4+E4', 'D3+F#3+A3', 'G3+B3+D4', 'A3+C#4+E4', 'D3+F#3+A3'];
    const roots = ['D2', 'G2', 'D2', 'A2', 'D2', 'G2', 'A2', 'D2'];
    chords.forEach((c, i) => {
        ev.push(...seq(`${c}:8`, 'crewHum', 0.5, i * 8).events);
        ev.push(...seq(`${c}:2 R:2 ${c}:2 R:2`, 'accordion', 0.13, i * 8).events);
        ev.push(...seq(`${roots[i]}:4 ${roots[i]}:4`, 'bass', 0.55, i * 8).events);
        for (let b = 0; b < 4; b++) ev.push({ t: i * 8 + b * 2, dur: 1, inst: b % 2 ? 'tick' : 'drum', f: 65, vel: b % 2 ? 0.2 : 0.6 });
    });
    return { name: 'leine', unit: 60 / 104 / 2, len: 64, events: ev, gain: 0.5 };
}

function songAtem() { // "Atem holen" – at the fire; the Halstuch motif on a lonely accordion
    const mel = 'D4:2 F4:2 A4:4 G4:1 F4:1 E4:2 D4:4 | A3:2 D4:2 F4:4 E4:2 C4:2 D4:4';
    const ev = [...seq(mel, 'accordionSolo', 0.35).events];
    const pads = ['D3+A3+F4', 'A#2+F3+D4', 'C3+G3+E4', 'D3+A3+F4'];
    pads.forEach((c, i) => ev.push(...seq(`${c}:8`, 'pad', 0.12, i * 8).events));
    return { name: 'atem', unit: 60 / 62 / 2, len: 32, events: ev, gain: 0.55 };
}

function songTitel() { // title: the Halstuch motif on a solo fiddle over the sea
    const mel = 'D5:3 F5:3 A5:6 G5:2 F5:1 E5:3 D5:3 R:6 | A4:3 D5:3 F5:6 E5:3 C5:3 D5:6 R:6';
    const ev = [...seq(mel, 'fiddle', 0.4).events];
    ['D3+A3', 'D3+A3', 'A#2+F3', 'C3+G3', 'D3+A3', 'D3+A3'].forEach((c, i) => ev.push(...seq(`${c}:12`, 'pad', 0.14, i * 12).events));
    return { name: 'titel', unit: 60 / 58 / 3, len: 72, events: ev, gain: 0.55 };
}

export class Audio {
    constructor() {
        this.ctx = null;
        this.mood = 'silence';
        this.song = null;
        this.crewBoost = 0;
        this.songs = { salz: songSalz(), leine: songLeine(), atem: songAtem(), titel: songTitel() };
        this.muted = false;
    }
    unlock() {
        if (this.ctx) { if (this.ctx.state === 'suspended') this.ctx.resume(); return; }
        const C = window.AudioContext || window.webkitAudioContext;
        if (!C) return;
        const ctx = this.ctx = new C();
        this.master = ctx.createGain(); this.master.gain.value = 0.8;
        const comp = ctx.createDynamicsCompressor();
        comp.threshold.value = -16; comp.ratio.value = 4;
        this.master.connect(comp).connect(ctx.destination);
        // gentle "64-bit" sampler colour: soft quantization curve on the music bus
        const crush = ctx.createWaveShaper();
        const curve = new Float32Array(1024);
        for (let i = 0; i < 1024; i++) { const x = i / 511.5 - 1; curve[i] = Math.round(x * 48) / 48 * 0.6 + x * 0.4; }
        crush.curve = curve;
        this.reverb = ctx.createConvolver();
        const len = ctx.sampleRate * 2.6, ir = ctx.createBuffer(2, len, ctx.sampleRate);
        for (let ch = 0; ch < 2; ch++) { const d = ir.getChannelData(ch); for (let i = 0; i < len; i++) d[i] = (Math.random() * 2 - 1) * Math.pow(1 - i / len, 2.6); }
        this.reverb.buffer = ir;
        const wet = ctx.createGain(); wet.gain.value = 0.32;
        this.reverb.connect(wet).connect(this.master);
        this.music = ctx.createGain(); this.music.gain.value = 0;
        this.music.connect(crush).connect(this.master);
        crush.connect(this.reverb);
        this.crew = ctx.createGain(); this.crew.gain.value = 0;
        this.crew.connect(this.master); this.crew.connect(this.reverb);
        this.sfxBus = ctx.createGain(); this.sfxBus.gain.value = 0.7;
        this.sfxBus.connect(this.master); this.sfxBus.connect(this.reverb);
        this.amb = ctx.createGain(); this.amb.gain.value = 0;
        this.amb.connect(this.master);
        this.noise = ctx.createBuffer(1, ctx.sampleRate * 2, ctx.sampleRate);
        const nd = this.noise.getChannelData(0);
        for (let i = 0; i < nd.length; i++) nd[i] = Math.random() * 2 - 1;
        this.startAmbience();
        this.nextTime = ctx.currentTime + 0.1;
        this.pos = 0;
        setInterval(() => this.schedule(), 40);
        this.applyMood(true);
    }
    setMuted(m) { this.muted = m; if (this.master) this.master.gain.value = m ? 0 : 0.8; }

    startAmbience() {
        const ctx = this.ctx;
        const mk = (type, f, q) => { const s = ctx.createBufferSource(); s.buffer = this.noise; s.loop = true; const fl = ctx.createBiquadFilter(); fl.type = type; fl.frequency.value = f; fl.Q.value = q; const g = ctx.createGain(); s.connect(fl).connect(g); s.start(); return { s, fl, g }; };
        this.waves = mk('lowpass', 520, 0.4);
        this.waves.g.connect(this.amb);
        const lfo = ctx.createOscillator(); lfo.frequency.value = 0.11;
        const lg = ctx.createGain(); lg.gain.value = 0.35;
        lfo.connect(lg).connect(this.waves.g.gain); lfo.start();
        this.waves.g.gain.value = 0.55;
        this.wind = mk('bandpass', 700, 0.8);
        this.wind.g.gain.value = 0;
        this.wind.g.connect(this.amb);
        this.drone = ctx.createOscillator(); this.drone.type = 'sawtooth'; this.drone.frequency.value = 41.2;
        const dl = ctx.createBiquadFilter(); dl.type = 'lowpass'; dl.frequency.value = 120;
        this.droneG = ctx.createGain(); this.droneG.gain.value = 0;
        this.drone.connect(dl).connect(this.droneG).connect(this.master); this.drone.start();
    }

    setMood(m) {
        if (m === this.mood) return;
        this.mood = m;
        if (this.ctx) this.applyMood();
    }
    applyMood(first = false) {
        const ctx = this.ctx, now = ctx.currentTime;
        const target = { land: 'salz', sea: 'leine', fire: 'atem', title: 'titel', death: null, silence: null }[this.mood];
        const fade = (param, v, t = 1.5) => { param.cancelScheduledValues(now); param.setValueAtTime(param.value, now); param.linearRampToValueAtTime(v, now + t); };
        fade(this.music.gain, 0, first ? 0.01 : 1.0);
        fade(this.droneG.gain, this.mood === 'death' ? 0.25 : 0, 1.5);
        fade(this.amb.gain, this.mood === 'death' ? 0.1 : (this.mood === 'sea' ? 0.55 : 0.32), 2);
        fade(this.wind.g.gain, this.mood === 'sea' ? 0.35 : 0.05, 2);
        fade(this.crew.gain, this.mood === 'sea' ? 0.16 : 0, 2);
        clearTimeout(this.swapT);
        this.swapT = setTimeout(() => {
            this.song = target ? this.songs[target] : null;
            this.pos = 0; this.nextTime = this.ctx.currentTime + 0.05;
            if (this.song) { const n = this.ctx.currentTime; this.music.gain.cancelScheduledValues(n); this.music.gain.setValueAtTime(0, n); this.music.gain.linearRampToValueAtTime(this.song.gain * (this.mood === 'land' ? 0.55 : 1), n + 2.5); }
        }, first ? 10 : 1050);
    }

    crewSing() {
        if (!this.ctx) return;
        const now = this.ctx.currentTime;
        this.crew.gain.cancelScheduledValues(now);
        this.crew.gain.setValueAtTime(this.crew.gain.value, now);
        this.crew.gain.linearRampToValueAtTime(0.42, now + 1);
        this.crew.gain.linearRampToValueAtTime(0.42, now + 40);
        this.crew.gain.linearRampToValueAtTime(this.mood === 'sea' ? 0.16 : 0, now + 46);
    }

    schedule() {
        if (!this.ctx || !this.song) return;
        const song = this.song;
        const ahead = this.ctx.currentTime + 0.25;
        while (this.nextTime < ahead) {
            const unitIdx = this.pos % song.len;
            for (const e of song.events) if (e.t === unitIdx) this.play(e, this.nextTime, e.dur * song.unit);
            this.pos++;
            this.nextTime += song.unit;
        }
    }

    play(e, t, dur) {
        const ctx = this.ctx;
        const out = (e.inst === 'crewLead' || e.inst === 'crewHum') ? this.crew : this.music;
        const env = (g, a, peak, d, rel) => { g.gain.setValueAtTime(0.0001, t); g.gain.exponentialRampToValueAtTime(peak, t + a); g.gain.setValueAtTime(peak, t + Math.max(a, d - rel)); g.gain.exponentialRampToValueAtTime(0.0001, t + d); };
        const osc = (type, f, detune = 0) => { const o = ctx.createOscillator(); o.type = type; o.frequency.value = f; o.detune.value = detune; return o; };
        const v = e.vel;
        switch (e.inst) {
            case 'whistle': {
                const o = osc('triangle', e.f), o2 = osc('sine', e.f * 2);
                const vib = osc('sine', 5.8), vg = ctx.createGain(); vg.gain.setValueAtTime(0, t); vg.gain.linearRampToValueAtTime(e.f * 0.006, t + 0.25);
                vib.connect(vg).connect(o.frequency);
                const g = ctx.createGain(), g2 = ctx.createGain(); g2.gain.value = 0.15;
                o.connect(g); o2.connect(g2).connect(g); g.connect(out);
                env(g, 0.025, 0.22 * v, dur * 0.95, 0.06);
                [o, o2, vib].forEach(x => { x.start(t); x.stop(t + dur + 0.05); });
                break;
            }
            case 'fiddle': {
                const o = osc('sawtooth', e.f), bp = ctx.createBiquadFilter(); bp.type = 'lowpass'; bp.frequency.value = 2400; bp.Q.value = 2;
                const vib = osc('sine', 5.3), vg = ctx.createGain(); vg.gain.setValueAtTime(0, t); vg.gain.linearRampToValueAtTime(e.f * 0.008, t + 0.3);
                vib.connect(vg).connect(o.frequency);
                const g = ctx.createGain(); o.connect(bp).connect(g).connect(out);
                env(g, 0.07, 0.11 * v, dur * 0.97, 0.12);
                [o, vib].forEach(x => { x.start(t); x.stop(t + dur + 0.05); });
                break;
            }
            case 'accordion': case 'accordionSolo': {
                const solo = e.inst === 'accordionSolo';
                const g = ctx.createGain(), lp = ctx.createBiquadFilter(); lp.type = 'lowpass'; lp.frequency.value = solo ? 1700 : 1300;
                const trem = osc('sine', 5.2), tg = ctx.createGain(); tg.gain.value = 0.25;
                const mix = ctx.createGain(); mix.gain.value = 1;
                trem.connect(tg).connect(mix.gain);
                for (const d of [-7, 7]) { const o = osc('sawtooth', e.f, d); o.connect(lp); o.start(t); o.stop(t + dur + 0.05); }
                const sq = osc('square', e.f / 2); const sg = ctx.createGain(); sg.gain.value = 0.25; sq.connect(sg).connect(lp); sq.start(t); sq.stop(t + dur + 0.05);
                lp.connect(mix).connect(g).connect(out);
                env(g, solo ? 0.08 : 0.04, 0.07 * v, dur * 0.98, 0.1);
                trem.start(t); trem.stop(t + dur + 0.05);
                break;
            }
            case 'pad': {
                const g = ctx.createGain(), lp = ctx.createBiquadFilter(); lp.type = 'lowpass'; lp.frequency.value = 700;
                for (const d of [-9, 0, 9]) { const o = osc('sawtooth', e.f, d); o.connect(lp); o.start(t); o.stop(t + dur + 0.1); }
                lp.connect(g).connect(out);
                env(g, Math.min(1.2, dur * 0.3), 0.05 * v, dur, Math.min(1.2, dur * 0.4));
                break;
            }
            case 'bass': {
                const o = osc('triangle', e.f), s = osc('sine', e.f / 2), g = ctx.createGain();
                o.connect(g); s.connect(g); g.connect(out);
                g.gain.setValueAtTime(0.0001, t); g.gain.exponentialRampToValueAtTime(0.3 * v, t + 0.01); g.gain.exponentialRampToValueAtTime(0.0001, t + Math.min(dur, 0.6));
                [o, s].forEach(x => { x.start(t); x.stop(t + dur + 0.05); });
                break;
            }
            case 'drum': {
                const o = osc('sine', e.f * 1.6), g = ctx.createGain();
                o.frequency.setValueAtTime(e.f * 1.8, t); o.frequency.exponentialRampToValueAtTime(e.f * 0.7, t + 0.18);
                o.connect(g).connect(out);
                g.gain.setValueAtTime(0.5 * v, t); g.gain.exponentialRampToValueAtTime(0.0001, t + 0.3);
                o.start(t); o.stop(t + 0.35);
                this.noiseHit(t, 300, 0.06, 0.12 * v, out, 'lowpass');
                break;
            }
            case 'tick': this.noiseHit(t, 2200, 0.03, 0.18 * v, out, 'bandpass'); break;
            case 'crewLead': case 'crewHum': {
                // a few rough male voices singing "oh" through formant filters
                const hum = e.inst === 'crewHum';
                const g = ctx.createGain();
                const f1 = ctx.createBiquadFilter(); f1.type = 'bandpass'; f1.frequency.value = hum ? 420 : 560; f1.Q.value = 6;
                const f2 = ctx.createBiquadFilter(); f2.type = 'bandpass'; f2.frequency.value = hum ? 780 : 900; f2.Q.value = 8;
                const fg2 = ctx.createGain(); fg2.gain.value = 0.6;
                const voices = hum ? [-12, 3] : [-14, -4, 6, 13];
                for (const d of voices) {
                    const o = osc('sawtooth', hum ? e.f : e.f / 2, d + (Math.random() - 0.5) * 8);
                    const vib = osc('sine', 4.5 + Math.random()), vg = ctx.createGain(); vg.gain.value = e.f * 0.006;
                    vib.connect(vg).connect(o.frequency);
                    o.connect(f1); o.connect(f2);
                    o.start(t); o.stop(t + dur + 0.2); vib.start(t); vib.stop(t + dur + 0.2);
                }
                f1.connect(g); f2.connect(fg2).connect(g); g.connect(out);
                env(g, hum ? 0.4 : 0.08, (hum ? 0.18 : 0.5) * v, dur + 0.1, hum ? 0.5 : 0.15);
                break;
            }
        }
    }

    noiseHit(t, f, dur, vol, out, type = 'bandpass', q = 1) {
        const ctx = this.ctx;
        const s = ctx.createBufferSource(); s.buffer = this.noise;
        const fl = ctx.createBiquadFilter(); fl.type = type; fl.frequency.value = f; fl.Q.value = q;
        const g = ctx.createGain();
        s.connect(fl).connect(g).connect(out || this.sfxBus);
        g.gain.setValueAtTime(vol, t); g.gain.exponentialRampToValueAtTime(0.0001, t + dur);
        s.start(t, Math.random()); s.stop(t + dur + 0.02);
        return fl;
    }
    tone(t, type, f0, f1, dur, vol) {
        const ctx = this.ctx, o = ctx.createOscillator(), g = ctx.createGain();
        o.type = type; o.frequency.setValueAtTime(f0, t); o.frequency.exponentialRampToValueAtTime(Math.max(1, f1), t + dur);
        o.connect(g).connect(this.sfxBus);
        g.gain.setValueAtTime(vol, t); g.gain.exponentialRampToValueAtTime(0.0001, t + dur);
        o.start(t); o.stop(t + dur + 0.02);
    }

    sfx(name) {
        if (!this.ctx) return;
        const t = this.ctx.currentTime;
        switch (name) {
            case 'swing': { const f = this.noiseHit(t, 900, 0.16, 0.35, null, 'bandpass', 1.5); f.frequency.exponentialRampToValueAtTime(3200, t + 0.14); break; }
            case 'heavy': { const f = this.noiseHit(t, 400, 0.3, 0.5, null, 'bandpass', 1.2); f.frequency.exponentialRampToValueAtTime(1800, t + 0.28); break; }
            case 'hit': this.noiseHit(t, 1400, 0.08, 0.5); this.tone(t, 'square', 220, 90, 0.09, 0.2); break;
            case 'crit': this.noiseHit(t, 900, 0.25, 0.7); this.tone(t, 'sawtooth', 160, 40, 0.3, 0.35); break;
            case 'hurt': this.noiseHit(t, 500, 0.2, 0.6, null, 'lowpass'); this.tone(t, 'square', 140, 60, 0.18, 0.25); break;
            case 'roll': { const f = this.noiseHit(t, 300, 0.3, 0.3, null, 'lowpass'); f.frequency.linearRampToValueAtTime(900, t + 0.2); break; }
            case 'parry': this.tone(t, 'triangle', 1800, 1500, 0.05, 0.12); break;
            case 'parried': this.tone(t, 'square', 2400, 2300, 0.25, 0.25); this.tone(t, 'triangle', 3100, 3000, 0.4, 0.15); this.noiseHit(t, 3000, 0.1, 0.4); break;
            case 'drink': [0, 0.12, 0.24, 0.36].forEach(d => this.tone(t + d, 'sine', 320 + Math.random() * 60, 200, 0.08, 0.15)); break;
            case 'heal': [0, 0.08, 0.16].forEach((d, i) => this.tone(t + d, 'triangle', 660 * (1 + i * 0.25), 700 * (1 + i * 0.25), 0.4, 0.12)); break;
            case 'pinch': this.noiseHit(t, 2500, 0.05, 0.4); this.noiseHit(t + 0.06, 2800, 0.05, 0.35); break;
            case 'bones': for (let k = 0; k < 6; k++) this.noiseHit(t + k * 0.05 + Math.random() * 0.03, 1500 + Math.random() * 1500, 0.04, 0.35, null, 'bandpass', 4); break;
            case 'crush': this.noiseHit(t, 600, 0.3, 0.8, null, 'lowpass'); for (let k = 0; k < 5; k++) this.noiseHit(t + k * 0.03, 2000, 0.05, 0.4, null, 'bandpass', 3); break;
            case 'cannon': this.tone(t, 'sine', 90, 30, 0.6, 0.9); this.noiseHit(t, 500, 0.7, 0.9, null, 'lowpass'); break;
            case 'pistol': this.tone(t, 'square', 200, 40, 0.15, 0.4); this.noiseHit(t, 1500, 0.25, 0.8, null, 'lowpass'); break;
            case 'tide': { const f = this.noiseHit(t, 300, 0.6, 0.5, null, 'bandpass', 2); f.frequency.exponentialRampToValueAtTime(2000, t + 0.5); this.tone(t, 'sine', 440, 880, 0.5, 0.15); break; }
            case 'crash': this.noiseHit(t, 250, 0.5, 0.9, null, 'lowpass'); this.tone(t, 'sawtooth', 80, 40, 0.4, 0.3); break;
            case 'roar': this.tone(t, 'sawtooth', 70, 45, 1.2, 0.35); this.noiseHit(t, 200, 1.0, 0.5, null, 'lowpass'); break;
            case 'death': this.tone(t, 'sine', 110, 27, 3, 0.6); this.noiseHit(t, 150, 2.5, 0.6, null, 'lowpass'); break;
            case 'pearls': [0, 0.07, 0.14, 0.21, 0.28].forEach((d, i) => this.tone(t + d, 'triangle', 880 + i * 220, 900 + i * 220, 0.3, 0.12)); break;
            case 'kindle': this.noiseHit(t, 800, 1.2, 0.5, null, 'bandpass', 0.7); this.tone(t, 'sine', 196, 196, 1.6, 0.2); this.tone(t + 0.1, 'sine', 293.7, 293.7, 1.6, 0.15); break;
            case 'chest': this.tone(t, 'square', 130, 90, 0.2, 0.2); [0.2, 0.3, 0.4].forEach((d, i) => this.tone(t + d, 'triangle', 523 * (1 + i * 0.26), 530 * (1 + i * 0.26), 0.35, 0.14)); break;
            case 'level': [0, 0.1, 0.2].forEach((d, i) => this.tone(t + d, 'triangle', 392 * Math.pow(1.26, i), 392 * Math.pow(1.26, i), 0.6, 0.15)); break;
            case 'lock': this.tone(t, 'square', 1200, 1200, 0.04, 0.08); break;
            case 'rise': this.tone(t, 'sine', 48, 70, 2.2, 0.7); this.noiseHit(t, 180, 2.4, 0.7, null, 'lowpass'); for (let k = 0; k < 9; k++) this.tone(t + 0.4 + k * 0.17 + Math.random() * 0.1, 'sine', 300 + Math.random() * 500, 900 + Math.random() * 600, 0.07, 0.08); this.noiseHit(t + 1.4, 900, 0.9, 0.5, null, 'bandpass', 0.8); break;
            case 'zap': for (let k = 0; k < 6; k++) this.noiseHit(t + k * 0.035, 3000 + Math.random() * 3000, 0.06, 0.45, null, 'bandpass', 2); this.tone(t, 'sawtooth', 1400, 120, 0.35, 0.22); this.noiseHit(t + 0.05, 400, 0.6, 0.6, null, 'lowpass'); break;
            case 'charge': this.tone(t, 'sine', 300, 900, 0.22, 0.12); this.noiseHit(t, 2500, 0.2, 0.2, null, 'bandpass', 3); break;
            case 'board': this.noiseHit(t, 400, 0.4, 0.4, null, 'lowpass'); this.tone(t, 'triangle', 180, 120, 0.3, 0.1); break;
            case 'click': this.tone(t, 'square', 900, 900, 0.03, 0.06); break;
        }
    }

    update(dt, game) {
        if (!this.ctx) return;
        // fire crackle near fires
        this.crackleT = (this.crackleT || 0) - dt;
        if (this.crackleT <= 0) {
            this.crackleT = 0.05 + Math.random() * 0.2;
            let d = 99;
            for (const f of game.map.fires) d = Math.min(d, Math.hypot(game.player.x - f.x, game.player.z - f.z));
            if (d < 8) this.noiseHit(this.ctx.currentTime, 1500 + Math.random() * 2500, 0.03, 0.25 * (1 - d / 8), null, 'bandpass', 3);
        }
    }
}
