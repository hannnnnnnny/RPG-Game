import Phaser from "phaser";
import { generateLoot, goldForKill } from "../../core/lootGenerator";
import { useGameStore } from "../../store/useGameStore";

const WORLD_W = 1450;
const WORLD_H = 900;
const SRC_TILE = 16;
const TILE_SCALE = 3;
const DISP_TILE = SRC_TILE * TILE_SCALE; // 48px on screen

const COLS = Math.ceil(WORLD_W / DISP_TILE);
const ROWS = Math.ceil(WORLD_H / DISP_TILE);

type TileType = "wall" | "floor" | "path" | "puddle" | "rubble";

type EnemyActor = {
  body: Phaser.Physics.Arcade.Sprite;
  hp: number;
  attackCooldown: number;
};

const ROOMS = [
  { x: 60, y: 80, w: 350, h: 180 },
  { x: 360, y: 185, w: 520, h: 170 },
  { x: 760, y: 310, w: 350, h: 210 },
  { x: 1040, y: 470, w: 240, h: 260 },
  { x: 420, y: 540, w: 520, h: 180 }
];

const PUDDLES: Array<{ cx: number; cy: number; rx: number; ry: number }> = [
  { cx: 685, cy: 260, rx: 95, ry: 35 },
  { cx: 980, cy: 445, rx: 110, ry: 45 },
  { cx: 555, cy: 640, rx: 85, ry: 29 }
];

export class MineIntroScene extends Phaser.Scene {
  private player!: Phaser.Physics.Arcade.Sprite;
  private playerFacing: "down" | "up" | "left" | "right" = "down";
  private cursors!: Phaser.Types.Input.Keyboard.CursorKeys;
  private keys!: Record<string, Phaser.Input.Keyboard.Key>;
  private enemies: EnemyActor[] = [];
  private stamina = 100;
  private health = 100;
  private focus = 30;
  private attacking = false;
  private rollUntil = 0;
  private attackCooldown = 0;
  private dwarfZone!: Phaser.GameObjects.Zone;
  private totemZone!: Phaser.GameObjects.Zone;
  private exitZone!: Phaser.GameObjects.Zone;
  private objectiveText!: Phaser.GameObjects.Text;
  private lastHudSync = 0;
  private totemTouchedInScene = false;
  private walls!: Phaser.Physics.Arcade.StaticGroup;
  private wallGrid: boolean[][] = [];

  constructor() {
    super("MineIntroScene");
  }

  create() {
    this.physics.world.setBounds(0, 0, WORLD_W, WORLD_H);
    this.cameras.main.setBounds(0, 0, WORLD_W, WORLD_H);

    this.buildWorldTexture();
    this.buildPlayerSheet();
    this.buildCorruptedDwarfSheet();
    this.buildInjuredDwarfSprite();
    this.buildTotemSprite();
    this.buildExitSprite();
    this.buildSlashSprite();
    this.createAnimations();

    this.add.image(0, 0, "world_map").setOrigin(0, 0).setDisplaySize(WORLD_W, WORLD_H).setDepth(0);

    this.buildWallColliders();
    this.createControls();
    this.createPlayer();
    this.createEnemies();
    this.physics.add.collider(this.player, this.walls);
    this.enemies.forEach((enemy) => this.physics.add.collider(enemy.body, this.walls));
    this.createInteractables();
    this.createObjectiveText();

    this.cameras.main.startFollow(this.player, true, 0.12, 0.12);

    useGameStore.getState().setDialogue({
      speaker: "克哈低语",
      text: "往前。那些矿灯已经死了，但你的血还记得路。",
      tone: "whisper"
    });
  }

  update(time: number, delta: number) {
    this.updatePlayer(time, delta);
    this.updateEnemies(delta);
    this.handleInteraction();
    this.syncHud(time);
  }

  // ---------- Tile / world generation ----------

  private classifyTile(col: number, row: number): TileType {
    const cx = col * DISP_TILE + DISP_TILE / 2;
    const cy = row * DISP_TILE + DISP_TILE / 2;
    const inRoom = ROOMS.some((r) => cx >= r.x && cx <= r.x + r.w && cy >= r.y && cy <= r.y + r.h);
    if (!inRoom) return "wall";
    const inPuddle = PUDDLES.some((p) => {
      const dx = (cx - p.cx) / p.rx;
      const dy = (cy - p.cy) / p.ry;
      return dx * dx + dy * dy <= 1;
    });
    if (inPuddle) return "puddle";
    // Path connecting the rooms (simple diagonal corridor)
    if (this.isPathTile(cx, cy)) return "path";
    return "floor";
  }

  private isPathTile(cx: number, cy: number): boolean {
    const points: Array<[number, number]> = [
      [200, 170],
      [620, 270],
      [930, 415],
      [1160, 600],
      [680, 630]
    ];
    for (let i = 0; i < points.length - 1; i += 1) {
      const [x1, y1] = points[i];
      const [x2, y2] = points[i + 1];
      const t = Phaser.Math.Distance.Between(cx, cy, x1, y1) + Phaser.Math.Distance.Between(cx, cy, x2, y2);
      const segLen = Phaser.Math.Distance.Between(x1, y1, x2, y2);
      if (Math.abs(t - segLen) < 26) return true;
    }
    return false;
  }

  private buildWorldTexture() {
    const w = COLS * SRC_TILE;
    const h = ROWS * SRC_TILE;
    const tex = this.textures.createCanvas("world_map", w, h)!;
    const ctx = tex.getContext();
    ctx.imageSmoothingEnabled = false;

    this.wallGrid = [];
    for (let r = 0; r < ROWS; r += 1) {
      this.wallGrid[r] = [];
      for (let c = 0; c < COLS; c += 1) {
        const type = this.classifyTile(c, r);
        this.wallGrid[r][c] = type === "wall";
        this.drawTile(ctx, c * SRC_TILE, r * SRC_TILE, type, c, r);
      }
    }
    tex.refresh();
  }

