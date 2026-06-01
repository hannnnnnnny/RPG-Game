import { lazy, Suspense, useMemo, useState } from "react";
import {
  Backpack,
  Coins,
  Heart,
  RotateCcw,
  ScrollText,
  Shield,
  Skull,
  Sparkles,
  Swords,
  Zap
} from "lucide-react";
import { useGameStore } from "../store/useGameStore";
import type { Gender, PlayerProfile } from "../core/types";

const PhaserGame = lazy(() => import("../game/PhaserGame").then((module) => ({ default: module.PhaserGame })));

export function App() {
  const profile = useGameStore((state) => state.profile);

  if (!profile) {
    return <CharacterCreator />;
  }

  return <GameShell />;
}

function CharacterCreator() {
  const createProfile = useGameStore((state) => state.createProfile);
  const [name, setName] = useState("无名者");
  const [gender, setGender] = useState<Gender>("unknown");
  const [appearance, setAppearance] = useState<PlayerProfile["appearance"]>("ashen");

  return (
    <main className="creation-screen">
      <section className="creation-panel" aria-labelledby="creation-title">
        <div className="brand-mark">潮蚀之环</div>
        <h1 id="creation-title">黑潮矿区醒来的人</h1>
        <p>矿井深处只剩一个名字还没有被黑潮吞掉。</p>

        <label>
          名字
          <input value={name} onChange={(event) => setName(event.target.value)} maxLength={18} />
        </label>

        <div className="field-label">性别</div>
        <div className="segmented" aria-label="性别">
          {[
            ["unknown", "未知"],
            ["female", "女性"],
            ["male", "男性"]
          ].map(([value, label]) => (
            <button
              className={gender === value ? "selected" : ""}
              key={value}
              onClick={() => setGender(value as Gender)}
              type="button"
            >
              {label}
            </button>
          ))}
        </div>

        <div className="field-label">外貌</div>
        <div className="appearance-grid" aria-label="外貌">
          {[
            ["ashen", "灰烬旅人"],
            ["wanderer", "破斗篷"],
            ["miner", "矿区幸存者"],
            ["noble", "失落贵族"]
          ].map(([value, label]) => (
            <button
              className={appearance === value ? "selected" : ""}
              key={value}
              onClick={() => setAppearance(value as PlayerProfile["appearance"])}
              type="button"
            >
              <span className={`portrait ${value}`} />
              {label}
            </button>
          ))}
        </div>

        <button
          className="primary-action"
          disabled={!name.trim()}
          onClick={() => createProfile({ name: name.trim(), gender, appearance })}
          type="button"
        >
          <Sparkles size={18} />
          进入矿井
        </button>
      </section>
    </main>
  );
}

function GameShell() {
  return (
    <main className="game-shell">
      <section className="play-area" aria-label="游戏区域">
        <Suspense fallback={<div className="canvas-loading">矿灯正在点亮...</div>}>
          <PhaserGame />
        </Suspense>
        <GameHud />
        <DisiAvatar />
        <DialoguePanel />
        <ChoicePanel />
        <VisionOverlay />
      </section>
      <aside className="side-panel" aria-label="角色与任务">
        <CharacterPanel />
        <InventoryPanel />
        <QuestPanel />
        <LogPanel />
      </aside>
    </main>
  );
}

function DisiAvatar() {
  const corruption = useGameStore((state) => state.worldState.corruption);
  const profile = useGameStore((state) => state.profile);
  const stage = corruption <= 25 ? 1 : corruption <= 55 ? 2 : 3;
  const labels = ["神志清明", "渗透中", "意志崩碎"];
  return (
    <div className={`disi-avatar stage-${stage}`} aria-label="迪西状态">
      <img src={`assets/characters/disi_stage${stage}.jpg`} alt={`迪西第${stage}阶段`} />
      <div className="disi-avatar-meta">
        <strong>{profile?.name ?? "迪西"}</strong>
        <span>污染 {corruption} · {labels[stage - 1]}</span>
      </div>
    </div>
  );
}

