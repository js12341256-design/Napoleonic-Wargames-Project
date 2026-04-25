//! Headless CLI for Grand Campaign 1805.
//!
//! Subcommands:
//!
//! - `load <scenario.json>` — parse, validate, print placeholder/integrity
//!   counts and the canonical-state hash.
//! - `move-all-to-capital <scenario.json>` — for every corps, attempt
//!   to issue a `Move` order toward its owning power's capital,
//!   honouring movement budgets.  Prints one line per resolution and
//!   the post-script state hash.  This is the §16.3 Phase 2 gate's
//!   "move every corps to capital" script.

use std::path::PathBuf;
use std::process::ExitCode;

use gc1805_core::economy::resolve_economic_phase;
use gc1805_core::movement::{MovementPlan, validate_or_reject};
use gc1805_core::orders::{MoveOrder, Order};
use gc1805_core::{MapGraph, load_scenario_str};
use gc1805_core_schema::canonical_hash;
use gc1805_core_schema::events::Event;
use gc1805_core_schema::ids::AreaId;
use gc1805_core_schema::tables::EconomyTable;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let cmd = args.next().unwrap_or_default();
    match cmd.as_str() {
        "load" => match args.next().map(PathBuf::from) {
            Some(p) => cmd_load(&p),
            None => usage_err(),
        },
        "move-all-to-capital" => match args.next().map(PathBuf::from) {
            Some(p) => cmd_move_all_to_capital(&p),
            None => usage_err(),
        },
        "economic-phase" => {
            match args.next().map(PathBuf::from) {
                Some(scenario_path) => {
                    // Optional --tables <path>
                    let tables_path: Option<PathBuf> = {
                        let flag = args.next();
                        if flag.as_deref() == Some("--tables") {
                            args.next().map(PathBuf::from)
                        } else {
                            None
                        }
                    };
                    cmd_economic_phase(&scenario_path, tables_path.as_deref())
                }
                None => usage_err(),
            }
        }
        "" | "help" | "--help" | "-h" => {
            print_help();
            ExitCode::SUCCESS
        }
        other => {
            eprintln!("gc1805: unknown subcommand `{other}`");
            print_help();
            ExitCode::from(2)
        }
    }
}

fn print_help() {
    println!(
        "gc1805 — headless runner\n\
\n\
USAGE:\n\
    gc1805 <SUBCOMMAND>\n\
\n\
SUBCOMMANDS:\n\
    load <scenario.json>                  Parse + validate; print state hash.\n\
    move-all-to-capital <scenario.json>   Try to march every corps home.\n\
    economic-phase <scenario.json>        Run one economic phase turn.\n\
             [--tables <economy.json>]\n\
    help                                  Show this message."
    );
}

fn usage_err() -> ExitCode {
    eprintln!("gc1805: missing scenario.json argument");
    print_help();
    ExitCode::from(2)
}

fn cmd_load(path: &std::path::Path) -> ExitCode {
    let json = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("read {}: {e}", path.display());
            return ExitCode::from(1);
        }
    };
    match load_scenario_str(&json) {
        Ok((scenario, report)) => {
            let hash = canonical_hash(&scenario).expect("canonical hash");
            println!("scenario_id     = {}", scenario.scenario_id);
            println!("schema_version  = {}", scenario.schema_version);
            println!("rules_version   = {}", scenario.rules_version);
            println!("powers          = {}", scenario.powers.len());
            println!("areas           = {}", scenario.areas.len());
            println!("corps           = {}", scenario.corps.len());
            println!("placeholders    = {}", report.placeholder_paths.len());
            println!("integrity_issues= {}", report.integrity.len());
            println!("state_hash      = {hash}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("load failed: {e}");
            ExitCode::from(1)
        }
    }
}

