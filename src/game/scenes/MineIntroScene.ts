import Phaser from "phaser";
import { generateLoot, goldForKill } from "../../core/lootGenerator";
import { useGameStore } from "../../store/useGameStore";

type EnemyActor = {
  body: Phaser.Physics.Arcade.Image;
  hp: number;
  attackCooldown: number;
};

export class MineIntroScene extends Phaser.Scene {
  private player!: Phaser.Physics.Arcade.Image;
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

  constructor() {
    super("MineIntroScene");
  }

  preload() {
    this.load.image("player_img", "assets/characters/disi_stage1.jpg");
    this.load.image("dwarf_injured", "assets/characters/dwarf_injured.jpg");
    this.load.image("dwarf_corrupted", "assets/characters/dwarf_corrupted.jpg");
  }

  create() {
    this.physics.world.setBounds(0, 0, 1450, 900);
    this.cameras.main.setBounds(0, 0, 1450, 900);

    this.drawMine();
    this.createControls();
    this.createPlayer();
    this.createEnemies();
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

  private drawMine() {
    const graphics = this.add.graphics();

    graphics.fillStyle(0x0b0d10, 1);
    graphics.fillRect(0, 0, 1450, 900);

    graphics.fillStyle(0x16181b, 1);
    graphics.fillRoundedRect(60, 80, 350, 180, 16);
    graphics.fillRoundedRect(360, 185, 520, 170, 16);
    graphics.fillRoundedRect(760, 310, 350, 210, 16);
    graphics.fillRoundedRect(1040, 470, 240, 260, 16);
    graphics.fillRoundedRect(420, 540, 520, 180, 16);

    graphics.fillStyle(0x241528, 0.86);
    graphics.fillEllipse(685, 260, 190, 70);
    graphics.fillEllipse(980, 445, 220, 90);
    graphics.fillEllipse(555, 640, 170, 58);

    graphics.lineStyle(3, 0x775f35, 0.65);
    for (let i = 0; i < 11; i += 1) {
      graphics.lineBetween(120 + i * 110, 120 + (i % 2) * 55, 180 + i * 105, 160 + (i % 2) * 60);
    }

    graphics.lineStyle(2, 0x4c354d, 0.5);
    for (let i = 0; i < 20; i += 1) {
      graphics.lineBetween(90 + i * 60, 820, 140 + i * 58, 760 - (i % 4) * 24);
    }

    this.add.text(96, 94, "黑潮矿区", {
      color: "#c9b77d",
      fontFamily: "serif",
      fontSize: "24px"
    });
  }

  private createControls() {
    this.cursors = this.input.keyboard!.createCursorKeys();
    this.keys = this.input.keyboard!.addKeys("W,A,S,D,J,E,SPACE") as Record<string, Phaser.Input.Keyboard.Key>;
  }

  private createPlayer() {
    this.player = this.physics.add.image(155, 165, "player_img");
    this.player.setDisplaySize(56, 64);
    const tex = this.player;
    const radius = Math.min(tex.width, tex.height) / 2;
    tex.setCircle(radius, (tex.width - radius * 2) / 2, (tex.height - radius * 2) / 2);
    tex.setCollideWorldBounds(true);
  }

  private createEnemies() {
    [
      [510, 245],
      [825, 395],
      [1075, 575]
    ].forEach(([x, y]) => {
      const body = this.physics.add.image(x, y, "dwarf_corrupted");
      body.setDisplaySize(52, 58);
      const radius = Math.min(body.width, body.height) / 2;
      body.setCircle(radius, (body.width - radius * 2) / 2, (body.height - radius * 2) / 2);
      body.setCollideWorldBounds(true);
      this.enemies.push({ body, hp: 32, attackCooldown: 0 });
    });
  }

  private createInteractables() {
    this.dwarfZone = this.add.zone(395, 230, 72, 72);
    this.physics.add.existing(this.dwarfZone, true);
    this.add.image(395, 230, "dwarf_injured").setDisplaySize(44, 50);
    this.add.text(365, 260, "受伤矮人", { color: "#b99f82", fontSize: "13px" });

    this.totemZone = this.add.zone(910, 382, 82, 82);
    this.physics.add.existing(this.totemZone, true);
    this.add.polygon(910, 382, [0, -32, 24, -8, 16, 30, -18, 28, -26, -10], 0x45224a, 1);
    this.add.text(870, 422, "图腾残片", { color: "#caa7d9", fontSize: "13px" });

    this.exitZone = this.add.zone(1220, 650, 90, 120);
    this.physics.add.existing(this.exitZone, true);
    this.add.rectangle(1220, 650, 78, 96, 0x151f20, 1).setStrokeStyle(2, 0x79a896);
    this.add.text(1188, 709, "矿井出口", { color: "#9fd0c1", fontSize: "13px" });
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

  private updatePlayer(time: number, delta: number) {
    const speed = time < this.rollUntil ? 275 : 150;
    const velocity = new Phaser.Math.Vector2(0, 0);
    if (this.cursors.left.isDown || this.keys.A.isDown) velocity.x -= 1;
    if (this.cursors.right.isDown || this.keys.D.isDown) velocity.x += 1;
    if (this.cursors.up.isDown || this.keys.W.isDown) velocity.y -= 1;
    if (this.cursors.down.isDown || this.keys.S.isDown) velocity.y += 1;
    velocity.normalize().scale(speed);
    this.player.setVelocity(velocity.x, velocity.y);

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
    const slash = this.add.circle(this.player.x, this.player.y, 58, 0xc9b77d, 0.22).setDepth(9);
    slash.setStrokeStyle(2, 0xe8d9a6, 0.7);

    this.enemies.forEach((enemy) => {
      const distance = Phaser.Math.Distance.Between(this.player.x, this.player.y, enemy.body.x, enemy.body.y);
      if (distance < 78) {
        enemy.hp -= 18;
        enemy.body.setTint(0xe2bddc);
        this.time.delayedCall(80, () => enemy.body.clearTint());
        if (enemy.hp <= 0) this.killEnemy(enemy);
      }
    });

    this.time.delayedCall(120, () => {
      slash.destroy();
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
          image: "assets/characters/dwarf_captain_vision.jpg",
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