  private buildWallColliders() {
    this.walls = this.physics.add.staticGroup();
    const used: boolean[][] = this.wallGrid.map((row) => row.map(() => false));
    // Greedy merge into horizontal strips per row to cut body count.
    for (let r = 0; r < ROWS; r += 1) {
      let c = 0;
      while (c < COLS) {
        if (!this.wallGrid[r][c] || used[r][c]) {
          c += 1;
          continue;
        }
        let runEnd = c;
        while (runEnd < COLS && this.wallGrid[r][runEnd] && !used[r][runEnd]) {
          used[r][runEnd] = true;
          runEnd += 1;
        }
        const width = (runEnd - c) * DISP_TILE;
        const x = c * DISP_TILE + width / 2;
        const y = r * DISP_TILE + DISP_TILE / 2;
        const block = this.add.rectangle(x, y, width, DISP_TILE, 0x000000, 0).setDepth(0);
        this.physics.add.existing(block, true);
        this.walls.add(block);
        c = runEnd;
      }
    }
  }

  private drawTile(ctx: CanvasRenderingContext2D, x: number, y: number, type: TileType, c: number, r: number) {
    const seed = (c * 73856093) ^ (r * 19349663);
    const rand = (n: number) => Math.abs(Math.sin(seed * (n + 1))) % 1;

    if (type === "wall") {
      ctx.fillStyle = "#1b1418";
      ctx.fillRect(x, y, SRC_TILE, SRC_TILE);
      ctx.fillStyle = "#241a24";
      ctx.fillRect(x, y, SRC_TILE, 2);
      ctx.fillRect(x, y, 2, SRC_TILE);
      ctx.fillStyle = "#0f0a10";
      ctx.fillRect(x, y + SRC_TILE - 2, SRC_TILE, 2);
      ctx.fillRect(x + SRC_TILE - 2, y, 2, SRC_TILE);
      // speckle
      ctx.fillStyle = "#3a2645";
      if (rand(1) < 0.18) ctx.fillRect(x + 4 + Math.floor(rand(2) * 8), y + 4 + Math.floor(rand(3) * 8), 1, 1);
      if (rand(4) < 0.1) ctx.fillRect(x + 3 + Math.floor(rand(5) * 9), y + 3 + Math.floor(rand(6) * 9), 2, 1);
      return;
    }

    if (type === "puddle") {
      ctx.fillStyle = "#2b1638";
      ctx.fillRect(x, y, SRC_TILE, SRC_TILE);
      ctx.fillStyle = "#3d1d4a";
      ctx.fillRect(x + 2, y + 2, 4, 1);
      ctx.fillRect(x + 8, y + 6, 5, 1);
      ctx.fillRect(x + 3, y + 11, 6, 1);
      ctx.fillStyle = "#6c39a6";
      ctx.fillRect(x + 5, y + 4, 2, 1);
      ctx.fillRect(x + 10, y + 9, 1, 1);
      return;
    }

    if (type === "path") {
      ctx.fillStyle = "#4a3a26";
      ctx.fillRect(x, y, SRC_TILE, SRC_TILE);
      ctx.fillStyle = "#5e4a30";
      for (let i = 0; i < 6; i += 1) {
        const px = x + Math.floor(rand(i + 1) * SRC_TILE);
        const py = y + Math.floor(rand(i + 7) * SRC_TILE);
        ctx.fillRect(px, py, 1, 1);
      }
      ctx.fillStyle = "#352716";
      ctx.fillRect(x + 4, y + 9, 2, 1);
      ctx.fillRect(x + 11, y + 3, 1, 1);
      return;
    }

    // floor (default dark cave floor with subtle variation)
    const variant = rand(2);
    const base = variant < 0.6 ? "#262024" : variant < 0.85 ? "#2c2228" : "#221c20";
    ctx.fillStyle = base;
    ctx.fillRect(x, y, SRC_TILE, SRC_TILE);
    ctx.fillStyle = "#1a1318";
    ctx.fillRect(x, y + SRC_TILE - 1, SRC_TILE, 1);
    ctx.fillRect(x + SRC_TILE - 1, y, 1, SRC_TILE);
    ctx.fillStyle = "#352a32";
    ctx.fillRect(x, y, SRC_TILE, 1);
    ctx.fillRect(x, y, 1, SRC_TILE);
    // pebbles
    if (rand(3) < 0.18) {
      ctx.fillStyle = "#4a3a44";
      ctx.fillRect(x + 3 + Math.floor(rand(4) * 9), y + 3 + Math.floor(rand(5) * 9), 1, 1);
    }
    if (rand(6) < 0.08) {
      ctx.fillStyle = "#7a5b85";
      ctx.fillRect(x + 5 + Math.floor(rand(7) * 6), y + 5 + Math.floor(rand(8) * 6), 1, 1);
    }
  }

  // ---------- Player sprite sheet ----------

  private fill(ctx: CanvasRenderingContext2D, x: number, y: number, w: number, h: number, color: string) {
    ctx.fillStyle = color;
    ctx.fillRect(x, y, w, h);
  }

  private buildPlayerSheet() {
    // 18×32 per frame, 4 directions × 3 frames (neutral / step-L / step-R) = 12 frames.
    const FW = 18;
    const FH = 32;
    const FRAMES = 12;
    const tex = this.textures.createCanvas("player_sheet", FW * FRAMES, FH)!;
    const ctx = tex.getContext();
    ctx.imageSmoothingEnabled = false;
    ctx.clearRect(0, 0, FW * FRAMES, FH);

    const P = {
      outline: "#0a0510",
      hair: "#2a1505",
      hairLight: "#4a2818",
      hairShade: "#160702",
      skin: "#ebcaa7",
      skinLight: "#f7dec0",
      skinShade: "#b5876a",
      eye: "#1a0d05",
      eyeWhite: "#f4e0c4",
      shirt: "#3d1f55",
      shirtLight: "#5a2d7a",
      shirtShade: "#231036",
      accent: "#b06fe5",
      cape: "#1a0726",
      capeShade: "#0a0210",
      pants: "#1f1828",
      pantsLight: "#2d2438",
      pantsShade: "#0e0815",
      boot: "#150810",
      bootHi: "#3a2030",
      buckle: "#a6822f"
    };

    for (let i = 0; i < FRAMES; i += 1) {
      const dir = Math.floor(i / 3);
      const step = i % 3;
      const ox = i * FW;
      if (dir === 0) this.drawPlayerDown(ctx, ox, step, P);
      else if (dir === 1) this.drawPlayerSide(ctx, ox, step, P, "left");
      else if (dir === 2) this.drawPlayerUp(ctx, ox, step, P);
      else this.drawPlayerSide(ctx, ox, step, P, "right");
      tex.add(String(i), 0, ox, 0, FW, FH);
    }
    tex.refresh();
  }

