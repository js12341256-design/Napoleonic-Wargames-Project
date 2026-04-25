//! xtask — workspace automation.
//!
//! Subcommands implemented:
//!
//! - `dump-schemas <out_dir>` — emits a JSON Schema for every persisted
//!   root type (`Scenario` for now) to `<out_dir>/<name>.schema.json`.

use std::path::PathBuf;

use schemars::schema_for;

fn main() {
    let mut args = std::env::args().skip(1);
    let cmd = args.next().unwrap_or_default();
    match cmd.as_str() {
        "dump-schemas" => {
            let out: PathBuf = args
                .next()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("data/schemas"));
            dump_schemas(&out).unwrap_or_else(|e| {
                eprintln!("dump-schemas failed: {e}");
                std::process::exit(1);
            });
        }
        "" | "help" | "--help" | "-h" => {
            print_help();
        }
        other => {
            eprintln!("xtask: unknown subcommand `{other}`");
            print_help();
            std::process::exit(2);
        }
    }
}

fn print_help() {
    println!(
        "xtask — Grand Campaign 1805 build helpers\n\
\n\
USAGE:\n\
    xtask <SUBCOMMAND>\n\
\n\
SUBCOMMANDS:\n\
    dump-schemas <out_dir>   Write JSON Schema files for persisted roots.\n\
    help                     Show this message."
    );
}

fn dump_schemas(out_dir: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(out_dir)?;

    let scenario = schema_for!(gc1805_core_schema::scenario::Scenario);
    write(out_dir, "scenario.schema.json", &scenario)?;

    let combat = schema_for!(gc1805_core_schema::tables::CombatTable);
    write(out_dir, "combat.schema.json", &combat)?;

    let attrition = schema_for!(gc1805_core_schema::tables::AttritionTable);
    write(out_dir, "attrition.schema.json", &attrition)?;

    let weather = schema_for!(gc1805_core_schema::tables::WeatherTable);
    write(out_dir, "weather.schema.json", &weather)?;

    let economy = schema_for!(gc1805_core_schema::tables::EconomyTable);
    write(out_dir, "economy.schema.json", &economy)?;

    let pp = schema_for!(gc1805_core_schema::tables::PpModifiersTable);
    write(out_dir, "pp_modifiers.schema.json", &pp)?;

    let leader_cas = schema_for!(gc1805_core_schema::tables::LeaderCasualtyTable);
    write(out_dir, "leader_casualty.schema.json", &leader_cas)?;

    let morale = schema_for!(gc1805_core_schema::tables::MoraleTable);
    write(out_dir, "morale.schema.json", &morale)?;

    let naval = schema_for!(gc1805_core_schema::tables::NavalCombatTable);
    write(out_dir, "naval_combat.schema.json", &naval)?;

    let minor_act = schema_for!(gc1805_core_schema::tables::MinorActivationTable);
    write(out_dir, "minor_activation.schema.json", &minor_act)?;

    println!("Wrote 10 schema files to {}", out_dir.display());
    Ok(())
}

fn write<T: serde::Serialize>(dir: &std::path::Path, name: &str, value: &T) -> std::io::Result<()> {
    let s = serde_json::to_string_pretty(value).map_err(std::io::Error::other)?;
    let path = dir.join(name);
    std::fs::write(&path, s)?;
    println!("  {}", path.display());
    Ok(())
}
