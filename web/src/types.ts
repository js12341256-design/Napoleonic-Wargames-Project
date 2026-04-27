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

export interface PowerEconomy {
  power: string;
  treasury: number;
  income_per_day: number;
  expenditure_per_day: number;
  manpower_pool: number;
  manpower_cap: number;
  manpower_recovery: number;
  factories: number;
  war_exhaustion: number;
}

export interface DivisionTemplate {
  id: number;
  name: string;
  power: string;
  battalions: number;
  cavalrySquadrons: number;
  artilleryBatteries: number;
  tactic: "Column" | "Line" | "Square" | "SkirmishScreen";
}

export interface GameEventOption {
  label: string;
  effects: string[];
}

export interface GameEvent {
  id: number;
  title: string;
  description: string;
  firesFor: string;
  options: GameEventOption[];
}

export type Government = "Empire" | "AbsoluteMonarchy" | "ConstitutionalMonarchy" | "Republic";

export type Faction =
  | "Military"
  | "Nobility"
  | "Clergy"
  | "Merchants"
  | "Peasantry"
  | "Revolutionaries";

export interface StabilityEffects {
  income_modifier: number;
  manpower_modifier: number;
  revolt_chance: number;
  civil_war_risk: boolean;
}

export interface PowerPoliticsData {
  power: string;
  legitimacy: number;
  stability: number;
  government: Government;
  ruling_faction: Faction;
  faction_support: Record<string, number>;
  puppets: string[];
  overlord: string | null;
}

export interface BattleSide {
  power: string;
  commander: string;
  strength: number;
  tactic: 'Column' | 'Line' | 'Square' | 'SkirmishScreen';
}

export interface BattleEvent {
  territory: string;
  attacker: BattleSide;
  defender: BattleSide;
  outcome: 'attacker_advances' | 'stalemate' | 'defender_holds';
  attackerCasualties: number;
  defenderCasualties: number;
}

export type FocusEffect = Record<string, any>

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

export interface TerritoryInfo {
  id: string;
  name: string;
  owner: string;
  terrain: 'Plains' | 'Mountains' | 'Forest' | 'Coast' | 'River';
  corps: { name: string; strength: number; marshal?: string }[];
  goldPerDay: number;
  manpowerPerMonth: number;
}