  // Step variation: returns [leftFootDy, rightFootDy] — negative means lifted up.
  private footOffsets(step: number): [number, number] {
    if (step === 1) return [-1, 0];
    if (step === 2) return [0, -1];
    return [0, 0];
  }

  private drawPlayerDown(ctx: CanvasRenderingContext2D, ox: number, step: number, P: Record<string, string>) {
    // ===== Head & hair (rows 1–10) =====
    // Hair silhouette outline
    this.fill(ctx, ox + 6, 1, 6, 1, P.outline);
    this.fill(ctx, ox + 5, 2, 1, 1, P.outline);
    this.fill(ctx, ox + 12, 2, 1, 1, P.outline);
    this.fill(ctx, ox + 4, 3, 1, 4, P.outline);
    this.fill(ctx, ox + 13, 3, 1, 4, P.outline);
    // Hair fill
    this.fill(ctx, ox + 6, 2, 6, 1, P.hair);
    this.fill(ctx, ox + 5, 3, 8, 4, P.hair);
    // Hair highlights — top-left strand catching light
    this.fill(ctx, ox + 6, 3, 1, 3, P.hairLight);
    this.fill(ctx, ox + 7, 3, 1, 1, P.hairLight);
    // Hair shadow on right side
    this.fill(ctx, ox + 11, 4, 1, 3, P.hairShade);
    // Bangs hanging down forehead
    this.fill(ctx, ox + 5, 7, 1, 1, P.hair);
    this.fill(ctx, ox + 12, 7, 1, 1, P.hair);
    this.fill(ctx, ox + 6, 7, 2, 1, P.hairLight);
    this.fill(ctx, ox + 10, 7, 2, 1, P.hairLight);
    // Skin face
    this.fill(ctx, ox + 6, 7, 6, 4, P.skin);
    // Face shading right side
    this.fill(ctx, ox + 11, 8, 1, 3, P.skinShade);
    // Face highlight
    this.fill(ctx, ox + 6, 8, 1, 1, P.skinLight);
    // Eyes (with whites for definition)
    this.fill(ctx, ox + 7, 9, 1, 1, P.eyeWhite);
    this.fill(ctx, ox + 10, 9, 1, 1, P.eyeWhite);
    this.fill(ctx, ox + 7, 9, 1, 1, P.eye);
    this.fill(ctx, ox + 10, 9, 1, 1, P.eye);
    // Brow shadow
    this.fill(ctx, ox + 7, 8, 1, 1, P.hairShade);
    this.fill(ctx, ox + 10, 8, 1, 1, P.hairShade);
    // Mouth / chin shadow
    this.fill(ctx, ox + 8, 10, 2, 1, P.skinShade);
    // Jaw outline
    this.fill(ctx, ox + 5, 10, 1, 1, P.outline);
    this.fill(ctx, ox + 12, 10, 1, 1, P.outline);
    // Neck
    this.fill(ctx, ox + 7, 11, 4, 1, P.skinShade);
    this.fill(ctx, ox + 6, 11, 1, 1, P.outline);
    this.fill(ctx, ox + 11, 11, 1, 1, P.outline);

    // ===== Torso (rows 12–18) =====
    // Outline shoulders + sides (slim chibi torso)
    this.fill(ctx, ox + 4, 12, 1, 7, P.outline);
    this.fill(ctx, ox + 13, 12, 1, 7, P.outline);
    // Shirt body
    this.fill(ctx, ox + 5, 12, 8, 7, P.shirt);
    // Shirt highlight (left chest)
    this.fill(ctx, ox + 5, 13, 1, 5, P.shirtLight);
    // Shirt shadow (right side)
    this.fill(ctx, ox + 12, 13, 1, 5, P.shirtShade);
    // V-neck collar
    this.fill(ctx, ox + 8, 12, 2, 1, P.outline);
    this.fill(ctx, ox + 8, 13, 2, 1, P.shirtShade);
    // Arms (sleeves)
    this.fill(ctx, ox + 4, 13, 1, 5, P.shirtShade);
    this.fill(ctx, ox + 13, 13, 1, 5, P.shirtShade);
    // Hands
    this.fill(ctx, ox + 4, 18, 1, 1, P.skin);
    this.fill(ctx, ox + 13, 18, 1, 1, P.skin);
    this.fill(ctx, ox + 3, 19, 1, 1, P.outline);
    this.fill(ctx, ox + 14, 19, 1, 1, P.outline);

    // ===== Belt (row 19) =====
    this.fill(ctx, ox + 4, 19, 10, 1, P.boot);
    this.fill(ctx, ox + 8, 19, 2, 1, P.buckle);

    // ===== Pants (rows 20–25) =====
    this.fill(ctx, ox + 4, 20, 1, 6, P.outline);
    this.fill(ctx, ox + 13, 20, 1, 6, P.outline);
    this.fill(ctx, ox + 5, 20, 8, 6, P.pants);
    this.fill(ctx, ox + 5, 20, 1, 6, P.pantsLight);
    this.fill(ctx, ox + 12, 20, 1, 6, P.pantsShade);
    // Leg separation
    this.fill(ctx, ox + 8, 21, 2, 5, P.pantsShade);
    this.fill(ctx, ox + 9, 21, 1, 5, P.outline);

    // ===== Boots (rows 26–30) with step variation =====
    const [lDy, rDy] = this.footOffsets(step);
    // Left boot (screen-left, x=5–8)
    this.fill(ctx, ox + 5, 26 + lDy, 3, 1, P.bootHi);
    this.fill(ctx, ox + 4, 27 + lDy, 1, 3, P.outline);
    this.fill(ctx, ox + 5, 27 + lDy, 3, 3, P.boot);
    this.fill(ctx, ox + 5, 27 + lDy, 1, 1, P.bootHi);
    this.fill(ctx, ox + 8, 27 + lDy, 1, 3, P.outline);
    this.fill(ctx, ox + 4, 30 + lDy, 5, 1, P.outline);
    // Right boot (screen-right, x=10–13)
    this.fill(ctx, ox + 10, 26 + rDy, 3, 1, P.bootHi);
    this.fill(ctx, ox + 9, 27 + rDy, 1, 3, P.outline);
    this.fill(ctx, ox + 10, 27 + rDy, 3, 3, P.boot);
    this.fill(ctx, ox + 10, 27 + rDy, 1, 1, P.bootHi);
    this.fill(ctx, ox + 13, 27 + rDy, 1, 3, P.outline);
    this.fill(ctx, ox + 9, 30 + rDy, 5, 1, P.outline);
  }

