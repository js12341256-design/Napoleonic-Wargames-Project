import React, { useEffect, useMemo, useRef, useState } from 'react'
import { geoMercator, geoPath } from 'd3-geo'

/* ───────── constants ───────── */
const SVG_W = 1600
const SVG_H = 1000
const MIN_ZOOM = 0.3
const MAX_ZOOM = 20

/* ── Omniatlas-style historical political map palette ── */
const POWER_COLORS: Record<string, string> = {
  FRA: '#9B7B9E',   // muted mauve-purple
  GBR: '#E84090',   // magenta/hot pink
  AUS: '#FF6B35',   // bright orange
  PRU: '#7090A8',   // blue-gray
  RUS: '#C8A8D4',   // light purple/lavender
  SPA: '#D4C4A8',   // pale tan/beige
  OTT: '#E8E8E0',   // light gray/off-white
}

/* Minor region palettes — region bucket determines shade */
const MINOR_COLORS: Record<string, string> = {
  german:    '#F0A080',  // salmon/coral
  italian:   '#D4B840',  // yellow-gold
  scandinavian: '#88A878', // sage green
  iberian:   '#E8C0C0',  // light pink
  north_africa: '#C8B898', // sand/tan
  balkans:   '#FFB8A0',  // coral variant
  eastern:   '#F8B870',  // orange-peach
  default:   '#C8C8B8',  // unowned gray-beige
}

/* Area → region bucket mapping */
const AREA_REGION: Record<string, string> = {
  AREA_BERG: 'german', AREA_BRUNSWICK: 'german', AREA_DARMSTADT: 'german',
  AREA_FRANKFURT: 'german', AREA_HAMBURG: 'german', AREA_HANOVER: 'german',
  AREA_HOLSTEIN: 'german', AREA_KARLSRUHE: 'german', AREA_KASSEL: 'german',
  AREA_KIEL: 'german', AREA_KLEVES: 'german', AREA_LUBECK: 'german',
  AREA_MECKLENBURG: 'german', AREA_NASSAU: 'german', AREA_OLDENBURG: 'german',
  AREA_STUTTGART: 'german', AREA_THURINGIA: 'german', AREA_WURZBURG: 'german',
  AREA_SWEDISH_POMERANIA: 'german', AREA_COLOGNE: 'german', AREA_BAVARIA: 'german',
  AREA_MUNICH: 'german', AREA_INGOLSTADT: 'german',

  AREA_FLORENCE: 'italian', AREA_LOMBARDY: 'italian', AREA_MANTUA: 'italian',
  AREA_MILAN: 'italian', AREA_NAPLES: 'italian', AREA_PALERMO: 'italian',
  AREA_PIEDMONT: 'italian', AREA_ROMAGNA: 'italian', AREA_ROME: 'italian',
  AREA_SARDINIA_ISLAND: 'italian', AREA_SICILY: 'italian', AREA_TURIN: 'italian',
  AREA_VALLETTA: 'italian', AREA_VENETIA: 'italian', AREA_VENICE: 'italian',
  AREA_ALESSANDRIA: 'italian', AREA_CORFU: 'italian', AREA_MALTA: 'italian',

  AREA_DENMARK: 'scandinavian', AREA_FINLAND: 'scandinavian', AREA_NORWAY: 'scandinavian',
  AREA_STOCKHOLM: 'scandinavian', AREA_SWEDEN_SOUTH: 'scandinavian',
  AREA_COPENHAGEN: 'scandinavian',

  AREA_PORTUGAL: 'iberian', AREA_LISBON: 'iberian', AREA_ELVAS: 'iberian',

  AREA_ALGIERS: 'north_africa', AREA_BENGHAZI: 'north_africa', AREA_CAIRO: 'north_africa',
  AREA_EGYPT: 'north_africa', AREA_FEZ: 'north_africa', AREA_MOROCCO: 'north_africa',
  AREA_TRIPOLI: 'north_africa', AREA_TUNIS: 'north_africa', AREA_CYRENAICA: 'north_africa',

  AREA_BELGRADE: 'balkans', AREA_BOSNIA: 'balkans', AREA_ILLYRIA: 'balkans',
  AREA_MOREA: 'balkans', AREA_SARAJEVO: 'balkans', AREA_ATHENS: 'balkans',
  AREA_RHODESIA: 'balkans', AREA_RHODES: 'balkans',

  AREA_WARSAW: 'eastern', AREA_KRAKOW: 'eastern', AREA_GALICIA_EAST: 'eastern',
  AREA_GALICIA_WEST: 'eastern', AREA_POSEN: 'eastern', AREA_MOLDAVIA: 'eastern',
  AREA_IASI: 'eastern', AREA_BUCHAREST: 'eastern', AREA_TRANSYLVANIA: 'eastern',
  AREA_CLUJ: 'eastern', AREA_CRIMEA: 'eastern', AREA_SEVASTOPOL: 'eastern',
}

