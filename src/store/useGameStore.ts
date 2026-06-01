import { create } from "zustand";
import { persist } from "zustand/middleware";
import { approveStateChange } from "../core/aidlcRules";
import { initialWorldState } from "../core/worldState";
import type {
  ActiveChoice,
  CombatSnapshot,
  DialogueState,
  ItemInstance,
  PlayerProfile,
  StateChangeRequest,
  WorldState
} from "../core/types";

type VisionOverlay = {
  image: string;
  caption?: string;
};

type GameStore = {
  profile: PlayerProfile | null;
  worldState: WorldState;
  combat: CombatSnapshot;
  inventory: ItemInstance[];
  equipped: Partial<Record<string, string>>;
  dialogue: DialogueState | null;
  activeChoice: ActiveChoice | null;
  vision: VisionOverlay | null;
  log: string[];
  createProfile: (profile: PlayerProfile) => void;
  setCombat: (combat: CombatSnapshot) => void;
  setDialogue: (dialogue: DialogueState | null) => void;
  setActiveChoice: (choice: ActiveChoice | null) => void;
  setVision: (vision: VisionOverlay | null) => void;
  addItem: (item: ItemInstance) => void;
  addGold: (amount: number) => void;
  equipItem: (itemId: string) => void;
  requestStateChange: (request: StateChangeRequest) => boolean;
  resetRun: () => void;
};

const initialCombat: CombatSnapshot = {
  health: 100,
  maxHealth: 100,
  stamina: 100,
  maxStamina: 100,
  focus: 30,
  maxFocus: 30
};

function setPath(worldState: WorldState, path: string, value: boolean | number | string): WorldState {
  if (path.startsWith("flags.")) {
    const flagName = path.replace("flags.", "");
    return {
      ...worldState,
      flags: {
        ...worldState.flags,
        [flagName]: value
      }
    };
  }

  if (path === "sanity" || path === "corruption" || path === "parasiteLoad" || path === "gold") {
    return {
      ...worldState,
      [path]: Number(value)
    };
  }

  if (path === "vesselAwakening") {
    return {
      ...worldState,
      vesselAwakening: Number(value) as WorldState["vesselAwakening"]
    };
  }

  return worldState;
}

export const useGameStore = create<GameStore>()(
  persist(
    (set, get) => ({
      profile: null,
      worldState: initialWorldState,
      combat: initialCombat,
      inventory: [],
      equipped: {},
      dialogue: null,
      activeChoice: null,
      vision: null,
      log: ["存档初始化。"],
      createProfile: (profile) =>
        set((state) => ({
          profile,
          worldState: {
            ...state.worldState,
            flags: {
              ...state.worldState.flags,
              awakenedByKhah: true
            },
            vesselAwakening: 1
          },
          dialogue: {
            speaker: "克哈低语",
            text: `${profile.name}，醒来。石头正在合拢，而你不该死在这里。`,
            tone: "whisper"
          },
          log: [`${profile.name} 在黑潮矿区苏醒。`, ...state.log]
        })),
      setCombat: (combat) => set({ combat }),
      setDialogue: (dialogue) => set({ dialogue }),
      setActiveChoice: (choice) => set({ activeChoice: choice }),
      setVision: (vision) => set({ vision }),
      addItem: (item) =>
        set((state) => ({
          inventory: [item, ...state.inventory],
          log: [`获得装备：${item.name}`, ...state.log].slice(0, 16)
        })),
      addGold: (amount) =>
        set((state) => ({
          worldState: {
            ...state.worldState,
            gold: state.worldState.gold + amount
          },
          log: [`获得 ${amount} 金币。`, ...state.log].slice(0, 16)
        })),
      equipItem: (itemId) =>
        set((state) => {
          const item = state.inventory.find((entry) => entry.id === itemId);
          if (!item) return state;
          return {
            equipped: {
              ...state.equipped,
              [item.slot]: item.id
            },
            log: [`装备：${item.name}`, ...state.log].slice(0, 16)
          };
        }),
      requestStateChange: (request) => {
        const state = get();
        const decision = approveStateChange(request, state.worldState);
        if (!decision.approved) {
          set({
            dialogue: {
              speaker: request.requestedBy,
              text: decision.reason,
              tone: "warning"
            }
          });
          return false;
        }

        const nextWorld = request.effects.reduce(
          (current, effect) => setPath(current, effect.path, effect.value),
          state.worldState
        );

        set({
          worldState: nextWorld,
          log: [`世界状态变更：${request.type}`, ...state.log].slice(0, 16)
        });
        return true;
      },
      resetRun: () =>
        set({
          profile: null,
          worldState: initialWorldState,
          combat: initialCombat,
          inventory: [],
          equipped: {},
          dialogue: null,
          activeChoice: null,
          vision: null,
          log: ["已重置本地测试存档。"]
        })
    }),
    {
      name: "tides-of-khah-save-v1",
      partialize: (state) => ({
        profile: state.profile,
        worldState: state.worldState,
        inventory: state.inventory,
        equipped: state.equipped,
        log: state.log
      })
    }
  )
);