  private drawPlayerUp(ctx: CanvasRenderingContext2D, ox: number, step: number, P: Record<string, string>) {
    // ===== Head — all hair, no face =====
    this.fill(ctx, ox + 6, 1, 6, 1, P.outline);
    this.fill(ctx, ox + 5, 2, 1, 1, P.outline);
    this.fill(ctx, ox + 12, 2, 1, 1, P.outline);
    this.fill(ctx, ox + 4, 3, 1, 8, P.outline);
    this.fill(ctx, ox + 13, 3, 1, 8, P.outline);
    this.fill(ctx, ox + 6, 2, 6, 1, P.hair);
    this.fill(ctx, ox + 5, 3, 8, 8, P.hair);
    // Crown highlight
    this.fill(ctx, ox + 6, 3, 6, 1, P.hairLight);
    this.fill(ctx, ox + 7, 4, 4, 1, P.hairLight);
    // Hair tips fading at the nape
    this.fill(ctx, ox + 5, 10, 8, 1, P.hairShade);
    // Tiny ear hint on each side
    this.fill(ctx, ox + 5, 9, 1, 1, P.skinShade);
    this.fill(ctx, ox + 12, 9, 1, 1, P.skinShade);
    // Neck
    this.fill(ctx, ox + 7, 11, 4, 1, P.skinShade);
    this.fill(ctx, ox + 6, 11, 1, 1, P.outline);
    this.fill(ctx, ox + 11, 11, 1, 1, P.outline);

    // ===== Torso (back view) =====
    this.fill(ctx, ox + 3, 12, 1, 7, P.outline);
    this.fill(ctx, ox + 14, 12, 1, 7, P.outline);
    this.fill(ctx, ox + 4, 12, 1, 1, P.outline);
    this.fill(ctx, ox + 13, 12, 1, 1, P.outline);
    // Cape covering most of back
    this.fill(ctx, ox + 4, 12, 10, 7, P.cape);
    this.fill(ctx, ox + 5, 13, 8, 5, P.cape);
    // Cape highlight along left edge
    this.fill(ctx, ox + 5, 13, 1, 5, P.capeShade);
    this.fill(ctx, ox + 12, 13, 1, 5, P.capeShade);
    // Cape clasp at top
    this.fill(ctx, ox + 8, 12, 2, 1, P.buckle);
    // Shoulder slivers of shirt peeking
    this.fill(ctx, ox + 4, 13, 1, 4, P.shirt);
    this.fill(ctx, ox + 13, 13, 1, 4, P.shirt);
    // Arms
    this.fill(ctx, ox + 4, 18, 1, 1, P.skin);
    this.fill(ctx, ox + 13, 18, 1, 1, P.skin);
    this.fill(ctx, ox + 3, 19, 1, 1, P.outline);
    this.fill(ctx, ox + 14, 19, 1, 1, P.outline);
    // Belt
    this.fill(ctx, ox + 4, 19, 10, 1, P.boot);

    // Pants
    this.fill(ctx, ox + 4, 20, 1, 6, P.outline);
    this.fill(ctx, ox + 13, 20, 1, 6, P.outline);
    this.fill(ctx, ox + 5, 20, 8, 6, P.pants);
    this.fill(ctx, ox + 12, 20, 1, 6, P.pantsShade);
    this.fill(ctx, ox + 8, 21, 2, 5, P.pantsShade);
    this.fill(ctx, ox + 9, 21, 1, 5, P.outline);

    // Boots — step swapped (walking away from camera)
    const [lDy, rDy] = this.footOffsets(step);
    this.fill(ctx, ox + 5, 26 + lDy, 3, 1, P.bootHi);
    this.fill(ctx, ox + 4, 27 + lDy, 1, 3, P.outline);
    this.fill(ctx, ox + 5, 27 + lDy, 3, 3, P.boot);
    this.fill(ctx, ox + 8, 27 + lDy, 1, 3, P.outline);
    this.fill(ctx, ox + 4, 30 + lDy, 5, 1, P.outline);
    this.fill(ctx, ox + 10, 26 + rDy, 3, 1, P.bootHi);
    this.fill(ctx, ox + 9, 27 + rDy, 1, 3, P.outline);
    this.fill(ctx, ox + 10, 27 + rDy, 3, 3, P.boot);
    this.fill(ctx, ox + 13, 27 + rDy, 1, 3, P.outline);
    this.fill(ctx, ox + 9, 30 + rDy, 5, 1, P.outline);
  }