function minorColor(aid: string): string {
  const bucket = AREA_REGION[aid] ?? 'default'
  return MINOR_COLORS[bucket] ?? MINOR_COLORS.default
}

/* Ocean / land / border styling */
const OCEAN     = '#B8D4E8'   // light blue-gray
const LAND_BASE = '#E8E4D8'   // unowned parchment (fallback)
const BORDER_CLR = '#2a2a2a'  // dark thin border
const COAST_CLR  = '#2a2a2a'  // dark coastline
const RIVER_CLR  = 'rgba(60,120,200,0.55)'
const LAKE_CLR   = '#B8D4E8'  // match ocean

/* Lighten hex by amount (0-255 per channel) */
function lightenHex(hex: string, amount: number): string {
  const n = hex.replace('#', '')
  if (n.length !== 6) return hex
  const v = Number.parseInt(n, 16)
  const r = Math.min(255, (v >> 16) + amount)
  const g = Math.min(255, ((v >> 8) & 0xff) + amount)
  const b = Math.min(255, (v & 0xff) + amount)
  return `#${[r, g, b].map(c => c.toString(16).padStart(2, '0')).join('')}`
}

const POWER_NAMES: Record<string, string> = {
  FRA: 'France', GBR: 'Britain', AUS: 'Austria',
  PRU: 'Prussia', RUS: 'Russia', SPA: 'Spain', OTT: 'Ottoman',
}

/* ───────── front-line types ───────── */
export interface AttackArrow {
  fromArea: string
  toArea: string
  attacker: string
  strength: number
}
export interface ContestedArea {
  areaId: string
  attacker: string
  defender: string
  pressure: number
}
export interface BattleToast {
  area: string
  areaName: string
  attacker: string
  result: 'AttackerAdvances' | 'Stalemate' | 'DefenderHolds' | 'DefenderRoutes'
  timestamp: number
}

type CorpsInfo = { id: string; owner: string; sp: number; area: string; displayName: string }

interface MapViewProps {
  scenarioData: any
  powerStates: Record<string, any>
  currentTurn: number
  onEndTurn: () => void
  attackArrows?: AttackArrow[]
  contestedAreas?: ContestedArea[]
  battleToasts?: BattleToast[]
}

function clamp(v: number, lo: number, hi: number) {
  return Math.max(lo, Math.min(hi, v))
}

function areaOwnerColor(area: any, aid: string): string {
  const o = area?.owner
  if (!o) return LAND_BASE
  if (o.kind === 'POWER' && o.power) return POWER_COLORS[o.power] ?? LAND_BASE
  if (o.kind === 'MINOR') return minorColor(aid)
  return LAND_BASE
}

function areaOwnerPowerId(area: any): string | null {
  const o = area?.owner
  if (o?.kind === 'POWER') return o.power || null
  return null
}

function isOttomanOwned(area: any): boolean {
  return area?.owner?.power === 'OTT'
}

function fmtArea(id: string, dn?: string) {
  if (dn) return dn
  return id.replace(/^AREA_/, '').replace(/_/g, ' ').replace(/\b\w/g, c => c.toUpperCase())
}
function shortLabel(n: string) {
  if (n.length <= 12) return n
  return n.replace('Saint', 'St.').replace('Petersburg', 'Pete.')
    .replace('Colonies', 'Cols.').replace('Swedish', 'Swed.')
}
function turnDate(start: any, turn: number) {
  const sy = Number(start?.year ?? 1805)
  const sm = Number(start?.month ?? 8)
  const st = Number(start?.turn ?? 1)
  const total = st - 1 + turn
  const y = sy + Math.floor(total / 12)
  const mi = ((sm - 1 + total) % 12 + 12) % 12
  return ['Jan','Feb','Mar','Apr','May','Jun','Jul','Aug','Sep','Oct','Nov','Dec'][mi] + ' ' + y
}
function ownerLabel(area: any): string {
  const o = area?.owner
  if (!o) return 'Neutral'
  if (o.kind === 'POWER') return POWER_NAMES[o.power] || o.power || 'Unknown'
  if (o.kind === 'MINOR') return (o.minor || 'Minor').replace('MINOR_', '').replace(/_/g, ' ')
  return 'Neutral'
}