function VisionOverlay() {
  const vision = useGameStore((state) => state.vision);
  const setVision = useGameStore((state) => state.setVision);
  if (!vision) return null;
  return (
    <div
      className="vision-overlay"
      role="dialog"
      aria-modal="true"
      onClick={() => setVision(null)}
    >
      <div className="vision-frame" onClick={(event) => event.stopPropagation()}>
        <img src={vision.image} alt="幻象" />
        {vision.caption && <p>{vision.caption}</p>}
        <button type="button" onClick={() => setVision(null)}>
          闭上眼
        </button>
      </div>
    </div>
  );
}

function GameHud() {
  const combat = useGameStore((state) => state.combat);
  const world = useGameStore((state) => state.worldState);

  return (
    <div className="hud">
      <Meter icon={<Heart size={16} />} label="生命" value={combat.health} max={combat.maxHealth} tone="red" />
      <Meter icon={<Zap size={16} />} label="体力" value={combat.stamina} max={combat.maxStamina} tone="green" />
      <Meter icon={<Sparkles size={16} />} label="专注" value={combat.focus} max={combat.maxFocus} tone="blue" />
      <div className="hud-pill">
        <Skull size={15} />
        污染 {world.corruption}
      </div>
      <div className="hud-pill">
        <Shield size={15} />
        觉醒 {world.vesselAwakening}
      </div>
      <div className="hud-pill">
        <Coins size={15} />
        {world.gold}
      </div>
    </div>
  );
}

function Meter({
  icon,
  label,
  value,
  max,
  tone
}: {
  icon: React.ReactNode;
  label: string;
  value: number;
  max: number;
  tone: "red" | "green" | "blue";
}) {
  const percent = Math.max(0, Math.min(100, (value / max) * 100));
  return (
    <div className="meter">
      <span>
        {icon}
        {label}
      </span>
      <div className="meter-track">
        <div className={`meter-fill ${tone}`} style={{ width: `${percent}%` }} />
      </div>
      <b>
        {value}/{max}
      </b>
    </div>
  );
}

function DialoguePanel() {
  const dialogue = useGameStore((state) => state.dialogue);
  const setDialogue = useGameStore((state) => state.setDialogue);
  if (!dialogue) return null;

  const isKhah = dialogue.speaker === "克哈低语";

  return (
    <section className={`dialogue-panel ${dialogue.tone ?? ""}`} aria-live="polite">
      {isKhah && (
        <img className="dialogue-portrait" src="assets/characters/khah.jpg" alt="克哈" />
      )}
      <div>
        <strong>{dialogue.speaker}</strong>
        <p>{dialogue.text}</p>
      </div>
      <button aria-label="关闭对话" onClick={() => setDialogue(null)} type="button">
        ×
      </button>
    </section>
  );
}

function ChoicePanel() {
  const activeChoice = useGameStore((state) => state.activeChoice);
  const setActiveChoice = useGameStore((state) => state.setActiveChoice);
  const requestStateChange = useGameStore((state) => state.requestStateChange);
  const setDialogue = useGameStore((state) => state.setDialogue);
  const addGold = useGameStore((state) => state.addGold);

  if (!activeChoice) return null;

  const choose = (id: string) => {
    const effects = [
      { path: "flags.firstDwarfChoice", value: id },
      { path: "sanity", value: id === "save" ? 82 : id === "abandon" ? 70 : 64 },
      { path: "corruption", value: id === "kill" ? 13 : id === "abandon" ? 8 : 5 }
    ];
    const approved = requestStateChange({
      type: "record_first_choice",
      requestedBy: "受伤矮人",
      targetId: activeChoice.id,
      reason: `玩家选择：${id}`,
      effects
    });

    if (approved) {
      addGold(id === "save" ? 8 : 16);
      setDialogue({
        speaker: id === "save" ? "受伤矮人" : "克哈低语",
        text:
          id === "save"
            ? "他还活着。也许这会让灰灯镇多一个问题，也许多一个证人。"
            : id === "abandon"
              ? "很好。怜悯会让矿道坍得更慢，但不会让你活得更久。"
              : "血没有溅到你身上，它像认识你一样避开了。",
        tone: id === "save" ? "memory" : "whisper"
      });
    }
    setActiveChoice(null);
  };

  return (
    <section className="choice-panel" aria-label="永久选择">
      <h2>{activeChoice.title}</h2>
      <p>{activeChoice.body}</p>
      <div className="choice-list">
        {activeChoice.options.map((option) => (
          <button key={option.id} onClick={() => choose(option.id)} type="button">
            <strong>{option.label}</strong>
            <span>{option.description}</span>
          </button>
        ))}
      </div>
    </section>
  );
}