  private drawPlayerSide(ctx: CanvasRenderingContext2D, ox: number, step: number, P: Record<string, string>, dir: "left" | "right") {
    const flip = dir === "left";
    const f = (x: number, y: number, w: number, h: number, color: string) => {
      const fx = flip ? ox + 18 - x - w : ox + x;
      this.fill(ctx, fx, y, w, h, color);
    };

    // ===== Head/hair (profile) =====
    f(5, 1, 6, 1, P.outline);
    f(4, 2, 1, 1, P.outline);
    f(11, 2, 1, 1, P.outline);
    f(3, 3, 1, 4, P.outline);
    f(12, 3, 1, 8, P.outline);
    f(5, 2, 6, 1, P.hair);
    f(4, 3, 8, 4, P.hair);
    f(11, 6, 1, 5, P.hair); // hair behind ear flowing down
    f(5, 3, 1, 3, P.hairLight);
    f(10, 4, 1, 2, P.hairShade);
    // Face profile (front faces away from `flip` side)
    f(5, 7, 5, 4, P.skin);
    f(9, 8, 1, 3, P.skinShade);
    f(5, 8, 1, 1, P.skinLight);
    // Nose
    f(4, 8, 1, 1, P.skin);
    f(4, 9, 1, 1, P.skinShade);
    // Eye
    f(6, 8, 1, 1, P.eyeWhite);
    f(6, 8, 1, 1, P.eye);
    // Mouth
    f(5, 10, 2, 1, P.skinShade);
    // Jaw outline
    f(4, 10, 1, 1, P.outline);
    f(10, 10, 1, 1, P.outline);
    // Neck
    f(6, 11, 3, 1, P.skinShade);
    f(5, 11, 1, 1, P.outline);
    f(9, 11, 1, 1, P.outline);

    // ===== Torso (profile, slim) =====
    f(4, 12, 1, 7, P.outline);
    f(10, 12, 1, 7, P.outline);
    // Shirt body
    f(5, 12, 5, 7, P.shirt);
    f(5, 13, 1, 5, P.shirtLight);
    f(9, 13, 1, 5, P.shirtShade);
    // Arm swinging — direction depends on step
    let armDy = 0;
    if (step === 1) armDy = -1;
    if (step === 2) armDy = 1;
    f(5, 14 + armDy, 1, 4, P.shirtShade);
    // Hand
    f(5, 18 + armDy, 1, 1, P.skin);

    // ===== Belt + Pants =====
    f(4, 19, 7, 1, P.boot);
    f(7, 19, 2, 1, P.buckle);
    f(4, 20, 1, 6, P.outline);
    f(10, 20, 1, 6, P.outline);
    f(5, 20, 5, 6, P.pants);
    f(5, 20, 1, 6, P.pantsLight);
    f(9, 20, 1, 6, P.pantsShade);

    // ===== Boots — stride in/out, side view =====
    // Front leg is the one further forward (x=5-7), back leg behind (x=7-9)
    const stride = step === 1 ? -1 : step === 2 ? 1 : 0;
    // Front foot
    f(4 + stride, 26, 4, 1, P.bootHi);
    f(4 + stride, 27, 4, 3, P.boot);
    f(4 + stride, 30, 4, 1, P.outline);
    f(3 + stride, 27, 1, 3, P.outline);
    f(8 + stride, 27, 1, 3, P.outline);
    // Back foot
    f(7 - stride, 27, 3, 1, P.bootHi);
    f(7 - stride, 28, 3, 2, P.boot);
    f(7 - stride, 30, 3, 1, P.outline);
    f(6 - stride, 28, 1, 2, P.outline);
    f(10 - stride, 28, 1, 2, P.outline);
  }

  // ---------- Enemy sprite (corrupted dwarf) ----------

  private buildCorruptedDwarfSheet() {
    const FW = 16;
    const FH = 20;
    const FRAMES = 2;
    const tex = this.textures.createCanvas("enemy_sheet", FW * FRAMES, FH)!;
    const ctx = tex.getContext();
    ctx.imageSmoothingEnabled = false;
    ctx.clearRect(0, 0, FW * FRAMES, FH);

    for (let i = 0; i < FRAMES; i += 1) {
      const ox = i * FW;
      const bob = i === 0 ? 0 : -1; // breathing offset
      // beard / face dark
      this.fill(ctx, ox + 3, 2 + bob, 10, 5, "#2a1037");
      // helmet/hood
      this.fill(ctx, ox + 3, 1 + bob, 10, 2, "#180520");
      this.fill(ctx, ox + 5, 0 + bob, 6, 1, "#180520");
      // glowing eyes
      this.fill(ctx, ox + 6, 4 + bob, 1, 1, "#c97aff");
      this.fill(ctx, ox + 9, 4 + bob, 1, 1, "#c97aff");
      // beard tendrils
      this.fill(ctx, ox + 4, 7 + bob, 8, 2, "#3d1b50");
      this.fill(ctx, ox + 5, 9 + bob, 6, 1, "#3d1b50");
      // body / chestplate
      this.fill(ctx, ox + 3, 9 + bob, 10, 7, "#241333");
      this.fill(ctx, ox + 4, 10 + bob, 8, 1, "#3a1c4e");
      this.fill(ctx, ox + 7, 11 + bob, 2, 4, "#6e2da0");
      // arms
      this.fill(ctx, ox + 2, 10 + bob, 1, 5, "#160820");
      this.fill(ctx, ox + 13, 10 + bob, 1, 5, "#160820");
      // legs
      this.fill(ctx, ox + 4, 16, 4, 4, "#150820");
      this.fill(ctx, ox + 8, 16, 4, 4, "#150820");
      this.fill(ctx, ox + 4, 19, 4, 1, "#000000");
      this.fill(ctx, ox + 8, 19, 4, 1, "#000000");

      tex.add(String(i), 0, ox, 0, FW, FH);
    }
    tex.refresh();
  }

  // ---------- Injured dwarf NPC ----------

  private buildInjuredDwarfSprite() {
    const FW = 20;
    const FH = 14;
    const tex = this.textures.createCanvas("npc_dwarf", FW, FH)!;
    const ctx = tex.getContext();
    ctx.imageSmoothingEnabled = false;
    ctx.clearRect(0, 0, FW, FH);
    // dwarf lying down sideways
    // legs
    this.fill(ctx, 2, 8, 8, 3, "#3a2418");
    this.fill(ctx, 2, 10, 8, 1, "#1f120a");
    // torso
    this.fill(ctx, 8, 5, 8, 6, "#5e3424");
    this.fill(ctx, 9, 6, 6, 1, "#7d4730");
    // belt
    this.fill(ctx, 8, 8, 8, 1, "#1a0d06");
    // head + beard
    this.fill(ctx, 14, 3, 5, 6, "#dbb38a");
    this.fill(ctx, 14, 7, 5, 3, "#b48356");
    this.fill(ctx, 14, 1, 5, 3, "#7d4a26");
    // eye (closed)
    this.fill(ctx, 16, 5, 1, 1, "#1a0d05");
    // blood
    this.fill(ctx, 4, 11, 6, 1, "#7a1d24");
    this.fill(ctx, 5, 12, 4, 1, "#5a0f1a");
    // bandage
    this.fill(ctx, 11, 6, 3, 1, "#d8c9a0");
    tex.refresh();
  }

  // ---------- Totem fragment ----------

