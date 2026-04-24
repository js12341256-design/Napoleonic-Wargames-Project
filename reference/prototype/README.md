# Prototype — *The Dusk of the Old World*

This folder is a **visual reference**, not a build target.

It is the HTML/CSS/JS prototype produced by Claude Design for an original
Napoleonic-era grand-strategy game concept ("The Dusk of the Old World" — explicitly
not branded as *Empires in Arms*). The live project in this repo is a separate,
clean-room Rust/Bevy implementation (see `/docs/PROMPT.md`).

## Why it's here

- The parchment-and-ink aesthetic, type scale, and panel layouts are a target
  style for the desktop client (see Phase 14 in the master prompt).
- The area graph, corps composition, impulse queue, and turn-log layouts are
  useful precedent for the UI surface area we need to cover.

## Why it's **not** the project

- It is pure presentation — there is no rules engine, no determinism,
  no tests, no netcode.
- The data in `data.js` is illustrative. None of it is authoritative for the
  clean-room rules tables under `/data/tables/`.
- Any code here may be discarded once the Rust clients reach parity.

## Viewing

Open `index.html` in a browser, or use githack:

```
https://raw.githack.com/js12341256-design/project1/claude/implement-design-system-ciX1R/reference/prototype/index.html
```

Desktop-width (≥ 1200 px). Requires internet for the Google Fonts (Cormorant
Garamond, Inter, JetBrains Mono); falls back to system fonts otherwise.
