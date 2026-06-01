# Portrait Asset Spec — 角色创建立绘

## Where they land

Drop 5 PNG files in this directory, exact filenames:

```
public/assets/portraits/
├── ashen.png
├── wanderer.png
├── miner.png
├── noble.png
└── dwarf.png
```

The React code in `src/app/App.tsx` (`CharacterCreator`) already loads them via
`assets/portraits/${value}.png`. Missing files are hidden gracefully on error
so dev doesn't crash while art is in flight.

## Hard specs

| | Spec |
|---|---|
| Resolution | **48×48 px** preferred. 32×32 also accepted (CSS will upscale with `image-rendering: pixelated`). |
| Format | PNG, 8-bit indexed or 32-bit RGBA. **No alpha background** — paint Mine Black `#080a0b` as the solid plate. |
| Anti-aliasing | **None.** Hard 1px outlines, no half-pixels. |
| Palette | Limited to 10–12 colors max. Share across all 5 files for visual cohesion. |
| Outline | 1px solid `#080a0b` around silhouette. |
| Shading | 2–3 hard color steps. No gradients, no dithering for skin/cloth. Dither allowed only for metal speculars. |
| Light source | Upper-left. Highlight on left brow/shoulder, shadow on right jaw/torso. |
| Composition | Bust framing (head + upper chest). Head is **~½ to ⅗** of total height — chibi / dwarven proportions. |
| Pose | Slightly 3/4 angle facing left. Same angle across all 5 for grid harmony. |

## Master palette (use only these)

```
#080a0b   Mine Black           — background, outline
#1c1a16   Cloth dark
#3a3128   Cloth mid
#5a4938   Cloth light
#8c6530   Lantern Wick / leather
#d9b662   Miner's Lantern Gold — accents ≤5% of the tile, never as fill
#b79d86   Skin mid
#e0d0bd   Skin highlight
#404852   Cool shadow         — ashen-only
#5a3b3c   Wound / rust red    — corrupted moments
#8b5a9b   Khah whisper violet — never on the base portrait set, save for tainted overlay
#aaa08f   Tunnel mist grey    — hair, beard mid
```

All values come from `DESIGN.md` (in-world named tokens). Token consistency is
non-optional — the 5 PNGs must be co-paintable from one Aseprite palette swatch.

## 5 character briefs

Each portrait must share the palette and proportion above; only the distinguishing
elements below vary.

### 1. `ashen.png` — 灰烬旅人 (the player default)

**Concept**: The amnesiac protagonist. Pale, gaunt, eyes hollow, no clear class.
This is who you are when you wake up in the mine.

- **Headwear**: None. White-grey short hair, wind-tossed left.
- **Face**: High cheekbones. Eyes are two black voids (1px each), no iris detail.
- **Body**: Torn collar, frayed cloth around neck.
- **Distinguishing color**: Cool shadow `#404852` dominates the cloth; skin
  pushed toward `#e0d0bd` (palest of the 5).
- **Mood**: "I don't remember who I was."

### 2. `wanderer.png` — 破斗篷

**Concept**: A rogue / hedge-trader who's been on the road too long. Hood low.

- **Headwear**: Deep brown hood (`#3a3128` → `#1c1a16`) covering top half of face.
  Hood lined with a single 1px `#d9b662` gold stitch along the hem.
- **Face**: Only stubbled jaw and lower lip visible. Upper face is hood shadow.
- **Body**: Cloak collar.
- **Distinguishing color**: Cloth dark family + the gold thread accent.
- **Mood**: "I'm not telling you my real name."

### 3. `miner.png` — 矿区幸存者 (the dwarf-styled human miner)

**Concept**: Directly transposed from the reference sample. A human miner who
survived the cave-in but bears its marks.

- **Headwear**: Black miner cap, pointed back. A single `#d9b662` lantern pin
  on the brim (3×3 px max).
- **Face**: Thick curly brown beard (`#5a4938` → `#8c6530`) reaching mid-chest.
  Soot smudge under one eye (`#080a0b` ×2 px).
- **Body**: Leather coverall.
- **Distinguishing color**: Lantern Wick `#8c6530` is the dominant cloth tone.
- **Mood**: "I came down with seventy-six others."
- **Note**: This is the canon visual the user provided.