  private buildTotemSprite() {
    const FW = 18;
    const FH = 22;
    const FRAMES = 2;
    const tex = this.textures.createCanvas("totem", FW * FRAMES, FH)!;
    const ctx = tex.getContext();
    ctx.imageSmoothingEnabled = false;
    ctx.clearRect(0, 0, FW * FRAMES, FH);

    for (let i = 0; i < FRAMES; i += 1) {
      const ox = i * FW;
      const glow = i === 0 ? "#7a3aa6" : "#a45fd1";
      // base
      this.fill(ctx, ox + 5, 18, 8, 3, "#2a1535");
      this.fill(ctx, ox + 4, 20, 10, 2, "#170820");
      // shard
      this.fill(ctx, ox + 8, 3, 3, 16, "#3d1f55");
      this.fill(ctx, ox + 7, 5, 5, 12, "#582a78");
      this.fill(ctx, ox + 9, 4, 1, 14, glow);
      // top point
      this.fill(ctx, ox + 8, 1, 3, 2, "#3d1f55");
      this.fill(ctx, ox + 9, 0, 1, 1, glow);
      // runes
      this.fill(ctx, ox + 9, 9, 1, 1, "#f0b8ff");
      this.fill(ctx, ox + 9, 13, 1, 1, "#f0b8ff");

      tex.add(String(i), 0, ox, 0, FW, FH);
    }
    tex.refresh();
  }

  // ---------- Exit door ----------

  private buildExitSprite() {
    const FW = 24;
    const FH = 32;
    const tex = this.textures.createCanvas("exit_door", FW, FH)!;
    const ctx = tex.getContext();
    ctx.imageSmoothingEnabled = false;
    ctx.clearRect(0, 0, FW, FH);
    // frame
    this.fill(ctx, 2, 4, 20, 26, "#1c1410");
    this.fill(ctx, 3, 5, 18, 24, "#2e2218");
    // arch top
    this.fill(ctx, 6, 4, 12, 2, "#3d2c1c");
    this.fill(ctx, 8, 2, 8, 2, "#3d2c1c");
    // dark opening
    this.fill(ctx, 5, 7, 14, 21, "#080608");
    // dim light spill
    this.fill(ctx, 10, 24, 4, 4, "#1f3b3a");
    this.fill(ctx, 11, 26, 2, 2, "#3a6f6e");
    // moss outlines
    this.fill(ctx, 3, 28, 18, 1, "#3d5236");
    this.fill(ctx, 4, 30, 16, 1, "#2c3a26");
    tex.refresh();
  }

  // ---------- Slash sprite ----------

  private buildSlashSprite() {
    // Stardew-style 4-frame swoosh. Pivot at the left edge of each frame so
    // the sprite can be rotated around the player.
    const FW = 32;
    const FH = 48;
    const FRAMES = 4;
    const tex = this.textures.createCanvas("slash", FW * FRAMES, FH)!;
    const ctx = tex.getContext();
    ctx.imageSmoothingEnabled = false;
    ctx.clearRect(0, 0, FW * FRAMES, FH);

    const cx = 2;
    const cy = FH / 2;
    const rOuter = 26;
    const rInner = 18;
    const fullStart = -Math.PI / 2.4;
    const fullEnd = Math.PI / 2.4;

    const segments: Array<{ start: number; end: number; alpha: number }> = [
      // frame 0 — leading edge appears, top of arc
      { start: fullStart, end: -Math.PI / 8, alpha: 0.85 },
      // frame 1 — extends through the middle
      { start: fullStart, end: Math.PI / 6, alpha: 1.0 },
      // frame 2 — full arc fully visible
      { start: fullStart, end: fullEnd, alpha: 1.0 },
      // frame 3 — trailing fade, bottom of arc
      { start: Math.PI / 10, end: fullEnd, alpha: 0.5 }
    ];

    for (let i = 0; i < FRAMES; i += 1) {
      const ox = i * FW;
      const seg = segments[i];

      for (let a = seg.start; a <= seg.end; a += 0.02) {
        // Edge fade — pixels at the ends of the visible arc fade out for soft tips.
        const edgeT = Math.min((a - seg.start) / 0.25, (seg.end - a) / 0.25, 1);
        const alpha = seg.alpha * Math.max(0.15, edgeT);

        for (let r = rInner; r <= rOuter; r += 1) {
          const x = Math.round(cx + Math.cos(a) * r);
          const y = Math.round(cy + Math.sin(a) * r);
          if (x < 0 || x >= FW || y < 0 || y >= FH) continue;

          const tCore = (r - rInner) / (rOuter - rInner);
          // Outer rim is brightest white, inner core fades into pale blue.
          let color: string;
          if (tCore > 0.78) color = `rgba(255, 255, 255, ${alpha})`;
          else if (tCore > 0.45) color = `rgba(220, 232, 255, ${alpha * 0.75})`;
          else color = `rgba(150, 180, 230, ${alpha * 0.32})`;
          ctx.fillStyle = color;
          ctx.fillRect(ox + x, y, 1, 1);
        }
      }
      tex.add(String(i), 0, ox, 0, FW, FH);
    }
    tex.refresh();
  }

  // ---------- Animations ----------

  private createAnimations() {
    const make = (key: string, frames: number[], frameRate: number, repeat = -1) => {
      if (this.anims.exists(key)) return;
      this.anims.create({
        key,
        frames: frames.map((f) => ({ key: "player_sheet", frame: String(f) })),
        frameRate,
        repeat
      });
    };
    // Frames: 0–2 down, 3–5 left, 6–8 up, 9–11 right; per group: 0 neutral, 1 step-L, 2 step-R
    make("idle_down", [0], 1);
    make("idle_left", [3], 1);
    make("idle_up", [6], 1);
    make("idle_right", [9], 1);
    make("walk_down", [0, 1, 0, 2], 8);
    make("walk_left", [3, 4, 3, 5], 8);
    make("walk_up", [6, 7, 6, 8], 8);
    make("walk_right", [9, 10, 9, 11], 8);

    if (!this.anims.exists("enemy_idle")) {
      this.anims.create({
        key: "enemy_idle",
        frames: [
          { key: "enemy_sheet", frame: "0" },
          { key: "enemy_sheet", frame: "1" }
        ],
        frameRate: 2,
        repeat: -1
      });
    }
    if (!this.anims.exists("totem_pulse")) {
      this.anims.create({
        key: "totem_pulse",
        frames: [
          { key: "totem", frame: "0" },
          { key: "totem", frame: "1" }
        ],
        frameRate: 3,
        repeat: -1
      });
    }
    if (!this.anims.exists("slash_swing")) {
      this.anims.create({
        key: "slash_swing",
        frames: [
          { key: "slash", frame: "0" },
          { key: "slash", frame: "1" },
          { key: "slash", frame: "2" },
          { key: "slash", frame: "3" }
        ],
        frameRate: 28,
        repeat: 0
      });
    }
  }

