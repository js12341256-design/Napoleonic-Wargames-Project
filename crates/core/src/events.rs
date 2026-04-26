//! Historical Events system for Grand Campaign 1805.
//!
//! 25 scripted historical events with date/territory triggers,
//! player-facing options, and gameplay effects.

use std::collections::BTreeMap;

use gc1805_core_schema::ids::{AreaId, MarshalId, PowerId};

use crate::marshals::MarshalTrait;

// ── Core types ──────────────────────────────────────────────────────────

/// Unique event identifier (simple u32).
pub type EventId = u32;

/// How an event is triggered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventTrigger {
    DateReached { year: u16, month: u8, day: u8 },
    TerritoryLost(AreaId),
    TerritoryGained(AreaId),
    AlwaysTrue,
}

/// A gameplay effect applied when the player picks an option.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventEffect {
    ManpowerChange(i32),
    TreasuryChange(i32),
    RelationChange { power: PowerId, delta: i32 },
    DeclareWar(PowerId),
    PeaceOffer(PowerId),
    AddTrait(MarshalId, MarshalTrait),
    NewsMessage(String),
}

/// One player-facing choice within an event.
#[derive(Debug, Clone)]
pub struct EventOption {
    pub label: &'static str,
    pub effects: Vec<EventEffect>,
}

/// A historical event that can fire once (or repeatedly).
#[derive(Debug, Clone)]
pub struct HistoricalEvent {
    pub id: EventId,
    pub title: &'static str,
    pub description: &'static str,
    pub trigger: EventTrigger,
    pub options: Vec<EventOption>,
    pub fires_for: PowerId,
    pub repeatable: bool,
    pub fired: bool,
}

// ── Registry ────────────────────────────────────────────────────────────

/// Manages all historical events, checks triggers, and resolves choices.
#[derive(Debug)]
pub struct EventRegistry {
    pub events: BTreeMap<EventId, HistoricalEvent>,
}

impl EventRegistry {
    /// Create a registry with all 25 historical events.
    pub fn with_historical() -> Self {
        let mut events = BTreeMap::new();
        for e in all_historical_events() {
            events.insert(e.id, e);
        }
        EventRegistry { events }
    }

    /// Create an empty registry (for testing).
    pub fn new() -> Self {
        EventRegistry {
            events: BTreeMap::new(),
        }
    }

    /// Add a single event.
    pub fn add(&mut self, event: HistoricalEvent) {
        self.events.insert(event.id, event);
    }

    /// Check date-based triggers and mark matching events as pending.
    /// Returns IDs of events that should fire.
    pub fn advance_triggers(&mut self, year: u16, month: u8, day: u8) -> Vec<EventId> {
        let mut pending = Vec::new();
        for (id, event) in &self.events {
            if event.fired && !event.repeatable {
                continue;
            }
            match &event.trigger {
                EventTrigger::DateReached {
                    year: y,
                    month: m,
                    day: d,
                } => {
                    if year >= *y && (year > *y || month > *m || (month == *m && day >= *d)) {
                        pending.push(*id);
                    }
                }
                EventTrigger::AlwaysTrue => {
                    pending.push(*id);
                }
                _ => {}
            }
        }
        pending
    }

    /// Check territory-based triggers.
    pub fn check_territory_trigger(
        &mut self,
        area: &AreaId,
        gained: bool,
    ) -> Vec<EventId> {
        let mut pending = Vec::new();
        for (id, event) in &self.events {
            if event.fired && !event.repeatable {
                continue;
            }
            match &event.trigger {
                EventTrigger::TerritoryGained(a) if gained && a == area => {
                    pending.push(*id);
                }
                EventTrigger::TerritoryLost(a) if !gained && a == area => {
                    pending.push(*id);
                }
                _ => {}
            }
        }
        pending
    }

    /// Get all pending (unfired) events for a specific power, given current
    /// date triggers that have been advanced.
    pub fn get_pending_for_power(&self, power: &PowerId, fired_ids: &[EventId]) -> Vec<&HistoricalEvent> {
        fired_ids
            .iter()
            .filter_map(|id| self.events.get(id))
            .filter(|e| &e.fires_for == power && (!e.fired || e.repeatable))
            .collect()
    }

    /// Resolve a player choice: mark fired, return effects.
    pub fn resolve(&mut self, event_id: EventId, option_index: u8) -> Result<Vec<EventEffect>, String> {
        let event = self
            .events
            .get_mut(&event_id)
            .ok_or_else(|| format!("event {event_id} not found"))?;
        let idx = option_index as usize;
        if idx >= event.options.len() {
            return Err(format!(
                "option index {idx} out of range (event has {} options)",
                event.options.len()
            ));
        }
        event.fired = true;
        Ok(event.options[idx].effects.clone())
    }

