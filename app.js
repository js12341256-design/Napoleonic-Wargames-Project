/* The Dusk of the Old World — app wiring
   Renders panels, SVG map, and modal interactions against window.GAME_DATA. */

(function () {
  "use strict";

  const D = window.GAME_DATA;
  const $ = (sel, root = document) => root.querySelector(sel);
  const $$ = (sel, root = document) => Array.from(root.querySelectorAll(sel));
  const SVG_NS = "http://www.w3.org/2000/svg";
  const svgEl = (name, attrs = {}, parent) => {
    const el = document.createElementNS(SVG_NS, name);
    for (const k in attrs) {
      if (attrs[k] == null) continue;
      el.setAttribute(k, attrs[k]);
    }
    if (parent) parent.appendChild(el);
    return el;
  };

  const state = {
    selectedCorps: null,
    hoveredArea: null,
    mapMode: "strategic",
  };

  // ── Power -> CSS var mapping for fills ───────────────
  const powerColor = {
    FRA: "#2a3a6a",
    GBR: "#8c2a1a",
    AUS: "#b8a878",
    PRU: "#2b2b2b",
    RUS: "#3a5a3a",
    SPA: "#c4951f",
    OTT: "#6a3a7a",
    // Minor states on the board:
    POR: "#5a7a4a",
    PAP: "#c9b98a",
    NAP: "#a26a3a",
    BAV: "#6a90b0",
    SAX: "#9a7a5a",
    DEN: "#b35a3a",
    SWE: "#4a6a90",
    HRE: "#b8a878",
  };

  // ── Top bar phase steps ──────────────────────────────
  const PHASES = [
    { idx: "I",   nm: "Command", state: "done" },
    { idx: "II",  nm: "Economy", state: "done" },
    { idx: "III", nm: "Diplomacy", state: "done" },
    { idx: "IV",  nm: "Impulses", state: "active" },
    { idx: "V",   nm: "Attrition", state: "" },
    { idx: "VI",  nm: "Victory", state: "" },
  ];

  function renderPhaseBar() {
    const bar = $("#phase-bar");
    bar.innerHTML = "";
    PHASES.forEach((p) => {
      const el = document.createElement("div");
      el.className = "phase-step" + (p.state ? " " + p.state : "");
      el.innerHTML = `<span class="idx">PHASE ${p.idx}</span><span class="nm">${p.nm}</span>`;
      bar.appendChild(el);
    });
  }

  // ── Left sidebar: ledger stats ───────────────────────
  function renderLedger() {
    const me = D.powers.FRA;
    const root = $("#ledger");
    const rows = [
      { lbl: "Treasury (₣)", val: me.treasury, mono: true },
      { lbl: "Income / turn", val: `+${me.income}`, cls: "pos" },
      { lbl: "Manpower (SP)", val: me.manpower, mono: true },
      { lbl: "Prestige", val: me.pp, delta: me.ppDelta },
      { lbl: "Corps afield", val: D.corps.filter((c) => c.owner === "FRA").length },
      { lbl: "Supply lines", val: "6 / 7", cls: "neg" },
    ];
    root.innerHTML = rows
      .map((r) => {
        const dcls = r.delta == null ? "" : (r.delta > 0 ? "pos" : r.delta < 0 ? "neg" : "");
        const delta = r.delta != null
          ? `<span class="delta">${r.delta > 0 ? "+" : ""}${r.delta}</span>`
          : "";
        return `<div class="stat-row"><span class="lbl">${r.lbl}</span><span class="val ${r.cls || dcls}">${r.val}${delta}</span></div>`;
      })
      .join("");

    // Top bar echo
    $("#tr-pp").innerHTML = `${me.pp}<span class="delta">${me.ppDelta >= 0 ? "+" : ""}${me.ppDelta}</span>`;
    $("#tr-treasury").textContent = `₣${me.treasury}`;
    $("#tr-manpower").textContent = me.manpower;
    $("#tr-impulse").textContent = `${D.turn.impulse} / ${D.turn.totalImpulses}`;
    $("#turn-subtitle").textContent = `Anno ${D.turn.year} · ${D.turn.month} · Tour ${toRoman(D.turn.idx)}`;
  }

  function toRoman(n) {
    const map = [
      [10, "X"], [9, "IX"], [5, "V"], [4, "IV"], [1, "I"],
    ];
    let out = "";
    let x = n;
    for (const [v, s] of map) {
      while (x >= v) { out += s; x -= v; }
    }
    return out;
  }

  // ── Diplomacy panel ──────────────────────────────────
  function renderDiplomacy() {
    const root = $("#diplo-list");
    const rels = D.diplomacy.FRA;
    const order = ["GBR", "RUS", "AUS", "PRU", "SPA", "OTT"];
    root.innerHTML = order
      .map((id) => {
        const p = D.powers[id];
        const rel = rels[id];
        return `<div class="diplo-row">
          <span class="flag ${p.flagClass}"></span>
          <span class="nm">${p.name}</span>
          <span class="state state-${rel}">${rel}</span>
        </div>`;
      })
      .join("");
  }

  // ── PP sparkline (left) ──────────────────────────────
  function renderSparkline() {
    const svg = $("#pp-sparkline");
    svg.innerHTML = "";
    const w = 260, h = 48;
    const series = D.ppHistory;
    // Background ruled lines
    for (let i = 0; i < 4; i++) {
      const y = (i / 3) * (h - 8) + 4;
      svgEl("line", {
        x1: 0, x2: w, y1: y, y2: y,
        stroke: "#c7b68f", "stroke-width": 0.5,
        "stroke-dasharray": "1 3",
      }, svg);
    }
    const all = [].concat(...Object.values(series));
    const min = Math.min(...all), max = Math.max(...all);
    const plot = (vals, color, width, dash) => {
      const n = vals.length;
      const pts = vals.map((v, i) => {
        const x = (i / (n - 1)) * (w - 4) + 2;
        const y = h - 4 - ((v - min) / (max - min || 1)) * (h - 8);
        return `${x.toFixed(1)},${y.toFixed(1)}`;
      }).join(" ");
      svgEl("polyline", {
        points: pts, fill: "none", stroke: color,
        "stroke-width": width, "stroke-linejoin": "round",
        "stroke-linecap": "round",
        "stroke-dasharray": dash || "",
      }, svg);
    };
    // Rivals dimmed
    ["GBR", "RUS", "AUS", "PRU"].forEach((k) => {
      plot(series[k], powerColor[k], 1, "2 2");
    });
    // Player prominent
    plot(series.FRA, powerColor.FRA, 1.8);
    // End dot
    const fra = series.FRA;
    const x = w - 2, y = h - 4 - ((fra[fra.length - 1] - min) / (max - min || 1)) * (h - 8);
    svgEl("circle", { cx: x, cy: y, r: 2.2, fill: powerColor.FRA, stroke: "#1c1813", "stroke-width": 0.6 }, svg);
  }

  // ── Right sidebar: impulse queue ─────────────────────
  function renderImpulseQueue() {
    const root = $("#impulse-queue");
    root.innerHTML = D.impulseQueue
      .map((q) => {
        const p = D.powers[q.power];
        const cls =
          q.status === "active" ? "now"
          : q.status === "done" ? "passed"
          : "";
        return `<div class="impulse-row ${cls}">
          <span class="idx">${String(q.impulse).padStart(2, "0")}</span>
          <span style="display:flex;align-items:center;gap:8px">
            <span class="flag ${p.flagClass}"></span>
            <span class="nm">${p.name}</span>
          </span>
          <span class="roll">ROLL ${q.roll}</span>
        </div>
        <div style="font-family:var(--mono);font-size:10px;color:var(--ink-4);padding:0 8px 4px 38px">
          ${q.note || ""}
        </div>`;
      })
      .join("");
  }

  // ── Corps roster ────────────────────────────────────
  function renderCorps() {
    const root = $("#corps-list");
    const mine = D.corps.filter((c) => c.owner === "FRA");
    root.innerHTML = mine
      .map((c) => {
        const area = D.areas.find((a) => a.id === c.area);
        const total = c.inf + c.cav + c.art;
        const lead = c.leader ? (D.leaders[c.leader] || {}).name || c.leader : "—";
        return `<div class="corps-item${c.supply ? "" : " unsupplied"}" data-corps="${c.id}">
          <span class="tick"></span>
          <span class="nm">${c.name}
            <small>${lead.toUpperCase()} · ${area ? area.name : ""}${c.supply ? "" : " · UNSUPPLIED"}</small>
          </span>
          <span class="sp">
            <span class="total">${total}</span><br />
            ${c.inf}i ${c.cav}c ${c.art}a
          </span>
        </div>`;
      })
      .join("");

    $("#corps-meta").textContent = `${mine.length} corps`;

    $$("#corps-list .corps-item").forEach((el) => {
      el.addEventListener("click", () => {
        const id = el.getAttribute("data-corps");
        selectCorps(id);
      });
    });
  }

  // ── Dispatches ───────────────────────────────────────
  function renderDispatches() {
    const root = $("#dispatches");
    root.innerHTML = D.dispatches
      .map(
        (d) => `<div class="dispatch">
        ${d.seal ? '<span class="seal" aria-hidden="true"></span>' : ""}
        <div class="from">${d.from}</div>
        <div class="subject">${d.subject}</div>
        <div class="body">${d.body}</div>
      </div>`
      )
      .join("");
  }

  // ── Turn log ─────────────────────────────────────────
  function renderLog() {
    const root = $("#log-stream");
    const ICONS = { combat: "⚔", diplo: "✉", econ: "₣", highlight: "✦", normal: "·" };
    root.innerHTML = D.turnLog
      .map(
        (l) => `<div class="log-line ${l.c}">
        <span class="tm">${l.t}</span>
        <span class="ic">${ICONS[l.c] || "·"}</span>
        <span class="ev">${l.e}</span>
      </div>`
      )
      .join("");
    root.scrollTop = root.scrollHeight;
    $("#scrub-pos").textContent = `${D.turnLog.length} / ${D.turnLog.length}`;
  }

  // ── PP bars (bottom right) ───────────────────────────
  function renderPPBars() {
    const root = $("#pp-bars");
    const max = Math.max(...Object.values(D.powers).map((p) => p.pp));
    const order = ["FRA", "GBR", "RUS", "AUS", "PRU", "OTT", "SPA"];
    root.innerHTML = order
      .map((id) => {
        const p = D.powers[id];
        const pct = (p.pp / max) * 100;
        return `<div class="pp-bar ${id === "FRA" ? "me" : ""}">
        <span class="nm">${p.name}</span>
        <span class="track"><span class="fill" style="width:${pct.toFixed(1)}%;background:${id === "FRA" ? "var(--fra)" : powerColor[id]}"></span></span>
        <span class="v">${p.pp}</span>
      </div>`;
      })
      .join("");
  }

  // ── SVG Map ──────────────────────────────────────────
  function renderMap() {
    const svg = $("#map-svg");
    svg.innerHTML = "";

    // Defs: hatching pattern for sea, paper filter
    const defs = svgEl("defs", {}, svg);
    const pat = svgEl("pattern", {
      id: "sea-hatch", width: 14, height: 14,
      patternUnits: "userSpaceOnUse", patternTransform: "rotate(18)"
    }, defs);
    svgEl("rect", { x: 0, y: 0, width: 14, height: 14, fill: "#e7dcbd" }, pat);
    svgEl("path", { d: "M0 7 H14", stroke: "#c7b68f", "stroke-width": 0.6 }, pat);

    // Sea background
    svgEl("rect", {
      x: 0, y: 0, width: 1400, height: 900, fill: "url(#sea-hatch)"
    }, svg);

    // Decorative cartouche frame
    const frame = svgEl("g", { opacity: 0.7 }, svg);
    svgEl("rect", {
      x: 10, y: 10, width: 1380, height: 880,
      fill: "none", stroke: "#8a7d63", "stroke-width": 1.2
    }, frame);
    svgEl("rect", {
      x: 18, y: 18, width: 1364, height: 864,
      fill: "none", stroke: "#a89776", "stroke-width": 0.6,
      "stroke-dasharray": "3 4"
    }, frame);

    // Title cartouche top-left
    const tg = svgEl("g", { transform: "translate(40, 40)" }, svg);
    const tt = svgEl("text", {
      x: 0, y: 0,
      "font-family": "Cormorant Garamond, serif",
      "font-style": "italic",
      "font-size": 26, fill: "#1c1813"
    }, tg);
    tt.textContent = "Carte Générale de l'Europe";
    const ts = svgEl("text", {
      x: 0, y: 22,
      "font-family": "JetBrains Mono, monospace",
      "font-size": 10, fill: "#5c5240",
      "letter-spacing": "2"
    }, tg);
    ts.textContent = "MDCCCVI · Point-to-Point Theatre";

    // Draw edges (land + sea)
    const byId = Object.fromEntries(D.areas.map((a) => [a.id, a]));
    const edgeLayer = svgEl("g", { id: "edges" }, svg);
    D.edges.forEach(([a, b]) => {
      const A = byId[a], B = byId[b];
      if (!A || !B) return;
      svgEl("line", {
        x1: A.x, y1: A.y, x2: B.x, y2: B.y,
        stroke: "#8a7d63", "stroke-width": 1
      }, edgeLayer);
    });
    D.seaLinks.forEach(([a, b]) => {
      const A = byId[a], B = byId[b];
      if (!A || !B) return;
      svgEl("line", {
        x1: A.x, y1: A.y, x2: B.x, y2: B.y,
        stroke: "#5a6b90", "stroke-width": 0.9,
        "stroke-dasharray": "2 4", opacity: 0.7
      }, edgeLayer);
    });

    // Province nodes
    const nodeLayer = svgEl("g", { id: "provinces" }, svg);
    D.areas.forEach((a) => {
      const g = svgEl("g", {
        class: "prov", transform: `translate(${a.x},${a.y})`,
        "data-id": a.id, style: "cursor:pointer"
      }, nodeLayer);

      const r = a.capital ? 10 : 7;
      const fill = powerColor[a.owner] || "#b0a285";

      // Halo for capital
      if (a.capital) {
        svgEl("circle", {
          cx: 0, cy: 0, r: r + 4, fill: "none",
          stroke: fill, "stroke-width": 1, opacity: 0.4
        }, g);
      }
      svgEl("circle", {
        cx: 0, cy: 0, r, fill,
        stroke: "#1c1813", "stroke-width": 1.1
      }, g);

      // Star for capital
      if (a.capital) {
        svgEl("circle", { cx: 0, cy: 0, r: 2.4, fill: "#f2ead7" }, g);
      }

      // Fort ticks
      if (a.fort) {
        for (let i = 0; i < a.fort; i++) {
          svgEl("rect", {
            x: -r - 2 + i * 3, y: -r - 6,
            width: 2, height: 4,
            fill: "#1c1813"
          }, g);
        }
      }

      // Port glyph
      if (a.port) {
        svgEl("path", {
          d: "M -12 4 Q -10 8 -6 8",
          fill: "none", stroke: "#1c1813", "stroke-width": 1.2
        }, g);
      }

      // Label
      const lx = a.x > 1100 ? -10 : (r + 4);
      const anchor = a.x > 1100 ? "end" : "start";
      const label = svgEl("text", {
        x: lx, y: 3,
        "font-family": a.capital ? "Cormorant Garamond, serif" : "Inter, sans-serif",
        "font-size": a.capital ? 13 : 11,
        "font-weight": a.capital ? 500 : 400,
        "font-style": a.capital ? "italic" : "normal",
        fill: "#1c1813",
        "text-anchor": anchor,
        "paint-order": "stroke",
        stroke: "#f2ead7",
        "stroke-width": 3,
        "stroke-linejoin": "round"
      }, g);
      label.textContent = a.name;

      g.addEventListener("mouseenter", () => showTooltip(a, g));
      g.addEventListener("mouseleave", hideTooltip);
      g.addEventListener("click", (e) => {
        e.stopPropagation();
        openBattleFromArea(a);
      });
    });

    // Corps tokens
    const tokenLayer = svgEl("g", { id: "corps-tokens" }, svg);
    // group by area for stacking
    const byArea = {};
    D.corps.forEach((c) => { (byArea[c.area] = byArea[c.area] || []).push(c); });
    Object.entries(byArea).forEach(([areaId, list]) => {
      const A = byId[areaId];
      if (!A) return;
      list.forEach((c, i) => {
        const dx = 14, dy = 16;
        const cx = A.x + 12 + (i % 2) * dx;
        const cy = A.y + 14 + Math.floor(i / 2) * dy;

        const g = svgEl("g", {
          class: "corps", transform: `translate(${cx},${cy})`,
          "data-corps": c.id, style: "cursor:pointer"
        }, tokenLayer);

        const fill = powerColor[c.owner] || "#1c1813";
        const ring = svgEl("rect", {
          x: -11, y: -9, width: 22, height: 18,
          fill: "#f7f0dc", stroke: "#1c1813", "stroke-width": 1,
          rx: 1
        }, g);

        // Color chit
        svgEl("rect", {
          x: -10, y: -8, width: 6, height: 16,
          fill, stroke: "none"
        }, g);

        // Leader initial or ? for hidden
        const letter = c.hidden && c.owner !== "FRA" ? "?"
          : c.leader
          ? (D.leaders[c.leader] || {}).name[0]
          : c.name[0];
        const t = svgEl("text", {
          x: 1, y: 4,
          "font-family": "Cormorant Garamond, serif",
          "font-size": 12, "font-weight": 600,
          fill: "#1c1813", "text-anchor": "middle"
        }, g);
        t.textContent = letter;

        // Strength pips
        const sp = c.inf + c.cav + c.art;
        const bar = svgEl("rect", {
          x: -11, y: 10, width: Math.min(22, sp * 0.7),
          height: 2, fill
        }, g);

        if (!c.supply) {
          svgEl("circle", { cx: 11, cy: -9, r: 3, fill: "#8c2a1a", stroke: "#1c1813", "stroke-width": 0.7 }, g);
        }

        g.addEventListener("click", (e) => {
          e.stopPropagation();
          if (c.owner === "FRA") selectCorps(c.id);
          else openBattleAgainst(c);
        });
      });
    });

    // Supply trace from Paris → Strasbourg → Frankfurt → Munich → Milan
    const trace = ["PARIS", "STRASBOURG", "FRANKFURT", "MUNICH", "MILAN"];
    const trg = svgEl("g", { id: "supply-trace", opacity: 0.85 }, svg);
    for (let i = 0; i < trace.length - 1; i++) {
      const A = byId[trace[i]], B = byId[trace[i + 1]];
      svgEl("path", {
        d: `M ${A.x} ${A.y} L ${B.x} ${B.y}`,
        stroke: "#b48a2b",
        "stroke-width": 2.2,
        "stroke-dasharray": "4 2",
        fill: "none"
      }, trg);
    }

    // Click empty SVG clears selection
    svg.addEventListener("click", () => selectCorps(null));
  }

  // ── Tooltip ─────────────────────────────────────────
  function showTooltip(area, node) {
    const tt = $("#prov-tooltip");
    const power = D.powers[area.owner];
    const pname = power ? power.name : area.owner;
    const terrain = (area.terrain || "open").replace(/^\w/, (c) => c.toUpperCase());
    tt.innerHTML = `
      <div class="tt-name">
        <span>${area.name}</span>
        <span style="font-family:var(--mono);font-size:10px;color:var(--ink-4)">${area.id}</span>
      </div>
      <div class="tt-sub">${pname}${area.capital ? " · Capital" : ""}${area.port ? " · Port" : ""}</div>
      <div class="tt-row"><span class="lbl">Terrain</span><span>${terrain}</span></div>
      <div class="tt-row"><span class="lbl">Fortification</span><span>${"◆".repeat(area.fort || 0) || "—"}</span></div>
      <div class="tt-row"><span class="lbl">Revenue</span><span>₣${area.money || 0}</span></div>
      <div class="tt-row"><span class="lbl">Manpower</span><span>${area.mp || 0} SP</span></div>
    `;
    tt.style.display = "block";

    // Position near the node but inside the map-wrap
    const wrap = $("#map-wrap").getBoundingClientRect();
    const rect = node.getBoundingClientRect();
    let left = rect.right - wrap.left + 10;
    let top = rect.top - wrap.top + 4;
    if (left + 230 > wrap.width) left = rect.left - wrap.left - 230;
    if (top + 160 > wrap.height) top = wrap.height - 170;
    tt.style.left = Math.max(8, left) + "px";
    tt.style.top = Math.max(8, top) + "px";
  }
  function hideTooltip() { $("#prov-tooltip").style.display = "none"; }

  // ── Selection halos + corps list sync ───────────────
  function selectCorps(id) {
    state.selectedCorps = id;
    $$("#corps-list .corps-item").forEach((el) => {
      el.classList.toggle("selected", el.getAttribute("data-corps") === id);
    });
    // Redraw halo on map
    $$("#map-svg .corps-halo").forEach((el) => el.remove());
    if (!id) return;
    const c = D.corps.find((cc) => cc.id === id);
    if (!c) return;
    const A = D.areas.find((a) => a.id === c.area);
    const svg = $("#map-svg");
    svgEl("circle", {
      class: "corps-halo corps-ring selected",
      cx: A.x + 12, cy: A.y + 14, r: 18
    }, svg);

    // Open order builder on second click
    if (state.lastClickedCorps === id) openCOB(c);
    state.lastClickedCorps = id;
    setTimeout(() => (state.lastClickedCorps = null), 600);
  }

  function openCOB(corps) {
    const m = $("#modal-cob");
    const area = D.areas.find((a) => a.id === corps.area);
    const leader = corps.leader ? (D.leaders[corps.leader] || {}).name : "no commander";
    $("#cob-title").textContent = `Mandate · ${corps.name}`;
    $(".modal-head .mh-sub", m).textContent =
      `${corps.name} · Maréchal ${leader} · at ${area ? area.name : corps.area}`;
    m.style.display = "grid";
  }

  function openBattleFromArea(area) {
    // If FRA has corps here AND enemy corps adjacent or here, open battle
    const here = D.corps.filter((c) => c.area === area.id);
    const friendly = here.find((c) => c.owner === "FRA");
    const enemy = here.find((c) => c.owner !== "FRA") ||
      // neighbour enemy
      (() => {
        const neigh = D.edges
          .filter(([a, b]) => a === area.id || b === area.id)
          .map(([a, b]) => (a === area.id ? b : a));
        for (const n of neigh) {
          const e = D.corps.find((c) => c.area === n && c.owner !== "FRA");
          if (e) return e;
        }
        return null;
      })();
    if (friendly && enemy) openBattleAgainst(enemy);
  }

  function openBattleAgainst(_enemy) {
    $("#modal-battle").style.display = "grid";
  }

  // ── Map controls ────────────────────────────────────
  function wireControls() {
    $$(".map-controls button[data-mode]").forEach((b) => {
      b.addEventListener("click", () => {
        $$(".map-controls button[data-mode]").forEach((x) => x.classList.remove("active"));
        b.classList.add("active");
        state.mapMode = b.getAttribute("data-mode");
      });
    });

    $$(".map-controls button[data-zoom]").forEach((b) => {
      b.addEventListener("click", () => {
        const svg = $("#map-svg");
        const vb = svg.getAttribute("viewBox").split(" ").map(Number);
        const dir = b.getAttribute("data-zoom") === "in" ? 0.88 : 1.14;
        const nw = vb[2] * dir, nh = vb[3] * dir;
        const cx = vb[0] + vb[2] / 2, cy = vb[1] + vb[3] / 2;
        svg.setAttribute("viewBox", `${cx - nw / 2} ${cy - nh / 2} ${nw} ${nh}`);
      });
    });

    // Modal close
    $$("[data-close]").forEach((b) =>
      b.addEventListener("click", () => {
        $("#modal-cob").style.display = "none";
        $("#modal-battle").style.display = "none";
      })
    );
    document.addEventListener("keydown", (e) => {
      if (e.key === "Escape") {
        $("#modal-cob").style.display = "none";
        $("#modal-battle").style.display = "none";
      }
    });

    // End impulse button: open COB for first unmoved corps (demo)
    $("#btn-end-phase").addEventListener("click", () => {
      const c = D.corps.find((cc) => cc.owner === "FRA" && !cc.moved);
      if (c) { selectCorps(c.id); openCOB(c); }
    });

    // Action pill selection in COB
    $$("#modal-cob .action-pill").forEach((p) =>
      p.addEventListener("click", () => {
        $$("#modal-cob .action-pill").forEach((x) => x.classList.remove("selected"));
        p.classList.add("selected");
      })
    );
    // Formation option selection in Battle
    $$("#modal-battle .formation-grid").forEach((grid) => {
      $$(".form-opt", grid).forEach((o) =>
        o.addEventListener("click", () => {
          $$(".form-opt", grid).forEach((x) => x.classList.remove("selected"));
          o.classList.add("selected");
        })
      );
    });
  }

  // ── Init ────────────────────────────────────────────
  function init() {
    renderPhaseBar();
    renderLedger();
    renderDiplomacy();
    renderSparkline();
    renderImpulseQueue();
    renderCorps();
    renderDispatches();
    renderLog();
    renderPPBars();
    renderMap();
    wireControls();
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
})();