  // ---------- Entities ----------

  private createControls() {
    this.cursors = this.input.keyboard!.createCursorKeys();
    this.keys = this.input.keyboard!.addKeys("W,A,S,D,J,E,SPACE") as Record<string, Phaser.Input.Keyboard.Key>;
  }

  private createPlayer() {
    this.player = this.physics.add.sprite(155, 165, "player_sheet", "0");
    this.player.setOrigin(0.5, 0.88);
    this.player.setScale(2); // chibi: smaller than tile scale so player feels SDV-sized
    const bodyW = 10;
    const bodyH = 6;
    this.player.body!.setSize(bodyW, bodyH);
    this.player.body!.setOffset((18 - bodyW) / 2, 32 - bodyH - 1);
    this.player.setCollideWorldBounds(true);
    this.player.setDepth(5);
    this.player.play("idle_down");
  }

  private createEnemies() {
    [
      [510, 245],
      [825, 395],
      [1075, 575]
    ].forEach(([x, y]) => {
      const body = this.physics.add.sprite(x, y, "enemy_sheet", "0");
      body.setOrigin(0.5, 0.85);
      body.setScale(TILE_SCALE);
      body.body!.setSize(10, 7);
      body.body!.setOffset(3, 12);
      body.setCollideWorldBounds(true);
      body.setDepth(5);
      body.play("enemy_idle");
      this.enemies.push({ body, hp: 32, attackCooldown: 0 });
    });
  }

  private createInteractables() {
    this.dwarfZone = this.add.zone(395, 230, 80, 80);
    this.physics.add.existing(this.dwarfZone, true);
    this.add.image(395, 230, "npc_dwarf").setScale(TILE_SCALE).setOrigin(0.5, 0.8).setDepth(4);
    this.add.text(355, 270, "受伤矮人", { color: "#d9bda0", fontSize: "13px" }).setDepth(6);

    this.totemZone = this.add.zone(910, 382, 90, 90);
    this.physics.add.existing(this.totemZone, true);
    const totem = this.add.sprite(910, 382, "totem", "0").setScale(TILE_SCALE).setOrigin(0.5, 0.85).setDepth(4);
    totem.play("totem_pulse");
    this.add.text(862, 432, "图腾残片", { color: "#caa7d9", fontSize: "13px" }).setDepth(6);

    this.exitZone = this.add.zone(1220, 650, 90, 130);
    this.physics.add.existing(this.exitZone, true);
    this.add.image(1220, 650, "exit_door").setScale(TILE_SCALE).setOrigin(0.5, 0.7).setDepth(4);
    this.add.text(1184, 740, "矿井出口", { color: "#9fd0c1", fontSize: "13px" }).setDepth(6);

    this.add.text(80, 96, "黑潮矿区", {
      color: "#d9bf86",
      fontFamily: "serif",
      fontSize: "22px"
    }).setDepth(6);
  }

  private createObjectiveText() {
    this.objectiveText = this.add
      .text(24, 500, "目标：沿矿灯残影逃出矿井。", {
        color: "#d9cfb7",
        fontSize: "14px",
        backgroundColor: "#111417cc",
        padding: { x: 12, y: 8 }
      })
      .setScrollFactor(0)
      .setDepth(20);
  }

  // ---------- Update loop ----------

  private updatePlayer(time: number, delta: number) {
    const speed = time < this.rollUntil ? 240 : 140;
    const velocity = new Phaser.Math.Vector2(0, 0);
    if (this.cursors.left.isDown || this.keys.A.isDown) velocity.x -= 1;
    if (this.cursors.right.isDown || this.keys.D.isDown) velocity.x += 1;
    if (this.cursors.up.isDown || this.keys.W.isDown) velocity.y -= 1;
    if (this.cursors.down.isDown || this.keys.S.isDown) velocity.y += 1;

    const moving = velocity.lengthSq() > 0;
    if (moving) {
      if (Math.abs(velocity.x) > Math.abs(velocity.y)) {
        this.playerFacing = velocity.x < 0 ? "left" : "right";
      } else {
        this.playerFacing = velocity.y < 0 ? "up" : "down";
      }
    }
    velocity.normalize().scale(speed);
    this.player.setVelocity(velocity.x, velocity.y);

    const anim = moving ? `walk_${this.playerFacing}` : `idle_${this.playerFacing}`;
    if (this.player.anims.currentAnim?.key !== anim) {
      this.player.play(anim, true);
    }

    if (Phaser.Input.Keyboard.JustDown(this.keys.SPACE) && this.stamina >= 28) {
      this.stamina -= 28;
      this.rollUntil = time + 260;
      this.player.setTint(0x98d8cc);
      this.time.delayedCall(260, () => this.player.clearTint());
    }

    this.attackCooldown -= delta;
    if (Phaser.Input.Keyboard.JustDown(this.keys.J) && this.attackCooldown <= 0) {
      this.performAttack();
      this.attackCooldown = 420;
    }

    if (time >= this.rollUntil) {
      this.stamina = Math.min(100, this.stamina + delta * 0.022);
    }
  }