function CharacterPanel() {
  const profile = useGameStore((state) => state.profile)!;
  const world = useGameStore((state) => state.worldState);
  const resetRun = useGameStore((state) => state.resetRun);

  return (
    <section className="panel-section">
      <div className="section-title">
        <Swords size={18} />
        <h2>{profile.name}</h2>
      </div>
      <dl className="stat-grid">
        <div>
          <dt>理智</dt>
          <dd>{world.sanity}</dd>
        </div>
        <div>
          <dt>污染</dt>
          <dd>{world.corruption}</dd>
        </div>
        <div>
          <dt>容器</dt>
          <dd>{world.vesselAwakening}</dd>
        </div>
        <div>
          <dt>世界</dt>
          <dd>{world.worldTier}</dd>
        </div>
      </dl>
      <button className="quiet-button" onClick={resetRun} type="button">
        <RotateCcw size={15} />
        重置旅程
      </button>
    </section>
  );
}

function InventoryPanel() {
  const inventory = useGameStore((state) => state.inventory);
  const equipped = useGameStore((state) => state.equipped);
  const equipItem = useGameStore((state) => state.equipItem);

  return (
    <section className="panel-section inventory-section">
      <div className="section-title">
        <Backpack size={18} />
        <h2>背包</h2>
      </div>
      {inventory.length === 0 ? (
        <p className="muted">矿袋空着，只剩黑潮和铁锈的气味。</p>
      ) : (
        <div className="item-list">
          {inventory.slice(0, 8).map((item) => (
            <button
              className={`item-row ${item.quality}`}
              key={item.id}
              onClick={() => equipItem(item.id)}
              type="button"
            >
              <span>
                <strong>{item.name}</strong>
                <small>
                  {item.quality} · 强度 {item.itemPower}
                  {equipped[item.slot] === item.id ? " · 已装备" : ""}
                </small>
              </span>
              <em>{item.affixes[0]?.label}+{item.affixes[0]?.value}</em>
            </button>
          ))}
        </div>
      )}
    </section>
  );
}

function QuestPanel() {
  const world = useGameStore((state) => state.worldState);
  const objectives = useMemo(() => {
    const list = ["逃出黑潮矿区"];
    if (!world.flags.firstDwarfChoice) list.push("处理受伤矮人的命运");
    if (!world.flags.touchedTotemFragment) list.push("触碰图腾残片");
    if (world.flags.touchedTotemFragment && !world.flags.escapedMine) list.push("前往矿井出口");
    if (world.flags.escapedMine) list.push("灰灯镇的路还在雾里");
    return list;
  }, [world.flags]);

  return (
    <section className="panel-section">
      <div className="section-title">
        <ScrollText size={18} />
        <h2>任务</h2>
      </div>
      <ul className="quest-list">
        {objectives.map((objective) => (
          <li key={objective}>{objective}</li>
        ))}
      </ul>
    </section>
  );
}

function LogPanel() {
  const log = useGameStore((state) => state.log);

  return (
    <section className="panel-section log-section">
      <div className="section-title">
        <ScrollText size={18} />
        <h2>日志</h2>
      </div>
      <ol>
        {log.slice(0, 6).map((entry, index) => (
          <li key={`${entry}-${index}`}>{entry}</li>
        ))}
      </ol>
    </section>
  );
}
