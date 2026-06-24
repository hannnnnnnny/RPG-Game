// Procedural SFX generator for 《潮蚀之环》.
// Writes small 22050Hz mono 16-bit WAV files into godot/assets/audio/.
// Run: node tools/gen_sfx.mjs
// Synthesis is deliberately simple (noise/sine + envelopes) so the SFX are
// tiny, license-free, and match the lo-fi pixel mood.

import { writeFileSync, mkdirSync } from "node:fs";
import { dirname } from "node:path";

const SR = 22050;
const OUT = "godot/assets/audio";
mkdirSync(OUT, { recursive: true });

function writeWav(name, samples) {
  const n = samples.length;
  const buf = Buffer.alloc(44 + n * 2);
  // RIFF header
  buf.write("RIFF", 0);
  buf.writeUInt32LE(36 + n * 2, 4);
  buf.write("WAVE", 8);
  buf.write("fmt ", 12);
  buf.writeUInt32LE(16, 16);
  buf.writeUInt16LE(1, 20); // PCM
  buf.writeUInt16LE(1, 22); // mono
  buf.writeUInt32LE(SR, 24);
  buf.writeUInt32LE(SR * 2, 28);
  buf.writeUInt16LE(2, 32);
  buf.writeUInt16LE(16, 34);
  buf.write("data", 36);
  buf.writeUInt32LE(n * 2, 40);
  for (let i = 0; i < n; i++) {
    let v = Math.max(-1, Math.min(1, samples[i]));
    buf.writeInt16LE((v * 32767) | 0, 44 + i * 2);
  }
  writeFileSync(`${OUT}/${name}.wav`, buf);
  console.log(`wrote ${OUT}/${name}.wav (${(n / SR).toFixed(2)}s)`);
}

const rnd = () => Math.random() * 2 - 1;
function lowpass(arr, a) {
  let prev = 0;
  for (let i = 0; i < arr.length; i++) {
    prev = prev + a * (arr[i] - prev);
    arr[i] = prev;
  }
  return arr;
}

// --- swing: soft filtered-noise whoosh ---
function swing() {
  const dur = 0.16;
  const n = (SR * dur) | 0;
  const s = new Float32Array(n);
  for (let i = 0; i < n; i++) {
    const t = i / n;
    const env = Math.pow(1 - t, 2) * Math.min(1, t * 8); // fast in, decay out
    s[i] = rnd() * env;
  }
  lowpass(s, 0.18);
  for (let i = 0; i < n; i++) s[i] *= 0.55;
  return s;
}

// --- hit: low thud with a noise transient ---
function hit() {
  const dur = 0.18;
  const n = (SR * dur) | 0;
  const s = new Float32Array(n);
  for (let i = 0; i < n; i++) {
    const t = i / n;
    const env = Math.pow(1 - t, 3);
    const freq = 150 - 60 * t; // pitch drops
    const tone = Math.sin((2 * Math.PI * freq * i) / SR);
    const click = i < SR * 0.02 ? rnd() * (1 - i / (SR * 0.02)) : 0;
    s[i] = (tone * 0.7 + click * 0.6) * env;
  }
  return s;
}

// --- pickup: two ascending blips ---
function pickup() {
  const dur = 0.2;
  const n = (SR * dur) | 0;
  const s = new Float32Array(n);
  for (let i = 0; i < n; i++) {
    const t = i / n;
    const freq = t < 0.5 ? 620 : 880;
    const local = t < 0.5 ? t / 0.5 : (t - 0.5) / 0.5;
    const env = Math.pow(1 - local, 2) * Math.min(1, local * 10);
    s[i] = Math.sin((2 * Math.PI * freq * i) / SR) * env * 0.45;
  }
  return s;
}

// --- step: very short low tick ---
function step() {
  const dur = 0.07;
  const n = (SR * dur) | 0;
  const s = new Float32Array(n);
  for (let i = 0; i < n; i++) {
    const t = i / n;
    const env = Math.pow(1 - t, 4);
    s[i] = (rnd() * 0.5 + Math.sin((2 * Math.PI * 90 * i) / SR) * 0.5) * env * 0.35;
  }
  lowpass(s, 0.3);
  return s;
}

// --- ambient: low seamless drone loop (~2.4s) ---
function ambient() {
  const dur = 2.4;
  const n = (SR * dur) | 0;
  const s = new Float32Array(n);
  for (let i = 0; i < n; i++) {
    const trem = 0.7 + 0.3 * Math.sin((2 * Math.PI * 0.25 * i) / SR);
    const a = Math.sin((2 * Math.PI * 55 * i) / SR);
    const b = Math.sin((2 * Math.PI * 82.4 * i) / SR);
    const c = Math.sin((2 * Math.PI * 36.7 * i) / SR);
    s[i] = (a * 0.5 + b * 0.3 + c * 0.4) * trem * 0.16;
  }
  // crossfade ends so the loop is seamless
  const fade = (SR * 0.15) | 0;
  for (let i = 0; i < fade; i++) {
    const k = i / fade;
    s[i] *= k;
    s[n - 1 - i] *= k;
  }
  lowpass(s, 0.5);
  return s;
}

writeWav("swing", swing());
writeWav("hit", hit());
writeWav("pickup", pickup());
writeWav("step", step());
writeWav("ambient", ambient());
console.log("done");
