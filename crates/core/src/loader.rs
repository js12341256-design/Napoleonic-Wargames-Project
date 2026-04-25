//! Scenario loader.
//!
//! Reads a `Scenario` from JSON, runs structural validation, and
//! returns the loaded value plus a [`LoadReport`] listing any
//! placeholders the designer still owes us (PROMPT.md §6.1).

use gc1805_core_schema::{Scenario, SchemaError, SCHEMA_VERSION};
use serde_json::Value;

use crate::validate::{validate_scenario, IntegrityIssue};

/// Diagnostic record produced by [`load_scenario_str`].  A scenario
/// can be loaded successfully even with placeholders or other
/// integrity issues; the report tells the caller what to surface.
#[derive(Debug, Clone, Default)]
pub struct LoadReport {
    /// Dotted JSON paths where a `{"_placeholder": true}` value was
    /// encountered.  Non-empty implies the scenario is
    /// `unplayable_in_release`.
    pub placeholder_paths: Vec<String>,
    /// Integrity findings (dangling references, non-symmetric
    /// adjacency, etc.) reported by [`validate_scenario`].
    pub integrity: Vec<IntegrityIssue>,
}

impl LoadReport {
    pub fn is_clean(&self) -> bool {
        self.placeholder_paths.is_empty() && self.integrity.is_empty()
    }
}

/// Errors that prevent loading entirely.
#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("schema error: {0}")]
    Schema(#[from] SchemaError),
}

/// Load a scenario from a JSON string.
///
/// Always returns the parsed scenario plus a report when JSON parsing
/// succeeds.  Hard errors come from JSON malformation, schema-version
/// mismatches, or values whose types disagree with the Rust schema.
pub fn load_scenario_str(json: &str) -> Result<(Scenario, LoadReport), LoadError> {
    // First pass: parse to `Value` so we can scan for placeholders
    // before serde possibly drops the markers.
    let raw: Value = serde_json::from_str(json)?;

    // Schema-version check happens against the raw payload so that
    // future migrations have a clean place to hook in.
    let found_version = raw
        .get("schema_version")
        .and_then(Value::as_u64)
        .ok_or_else(|| {
            LoadError::Schema(SchemaError::InvalidValue {
                path: "schema_version".into(),
                reason: "missing or non-integer".into(),
            })
        })?;
    if (found_version as u32) != SCHEMA_VERSION {
        return Err(LoadError::Schema(SchemaError::SchemaVersion {
            found: found_version as u32,
            min: SCHEMA_VERSION,
            max: SCHEMA_VERSION,
        }));
    }

    let mut report = LoadReport::default();
    scan_placeholders(&raw, "$", &mut report.placeholder_paths);

    // Second pass: typed deserialization.
    let mut scenario: Scenario = serde_json::from_value(raw)?;

    // Initialise the live PowerState from PowerSetup.starting_* if not
    // already present in the JSON.  This keeps the scenario file
    // authoring-friendly: scenario authors only write the immutable
    // setup; the mutable state defaults from it.
    initialize_power_state(&mut scenario);

    // Structural validation.
    report.integrity = validate_scenario(&scenario);

    Ok((scenario, report))
}

fn initialize_power_state(s: &mut Scenario) {
    use gc1805_core_schema::scenario::{PowerState, TaxPolicy};
    for (id, setup) in &s.powers {
        s.power_state
            .entry(id.clone())
            .or_insert_with(|| PowerState {
                treasury: setup.starting_treasury,
                manpower: setup.starting_manpower,
                prestige: setup.starting_pp,
                tax_policy: TaxPolicy::Standard,
            });
    }
}

fn scan_placeholders(value: &Value, path: &str, out: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            if map
                .get("_placeholder")
                .and_then(Value::as_bool)
                .unwrap_or(false)
                && map.len() == 1
            {
                out.push(path.to_owned());
                return;
            }
            for (k, v) in map {
                let next = format!("{path}.{k}");
                scan_placeholders(v, &next, out);
            }
        }
        Value::Array(arr) => {
            for (i, v) in arr.iter().enumerate() {
                let next = format!("{path}[{i}]");
                scan_placeholders(v, &next, out);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholder_scan_finds_simple_marker() {
        let json = r#"{"x": {"_placeholder": true}}"#;
        let v: Value = serde_json::from_str(json).unwrap();
        let mut out = Vec::new();
        scan_placeholders(&v, "$", &mut out);
        assert_eq!(out, vec!["$.x".to_string()]);
    }

    #[test]
    fn placeholder_scan_finds_nested_markers() {
        let json = r#"{"a": [1, {"_placeholder": true}], "b": {"c": {"_placeholder": true}}}"#;
        let v: Value = serde_json::from_str(json).unwrap();
        let mut out = Vec::new();
        scan_placeholders(&v, "$", &mut out);
        out.sort();
        assert_eq!(out, vec!["$.a[1]".to_string(), "$.b.c".to_string()]);
    }

    #[test]
    fn schema_version_mismatch_errors() {
        let json = r#"{"schema_version": 9999}"#;
        let r = load_scenario_str(json);
        assert!(matches!(
            r,
            Err(LoadError::Schema(SchemaError::SchemaVersion {
                found: 9999,
                ..
            }))
        ));
    }
}
