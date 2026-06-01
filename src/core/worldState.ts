import type { WorldState } from "./types";

export const initialWorldState: WorldState = {
  worldTier: 1,
  sanity: 78,
  corruption: 5,
  vesselAwakening: 0,
  parasiteLoad: 0,
  gold: 0,
  flags: {
    awakenedByKhah: false,
    metInjuredDwarf: false,
    firstDwarfChoice: "",
    touchedTotemFragment: false,
    escapedMine: false
  }
};