    /// Serialize pending events for a power as JSON (for WASM export).
    pub fn pending_events_json(&self, power: &PowerId, fired_ids: &[EventId]) -> String {
        let pending = self.get_pending_for_power(power, fired_ids);
        let json_events: Vec<serde_json::Value> = pending
            .iter()
            .map(|e| {
                let options: Vec<serde_json::Value> = e
                    .options
                    .iter()
                    .map(|o| {
                        let effects: Vec<String> = o.effects.iter().map(effect_summary).collect();
                        serde_json::json!({
                            "label": o.label,
                            "effects": effects,
                        })
                    })
                    .collect();
                serde_json::json!({
                    "id": e.id,
                    "title": e.title,
                    "description": e.description,
                    "firesFor": e.fires_for.as_str(),
                    "options": options,
                })
            })
            .collect();
        serde_json::to_string(&json_events).unwrap_or_else(|_| "[]".to_string())
    }
}

/// Human-readable summary of an effect for the UI.
fn effect_summary(effect: &EventEffect) -> String {
    match effect {
        EventEffect::ManpowerChange(n) => {
            if *n >= 0 {
                format!("+{n} manpower")
            } else {
                format!("{n} manpower")
            }
        }
        EventEffect::TreasuryChange(n) => {
            if *n >= 0 {
                format!("+{n} treasury")
            } else {
                format!("{n} treasury")
            }
        }
        EventEffect::RelationChange { power, delta } => {
            let sign = if *delta >= 0 { "+" } else { "" };
            format!("{sign}{delta} relations with {}", power.as_str())
        }
        EventEffect::DeclareWar(p) => format!("Declare war on {}", p.as_str()),
        EventEffect::PeaceOffer(p) => format!("Peace offer to {}", p.as_str()),
        EventEffect::AddTrait(m, t) => format!("Add {:?} to {}", t, m.as_str()),
        EventEffect::NewsMessage(msg) => msg.clone(),
    }
}

// ── 25 Historical Events ────────────────────────────────────────────────

fn fra() -> PowerId { PowerId::from("FRA") }
fn gbr() -> PowerId { PowerId::from("GBR") }
fn rus() -> PowerId { PowerId::from("RUS") }
fn aus() -> PowerId { PowerId::from("AUS") }
fn pru() -> PowerId { PowerId::from("PRU") }
fn spa() -> PowerId { PowerId::from("SPA") }

