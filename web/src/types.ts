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
