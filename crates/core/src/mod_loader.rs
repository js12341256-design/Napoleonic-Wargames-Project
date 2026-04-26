//! Data-driven mod loader (PROMPT.md §16.17).
//!
//! Mods may override authored data files under `scenarios/`, `tables/`,
//! and `locales/`. They do not execute code. Resolution is deterministic:
//! base game first, then active mods in alphabetical order, with later
//! mods winning.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    /// Relative paths to files that override base game data.
    pub overrides: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ModLoader {
    pub base_data_dir: PathBuf,
    pub mods_dir: PathBuf,
    /// Active mod IDs in load order.
    pub active_mods: Vec<String>,
}

impl ModLoader {
    pub fn new(base_data_dir: PathBuf, mods_dir: PathBuf) -> Self {
        Self {
            base_data_dir,
            mods_dir,
            active_mods: Vec::new(),
        }
    }

    /// Scan `mods_dir` for subdirectories containing `mod.json` and
    /// return manifests sorted alphabetically by mod ID.
    pub fn discover_mods(&self) -> Result<Vec<ModManifest>, String> {
        let mut manifests: Vec<ModManifest> = Vec::new();
        let entries = fs::read_dir(&self.mods_dir).map_err(|err| {
            format!(
                "failed to read mods dir `{}`: {err}",
                self.mods_dir.display()
            )
        })?;

        for entry in entries {
            let entry = entry.map_err(|err| format!("failed to read mods dir entry: {err}"))?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let manifest_path = path.join("mod.json");
            if !manifest_path.is_file() {
                continue;
            }

            let manifest_json = fs::read_to_string(&manifest_path)
                .map_err(|err| format!("failed to read `{}`: {err}", manifest_path.display()))?;
            let manifest: ModManifest = serde_json::from_str(&manifest_json)
                .map_err(|err| format!("failed to parse `{}`: {err}", manifest_path.display()))?;
            manifests.push(manifest);
        }

        manifests.sort_by(|left, right| left.id.cmp(&right.id));
        Ok(manifests)
    }

    /// Returns the path to the file: check active mods in reverse order
    /// (last wins), fall back to `base_data_dir` if no mod overrides it.
    pub fn resolve_file(&self, relative_path: &str) -> PathBuf {
        let relative = Path::new(relative_path);
        if relative.is_absolute() {
            return relative.to_path_buf();
        }

        let manifests_by_id: BTreeMap<String, ModManifest> = self
            .discover_mods()
            .unwrap_or_default()
            .into_iter()
            .map(|manifest| (manifest.id.clone(), manifest))
            .collect();

        for mod_id in self.active_mods.iter().rev() {
            let Some(manifest) = manifests_by_id.get(mod_id) else {
                continue;
            };

            if !manifest.overrides.iter().any(|item| item == relative_path) {
                continue;
            }

            let candidate = self.mods_dir.join(mod_id).join(relative);
            if candidate.is_file() {
                return candidate;
            }
        }

        self.base_data_dir.join(relative)
    }