fn all_historical_events() -> Vec<HistoricalEvent> {
    vec![
        // ── FRANCE ──────────────────────────────────────────────────
        // 1. The Emperor Crowns Himself
        HistoricalEvent {
            id: 1,
            title: "The Emperor Crowns Himself",
            description: "In the Cathedral of Notre-Dame, Napoleon Bonaparte takes the crown from \
                the Pope's hands and places it upon his own head. The assembled dignitaries watch \
                in stunned silence as a new era begins for France.",
            trigger: EventTrigger::DateReached { year: 1804, month: 12, day: 2 },
            options: vec![
                EventOption {
                    label: "Grand Ceremony",
                    effects: vec![
                        EventEffect::ManpowerChange(5),
                        EventEffect::TreasuryChange(-10),
                        EventEffect::NewsMessage("Napoleon crowned Emperor in grand ceremony!".into()),
                    ],
                },
                EventOption {
                    label: "Simple Ceremony",
                    effects: vec![
                        EventEffect::TreasuryChange(15),
                        EventEffect::NewsMessage("Napoleon crowned Emperor in modest ceremony.".into()),
                    ],
                },
            ],
            fires_for: fra(),
            repeatable: false,
            fired: false,
        },
        // 2. The Continental System
        HistoricalEvent {
            id: 2,
            title: "The Continental System",
            description: "Napoleon decrees that all European ports under French influence shall be \
                closed to British trade. The Berlin Decree aims to strangle Britain's economy, \
                but enforcing it will strain relations with every continental power.",
            trigger: EventTrigger::DateReached { year: 1806, month: 11, day: 21 },
            options: vec![
                EventOption {
                    label: "Enforce Strictly",
                    effects: vec![
                        EventEffect::RelationChange { power: gbr(), delta: -30 },
                        EventEffect::RelationChange { power: aus(), delta: -10 },
                        EventEffect::RelationChange { power: pru(), delta: -10 },
                        EventEffect::RelationChange { power: rus(), delta: -10 },
                        EventEffect::TreasuryChange(20),
                    ],
                },
                EventOption {
                    label: "Partial Enforcement",
                    effects: vec![
                        EventEffect::RelationChange { power: gbr(), delta: -15 },
                        EventEffect::TreasuryChange(5),
                    ],
                },
            ],
            fires_for: fra(),
            repeatable: false,
            fired: false,
        },
        // 3. Tilsit Negotiations
        HistoricalEvent {
            id: 3,
            title: "Tilsit Negotiations",
            description: "On a raft in the middle of the Niemen River, Napoleon meets Tsar \
                Alexander I. The two emperors discuss the future of Europe — a historic moment \
                that could reshape the continental balance of power.",
            trigger: EventTrigger::DateReached { year: 1807, month: 6, day: 25 },
            options: vec![
                EventOption {
                    label: "Generous Terms",
                    effects: vec![
                        EventEffect::RelationChange { power: rus(), delta: 30 },
                        EventEffect::PeaceOffer(rus()),
                    ],
                },
                EventOption {
                    label: "Harsh Terms",
                    effects: vec![
                        EventEffect::RelationChange { power: rus(), delta: -10 },
                        EventEffect::ManpowerChange(10),
                    ],
                },
            ],
            fires_for: fra(),
            repeatable: false,
            fired: false,
        },
        // 4. Spain Succession Crisis
        HistoricalEvent {
            id: 4,
            title: "Spain Succession Crisis",
            description: "The Bourbon dynasty in Spain is in turmoil. Napoleon sees an opportunity \
                to extend French influence over the Iberian Peninsula, but installing his brother \
                Joseph on the Spanish throne risks igniting fierce resistance.",
            trigger: EventTrigger::DateReached { year: 1808, month: 3, day: 15 },
            options: vec![
                EventOption {
                    label: "Place Joseph on Throne",
                    effects: vec![
                        EventEffect::RelationChange { power: spa(), delta: -40 },
                        EventEffect::DeclareWar(spa()),
                        EventEffect::ManpowerChange(-5),
                    ],
                },
                EventOption {
                    label: "Support Spanish King",
                    effects: vec![
                        EventEffect::RelationChange { power: spa(), delta: 15 },
                    ],
                },
            ],
            fires_for: fra(),
            repeatable: false,
            fired: false,
        },
        // 5. The Grande Armée Marches East
        HistoricalEvent {
            id: 5,
            title: "The Grande Armée Marches East",
            description: "Six hundred thousand men cross the Niemen into Russian territory — the \
                largest army Europe has ever seen. The fate of the Empire hangs on the coming \
                campaign against the vast Russian interior.",
            trigger: EventTrigger::DateReached { year: 1812, month: 6, day: 24 },
            options: vec![
                EventOption {
                    label: "Full Invasion",
                    effects: vec![
                        EventEffect::ManpowerChange(-20),
                        EventEffect::DeclareWar(rus()),
                        EventEffect::NewsMessage("The Grande Armée invades Russia!".into()),
                    ],
                },
                EventOption {
                    label: "Probe Only",
                    effects: vec![
                        EventEffect::ManpowerChange(-5),
                        EventEffect::RelationChange { power: rus(), delta: -20 },
                    ],
                },
            ],
            fires_for: fra(),
            repeatable: false,
            fired: false,
        },
        // 6. Trafalgar Aftermath
        HistoricalEvent {
            id: 6,
            title: "Trafalgar Aftermath",
            description: "The combined Franco-Spanish fleet has been shattered off Cape Trafalgar. \
                Admiral Villeneuve is captured, and French naval power lies broken. The Emperor \
                must decide how to respond to this catastrophe at sea.",
            trigger: EventTrigger::DateReached { year: 1805, month: 10, day: 21 },
            options: vec![
                EventOption {
                    label: "Rebuild the Fleet",
                    effects: vec![
                        EventEffect::TreasuryChange(-30),
                        EventEffect::NewsMessage("France begins massive naval rebuilding program.".into()),
                    ],
                },
                EventOption {
                    label: "Accept Naval Inferiority",
                    effects: vec![
                        EventEffect::ManpowerChange(10),
                        EventEffect::NewsMessage("France focuses resources on the army.".into()),
                    ],
                },
            ],
            fires_for: fra(),
            repeatable: false,
            fired: false,
        },
        // 7. Marshals Demand Pay
        HistoricalEvent {
            id: 7,
            title: "Marshals Demand Pay",
            description: "Several of your most senior marshals have presented themselves at court, \
                demanding their promised estates and back pay. Their loyalty is not in question — \
                yet — but their patience wears thin.",
            trigger: EventTrigger::AlwaysTrue,
            options: vec![
                EventOption {
                    label: "Pay Them",
                    effects: vec![
                        EventEffect::TreasuryChange(-20),
                        EventEffect::NewsMessage("The marshals are satisfied with their rewards.".into()),
                    ],
                },
                EventOption {
                    label: "Deny Payment",
                    effects: vec![
                        EventEffect::ManpowerChange(-3),
                        EventEffect::NewsMessage("Disgruntled marshals mutter in the corridors.".into()),
                    ],
                },
            ],
            fires_for: fra(),
            repeatable: false,
            fired: false,
        },
        // 8. Conscription Resistance
        HistoricalEvent {
            id: 8,
            title: "Conscription Resistance",
            description: "Reports pour in from the provinces: young men are fleeing to the hills \
                rather than answer the call to arms. The prefects demand stronger enforcement, \
                while nobles petition for exemptions.",
            trigger: EventTrigger::AlwaysTrue,
            options: vec![
                EventOption {
                    label: "Enforce Conscription",
                    effects: vec![
                        EventEffect::ManpowerChange(15),
                        EventEffect::TreasuryChange(-5),
                    ],
                },
                EventOption {
                    label: "Exempt Nobles",
                    effects: vec![
                        EventEffect::ManpowerChange(5),
                        EventEffect::TreasuryChange(5),
                    ],
                },
            ],
            fires_for: fra(),
            repeatable: false,
            fired: false,
        },

        // ── BRITAIN ─────────────────────────────────────────────────
        // 9. Trafalgar Victory
        HistoricalEvent {
            id: 9,
            title: "Trafalgar Victory",
            description: "The Royal Navy has won a decisive victory off Cape Trafalgar, shattering \
                the Franco-Spanish fleet. But the triumph is bittersweet — Admiral Lord Nelson \
                has fallen on the deck of HMS Victory.",
            trigger: EventTrigger::DateReached { year: 1805, month: 10, day: 21 },
            options: vec![
                EventOption {
                    label: "Honor Nelson",
                    effects: vec![
                        EventEffect::ManpowerChange(5),
                        EventEffect::TreasuryChange(-10),
                        EventEffect::NewsMessage("The nation mourns Nelson and celebrates victory.".into()),
                    ],
                },
                EventOption {
                    label: "Focus on Victory",
                    effects: vec![
                        EventEffect::TreasuryChange(10),
                        EventEffect::NewsMessage("Britannia rules the waves!".into()),
                    ],
                },
            ],
            fires_for: gbr(),
            repeatable: false,
            fired: false,
        },
        // 10. Subsidize Coalition
        HistoricalEvent {
            id: 10,
            title: "Subsidize the Coalition",
            description: "Parliament debates funding for Britain's continental allies. Gold shipped \
                to Vienna and Berlin could keep armies in the field against Napoleon — but the \
                Treasury warns of mounting debts from the endless war.",
            trigger: EventTrigger::AlwaysTrue,
            options: vec![
                EventOption {
                    label: "Fund Austria & Prussia",
                    effects: vec![
                        EventEffect::TreasuryChange(-25),
                        EventEffect::RelationChange { power: aus(), delta: 20 },
                        EventEffect::RelationChange { power: pru(), delta: 20 },
                    ],
                },
                EventOption {
                    label: "Save the Money",
                    effects: vec![
                        EventEffect::TreasuryChange(10),
                        EventEffect::RelationChange { power: aus(), delta: -5 },
                    ],
                },
            ],
            fires_for: gbr(),
            repeatable: false,
            fired: false,
        },
        // 11. Orders in Council
        HistoricalEvent {
            id: 11,
            title: "Orders in Council",
            description: "The Cabinet issues the Orders in Council, declaring a blockade of all \
                ports from which the British flag is excluded. Neutral shipping must submit to \
                Royal Navy inspection or face seizure.",
            trigger: EventTrigger::DateReached { year: 1807, month: 1, day: 7 },
            options: vec![
                EventOption {
                    label: "Strict Blockade",
                    effects: vec![
                        EventEffect::TreasuryChange(15),
                        EventEffect::RelationChange { power: fra(), delta: -20 },
                    ],
                },
                EventOption {
                    label: "Lenient Enforcement",
                    effects: vec![
                        EventEffect::TreasuryChange(5),
                        EventEffect::RelationChange { power: fra(), delta: -5 },
                    ],
                },
            ],
            fires_for: gbr(),
            repeatable: false,
            fired: false,
        },
        // 12. Peninsula Opportunity
        HistoricalEvent {
            id: 12,
            title: "Peninsula Opportunity",
            description: "Spain is in open revolt against French occupation. The Iberian Peninsula \
                offers a second front — a chance to bleed Napoleon's armies in a brutal guerrilla \
                war. Wellington stands ready with an expeditionary force.",
            trigger: EventTrigger::DateReached { year: 1808, month: 8, day: 1 },
            options: vec![
                EventOption {
                    label: "Send Wellington",
                    effects: vec![
                        EventEffect::ManpowerChange(-10),
                        EventEffect::TreasuryChange(-15),
                        EventEffect::RelationChange { power: spa(), delta: 25 },
                        EventEffect::AddTrait(
                            MarshalId::from("MARSHAL_WELLINGTON"),
                            MarshalTrait::Tactician,
                        ),
                    ],
                },
                EventOption {
                    label: "Support from Sea Only",
                    effects: vec![
                        EventEffect::TreasuryChange(-5),
                        EventEffect::RelationChange { power: spa(), delta: 10 },
                    ],
                },
            ],
            fires_for: gbr(),
            repeatable: false,
            fired: false,
        },

        // ── RUSSIA ──────────────────────────────────────────────────
        // 13. Kutuzov Takes Command
        HistoricalEvent {
            id: 13,
            title: "Kutuzov Takes Command",
            description: "The Tsar's advisors urge him to replace the cautious Barclay de Tolly \
                with the aging but beloved Mikhail Kutuzov. The one-eyed veteran promises to \
                stop Napoleon — through patience, attrition, and Russian winter.",
            trigger: EventTrigger::DateReached { year: 1812, month: 8, day: 29 },
            options: vec![
                EventOption {
                    label: "Appoint Kutuzov",
                    effects: vec![
                        EventEffect::AddTrait(
                            MarshalId::from("MARSHAL_KUTUZOV"),
                            MarshalTrait::DefensiveGenius,
                        ),
                        EventEffect::ManpowerChange(5),
                        EventEffect::NewsMessage("Kutuzov takes supreme command of Russian forces.".into()),
                    ],
                },
                EventOption {
                    label: "Keep Barclay",
                    effects: vec![
                        EventEffect::AddTrait(
                            MarshalId::from("MARSHAL_BARCLAY"),
                            MarshalTrait::Logistics,
                        ),
                    ],
                },
            ],
            fires_for: rus(),
            repeatable: false,
            fired: false,
        },
        // 14. Scorched Earth Policy
        HistoricalEvent {
            id: 14,
            title: "Scorched Earth Policy",
            description: "As Napoleon's Grande Armée advances deeper into Russia, the generals \
                propose a terrible strategy: burn everything. Leave nothing for the enemy — \
                no food, no shelter, no forage. Mother Russia herself will be the weapon.",
            trigger: EventTrigger::AlwaysTrue,
            options: vec![
                EventOption {
                    label: "Burn Everything",
                    effects: vec![
                        EventEffect::ManpowerChange(-10),
                        EventEffect::TreasuryChange(-15),
                        EventEffect::NewsMessage("Russian forces adopt scorched earth tactics.".into()),
                    ],
                },
                EventOption {
                    label: "Hold Ground",
                    effects: vec![
                        EventEffect::ManpowerChange(-5),
                    ],
                },
            ],
            fires_for: rus(),
            repeatable: false,
            fired: false,
        },
        // 15. Peace Party vs War Party
        HistoricalEvent {
            id: 15,
            title: "Peace Party vs War Party",
            description: "The Russian court is split. The peace faction, led by influential nobles, \
                argues that accommodation with Napoleon is the only rational course. The war \
                party demands Russia stand firm against the Corsican tyrant.",
            trigger: EventTrigger::AlwaysTrue,
            options: vec![
                EventOption {
                    label: "Negotiate with Napoleon",
                    effects: vec![
                        EventEffect::RelationChange { power: fra(), delta: 20 },
                        EventEffect::PeaceOffer(fra()),
                    ],
                },
                EventOption {
                    label: "Fight On",
                    effects: vec![
                        EventEffect::ManpowerChange(10),
                        EventEffect::RelationChange { power: fra(), delta: -15 },
                    ],
                },
            ],
            fires_for: rus(),
            repeatable: false,
            fired: false,
        },

        // ── AUSTRIA ─────────────────────────────────────────────────
        // 16. Austerlitz Aftermath
        HistoricalEvent {
            id: 16,
            title: "Austerlitz Aftermath",
            description: "The Battle of the Three Emperors has ended in catastrophe. The Austrian \
                and Russian armies are shattered, and Napoleon stands triumphant. The Habsburg \
                Empire must now choose between humiliating peace and desperate resistance.",
            trigger: EventTrigger::DateReached { year: 1805, month: 12, day: 2 },
            options: vec![
                EventOption {
                    label: "Seek Peace",
                    effects: vec![
                        EventEffect::PeaceOffer(fra()),
                        EventEffect::TreasuryChange(-20),
                        EventEffect::RelationChange { power: fra(), delta: 15 },
                    ],
                },
                EventOption {
                    label: "Fight On",
                    effects: vec![
                        EventEffect::ManpowerChange(10),
                        EventEffect::TreasuryChange(-10),
                    ],
                },
            ],
            fires_for: aus(),
            repeatable: false,
            fired: false,
        },
        // 17. Habsburg Marriage Alliance
        HistoricalEvent {
            id: 17,
            title: "Habsburg Marriage Alliance",
            description: "Napoleon seeks to legitimize his dynasty through marriage to Archduchess \
                Marie Louise of Austria. The Habsburgs face a bitter choice: sacrifice a princess \
                for peace, or refuse the upstart Emperor and risk renewed war.",
            trigger: EventTrigger::DateReached { year: 1809, month: 10, day: 14 },
            options: vec![
                EventOption {
                    label: "Accept the Marriage",
                    effects: vec![
                        EventEffect::RelationChange { power: fra(), delta: 30 },
                        EventEffect::PeaceOffer(fra()),
                        EventEffect::NewsMessage("Marie Louise weds Napoleon — a new alliance.".into()),
                    ],
                },
                EventOption {
                    label: "Refuse",
                    effects: vec![
                        EventEffect::RelationChange { power: fra(), delta: -20 },
                        EventEffect::ManpowerChange(5),
                    ],
                },
            ],
            fires_for: aus(),
            repeatable: false,
            fired: false,
        },

        // ── PRUSSIA ─────────────────────────────────────────────────
        // 18. Jena Disaster
        HistoricalEvent {
            id: 18,
            title: "Jena Disaster",
            description: "The twin battles of Jena and Auerstedt have annihilated the Prussian \
                army. In a single day, the proud military legacy of Frederick the Great has been \
                swept away. Berlin lies open to French occupation.",
            trigger: EventTrigger::DateReached { year: 1806, month: 10, day: 14 },
            options: vec![
                EventOption {
                    label: "Capitulate",
                    effects: vec![
                        EventEffect::PeaceOffer(fra()),
                        EventEffect::TreasuryChange(-25),
                        EventEffect::ManpowerChange(-15),
                    ],
                },
                EventOption {
                    label: "Guerrilla War",
                    effects: vec![
                        EventEffect::ManpowerChange(-5),
                        EventEffect::RelationChange { power: fra(), delta: -20 },
                        EventEffect::NewsMessage("Prussian partisans take to the hills.".into()),
                    ],
                },
            ],
            fires_for: pru(),
            repeatable: false,
            fired: false,
        },
        // 19. Reform Movement
        HistoricalEvent {
            id: 19,
            title: "Reform Movement",
            description: "Scharnhorst, Gneisenau, and the reformers present their plan to modernize \
                the Prussian military. They propose abolishing the old caste system, introducing \
                universal conscription, and training a new generation of officers.",
            trigger: EventTrigger::AlwaysTrue,
            options: vec![
                EventOption {
                    label: "Accept Modernization",
                    effects: vec![
                        EventEffect::ManpowerChange(10),
                        EventEffect::TreasuryChange(-10),
                        EventEffect::NewsMessage("Prussia embarks on sweeping military reforms.".into()),
                    ],
                },
                EventOption {
                    label: "Keep Old Ways",
                    effects: vec![
                        EventEffect::TreasuryChange(5),
                    ],
                },
            ],
            fires_for: pru(),
            repeatable: false,
            fired: false,
        },

        // ── ALL POWERS (misc) ───────────────────────────────────────
        // 20–25: These fire for FRA by default, but are designed as generic.
        // In a full implementation each would be duplicated per power.

        // 20. Plague Outbreak
        HistoricalEvent {
            id: 20,
            title: "Plague Outbreak",
            description: "A virulent plague has broken out among the camp followers and is spreading \
                to the ranks. Army surgeons are overwhelmed, and the stench of the field hospitals \
                carries for miles. Decisive action is needed.",
            trigger: EventTrigger::AlwaysTrue,
            options: vec![
                EventOption {
                    label: "Quarantine",
                    effects: vec![
                        EventEffect::ManpowerChange(-8),
                        EventEffect::NewsMessage("Strict quarantine measures slow the plague.".into()),
                    ],
                },
                EventOption {
                    label: "Ignore It",
                    effects: vec![
                        EventEffect::ManpowerChange(-3),
                        EventEffect::TreasuryChange(-5),
                    ],
                },
            ],
            fires_for: fra(),
            repeatable: false,
            fired: false,
        },
        // 21. Harvest Failure
        HistoricalEvent {
            id: 21,
            title: "Harvest Failure",
            description: "A wet summer followed by early frost has devastated the harvest across \
                the countryside. Bread prices soar in the cities, and the army's commissariat \
                reports dwindling stores. Hunger stalks the land.",
            trigger: EventTrigger::AlwaysTrue,
            options: vec![
                EventOption {
                    label: "Import Grain",
                    effects: vec![
                        EventEffect::TreasuryChange(-15),
                        EventEffect::NewsMessage("Emergency grain imports stabilize food supply.".into()),
                    ],
                },
                EventOption {
                    label: "Requisition from Populace",
                    effects: vec![
                        EventEffect::ManpowerChange(-5),
                        EventEffect::TreasuryChange(-5),
                    ],
                },
            ],
            fires_for: gbr(),
            repeatable: false,
            fired: false,
        },
        // 22. Noble Conspiracy
        HistoricalEvent {
            id: 22,
            title: "Noble Conspiracy",
            description: "Your intelligence service has uncovered a conspiracy among disaffected \
                nobles. They plot to overthrow the current government and install a regent more \
                amenable to their interests. The evidence is damning.",
            trigger: EventTrigger::AlwaysTrue,
            options: vec![
                EventOption {
                    label: "Purge the Nobles",
                    effects: vec![
                        EventEffect::TreasuryChange(10),
                        EventEffect::ManpowerChange(-3),
                    ],
                },
                EventOption {
                    label: "Appease Them",
                    effects: vec![
                        EventEffect::TreasuryChange(-10),
                        EventEffect::ManpowerChange(2),
                    ],
                },
            ],
            fires_for: rus(),
            repeatable: false,
            fired: false,
        },
        // 23. Foreign Volunteers
        HistoricalEvent {
            id: 23,
            title: "Foreign Volunteers",
            description: "Word of your cause has spread abroad, and foreign volunteers are arriving \
                at the borders offering their services. Some are seasoned veterans, others are \
                idealistic youths. All seek glory under your banner.",
            trigger: EventTrigger::AlwaysTrue,
            options: vec![
                EventOption {
                    label: "Accept Volunteers",
                    effects: vec![
                        EventEffect::ManpowerChange(8),
                        EventEffect::TreasuryChange(-5),
                    ],
                },
                EventOption {
                    label: "Refuse Them",
                    effects: vec![
                        EventEffect::TreasuryChange(2),
                    ],
                },
            ],
            fires_for: aus(),
            repeatable: false,
            fired: false,
        },
        // 24. Winter Logistics Crisis
        HistoricalEvent {
            id: 24,
            title: "Winter Logistics Crisis",
            description: "Winter has arrived with a vengeance, and supply wagons are mired in frozen \
                mud. Horses collapse from exhaustion and cold, soldiers strip the dead for warm \
                clothing, and the quartermasters report critical shortages.",
            trigger: EventTrigger::AlwaysTrue,
            options: vec![
                EventOption {
                    label: "Prioritize Supply",
                    effects: vec![
                        EventEffect::TreasuryChange(-10),
                        EventEffect::NewsMessage("Supply lines reorganized for winter operations.".into()),
                    ],
                },
                EventOption {
                    label: "Push On",
                    effects: vec![
                        EventEffect::ManpowerChange(-8),
                        EventEffect::NewsMessage("The army presses forward through the bitter cold.".into()),
                    ],
                },
            ],
            fires_for: pru(),
            repeatable: false,
            fired: false,
        },
        // 25. War Exhaustion
        HistoricalEvent {
            id: 25,
            title: "War Exhaustion",
            description: "Years of ceaseless conflict have taken their toll. The populace grows \
                weary of war, mothers curse the recruiting sergeants, and even veteran soldiers \
                speak openly of peace. Something must give.",
            trigger: EventTrigger::AlwaysTrue,
            options: vec![
                EventOption {
                    label: "Seek Peace",
                    effects: vec![
                        EventEffect::ManpowerChange(5),
                        EventEffect::NewsMessage("Peace feelers are quietly extended.".into()),
                    ],
                },
                EventOption {
                    label: "Propaganda Campaign",
                    effects: vec![
                        EventEffect::TreasuryChange(-15),
                        EventEffect::ManpowerChange(3),
                        EventEffect::NewsMessage("Patriotic fervor is rekindled through propaganda.".into()),
                    ],
                },
            ],
            fires_for: fra(),
            repeatable: false,
            fired: false,
        },
    ]
}

// ── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_loads_25_events() {
        let reg = EventRegistry::with_historical();
        assert_eq!(reg.events.len(), 25);
    }

    #[test]
    fn all_events_have_options() {
        let reg = EventRegistry::with_historical();
        for (id, event) in &reg.events {
            assert!(
                !event.options.is_empty(),
                "event {id} '{}' has no options",
                event.title
            );
        }
    }

    #[test]
    fn date_trigger_fires_on_exact_date() {
        let mut reg = EventRegistry::new();
        reg.add(HistoricalEvent {
            id: 100,
            title: "Test Date Event",
            description: "Test",
            trigger: EventTrigger::DateReached { year: 1805, month: 10, day: 21 },
            options: vec![EventOption {
                label: "OK",
                effects: vec![EventEffect::TreasuryChange(10)],
            }],
            fires_for: fra(),
            repeatable: false,
            fired: false,
        });
        let pending = reg.advance_triggers(1805, 10, 21);
        assert_eq!(pending, vec![100]);
    }

    #[test]
    fn date_trigger_does_not_fire_before_date() {
        let mut reg = EventRegistry::new();
        reg.add(HistoricalEvent {
            id: 101,
            title: "Future Event",
            description: "Test",
            trigger: EventTrigger::DateReached { year: 1810, month: 6, day: 1 },
            options: vec![EventOption {
                label: "OK",
                effects: vec![],
            }],
            fires_for: fra(),
            repeatable: false,
            fired: false,
        });
        let pending = reg.advance_triggers(1805, 1, 1);
        assert!(pending.is_empty());
    }

    #[test]
    fn always_true_fires_immediately() {
        let mut reg = EventRegistry::new();
        reg.add(HistoricalEvent {
            id: 102,
            title: "Always Event",
            description: "Test",
            trigger: EventTrigger::AlwaysTrue,
            options: vec![EventOption {
                label: "OK",
                effects: vec![],
            }],
            fires_for: fra(),
            repeatable: false,
            fired: false,
        });
        let pending = reg.advance_triggers(1805, 1, 1);
        assert_eq!(pending, vec![102]);
    }

    #[test]
    fn fired_event_does_not_fire_again() {
        let mut reg = EventRegistry::new();
        reg.add(HistoricalEvent {
            id: 103,
            title: "Once Only",
            description: "Test",
            trigger: EventTrigger::AlwaysTrue,
            options: vec![EventOption {
                label: "OK",
                effects: vec![EventEffect::TreasuryChange(5)],
            }],
            fires_for: fra(),
            repeatable: false,
            fired: false,
        });
        let _ = reg.resolve(103, 0).unwrap();
        let pending = reg.advance_triggers(1805, 1, 1);
        assert!(pending.is_empty());
    }

    #[test]
    fn repeatable_event_fires_again() {
        let mut reg = EventRegistry::new();
        reg.add(HistoricalEvent {
            id: 104,
            title: "Repeating",
            description: "Test",
            trigger: EventTrigger::AlwaysTrue,
            options: vec![EventOption {
                label: "OK",
                effects: vec![],
            }],
            fires_for: fra(),
            repeatable: true,
            fired: false,
        });
        let _ = reg.resolve(104, 0).unwrap();
        let pending = reg.advance_triggers(1805, 1, 1);
        assert_eq!(pending, vec![104]);
    }

    #[test]
    fn resolve_returns_effects() {
        let mut reg = EventRegistry::new();
        reg.add(HistoricalEvent {
            id: 105,
            title: "Effects Test",
            description: "Test",
            trigger: EventTrigger::AlwaysTrue,
            options: vec![
                EventOption {
                    label: "Option A",
                    effects: vec![EventEffect::TreasuryChange(100)],
                },
                EventOption {
                    label: "Option B",
                    effects: vec![EventEffect::ManpowerChange(-5)],
                },
            ],
            fires_for: fra(),
            repeatable: false,
            fired: false,
        });
        let effects = reg.resolve(105, 1).unwrap();
        assert_eq!(effects.len(), 1);
        assert_eq!(effects[0], EventEffect::ManpowerChange(-5));
    }

    #[test]
    fn resolve_invalid_option_returns_error() {
        let mut reg = EventRegistry::new();
        reg.add(HistoricalEvent {
            id: 106,
            title: "Bounds Test",
            description: "Test",
            trigger: EventTrigger::AlwaysTrue,
            options: vec![EventOption {
                label: "Only",
                effects: vec![],
            }],
            fires_for: fra(),
            repeatable: false,
            fired: false,
        });
        assert!(reg.resolve(106, 5).is_err());
    }

    #[test]
    fn resolve_unknown_event_returns_error() {
        let mut reg = EventRegistry::new();
        assert!(reg.resolve(999, 0).is_err());
    }

    #[test]
    fn pending_events_json_filters_by_power() {
        let reg = EventRegistry::with_historical();
        let all_ids: Vec<EventId> = reg.events.keys().copied().collect();
        let fra_json = reg.pending_events_json(&fra(), &all_ids);
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&fra_json).unwrap();
        // All returned events should be for FRA
        for ev in &parsed {
            assert_eq!(ev["firesFor"].as_str().unwrap(), "FRA");
        }
        assert!(!parsed.is_empty());
    }

    #[test]
    fn territory_trigger_fires_on_loss() {
        let mut reg = EventRegistry::new();
        let area = AreaId::from("AREA_PARIS");
        reg.add(HistoricalEvent {
            id: 200,
            title: "Paris Lost",
            description: "Test",
            trigger: EventTrigger::TerritoryLost(area.clone()),
            options: vec![EventOption {
                label: "OK",
                effects: vec![],
            }],
            fires_for: fra(),
            repeatable: false,
            fired: false,
        });
        let pending = reg.check_territory_trigger(&area, false);
        assert_eq!(pending, vec![200]);
        // Gained should not trigger loss
        let pending2 = reg.check_territory_trigger(&area, true);
        assert!(pending2.is_empty());
    }

    #[test]
    fn effect_summary_formatting() {
        assert_eq!(effect_summary(&EventEffect::TreasuryChange(10)), "+10 treasury");
        assert_eq!(effect_summary(&EventEffect::TreasuryChange(-5)), "-5 treasury");
        assert_eq!(effect_summary(&EventEffect::ManpowerChange(3)), "+3 manpower");
        assert_eq!(
            effect_summary(&EventEffect::RelationChange {
                power: gbr(),
                delta: -20,
            }),
            "-20 relations with GBR"
        );
    }

    #[test]
    fn date_trigger_fires_after_date() {
        let mut reg = EventRegistry::new();
        reg.add(HistoricalEvent {
            id: 201,
            title: "Past Event",
            description: "Test",
            trigger: EventTrigger::DateReached { year: 1805, month: 1, day: 1 },
            options: vec![EventOption {
                label: "OK",
                effects: vec![],
            }],
            fires_for: fra(),
            repeatable: false,
            fired: false,
        });
        // Later year
        let pending = reg.advance_triggers(1806, 3, 15);
        assert_eq!(pending, vec![201]);
    }
}