/* ───────── component ───────── */
export default function MapView({
  scenarioData, powerStates, currentTurn, onEndTurn,
  attackArrows = [], contestedAreas = [], battleToasts = [],
}: MapViewProps) {
  const containerRef = useRef<HTMLDivElement>(null)
  const dragRef = useRef({ active: false, x: 0, y: 0, sx: 0, sy: 0, moved: false })

  const [zoom, setZoom] = useState(1)
  const [pan, setPan] = useState({ x: 0, y: 0 })
  const [selId, setSelId] = useState<string | null>(null)
  const [hovId, setHovId] = useState<string | null>(null)

  /* async geo data */
  const [land, setLand] = useState<any>(null)
  const [terrs, setTerrs] = useState<any>(null)
  const [rivers, setRivers] = useState<any>(null)
  const [lakes, setLakes] = useState<any>(null)
  const [centers, setCenters] = useState<Record<string, [number, number]> | null>(null)

  useEffect(() => {
    Promise.all([
      fetch('./ne_land.geojson').then(r => r.json()),
      fetch('./game-territories.geojson').then(r => r.json()),
      fetch('./ne_rivers.geojson').then(r => r.json()),
      fetch('./ne_lakes.geojson').then(r => r.json()),
      fetch('./area-centers.json').then(r => r.json()),
    ]).then(([a, b, c, d, e]) => {
      setLand(a); setTerrs(b); setRivers(c); setLakes(d); setCenters(e)
    }).catch(err => console.error('geo load failed', err))
  }, [])

  /* d3 projection — Mercator fitted to Europe + N.Africa + Middle East */
  const projection = useMemo(() => {
    const euroBox: any = {
      type: 'FeatureCollection',
      features: [{
        type: 'Feature', properties: {},
        geometry: { type: 'Polygon', coordinates: [[[-15, 22], [62, 22], [62, 72], [-15, 72], [-15, 22]]] },
      }],
    }
    return geoMercator().fitExtent([[10, 10], [SVG_W - 10, SVG_H - 10]], euroBox)
  }, [])

  const pathGen = useMemo(() => geoPath().projection(projection), [projection])

  const landPaths = useMemo(() => {
    if (!land) return []
    return (land as any).features.map((f: any) => pathGen(f) || '').filter(Boolean)
  }, [land, pathGen])

  const terrData = useMemo(() => {
    if (!terrs) return []
    const sa = scenarioData?.areas ?? {}
    return (terrs as any).features.map((f: any) => {
      const aid = f.properties?.id || f.id
      const a = sa[aid]
      return { aid, area: a, baseColor: areaOwnerColor(a, aid), path: pathGen(f) || '' }
    }).filter((t: any) => t.path)
  }, [terrs, scenarioData, pathGen])

  const riverPaths = useMemo(() => {
    if (!rivers) return []
    return (rivers as any).features.map((f: any) => pathGen(f) || '').filter(Boolean)
  }, [rivers, pathGen])

  const lakePaths = useMemo(() => {
    if (!lakes) return []
    return (lakes as any).features.map((f: any) => pathGen(f) || '').filter(Boolean)
  }, [lakes, pathGen])

  const corpsByArea = useMemo(() => {
    const sc = scenarioData?.corps ?? {}
    const g: Record<string, CorpsInfo[]> = {}
    Object.entries(sc).forEach(([id, raw]) => {
      const c = raw as any
      const sp = Number(c.infantry_sp || 0) + Number(c.cavalry_sp || 0) + Number(c.artillery_sp || 0)
      ;(g[c.area] ||= []).push({ id, owner: c.owner, area: c.area, sp, displayName: c.display_name || id })
    })
    return g
  }, [scenarioData])

  const contestedMap = useMemo(() => {
    const m: Record<string, ContestedArea> = {}
    for (const ca of contestedAreas) m[ca.areaId] = ca
    return m
  }, [contestedAreas])

  const [visibleToasts, setVisibleToasts] = useState<BattleToast[]>([])
  useEffect(() => {
    if (battleToasts.length === 0) return
    setVisibleToasts(battleToasts)
    const timer = setTimeout(() => setVisibleToasts([]), 3000)
    return () => clearTimeout(timer)
  }, [battleToasts])

  const selInfo = useMemo(() => {
    if (!selId) return null
    const a = scenarioData?.areas?.[selId]
    return { id: selId, area: a, corps: corpsByArea[selId] || [], name: fmtArea(selId, a?.display_name) }
  }, [selId, scenarioData, corpsByArea])

  /* ── zoom / pan ── */
  const handleWheel = (e: React.WheelEvent<HTMLDivElement>) => {
    e.preventDefault()
    const rect = containerRef.current?.getBoundingClientRect()
    if (!rect) return
    const mx = e.clientX - rect.left, my = e.clientY - rect.top
    const f = e.deltaY < 0 ? 1.14 : 1 / 1.14
    const nz = clamp(zoom * f, MIN_ZOOM, MAX_ZOOM)
    if (nz === zoom) return
    setPan({ x: mx - (mx - pan.x) * (nz / zoom), y: my - (my - pan.y) * (nz / zoom) })
    setZoom(nz)
  }
  const beginDrag = (e: React.MouseEvent) => {
    dragRef.current = { active: true, x: e.clientX, y: e.clientY, sx: pan.x, sy: pan.y, moved: false }
  }
  const duringDrag = (e: React.MouseEvent) => {
    const d = dragRef.current; if (!d.active) return
    const dx = e.clientX - d.x, dy = e.clientY - d.y
    if (Math.abs(dx) > 2 || Math.abs(dy) > 2) d.moved = true
    setPan({ x: d.sx + dx, y: d.sy + dy })
  }
  const endDrag = () => { dragRef.current.active = false }
  const stepZoom = (f: number) => {
    const r = containerRef.current?.getBoundingClientRect()
    const cx = r ? r.width / 2 : SVG_W / 2, cy = r ? r.height / 2 : SVG_H / 2
    const nz = clamp(zoom * f, MIN_ZOOM, MAX_ZOOM)
    setPan({ x: cx - (cx - pan.x) * (nz / zoom), y: cy - (cy - pan.y) * (nz / zoom) })
    setZoom(nz)
  }
  const resetView = () => { setZoom(1); setPan({ x: 0, y: 0 }) }

  /* ── render ── */
  return (
    <div style={{
      height: '100%',
      display: 'flex',
      flexDirection: 'column',
      background: OCEAN,
      fontFamily: "'Noto Serif', 'Times New Roman', serif",
      color: '#1a1a1a',
    }}>

      {/* ────────────────────────────────────────────────────
          POWER STATUS BAR — 60px, dark background
          ──────────────────────────────────────────────────── */}
      <div style={{
        height: 60,
        flexShrink: 0,
        background: 'linear-gradient(180deg, #1a1612 0%, #0e0c0a 100%)',
        borderBottom: '2px solid #3a3028',
        display: 'flex',
        alignItems: 'center',
        padding: '0 12px',
        gap: 10,
        overflowX: 'auto',
        zIndex: 20,
      }}>
        {Object.entries(POWER_NAMES).map(([pid, name]) => {
          const treasury = powerStates?.[pid]?.treasury ?? scenarioData?.powers?.[pid]?.starting_treasury ?? '—'
          const prestige = powerStates?.[pid]?.prestige ?? '—'
          const col = POWER_COLORS[pid] ?? '#aaa'
          /* compute fill width from treasury (max 300 shown as 100%) */
          const tNum = typeof treasury === 'number' ? treasury : 0
          const fillPct = Math.min(100, (tNum / 300) * 100)
          return (
            <div key={pid} style={{
              display: 'flex',
              alignItems: 'center',
              gap: 8,
              padding: '4px 10px',
              background: 'rgba(255,255,255,0.04)',
              border: `1px solid ${col}55`,
              borderRadius: 3,
              minWidth: 148,
              flexShrink: 0,
            }}>
              {/* Color swatch */}
              <div style={{ width: 10, height: 28, background: col, borderRadius: 1, flexShrink: 0 }} />
              <div style={{ display: 'flex', flexDirection: 'column', gap: 2, flex: 1, minWidth: 0 }}>
                {/* Name row */}
                <div style={{
                  color: col,
                  fontSize: 10,
                  fontWeight: 700,
                  letterSpacing: 1.2,
                  textTransform: 'uppercase',
                  fontFamily: 'Cinzel, serif',
                }}>
                  {pid} · {name}
                </div>
                {/* Treasury bar */}
                <div style={{ display: 'flex', alignItems: 'center', gap: 5 }}>
                  <div style={{
                    flex: 1,
                    height: 5,
                    background: 'rgba(255,255,255,0.1)',
                    borderRadius: 2,
                    overflow: 'hidden',
                  }}>
                    <div style={{
                      width: `${fillPct}%`,
                      height: '100%',
                      background: col,
                      borderRadius: 2,
                      transition: 'width 0.3s',
                    }} />
                  </div>
                  <span style={{ color: '#d4c098', fontSize: 10, fontWeight: 700, minWidth: 28, textAlign: 'right' }}>
                    {treasury}💰
                  </span>
                </div>
              </div>
            </div>
          )
        })}

        <div style={{ flex: 1 }} />

        {/* Campaign date in bar */}
        <div style={{
          color: '#c8b078',
          fontSize: 13,
          fontFamily: 'Cinzel, serif',
          fontWeight: 700,
          letterSpacing: 1,
          flexShrink: 0,
          padding: '0 8px',
        }}>
          {turnDate(scenarioData?.start, currentTurn)}
        </div>
      </div>

      {/* ────────────────────────────────────────────────────
          MAP VIEWPORT — fills remaining height
          ──────────────────────────────────────────────────── */}
      <div
        style={{ flex: 1, position: 'relative', overflow: 'hidden', cursor: dragRef.current.active ? 'grabbing' : 'grab' }}
        ref={containerRef}
        onWheel={handleWheel}
        onMouseDown={beginDrag}
        onMouseMove={duringDrag}
        onMouseUp={endDrag}
        onMouseLeave={endDrag}
      >
        <svg
          width="100%" height="100%"
          viewBox={`0 0 ${SVG_W} ${SVG_H}`}
          style={{
            transform: `translate(${pan.x}px,${pan.y}px) scale(${zoom})`,
            transformOrigin: '0 0',
            userSelect: 'none',
          }}
        >
          <defs>
            {/* Land clip path for keeping fills inside coastlines */}
            <clipPath id="landClip">
              {landPaths.map((d: string, i: number) => <path key={i} d={d} />)}
            </clipPath>

            {/* Glow filter for selected areas */}
            <filter id="selGlow" x="-20%" y="-20%" width="140%" height="140%">
              <feGaussianBlur stdDeviation="3" result="blur" />
              <feMerge>
                <feMergeNode in="blur" />
                <feMergeNode in="SourceGraphic" />
              </feMerge>
            </filter>

            {/* Battle pulse animation */}
            <filter id="battleGlow">
              <feGaussianBlur stdDeviation="4" result="blur" />
              <feMerge>
                <feMergeNode in="blur" />
                <feMergeNode in="blur" />
                <feMergeNode in="SourceGraphic" />
              </feMerge>
            </filter>

            {/* Arrowhead markers per power */}
            {Object.entries(POWER_COLORS).map(([pid, col]) => (
              <marker key={`ah-${pid}`} id={`arrowhead-${pid}`} markerWidth="10" markerHeight="7" refX="9" refY="3.5" orient="auto">
                <polygon points="0 0, 10 3.5, 0 7" fill={col} />
              </marker>
            ))}

            <style>{`
              @keyframes battlePulse {
                0%,100% { stroke: rgba(200,20,20,0.9); stroke-width: 3; }
                50%      { stroke: rgba(200,20,20,0.2); stroke-width: 1.5; }
              }
              .battle-flash { animation: battlePulse 0.8s infinite; fill: none; pointer-events: none; }
              @keyframes fadeInMap { from { opacity:0 } to { opacity:1 } }
              .map-toast { animation: fadeInMap 0.3s ease-out; }
            `}</style>
          </defs>

          {/* ── Ocean base ── */}
          <rect width={SVG_W} height={SVG_H} fill={OCEAN} />

          {/* Subtle ocean texture (very light noise) */}
          <rect width={SVG_W} height={SVG_H} fill="none"
            stroke="rgba(100,150,200,0.12)" strokeWidth={0} />

          {/* ── Land base (for areas without scenario data) ── */}
          {landPaths.map((d: string, i: number) => (
            <path key={`lb-${i}`} d={d} fill={LAND_BASE} stroke="none" />
          ))}

          {/* ── Territory fills — clipped to land ── */}
          <g clipPath="url(#landClip)">
            {terrData.map((t: any) => {
              const isSelected = selId === t.aid
              const isHovered  = hovId === t.aid && !isSelected
              const fill = isHovered
                ? lightenHex(t.baseColor, 40)
                : t.baseColor
              return (
                <path
                  key={t.aid}
                  d={t.path}
                  fill={fill}
                  fillOpacity={1}
                  stroke="none"
                  style={{ cursor: 'pointer', transition: 'fill 80ms ease' }}
                  onMouseEnter={() => setHovId(t.aid)}
                  onMouseLeave={() => setHovId(c => c === t.aid ? null : c)}
                  onClick={e => { e.stopPropagation(); if (!dragRef.current.moved) setSelId(t.aid) }}
                />
              )
            })}
          </g>

          {/* ── Territory borders (inside land) ── */}
          <g clipPath="url(#landClip)" style={{ pointerEvents: 'none' }}>
            {terrData.map((t: any) => (
              <path
                key={`b-${t.aid}`}
                d={t.path}
                fill="none"
                stroke={BORDER_CLR}
                strokeWidth={0.5}
                strokeOpacity={0.7}
              />
            ))}
          </g>

          {/* ── Selected territory — gold highlight border ── */}
          {selId && terrData.map((t: any) => {
            if (t.aid !== selId) return null
            return (
              <path
                key={`sel-${t.aid}`}
                d={t.path}
                fill="rgba(255,215,0,0.18)"
                stroke="#FFD700"
                strokeWidth={2}
                style={{ pointerEvents: 'none' }}
                filter="url(#selGlow)"
              />
            )
          })}

          {/* ── Coastlines (land outline) — dark thin lines ── */}
          {landPaths.map((d: string, i: number) => (
            <path key={`co-${i}`} d={d} fill="none" stroke={COAST_CLR} strokeWidth={0.8} strokeOpacity={0.85} style={{ pointerEvents: 'none' }} />
          ))}

          {/* ── Lakes ── */}
          {lakePaths.map((d: string, i: number) => (
            <path key={`lk-${i}`} d={d} fill={LAKE_CLR} stroke={BORDER_CLR} strokeWidth={0.4} strokeOpacity={0.5} style={{ pointerEvents: 'none' }} />
          ))}

          {/* ── Rivers ── */}
          {riverPaths.map((d: string, i: number) => (
            <path key={`rv-${i}`} d={d} fill="none" stroke={RIVER_CLR} strokeWidth={0.6} style={{ pointerEvents: 'none' }} />
          ))}

          {/* ── Territory labels ── */}
          {centers && terrData.map((t: any) => {
            const c = centers[t.aid]
            if (!c) return null
            const p = projection(c as [number, number])
            if (!p) return null
            const lbl = shortLabel(fmtArea(t.aid, t.area?.display_name))
            /* Use dark text for Ottoman (light fill), else dark */
            const txtColor = '#1a1a1a'
            return (
              <text
                key={`lbl-${t.aid}`}
                x={p[0]} y={p[1]}
                textAnchor="middle"
                dominantBaseline="central"
                style={{
                  pointerEvents: 'none',
                  fontFamily: "'Noto Serif', Georgia, serif",
                  fontWeight: 600,
                  fontSize: 6.5,
                  letterSpacing: 0.4,
                  fill: txtColor,
                }}
                stroke="rgba(255,255,255,0.65)"
                strokeWidth={1.8}
                paintOrder="stroke"
              >
                {lbl}
              </text>
            )
          })}

          {/* ── Corps markers ── */}
          {centers && Object.entries(corpsByArea).flatMap(([aid, ac]) => {
            const c = centers[aid]
            if (!c) return []
            const p = projection(c as [number, number])
            if (!p) return []
            return ac.map((corps, i) => {
              const ox = (i % 3) * 14 - 14
              const oy = Math.floor(i / 3) * 14 + 10
              const col = POWER_COLORS[corps.owner] || '#666'
              return (
                <g key={corps.id} style={{ pointerEvents: 'none' }}>
                  {/* Flag pin */}
                  <circle cx={p[0] + ox} cy={p[1] + oy} r={7} fill="white" stroke={BORDER_CLR} strokeWidth={0.8} />
                  <circle cx={p[0] + ox} cy={p[1] + oy} r={5.5} fill={col} />
                  <text
                    x={p[0] + ox} y={p[1] + oy + 0.5}
                    textAnchor="middle" dominantBaseline="central"
                    style={{ fill: '#fff', fontSize: 5, fontWeight: 700, fontFamily: 'sans-serif' }}
                    stroke="rgba(0,0,0,0.5)" strokeWidth={1} paintOrder="stroke"
                  >
                    {corps.sp}
                  </text>
                </g>
              )
            })
          })}

          {/* ── Contested areas — split color overlay ── */}
          <g clipPath="url(#landClip)" style={{ pointerEvents: 'none' }}>
            {terrData.map((t: any) => {
              const ca = contestedMap[t.aid]
              if (!ca) return null
              const attCol = POWER_COLORS[ca.attacker] || '#888'
              const defCol = POWER_COLORS[ca.defender] || '#888'
              const splitPct = 50 + (ca.pressure / 2)
              const clipId = `split-${t.aid}`
              return (
                <g key={`contested-${t.aid}`}>
                  <defs>
                    <clipPath id={`${clipId}-left`}>
                      <rect x={0} y={0} width={SVG_W * splitPct / 100} height={SVG_H} />
                    </clipPath>
                    <clipPath id={`${clipId}-right`}>
                      <rect x={SVG_W * splitPct / 100} y={0} width={SVG_W * (100 - splitPct) / 100} height={SVG_H} />
                    </clipPath>
                  </defs>
                  <path d={t.path} fill={attCol} fillOpacity={0.45} clipPath={`url(#${clipId}-left)`} />
                  <path d={t.path} fill={defCol} fillOpacity={0.45} clipPath={`url(#${clipId}-right)`} />
                  <path d={t.path} fill="none" stroke="rgba(100,20,20,0.7)" strokeWidth={1.5} strokeDasharray="4 2 1 2" />
                </g>
              )
            })}
          </g>

          {/* ── Battle flash on contested areas ── */}
          <g clipPath="url(#landClip)" style={{ pointerEvents: 'none' }}>
            {terrData.map((t: any) => {
              if (!contestedMap[t.aid]) return null
              return (
                <path key={`flash-${t.aid}`} d={t.path} className="battle-flash" filter="url(#battleGlow)" />
              )
            })}
          </g>

          {/* ── Attack arrows ── */}
          {centers && attackArrows.map((arrow, i) => {
            const fromC = centers[arrow.fromArea]
            const toC   = centers[arrow.toArea]
            if (!fromC || !toC) return null
            const p1 = projection(fromC as [number, number])
            const p2 = projection(toC   as [number, number])
            if (!p1 || !p2) return null
            const col       = POWER_COLORS[arrow.attacker] || '#555'
            const thickness = Math.max(2, Math.min(6, arrow.strength / 10000))
            return (
              <line
                key={`arrow-${i}`}
                x1={p1[0]} y1={p1[1]} x2={p2[0]} y2={p2[1]}
                stroke={col} strokeWidth={thickness} strokeOpacity={0.9}
                markerEnd={`url(#arrowhead-${arrow.attacker})`}
                style={{ pointerEvents: 'none' }}
              />
            )
          })}

          {/* ── Map title cartouche (bottom-left corner of SVG) ── */}
          <g transform="translate(24 940)" style={{ pointerEvents: 'none' }}>
            <rect x={0} y={0} width={240} height={50} rx={4}
              fill="rgba(240,235,220,0.88)" stroke={BORDER_CLR} strokeWidth={0.8} />
            <text x={120} y={18} textAnchor="middle"
              style={{ fontFamily: 'Cinzel Decorative, Cinzel, serif', fontSize: 13, fontWeight: 700, fill: '#2a1a0a', letterSpacing: 1 }}>
              Grand Campaign
            </text>
            <text x={120} y={38} textAnchor="middle"
              style={{ fontFamily: 'Cinzel, serif', fontSize: 11, fill: '#3a2a10', letterSpacing: 3 }}>
              1805
            </text>
          </g>

          {/* ── Compass rose (bottom-right) ── */}
          <g transform="translate(1540 940)" style={{ pointerEvents: 'none' }}>
            <circle cx={0} cy={0} r={32} fill="rgba(240,235,220,0.75)" stroke={BORDER_CLR} strokeWidth={0.8} />
            <path d="M0 -22 L5 0 L0 22 L-5 0 Z" fill="#2a1a0a" />
            <path d="M-22 0 L0 5 L22 0 L0 -5 Z" fill="#7a6a58" />
            <text x={0} y={-28} textAnchor="middle"
              style={{ fontSize: 10, fontFamily: 'Cinzel, serif', fontWeight: 700, fill: '#2a1a0a' }}>N</text>
          </g>
        </svg>

        {/* ── Battle toasts ── */}
        {visibleToasts.length > 0 && (
          <div style={{ position: 'absolute', bottom: 70, right: 14, display: 'flex', flexDirection: 'column', gap: 6, zIndex: 30 }}>
            {visibleToasts.map((toast, i) => {
              const resultText = toast.result === 'AttackerAdvances' ? `${POWER_NAMES[toast.attacker] ?? toast.attacker} advances!`
                : toast.result === 'DefenderRoutes' ? `${POWER_NAMES[toast.attacker] ?? toast.attacker} routs the enemy!`
                : toast.result === 'Stalemate' ? 'Stalemate!'
                : 'Defender holds!'
              const attCol = POWER_COLORS[toast.attacker] ?? '#888'
              return (
                <div key={`toast-${i}`} className="map-toast" style={{
                  background: 'rgba(240,235,220,0.96)',
                  border: `2px solid ${attCol}`,
                  padding: '8px 14px',
                  borderRadius: 4,
                  minWidth: 220,
                  boxShadow: '0 4px 16px rgba(0,0,0,0.35)',
                  color: '#1a1a1a',
                  fontFamily: 'Noto Serif, serif',
                }}>
                  <div style={{ fontSize: 13, fontWeight: 700 }}>
                    ⚔️ {toast.areaName}: {resultText}
                  </div>
                </div>
              )
            })}
          </div>
        )}

        {/* ── Zoom controls ── */}
        <div style={{ position: 'absolute', top: 14, right: 14, display: 'flex', flexDirection: 'column', gap: 4 }}>
          {[
            { label: '+', fn: () => stepZoom(1.25) },
            { label: '−', fn: () => stepZoom(1 / 1.25) },
            { label: '↺', fn: resetView },
          ].map(b => (
            <button key={b.label} onClick={b.fn} style={{
              background: 'rgba(240,235,220,0.92)',
              color: '#2a1a0a',
              border: `1px solid ${BORDER_CLR}`,
              width: 36,
              height: 36,
              cursor: 'pointer',
              fontFamily: 'Cinzel, serif',
              fontSize: b.label === '↺' ? 14 : 20,
              fontWeight: 700,
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              borderRadius: 3,
              boxShadow: '0 1px 4px rgba(0,0,0,0.2)',
            }}>
              {b.label}
            </button>
          ))}
        </div>

        {/* ── Area inspector panel (left side, over map) ── */}
        <div style={{
          position: 'absolute',
          left: 14,
          bottom: 60,
          background: 'rgba(240,235,222,0.97)',
          border: `1px solid ${BORDER_CLR}`,
          padding: '12px 14px',
          width: 260,
          borderRadius: 4,
          boxShadow: '0 4px 16px rgba(0,0,0,0.25)',
          fontFamily: 'Noto Serif, serif',
        }}>
          {selInfo ? (
            <>
              <div style={{ fontSize: 16, fontWeight: 700, color: '#1a1a1a', borderBottom: `1px solid ${BORDER_CLR}`, paddingBottom: 6, marginBottom: 8 }}>
                {selInfo.name}
              </div>
              <div style={{ fontSize: 11, color: '#6a5a40', letterSpacing: 1.2, textTransform: 'uppercase', marginBottom: 2 }}>Owner</div>
              <div style={{ fontSize: 13, color: '#1a1a1a', marginBottom: 8 }}>{ownerLabel(selInfo.area)}</div>
              <div style={{ fontSize: 11, color: '#6a5a40', letterSpacing: 1.2, textTransform: 'uppercase', marginBottom: 2 }}>Terrain</div>
              <div style={{ fontSize: 13, color: '#1a1a1a', textTransform: 'capitalize', marginBottom: 8 }}>
                {(selInfo.area?.terrain || 'unknown').toLowerCase()}
              </div>
              {selInfo.corps.length > 0 && (
                <>
                  <div style={{ fontSize: 11, color: '#6a5a40', letterSpacing: 1.2, textTransform: 'uppercase', marginBottom: 4 }}>Forces</div>
                  {selInfo.corps.map(c => {
                    const col = POWER_COLORS[c.owner] ?? '#888'
                    return (
                      <div key={c.id} style={{ fontSize: 12, color: '#1a1a1a', display: 'flex', alignItems: 'center', gap: 6, marginBottom: 3 }}>
                        <div style={{ width: 8, height: 8, borderRadius: '50%', background: col, flexShrink: 0 }} />
                        {c.displayName} · {c.sp} SP
                      </div>
                    )
                  })}
                </>
              )}
            </>
          ) : (
            <div style={{ fontSize: 13, color: '#5a4a30', lineHeight: 1.55 }}>
              Click a territory to inspect its owner, terrain, and military presence.
            </div>
          )}
        </div>

        {/* ── Legend ── */}
        <div style={{
          position: 'absolute',
          right: 14,
          bottom: 60,
          background: 'rgba(240,235,222,0.97)',
          border: `1px solid ${BORDER_CLR}`,
          padding: '10px 12px',
          width: 180,
          borderRadius: 4,
          boxShadow: '0 4px 16px rgba(0,0,0,0.2)',
        }}>
          <div style={{ fontSize: 10, letterSpacing: 1.5, color: '#5a4a30', textTransform: 'uppercase', marginBottom: 8, fontFamily: 'Cinzel, serif' }}>
            Legend
          </div>
          {Object.entries(POWER_NAMES).map(([pid, name]) => (
            <div key={pid} style={{ display: 'flex', alignItems: 'center', gap: 6, marginBottom: 4 }}>
              <div style={{ width: 16, height: 10, background: POWER_COLORS[pid], border: `0.5px solid ${BORDER_CLR}`, flexShrink: 0 }} />
              <span style={{ fontSize: 11, color: '#1a1a1a', fontFamily: 'Noto Serif, serif' }}>{name}</span>
            </div>
          ))}
          <div style={{ marginTop: 6, borderTop: `0.5px solid #ccc`, paddingTop: 6 }}>
            {[
              { color: MINOR_COLORS.german, label: 'German States' },
              { color: MINOR_COLORS.italian, label: 'Italian States' },
              { color: MINOR_COLORS.scandinavian, label: 'Scandinavia' },
              { color: MINOR_COLORS.north_africa, label: 'N. Africa' },
            ].map(({ color, label }) => (
              <div key={label} style={{ display: 'flex', alignItems: 'center', gap: 6, marginBottom: 3 }}>
                <div style={{ width: 16, height: 10, background: color, border: `0.5px solid ${BORDER_CLR}`, flexShrink: 0 }} />
                <span style={{ fontSize: 10, color: '#4a3a20', fontFamily: 'Noto Serif, serif' }}>{label}</span>
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* ────────────────────────────────────────────────────
          BOTTOM BAR — End Turn
          ──────────────────────────────────────────────────── */}
      <div style={{
        height: 50,
        background: 'linear-gradient(180deg, #1a1612 0%, #0e0c0a 100%)',
        borderTop: '2px solid #3a3028',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        padding: '0 16px',
        flexShrink: 0,
      }}>
        <div style={{ color: '#c8b078', fontSize: 13, letterSpacing: 1.4, fontFamily: 'Cinzel, serif' }}>
          GRAND CAMPAIGN 1805 · Turn {currentTurn + 1} · {turnDate(scenarioData?.start, currentTurn)}
        </div>
        <button onClick={onEndTurn} style={{
          background: 'linear-gradient(180deg, #c8a030 0%, #8b6818 100%)',
          color: '#fff8e8',
          border: '1px solid #e8c060',
          borderRadius: 3,
          padding: '8px 24px',
          cursor: 'pointer',
          fontFamily: 'Cinzel, serif',
          fontSize: 12,
          fontWeight: 700,
          letterSpacing: 2,
          textTransform: 'uppercase',
          boxShadow: '0 2px 8px rgba(0,0,0,0.4)',
        }}>
          End Turn →
        </button>
      </div>
    </div>
  )
}
