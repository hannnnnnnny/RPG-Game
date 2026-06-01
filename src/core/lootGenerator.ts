import type { AffixInstance, EquipmentSlot, ItemInstance, ItemQuality, WorldTier } from "./types";

const slots: EquipmentSlot[] = ["mainHand", "chest", "hands", "boots", "ring", "totem"];

const baseNames: Record<EquipmentSlot, string[]> = {
  mainHand: ["矿工断刃", "灰灯短剑", "黑潮凿斧"],
  offHand: ["破裂法器"],
  head: ["污灰兜帽"],
  chest: ["矿井皮甲", "盐石胸甲"],
  hands: ["裂岩手套", "旧皮手套"],
  boots: ["逃亡者旧靴", "灰泥长靴"],
  amulet: ["残响项链"],
  ring: ["黑腕戒指", "灰灯铜戒"],
  totem: ["图腾碎屑", "矿井护符"]
};

const affixPool: Omit<AffixInstance, "id" | "value">[] = [
  { label: "近战伤害", category: "attack" },
  { label: "暴击率", category: "attack" },
  { label: "对感染者伤害", category: "attack" },
  { label: "最大生命", category: "defense" },
  { label: "黑潮抗性", category: "defense" },
  { label: "翻滚后伤害", category: "mobility" },
  { label: "体力回复", category: "mobility" },
  { label: "禁忌法术伤害", category: "forbidden" },
  { label: "理智稳定", category: "vessel" },
  { label: "金币掉落", category: "economy" }
];

function randomId(prefix: string) {
  return `${prefix}_${Math.random().toString(36).slice(2, 10)}`;
}

function pick<T>(items: T[]): T {
  return items[Math.floor(Math.random() * items.length)];
}

function qualityForSource(source: "enemy" | "elite" | "totem"): ItemQuality {
  const roll = Math.random();
  if (source === "totem") return roll > 0.55 ? "rare" : "common";
  if (source === "elite") return roll > 0.45 ? "rare" : "common";
  if (roll > 0.86) return "rare";
  if (roll > 0.45) return "common";
  return "broken";
}

function affixCount(quality: ItemQuality) {
  if (quality === "broken") return 1;
  if (quality === "common") return 2;
  if (quality === "rare") return 3;
  return 4;
}

export function generateLoot(source: "enemy" | "elite" | "totem", worldTier: WorldTier): ItemInstance {
  const slot = pick(slots);
  const quality = qualityForSource(source);
  const itemPower = worldTier * 10 + Math.floor(Math.random() * 8) + (source === "totem" ? 5 : 0);
  const chosenAffixes = Array.from({ length: affixCount(quality) }, (_, index) => {
    const affix = pick(affixPool);
    return {
      ...affix,
      id: randomId(`affix_${index}`),
      value: Math.max(2, Math.round(itemPower * (0.4 + Math.random() * 0.7)))
    };
  });

  return {
    id: randomId("item"),
    name: pick(baseNames[slot]),
    slot,
    quality,
    itemPower,
    upgradeLevel: 0,
    rerollCount: 0,
    affixes: chosenAffixes,
    coreEffect: source === "totem" ? "触碰图腾后，容器觉醒经验小幅提高。" : undefined
  };
}

export function goldForKill(source: "enemy" | "elite" | "boss", worldTier: WorldTier) {
  const base = source === "boss" ? 70 : source === "elite" ? 24 : 8;
  return base * worldTier + Math.floor(Math.random() * base);
}