  private performAttack() {
    if (this.attacking) return;
    this.attacking = true;

    const facingAngle = {
      down: Math.PI / 2,
      up: -Math.PI / 2,
      left: Math.PI,
      right: 0
    }[this.playerFacing];

    const slash = this.add.sprite(this.player.x, this.player.y, "slash", "0").setDepth(9);
    slash.setOrigin(0, 0.5);
    slash.setScale(TILE_SCALE);
    slash.setRotation(facingAngle);
    slash.play("slash_swing");
    slash.once(Phaser.Animations.Events.ANIMATION_COMPLETE, () => slash.destroy());

    const reach = 96;
    const halfArc = Math.PI / 2.4;
    this.enemies.forEach((enemy) => {
      const dx = enemy.body.x - this.player.x;
      const dy = enemy.body.y - this.player.y;
      const dist = Math.sqrt(dx * dx + dy * dy);
      if (dist > reach) return;
      const enemyAngle = Math.atan2(dy, dx);
      const diff = Phaser.Math.Angle.Wrap(enemyAngle - facingAngle);
      if (Math.abs(diff) > halfArc) return;
      enemy.hp -= 18;
      enemy.body.setTint(0xe2bddc);
      this.time.delayedCall(80, () => enemy.body.clearTint());
      const knockX = Math.cos(facingAngle) * 120;
      const knockY = Math.sin(facingAngle) * 120;
      enemy.body.setVelocity(knockX, knockY);
      this.time.delayedCall(120, () => enemy.body.setVelocity(0, 0));
      if (enemy.hp <= 0) this.killEnemy(enemy);
    });

    this.time.delayedCall(200, () => {
      this.attacking = false;
    });
  }

  private killEnemy(enemy: EnemyActor) {
    const store = useGameStore.getState();
    const gold = goldForKill("enemy", store.worldState.worldTier);
    store.addGold(gold);
    if (Math.random() > 0.36) {
      store.addItem(generateLoot("enemy", store.worldState.worldTier));
    }
    enemy.body.destroy();
    this.enemies = this.enemies.filter((entry) => entry !== enemy);
  }

  private updateEnemies(delta: number) {
    this.enemies.forEach((enemy) => {
      enemy.attackCooldown -= delta;
      const distance = Phaser.Math.Distance.Between(this.player.x, this.player.y, enemy.body.x, enemy.body.y);
      if (distance < 280) {
        this.physics.moveToObject(enemy.body, this.player, 62);
        enemy.body.flipX = this.player.x < enemy.body.x;
      } else {
        enemy.body.setVelocity(0, 0);
      }

      if (distance < 34 && enemy.attackCooldown <= 0 && this.time.now > this.rollUntil) {
        this.health = Math.max(0, this.health - 8);
        enemy.attackCooldown = 850;
        this.cameras.main.shake(70, 0.004);
        if (this.health <= 0) this.handlePlayerDown();
      }
    });
  }

  private handlePlayerDown() {
    this.health = 100;
    this.player.setPosition(155, 165);
    useGameStore.getState().setDialogue({
      speaker: "克哈低语",
      text: "死亡在这里没有耐心。站起来，再走一次。",
      tone: "whisper"
    });
  }

  private handleInteraction() {
    if (!Phaser.Input.Keyboard.JustDown(this.keys.E)) return;

    const store = useGameStore.getState();
    const nearDwarf = Phaser.Math.Distance.Between(this.player.x, this.player.y, this.dwarfZone.x, this.dwarfZone.y) < 78;
    const nearTotem = Phaser.Math.Distance.Between(this.player.x, this.player.y, this.totemZone.x, this.totemZone.y) < 86;
    const nearExit = Phaser.Math.Distance.Between(this.player.x, this.player.y, this.exitZone.x, this.exitZone.y) < 90;

    if (nearDwarf && !store.worldState.flags.firstDwarfChoice) {
      store.requestStateChange({
        type: "khah_whisper",
        requestedBy: "克哈低语",
        targetId: "injured_dwarf",
        reason: "玩家靠近第一个永久选择",
        effects: [{ path: "flags.metInjuredDwarf", value: true }]
      });
      store.setActiveChoice({
        id: "first_dwarf_choice",
        title: "第一个选择",
        body: "一个矮人倒在铁轨旁，手腕布条下有正在扩散的刺青。低语催你继续走。",
        options: [
          { id: "save", label: "救他", description: "理智稳定，但小镇可能承受感染风险。" },
          { id: "abandon", label: "抛下他", description: "听从低语，安全离开。" },
          { id: "kill", label: "终结他", description: "污染上升，猎人会认可这种冷酷。" }
        ]
      });
      return;
    }

    if (nearTotem && !store.worldState.flags.touchedTotemFragment) {
      const approved = store.requestStateChange({
        type: "touch_totem_fragment",
        requestedBy: "图腾残片",
        targetId: "mine_totem_fragment",
        reason: "玩家触碰第一块封印残片",
        effects: [
          { path: "flags.touchedTotemFragment", value: true },
          { path: "vesselAwakening", value: 2 },
          { path: "corruption", value: Math.min(100, store.worldState.corruption + 4) }
        ]
      });

      if (approved) {
        this.totemTouchedInScene = true;
        store.addItem(generateLoot("totem", store.worldState.worldTier));
        store.setVision({
          image: "/assets/characters/dwarf_captain_vision.jpg",
          caption: "矮人队长打开图腾的一瞬间，你看见紫色皮肤、黑色眼睛，以及一只虫子钻入王冠。"
        });
        store.setDialogue({
          speaker: "残响",
          text: "矮人队长打开图腾的一瞬间，你看见紫色皮肤、黑色眼睛，以及一只虫子钻入王冠。",
          tone: "memory"
        });
        this.objectiveText.setText("目标：矿井出口的黑潮退开了。前往右下方出口。");
      }
      return;
    }

    if (nearExit) {
      if (store.worldState.flags.escapedMine) {
        store.setDialogue({
          speaker: "矿井出口",
          text: "出口已经打开。下一版会从这里进入灰灯镇。",
          tone: "memory"
        });
        return;
      }

      const approved = store.requestStateChange({
        type: "escape_mine",
        requestedBy: "矿井出口",
        targetId: "mine_exit",
        reason: "玩家试图离开黑潮矿区",
        effects: [{ path: "flags.escapedMine", value: true }]
      });

      if (approved) {
        store.setDialogue({
          speaker: "克哈低语",
          text: "很好。现在去找那些还以为灯能挡住海的人。",
          tone: "whisper"
        });
        this.objectiveText.setText("第一版终点：你已逃出矿井。下一张地图将是灰灯镇。");
      }
    }
  }

  private syncHud(time: number) {
    if (time - this.lastHudSync < 100) return;
    this.lastHudSync = time;
    useGameStore.getState().setCombat({
      health: Math.round(this.health),
      maxHealth: 100,
      stamina: Math.round(this.stamina),
      maxStamina: 100,
      focus: this.focus,
      maxFocus: 30
    });

    if (this.totemTouchedInScene) {
      this.player.setTint(0xb692d2);
    }
  }
}
