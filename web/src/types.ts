export interface Marshal {
  id: number;
  name: string;
  power: string;
  skill: number; // 1-10
  traits: MarshalTrait[];
  assignedCorps?: number;
  portraitKey: string;
}

export type MarshalTrait =
  | "DefensiveGenius"
  | "Aggressive"
  | "CavalryMaster"
  | "Tactician"
  | "Logistics"
  | "Siege"
  | "NavalCommander"
  | "InspirationalLeader";

export interface DivisionTemplate {
  id: number;
  name: string;
  power: string;
  battalions: number;
  cavalrySquadrons: number;
  artilleryBatteries: number;
  tactic: "Column" | "Line" | "Square" | "SkirmishScreen";
}

export interface FocusEffect {
  ManpowerBonus?: number;
  AttackBonus?: number;
  DefenseBonus?: number;
  SupplyRangeBonus?: number;
  DiplomaticInfluence?: [string, number];
  UnlockUnit?: string;
  TreasuryBonus?: number;
  NavalBonus?: number;
  ResearchBonus?: number;
}

export interface Focus {
  id: number;
  name: string;
  description: string;
  power: string;
  cost_days: number;
  prerequisites: number[];
  effects: FocusEffect[];
  x: number;
  y: number;
  icon: string;
  category: string;
}

export interface FocusTreeData {
  power: string;
  focuses: Record<string, Focus>;
  completed: number[];
  in_progress: [number, number] | null;
}
