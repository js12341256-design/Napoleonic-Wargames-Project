# Movement rule cases

PROMPT.md §16.3 names this directory.  The authoritative rule cases
live as Rust unit tests in `crates/core/src/{map,movement}.rs::tests`
(see Phase 2 report) — those are what CI runs and what the build gate
cares about.

The YAML files in this directory are a human-readable mirror of those
tests.  They serve two purposes:

1. **Designer review.**  When a designer authors `data/tables/*.json`,
   they can read these cases to understand which graph topologies the
   pathfinder is expected to handle, without grepping Rust.
2. **Future YAML-driven test runner.**  A later phase can ship a
   `core::cases` loader that executes these files in lock-step with
   the Rust tests.  Phase 2 ships a small representative set; the
   complete enumeration matches the Rust suite name-for-name.

Schema:

```yaml
name:    "<test_name>"
kind:    "adjacency" | "hops" | "cost" | "validation"
given:
  areas: [...]
  edges: [{ from: ..., to: ..., cost: ... | "PLACEHOLDER" }]
  corps: [...]
  movement_rules: { ... }
when:
  ...               # depends on `kind`
then:
  ...               # depends on `kind`
```

Rust ground-truth lives in `crates/core/src/{map,movement}.rs::tests`.
If a YAML file disagrees with a Rust case, the Rust case wins (it is
what runs in CI).  Update the YAML to match.
