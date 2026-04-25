use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Locale {
    pub id: String,
    pub strings: BTreeMap<String, String>,
}

impl Locale {
    #[must_use]
    pub fn get<'a>(&'a self, key: &'a str) -> &'a str {
        self.strings
            .get(key)
            .map(std::string::String::as_str)
            .unwrap_or(key)
    }

    pub fn load_yaml(yaml_str: &str) -> Result<Self, String> {
        let mut locale_id: Option<String> = None;
        let mut strings = BTreeMap::new();
        let mut in_strings = false;

        for (index, raw_line) in yaml_str.lines().enumerate() {
            let line_number = index + 1;
            let trimmed = raw_line.trim();

            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if raw_line.starts_with(' ') || raw_line.starts_with('\t') {
                if !in_strings {
                    return Err(format!(
                        "line {line_number}: indented entry outside strings section"
                    ));
                }

                let (key, value) = parse_key_value(trimmed, line_number)?;
                if key.is_empty() {
                    return Err(format!("line {line_number}: empty string key"));
                }
                strings.insert(key.to_owned(), parse_scalar(value, line_number)?);
                continue;
            }

            let (key, value) = parse_key_value(trimmed, line_number)?;
            match key {
                "locale" => {
                    locale_id = Some(parse_scalar(value, line_number)?);
                    in_strings = false;
                }
                "version" => {
                    let parsed = parse_scalar(value, line_number)?;
                    if parsed.parse::<u32>().is_err() {
                        return Err(format!("line {line_number}: version must be an integer"));
                    }
                    in_strings = false;
                }
                "strings" => {
                    if value == "{}" {
                        in_strings = false;
                    } else if value.is_empty() {
                        in_strings = true;
                    } else {
                        return Err(format!(
                            "line {line_number}: strings must be followed by a block or {{}}"
                        ));
                    }
                }
                _ => {
                    return Err(format!(
                        "line {line_number}: unsupported top-level key {key}"
                    ));
                }
            }
        }

        let id = locale_id.ok_or_else(|| String::from("missing locale field"))?;
        Ok(Self { id, strings })
    }
}

fn parse_key_value(line: &str, line_number: usize) -> Result<(&str, &str), String> {
    let Some((key, value)) = line.split_once(':') else {
        return Err(format!("line {line_number}: expected key: value"));
    };

    Ok((key.trim(), value.trim()))
}

fn parse_scalar(value: &str, line_number: usize) -> Result<String, String> {
    if value.is_empty() {
        return Ok(String::new());
    }

    if value.starts_with('"') {
        if !value.ends_with('"') || value.len() < 2 {
            return Err(format!("line {line_number}: unterminated quoted string"));
        }

        let inner = &value[1..value.len() - 1];
        return Ok(inner.replace(r#"\""#, "\""));
    }

    Ok(value.to_owned())
}

#[cfg(test)]
mod tests {
    use super::Locale;

    const EN_YAML: &str = include_str!("../../../locales/en.yaml");
    const ZZ_YAML: &str = include_str!("../../../locales/zz.yaml");
    const FR_YAML: &str = include_str!("../../../locales/fr.yaml");

    fn load_en_locale() -> Locale {
        Locale::load_yaml(EN_YAML).expect("english locale should parse")
    }

    #[test]
    fn load_en_yaml_parses_correctly() {
        let locale = load_en_locale();
        assert_eq!(locale.id, "en");
        assert!(!locale.strings.is_empty());
    }

    #[test]
    fn get_known_key_returns_value() {
        let locale = load_en_locale();
        assert_eq!(locale.get("ui.end_turn"), "End Turn");
    }

    #[test]
    fn get_unknown_key_returns_key_as_fallback() {
        let locale = load_en_locale();
        assert_eq!(locale.get("missing.key"), "missing.key");
    }

    #[test]
    fn zz_locale_wraps_in_brackets() {
        let locale = Locale::load_yaml(ZZ_YAML).expect("pseudo locale should parse");
        assert_eq!(locale.get("power.FRA"), "[France]");
        assert_eq!(locale.get("ui.end_turn"), "[End Turn]");
    }

    #[test]
    fn placeholder_locale_returns_empty_strings_map() {
        let locale = Locale::load_yaml(FR_YAML).expect("placeholder locale should parse");
        assert!(locale.strings.is_empty());
    }

    #[test]
    fn locale_id_correct_after_load() {
        let locale = Locale::load_yaml(ZZ_YAML).expect("pseudo locale should parse");
        assert_eq!(locale.id, "zz");
    }

    #[test]
    fn all_en_keys_present() {
        let locale = load_en_locale();
        assert!(locale.strings.len() >= 30);
        assert_eq!(locale.strings.len(), 43);
    }

    #[test]
    fn get_power_fra_returns_france() {
        let locale = load_en_locale();
        assert_eq!(locale.get("power.FRA"), "France");
    }

    #[test]
    fn get_phase_name() {
        let locale = load_en_locale();
        assert_eq!(locale.get("phase.combat"), "Combat Phase");
    }

    #[test]
    fn get_order_name() {
        let locale = load_en_locale();
        assert_eq!(locale.get("order.forced_march"), "Forced March");
    }

    #[test]
    fn load_invalid_yaml_returns_err() {
        let invalid = "locale: en\nstrings\n  ui.turn: \"Turn\"\n";
        let error = Locale::load_yaml(invalid).expect_err("invalid yaml should fail");
        assert!(error.contains("expected key: value"));
    }

    #[test]
    fn quoted_values_are_unwrapped() {
        let locale =
            Locale::load_yaml("locale: en\nversion: 1\nstrings:\n  ui.confirm: \"Confirm\"\n")
                .expect("quoted values should parse");
        assert_eq!(locale.get("ui.confirm"), "Confirm");
    }

    #[test]
    fn unquoted_values_are_supported() {
        let locale = Locale::load_yaml("locale: fr\nversion: 1\nstrings:\n  ui.turn: Tour\n")
            .expect("unquoted values should parse");
        assert_eq!(locale.get("ui.turn"), "Tour");
    }

    #[test]
    fn comments_and_blank_lines_are_ignored() {
        let locale = Locale::load_yaml(
            "# comment\n\nlocale: en\nversion: 1\nstrings:\n  # inner comment\n  ui.turn: \"Turn\"\n",
        )
        .expect("comments should be ignored");
        assert_eq!(locale.get("ui.turn"), "Turn");
    }

    #[test]
    fn placeholder_locale_id_is_preserved() {
        let locale = Locale::load_yaml(FR_YAML).expect("placeholder locale should parse");
        assert_eq!(locale.id, "fr");
    }

    #[test]
    fn missing_locale_field_returns_err() {
        let error = Locale::load_yaml("version: 1\nstrings: {}\n")
            .expect_err("missing locale field should fail");
        assert!(error.contains("missing locale field"));
    }

    #[test]
    fn invalid_version_returns_err() {
        let error = Locale::load_yaml("locale: en\nversion: one\nstrings: {}\n")
            .expect_err("non-integer version should fail");
        assert!(error.contains("version must be an integer"));
    }

    #[test]
    fn strings_braces_form_is_supported() {
        let locale = Locale::load_yaml("locale: pl\nversion: 1\nstrings: {}\n")
            .expect("empty strings map should parse");
        assert!(locale.strings.is_empty());
    }

    #[test]
    fn indented_line_outside_strings_section_returns_err() {
        let error = Locale::load_yaml("locale: en\n  ui.turn: \"Turn\"\n")
            .expect_err("indented line outside strings should fail");
        assert!(error.contains("outside strings section"));
    }
}