fn cmd_move_all_to_capital(path: &std::path::Path) -> ExitCode {
    let json = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("read {}: {e}", path.display());
            return ExitCode::from(1);
        }
    };
    let (mut scenario, _report) = match load_scenario_str(&json) {
        Ok(x) => x,
        Err(e) => {
            eprintln!("load failed: {e}");
            return ExitCode::from(1);
        }
    };

    let map = MapGraph::from_scenario(&scenario);

    // Deterministic iteration: BTreeMap iteration is sorted by CorpsId.
    let plan_input: Vec<(gc1805_core_schema::ids::CorpsId, AreaId, AreaId)> = scenario
        .corps
        .iter()
        .filter_map(|(id, c)| {
            let capital = scenario.powers.get(&c.owner).map(|p| p.capital.clone())?;
            Some((id.clone(), c.area.clone(), capital))
        })
        .collect();

    let mut events: Vec<Event> = Vec::new();
    for (corps_id, from, to) in plan_input {
        if from == to {
            // Already home — emit a Hold-equivalent event for the log.
            let order = Order::Hold(gc1805_core::orders::HoldOrder {
                submitter: scenario.corps.get(&corps_id).unwrap().owner.clone(),
                corps: corps_id.clone(),
            });
            let plan = match gc1805_core::movement::validate_order(&scenario, &order) {
                Ok(p) => p,
                Err(_) => continue,
            };
            let ev = gc1805_core::movement::resolve_order(&mut scenario, &order, plan);
            log_event(&corps_id, &ev);
            events.push(ev);
            continue;
        }
        // Issue a Move along the shortest hop path; if the budget
        // doesn't reach the capital this turn, march toward it on the
        // longest legal step we can take.
        let path = match map.shortest_path_hops(&from, &to) {
            Some(p) => p,
            None => {
                eprintln!("{corps_id}: no path {from} → {to}; skipping");
                continue;
            }
        };
        let budget = match &scenario.movement_rules.movement_hops_per_turn {
            gc1805_core_schema::tables::Maybe::Value(v) => *v as usize,
            gc1805_core_schema::tables::Maybe::Placeholder(_) => path.len() - 1, // walk all the way
        };
        let target_idx = (path.len() - 1).min(budget);
        let target = path[target_idx].clone();
        let owner = scenario.corps.get(&corps_id).unwrap().owner.clone();
        let order = Order::Move(MoveOrder {
            submitter: owner,
            corps: corps_id.clone(),
            to: target,
        });
        match validate_or_reject(&scenario, &order) {
            Ok(plan) => {
                let ev = match plan {
                    MovementPlan::Move { .. }
                    | MovementPlan::Hold { .. }
                    | MovementPlan::ForcedMarch { .. }
                    | MovementPlan::InterceptionQueued { .. } => {
                        gc1805_core::movement::resolve_order(&mut scenario, &order, plan)
                    }
                };
                log_event(&corps_id, &ev);
                events.push(ev);
            }
            Err(rej_event) => {
                if let Event::OrderRejected(r) = &rej_event {
                    println!("{corps_id}: REJECTED [{}] {}", r.reason_code, r.message);
                }
                events.push(rej_event);
            }
        }
    }

    let hash = canonical_hash(&scenario).expect("canonical hash");
    println!("---");
    println!("events  = {}", events.len());
    println!("hash    = {hash}");
    ExitCode::SUCCESS
}

fn cmd_economic_phase(
    scenario_path: &std::path::Path,
    tables_path: Option<&std::path::Path>,
) -> ExitCode {
    let json = match std::fs::read_to_string(scenario_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("read {}: {e}", scenario_path.display());
            return ExitCode::from(1);
        }
    };
    let (mut scenario, _report) = match load_scenario_str(&json) {
        Ok(x) => x,
        Err(e) => {
            eprintln!("load failed: {e}");
            return ExitCode::from(1);
        }
    };

    let tables: EconomyTable = if let Some(tp) = tables_path {
        let t_json = match std::fs::read_to_string(tp) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("read tables {}: {e}", tp.display());
                return ExitCode::from(1);
            }
        };
        match serde_json::from_str(&t_json) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("parse tables: {e}");
                return ExitCode::from(1);
            }
        }
    } else {
        EconomyTable::default()
    };

    let events = resolve_economic_phase(&mut scenario, &tables);

    // Print events as JSON array.
    match serde_json::to_string_pretty(&events) {
        Ok(s) => println!("{s}"),
        Err(e) => {
            eprintln!("serialize events: {e}");
            return ExitCode::from(1);
        }
    }

    // Print treasury per power.
    println!("---");
    println!("Treasury after economic phase:");
    for (power_id, ps) in &scenario.power_state {
        println!("  {power_id}: {}", ps.treasury);
    }

    ExitCode::SUCCESS
}

fn log_event(corps: &gc1805_core_schema::ids::CorpsId, ev: &Event) {
    match ev {
        Event::MovementResolved(m) => {
            println!("{corps}: MOVE {}→{} ({} hops)", m.from, m.to, m.hops);
        }
        Event::ForcedMarchResolved(m) => {
            println!(
                "{corps}: FORCED_MARCH {}→{} ({} hops, −{} morale_q4)",
                m.from, m.to, m.hops, m.morale_loss_q4
            );
        }
        Event::InterceptionQueued(q) => {
            println!("{corps}: INTERCEPTION_QUEUED → {}", q.target_area);
        }
        Event::OrderRejected(r) => {
            println!("{corps}: REJECTED [{}] {}", r.reason_code, r.message);
        }
        _ => {} // economic and other events are not corps-specific
    }
}
