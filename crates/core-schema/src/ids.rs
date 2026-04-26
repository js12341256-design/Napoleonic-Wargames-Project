//! Stable, namespaced string identifiers for scenario entities.
//!
//! PROMPT.md §5.1: IDs are `UPPER_SNAKE_CASE`, namespaced by entity type,
//! and immutable for the lifetime of the entity.  We model each kind as a
//! distinct newtype so the type system catches `area_id` vs `corps_id`
//! mix-ups before they corrupt state.
//!
//! All ID newtypes serialize transparently as a JSON string and implement
//! `Ord` for use in `BTreeMap` (deterministic iteration per §2.2).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

macro_rules! id_newtype {
    (
        $(#[$meta:meta])*
        $name:ident, $prefix:literal
    ) => {
        $(#[$meta])*
        #[derive(
            Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord,
            Serialize, Deserialize, JsonSchema,
        )]
        #[serde(transparent)]
        pub struct $name(pub String);

        impl $name {
            /// Returns the expected ID prefix for this kind.
            pub const PREFIX: &'static str = $prefix;

            /// Returns the inner string slice.
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl From<&str> for $name {
            fn from(s: &str) -> Self {
                Self(s.to_owned())
            }
        }

        impl From<String> for $name {
            fn from(s: String) -> Self {
                Self(s)
            }
        }
    };
}

id_newtype!(
    /// Major power.  Three-letter ISO-ish code (`FRA`, `GBR`, …).
    /// Per §5.1, `PREFIX` is empty since the IDs themselves are short codes.
    PowerId,
    ""
);

id_newtype!(
    /// Minor country.  `MINOR_<NAME>`.
    MinorId,
    "MINOR_"
);

id_newtype!(
    /// Historical or generated leader.  `LEADER_<NAME>`.
    LeaderId,
    "LEADER_"
);

id_newtype!(
    /// Land area on the strategic map.  `AREA_<NAME>`.
    AreaId,
    "AREA_"
);

id_newtype!(
    /// Sea zone for naval movement and combat.  `SEA_<NAME>`.
    SeaZoneId,
    "SEA_"
);

id_newtype!(
    /// Corps (land formation).  Scenario IDs `CORPS_<PWR>_<NNN>`;
    /// runtime-spawned IDs `CORPS_<PWR>_<turn>_<counter>` per §5.1.
    CorpsId,
    "CORPS_"
);

id_newtype!(
    /// Fleet (naval formation).  Same scheme as corps.
    FleetId,
    "FLEET_"
);

id_newtype!(
    /// Marshal / commander.  `MARSHAL_<NAME>`.
    MarshalId,
    "MARSHAL_"
);

id_newtype!(
    /// Division template.  `DIVTPL_<NAME>`.
    DivisionTemplateId,
    "DIVTPL_"
);

/// Light validation: ID must be ASCII, non-empty, contain only
/// `[A-Z0-9_]`, and start with the type's required prefix.
pub fn validate_id(value: &str, required_prefix: &str) -> Result<(), String> {
    if value.is_empty() {
        return Err("empty id".into());
    }
    if !value.starts_with(required_prefix) {
        return Err(format!(
            "id `{value}` missing required prefix `{required_prefix}`"
        ));
    }
    for ch in value.chars() {
        if !(ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_') {
            return Err(format!("id `{value}` contains illegal character `{ch}`"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn power_id_serializes_as_plain_string() {
        let p = PowerId::from("FRA");
        let s = serde_json::to_string(&p).unwrap();
        assert_eq!(s, "\"FRA\"");
        let back: PowerId = serde_json::from_str(&s).unwrap();
        assert_eq!(p, back);
    }

    #[test]
    fn ord_is_lexicographic() {
        let mut ids = [
            PowerId::from("RUS"),
            PowerId::from("FRA"),
            PowerId::from("AUS"),
        ];
        ids.sort();
        assert_eq!(ids[0].as_str(), "AUS");
        assert_eq!(ids[2].as_str(), "RUS");
    }

    #[test]
    fn validate_id_basics() {
        assert!(validate_id("AREA_PARIS", "AREA_").is_ok());
        assert!(validate_id("MINOR_BAVARIA", "MINOR_").is_ok());
        assert!(validate_id("paris", "AREA_").is_err());
        assert!(validate_id("AREA_PARIS!", "AREA_").is_err());
        assert!(validate_id("PARIS", "AREA_").is_err());
        assert!(validate_id("", "").is_err());
    }
}
