export type Gender = "female" | "male" | "unknown";

export type PlayerProfile = {
  name: string;
  gender: Gender;
  appearance: "ashen" | "wanderer" | "miner" | "noble";
};

export type WorldTier = 1 | 2 | 3 | 4 | 5;

export type VesselAwakening = 0 | 1 | 2 | 3 | 4 | 5;

export type WorldState = {
  worldTier: WorldTier;
  sanity: number;
  corruption: number;
  vesselAwakening: VesselAwakening;
  parasiteLoad: number;
  gold: number;
  flags: Record<string, boolean | number | string>;
};

export type EquipmentSlot =
  | "mainHand"
  | "offHand"
  | "head"
  | "chest"
  | "hands"
  | "boots"
  | "amulet"
  | "ring"
  | "totem";

export type ItemQuality = "broken" | "common" | "rare" | "corrupted" | "relic" | "mythic";

export type AffixCategory =
  | "attack"
  | "defense"
  | "mobility"
  | "forbidden"
  | "vessel"
  | "economy";

export type AffixInstance = {
  id: string;
  label: string;
  category: AffixCategory;
  value: number;
};

export type ItemInstance = {
  id: string;
  name: string;
  slot: EquipmentSlot;
  quality: ItemQuality;
  itemPower: number;
  upgradeLevel: number;
  rerollCount: number;
  affixes: AffixInstance[];
  coreEffect?: string;
};

export type CombatSnapshot = {
  health: number;
  maxHealth: number;
  stamina: number;
  maxStamina: number;
  focus: number;
  maxFocus: number;
};

export type DialogueState = {
  speaker: string;
  text: string;
  tone?: "whisper" | "warning" | "memory";
};

export type ChoiceOption = {
  id: string;
  label: string;
  description: string;
};

export type ActiveChoice = {
  id: string;
  title: string;
  body: string;
  options: ChoiceOption[];
};

export type StateChangeRequest = {
  type: string;
  requestedBy: string;
  targetId: string;
  reason: string;
  effects: Array<{
    path: string;
    value: boolean | number | string;
  }>;
};

export type StateChangeDecision = {
  approved: boolean;
  reason: string;
};