### 4. `noble.png` — 失落贵族

**Concept**: A scion of a deposed house, traveling in disguise but unable to
fully hide the bearing.

- **Headwear**: None. Dark hair slicked straight back.
- **Face**: Sharp cheekbones, distant gaze (eyes look 5px to the right of center).
- **Body**: High-collared dark robe (`#1c1a16`). Single `#d9b662` gold ear-stud
  on the visible (right) ear, 2×2 px.
- **Distinguishing color**: Coldest of the warm-tones; muted purple-black hint
  in robe (`#1c1a20` — slight violet shift from generic `#1c1a16`).
- **Mood**: "My ancestors did this to you."

### 5. `dwarf.png` — 矿镇血脉 (new)

**Concept**: Dwarven descendant of the 77-expedition. Same world-language as
`miner` but shorter, hairier, redder-cheeked.

- **Headwear**: Wider miner cap than `miner.png`, slumped to one side.
- **Face**: Massive curly beard `#8c6530` → `#5a4938` covering the entire chest
  and obscuring the mouth completely. Two ruddy `#5a3b3c` cheek patches.
- **Body**: Barely visible under the beard.
- **Proportions**: Head closer to ⅗ of bust (squatter than the other 4).
- **Distinguishing color**: Most red of the 5 (`#5a3b3c` cheeks). Lantern Wick
  for the beard mid-tone.
- **Mood**: "We dug too deep. I won't say sorry."

## Generation prompts (Midjourney v6 / Stable Diffusion)

Base prompt — append the variant clause:

```
pixel art portrait, 48x48 resolution, SNES era, Final Fantasy IV character bust,
[VARIANT_DESCRIPTION], solid black background #080a0b, hard 1px black outline,
2-tone shading, no antialiasing, no gradients, no soft glow, limited 10-color
palette dominated by dark cool tones and one warm gold accent, dwarven
proportions with head approximately half of body height, slightly 3/4 angle
facing left, lit from upper left, mood: dark cthulhu mining post-apocalypse,
--style raw --ar 1:1 --v 6
```

| Variant | `[VARIANT_DESCRIPTION]` |
|---|---|
| ashen | `pale gaunt human, white-grey wind-tossed short hair, empty hollow eye sockets, torn cloth wrapped at neck, no headwear, cool blue shadow` |
| wanderer | `hooded rogue, deep brown hood with thin gold stitching along the hem, hood shadow covering upper half of face, only stubbled jaw visible` |
| miner | `human miner, pointed black miner cap with a single tiny glowing gold lantern pin on the brim, thick curly brown beard covering chest, soot stained under one eye` |
| noble | `fallen royal in disguise, high-collared dark robe with slight violet hint, dark hair slicked straight back, single gold ear-stud, sharp cheekbones, distant gaze` |
| dwarf | `stocky dwarven villager, wider lopsided miner cap, massive brown beard reaching mid-chest covering mouth, weathered red cheeks, head three-fifths of body height` |

Midjourney note: it may resist the 48×48 constraint. Generate at the largest
aspect (1024×1024), then in Aseprite scale down with **nearest neighbor + manual
cleanup** to enforce the pixel grid. Don't ship MJ output raw.

## Aseprite path (recommended)

1. Open the reference sample (the dwarf the user provided) → `Palette → Get Colors`.
2. `File → New 48×48 → RGB → Background Color: #080a0b`.
3. Paint variant per brief above. Re-use silhouette pieces across portraits where
   sensible (the cap shape on miner/dwarf, the eye placement on all 5).
4. Export each as `<value>.png` (8-bit indexed PNG is preferred, smaller and
   forces the palette discipline).
5. Drop into this directory and reload the app. No code changes needed.

## Anti-patterns (do not ship)

- Smooth edges or anti-aliased outlines → re-quantize.
- Transparent background → fill with `#080a0b`.
- A unique color per portrait → re-do from the master palette.
- Bright saturated highlights → check chroma against the master palette in OKLCH.
- Head ¼ of body or shorter → too "anime", not RPG portrait.
- Modern face proportions (eyes 1/3 down from top) → use chibi: eyes at midline.