    /// Resolve a file and deserialize it as `T`.
    pub fn load_with_mods<T: serde::de::DeserializeOwned>(
        &self,
        relative_path: &str,
    ) -> Result<T, String> {
        let path = self.resolve_file(relative_path);
        let json = fs::read_to_string(&path)
            .map_err(|err| format!("failed to read `{}`: {err}", path.display()))?;
        serde_json::from_str(&json)
            .map_err(|err| format!("failed to deserialize `{}`: {err}", path.display()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde::{Deserialize, Serialize};
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(1);

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    struct TestDoc {
        source: String,
        value: i32,
    }

    struct TestDirs {
        root: PathBuf,
        base: PathBuf,
        mods: PathBuf,
    }

    impl TestDirs {
        fn new() -> Self {
            let unique = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
            let root = std::env::temp_dir().join(format!(
                "gc1805-mod-loader-{}-{}",
                std::process::id(),
                unique
            ));
            if root.exists() {
                fs::remove_dir_all(&root).unwrap();
            }
            fs::create_dir_all(&root).unwrap();
            let base = root.join("data");
            let mods = root.join("mods");
            fs::create_dir_all(&base).unwrap();
            fs::create_dir_all(&mods).unwrap();
            Self { root, base, mods }
        }

        fn loader(&self) -> ModLoader {
            ModLoader::new(self.base.clone(), self.mods.clone())
        }
    }

    impl Drop for TestDirs {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn write_json(path: &Path, value: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, value).unwrap();
    }

    fn write_manifest(mods_dir: &Path, mod_id: &str, overrides: &[&str]) {
        let manifest = ModManifest {
            id: mod_id.to_string(),
            name: format!("{mod_id} name"),
            version: "0.1.0".to_string(),
            description: format!("{mod_id} description"),
            overrides: overrides.iter().map(|item| (*item).to_string()).collect(),
        };
        let manifest_json = serde_json::to_string_pretty(&manifest).unwrap();
        write_json(&mods_dir.join(mod_id).join("mod.json"), &manifest_json);
    }

    fn write_doc(path: &Path, source: &str, value: i32) {
        let json = format!(r#"{{"source":"{source}","value":{value}}}"#);
        write_json(path, &json);
    }

    #[test]
    fn discover_mods_empty_dir() {
        let dirs = TestDirs::new();
        let loader = dirs.loader();
        let manifests = loader.discover_mods().unwrap();
        assert!(manifests.is_empty());
    }

    #[test]
    fn discover_mods_finds_example() {
        let dirs = TestDirs::new();
        write_manifest(&dirs.mods, "example_mod", &["tables/economy.json"]);

        let loader = dirs.loader();
        let manifests = loader.discover_mods().unwrap();

        assert_eq!(manifests.len(), 1);
        assert_eq!(manifests[0].id, "example_mod");
        assert_eq!(
            manifests[0].overrides,
            vec!["tables/economy.json".to_string()]
        );
    }

    #[test]
    fn discover_mods_alphabetical_order() {
        let dirs = TestDirs::new();
        write_manifest(&dirs.mods, "zulu", &["tables/economy.json"]);
        write_manifest(&dirs.mods, "alpha", &["tables/economy.json"]);
        write_manifest(&dirs.mods, "mango", &["tables/economy.json"]);

        let loader = dirs.loader();
        let ids: Vec<String> = loader
            .discover_mods()
            .unwrap()
            .into_iter()
            .map(|manifest| manifest.id)
            .collect();

        assert_eq!(ids, vec!["alpha", "mango", "zulu"]);
    }

    #[test]
    fn resolve_file_base_when_no_mod() {
        let dirs = TestDirs::new();
        write_doc(&dirs.base.join("tables/economy.json"), "base", 10);

        let loader = dirs.loader();
        assert_eq!(
            loader.resolve_file("tables/economy.json"),
            dirs.base.join("tables/economy.json")
        );
    }

    #[test]
    fn resolve_file_mod_overrides_base() {
        let dirs = TestDirs::new();
        write_doc(&dirs.base.join("tables/economy.json"), "base", 10);
        write_manifest(&dirs.mods, "example_mod", &["tables/economy.json"]);
        write_doc(
            &dirs.mods.join("example_mod").join("tables/economy.json"),
            "example_mod",
            20,
        );

        let mut loader = dirs.loader();
        loader.active_mods = vec!["example_mod".to_string()];

        assert_eq!(
            loader.resolve_file("tables/economy.json"),
            dirs.mods.join("example_mod").join("tables/economy.json")
        );
    }

    #[test]
    fn resolve_file_last_mod_wins() {
        let dirs = TestDirs::new();
        write_doc(&dirs.base.join("tables/economy.json"), "base", 10);
        write_manifest(&dirs.mods, "alpha", &["tables/economy.json"]);
        write_manifest(&dirs.mods, "bravo", &["tables/economy.json"]);
        write_doc(
            &dirs.mods.join("alpha").join("tables/economy.json"),
            "alpha",
            20,
        );
        write_doc(
            &dirs.mods.join("bravo").join("tables/economy.json"),
            "bravo",
            30,
        );

        let mut loader = dirs.loader();
        loader.active_mods = vec!["alpha".to_string(), "bravo".to_string()];

        assert_eq!(
            loader.resolve_file("tables/economy.json"),
            dirs.mods.join("bravo").join("tables/economy.json")
        );
    }

    #[test]
    fn load_with_mods_reads_correct_file() {
        let dirs = TestDirs::new();
        write_doc(&dirs.base.join("tables/economy.json"), "base", 10);

        let loader = dirs.loader();
        let doc: TestDoc = loader.load_with_mods("tables/economy.json").unwrap();

        assert_eq!(
            doc,
            TestDoc {
                source: "base".to_string(),
                value: 10,
            }
        );
    }

    #[test]
    fn load_with_mods_mod_override_applied() {
        let dirs = TestDirs::new();
        write_doc(&dirs.base.join("tables/economy.json"), "base", 10);
        write_manifest(&dirs.mods, "example_mod", &["tables/economy.json"]);
        write_doc(
            &dirs.mods.join("example_mod").join("tables/economy.json"),
            "example_mod",
            25,
        );

        let mut loader = dirs.loader();
        loader.active_mods = vec!["example_mod".to_string()];
        let doc: TestDoc = loader.load_with_mods("tables/economy.json").unwrap();

        assert_eq!(
            doc,
            TestDoc {
                source: "example_mod".to_string(),
                value: 25,
            }
        );
    }

    #[test]
    fn manifest_deserializes_correctly() {
        let json = r#"{
            "id": "example_mod",
            "name": "Example Mod",
            "version": "0.1.0",
            "description": "Example",
            "overrides": ["tables/economy.json", "locales/en.json"]
        }"#;
        let manifest: ModManifest = serde_json::from_str(json).unwrap();

        assert_eq!(manifest.id, "example_mod");
        assert_eq!(manifest.name, "Example Mod");
        assert_eq!(manifest.version, "0.1.0");
        assert_eq!(manifest.description, "Example");
        assert_eq!(manifest.overrides.len(), 2);
    }

    #[test]
    fn manifest_serializes_correctly() {
        let manifest = ModManifest {
            id: "example_mod".to_string(),
            name: "Example Mod".to_string(),
            version: "0.1.0".to_string(),
            description: "Example".to_string(),
            overrides: vec!["tables/economy.json".to_string()],
        };
        let json = serde_json::to_string(&manifest).unwrap();

        assert!(json.contains("\"id\":\"example_mod\""));
        assert!(json.contains("\"overrides\":[\"tables/economy.json\"]"));
    }

    #[test]
    fn resolve_nested_path() {
        let dirs = TestDirs::new();
        write_doc(&dirs.base.join("tables/economy.json"), "base", 10);
        write_manifest(&dirs.mods, "nested_mod", &["tables/economy.json"]);
        write_doc(
            &dirs.mods.join("nested_mod").join("tables/economy.json"),
            "nested_mod",
            11,
        );

        let mut loader = dirs.loader();
        loader.active_mods = vec!["nested_mod".to_string()];

        assert_eq!(
            loader.resolve_file("tables/economy.json"),
            dirs.mods.join("nested_mod").join("tables/economy.json")
        );
    }

    #[test]
    fn no_active_mods_uses_base() {
        let dirs = TestDirs::new();
        write_doc(&dirs.base.join("tables/economy.json"), "base", 99);
        write_manifest(&dirs.mods, "example_mod", &["tables/economy.json"]);
        write_doc(
            &dirs.mods.join("example_mod").join("tables/economy.json"),
            "example_mod",
            12,
        );

        let loader = dirs.loader();
        let doc: TestDoc = loader.load_with_mods("tables/economy.json").unwrap();

        assert_eq!(doc.source, "base");
    }

    #[test]
    fn active_mod_not_overriding_uses_base() {
        let dirs = TestDirs::new();
        write_doc(&dirs.base.join("tables/economy.json"), "base", 99);
        write_manifest(&dirs.mods, "example_mod", &["tables/other.json"]);
        write_doc(
            &dirs.mods.join("example_mod").join("tables/economy.json"),
            "example_mod",
            12,
        );

        let mut loader = dirs.loader();
        loader.active_mods = vec!["example_mod".to_string()];
        let doc: TestDoc = loader.load_with_mods("tables/economy.json").unwrap();

        assert_eq!(doc.source, "base");
    }

    #[test]
    fn discover_mods_ignores_directories_without_manifest() {
        let dirs = TestDirs::new();
        fs::create_dir_all(dirs.mods.join("empty_mod")).unwrap();
        write_manifest(&dirs.mods, "real_mod", &["tables/economy.json"]);

        let loader = dirs.loader();
        let ids: Vec<String> = loader
            .discover_mods()
            .unwrap()
            .into_iter()
            .map(|manifest| manifest.id)
            .collect();

        assert_eq!(ids, vec!["real_mod"]);
    }

    #[test]
    fn resolve_file_missing_active_manifest_uses_base() {
        let dirs = TestDirs::new();
        write_doc(&dirs.base.join("tables/economy.json"), "base", 77);

        let mut loader = dirs.loader();
        loader.active_mods = vec!["missing_mod".to_string()];

        assert_eq!(
            loader.resolve_file("tables/economy.json"),
            dirs.base.join("tables/economy.json")
        );
    }

    #[test]
    fn load_with_mods_invalid_json_returns_error() {
        let dirs = TestDirs::new();
        write_json(&dirs.base.join("tables/economy.json"), "not valid json");

        let loader = dirs.loader();
        let error = loader
            .load_with_mods::<TestDoc>("tables/economy.json")
            .unwrap_err();

        assert!(error.contains("failed to deserialize"));
    }

    #[test]
    fn resolve_file_absolute_path_passthrough() {
        let dirs = TestDirs::new();
        let absolute = dirs.base.join("tables/economy.json");
        assert_eq!(
            dirs.loader().resolve_file(absolute.to_str().unwrap()),
            absolute
        );
    }
}
