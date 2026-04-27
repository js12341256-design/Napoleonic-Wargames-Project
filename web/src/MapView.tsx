import React, { useEffect, useMemo, useRef, useState } from 'react'
import { geoMercator, geoPath } from 'd3-geo'

/* ───────── constants ───────── */
const SVG_W = 1600
const SVG_H = 1000
const MIN_ZOOM = 0.3
const MAX_ZOOM = 20

const POWER_COLORS: Record<string, string> = {
  FRA: '#1565C0',
  GBR: '#B71C1C',
  AUS: '#F9A825',
  PRU: '#455A64',
  RUS: '#2E7D32',
  OTT: '#6A1B9A',
  SPA: '#E65100',
}
const NEUTRAL_COLOR = '#8D6E63'

const POWER_NAMES: Record<string, string> = {
  FRA: 'France',
  GBR: 'Britain',
  AUS: 'Austria',
  PRU: 'Prussia',
  RUS: 'Russia',
  SPA: 'Spain',
  OTT: 'Ottoman',
}

const OCEAN = '#1a2744'
const LAND_BASE = '#2a2a1e'
const BORDER_CLR = 'rgba(255,255,255,0.35)'
const COAST_CLR = 'rgba(180,200,240,0.45)'
const RIVER_CLR = 'rgba(60,100,180,0.45)'
const LAKE_CLR = '#1e3455'

/* ───────── terrain classification ───────── */
const MOUNTAIN_AREAS = new Set([
  'alps', 'carpathians', 'pyrenees', 'urals', 'tyrol', 'switzerland', 'savoy',
  'AREA_alps', 'AREA_carpathians', 'AREA_pyrenees', 'AREA_urals', 'AREA_tyrol', 'AREA_switzerland', 'AREA_savoy',
])
const FOREST_AREAS = new Set([
  'black_forest', 'bohemia', 'thuringia', 'lithuania', 'finland',
  'AREA_black_forest', 'AREA_bohemia', 'AREA_thuringia', 'AREA_lithuania', 'AREA_finland',
])
const COAST_AREAS = new Set([
  'brittany', 'normandy', 'naples', 'sicily', 'andalusia', 'catalonia', 'portugal', 'holland', 'denmark', 'norway', 'dalmatia', 'greece', 'crete',
  'AREA_brittany', 'AREA_normandy', 'AREA_naples', 'AREA_sicily', 'AREA_andalusia', 'AREA_catalonia', 'AREA_portugal', 'AREA_holland', 'AREA_denmark', 'AREA_norway', 'AREA_dalmatia', 'AREA_greece', 'AREA_crete',
])

/* Capital territories for star markers */
const CAPITALS: Record<string, string> = {
  'paris': 'FRA', 'AREA_paris': 'FRA', 'ile_de_france': 'FRA', 'AREA_ile_de_france': 'FRA',
  'london': 'GBR', 'AREA_london': 'GBR', 'england': 'GBR', 'AREA_england': 'GBR',
  'vienna': 'AUS', 'AREA_vienna': 'AUS', 'vie': 'AUS',
  'berlin': 'PRU', 'AREA_berlin': 'PRU', 'brandenburg': 'PRU', 'AREA_brandenburg': 'PRU',
  'moscow': 'RUS', 'AREA_moscow': 'RUS',
  'constantinople': 'OTT', 'AREA_constantinople': 'OTT',
  'madrid': 'SPA', 'AREA_madrid': 'SPA', 'castile': 'SPA', 'AREA_castile': 'SPA',
}

function getTerrainType(aid: string, area: any): 'mountain' | 'forest' | 'coast' | 'plains' {
  const terrain = (area?.terrain || '').toLowerCase()
  if (terrain.includes('mountain') || terrain.includes('alpine') || MOUNTAIN_AREAS.has(aid)) return 'mountain'
  if (terrain.includes('forest') || terrain.includes('wood') || FOREST_AREAS.has(aid)) return 'forest'
  if (terrain.includes('coast') || terrain.includes('port') || COAST_AREAS.has(aid)) return 'coast'
  return 'plains'
}

/* ───────── front line types ───────── */
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
  pressure: number // -100 to +100
}

export interface BattleToast {
  area: string
  areaName: string
  attacker: string
  result: 'AttackerAdvances' | 'Stalemate' | 'DefenderHolds' | 'DefenderRoutes'
  timestamp: number
}

/* ───────── helpers ───────── */
type CorpsInfo = { id: string; owner: string; sp: number; area: string; displayName: string }

interface MapViewProps {
  scenarioData: any
  powerStates: Record<string, any>
  currentTurn: number
  onEndTurn: () => void
  attackArrows?: AttackArrow[]
  contestedAreas?: ContestedArea[]
  battleToasts?: BattleToast[]
  selectedTerritory?: string | null
  onSelectTerritory?: (id: string | null) => void
}

function clamp(v: number, lo: number, hi: number) {
  return Math.max(lo, Math.min(hi, v))
}

function ownerColor(area: any): string {
  const o = area?.owner
  if (!o) return NEUTRAL_COLOR
  if (o.kind === 'POWER' && o.power) return POWER_COLORS[o.power] || NEUTRAL_COLOR
  return NEUTRAL_COLOR
}

function ownerPowerId(area: any): string | null {
  const o = area?.owner
  if (o?.kind === 'POWER') return o.power || null
  return null
}

function fmtArea(id: string, dn?: string) {
  if (dn) return dn
  return id.replace(/^AREA_/, '').replace(/_/g, ' ').replace(/\b\w/g, c => c.toUpperCase())
}

function shortLabel(n: string) {
  if (n.length <= 12) return n
  return n.replace('Saint', 'St.').replace('Petersburg', 'Pete.').replace('Colonies', 'Cols.').replace('Swedish', 'Swed.')
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

/** Darken a hex color by a percentage (0-1) */
function darkenColor(hex: string, amount: number): string {
  const r = parseInt(hex.slice(1, 3), 16)
  const g = parseInt(hex.slice(3, 5), 16)
  const b = parseInt(hex.slice(5, 7), 16)
  const dr = Math.round(r * (1 - amount))
  const dg = Math.round(g * (1 - amount))
  const db = Math.round(b * (1 - amount))
  return `#${dr.toString(16).padStart(2, '0')}${dg.toString(16).padStart(2, '0')}${db.toString(16).padStart(2, '0')}`
}

/* ───────── component ───────── */
export default function MapView({ scenarioData, powerStates, currentTurn, onEndTurn, attackArrows = [], contestedAreas = [], battleToasts = [], selectedTerritory, onSelectTerritory }: MapViewProps) {
  const containerRef = useRef<HTMLDivElement>(null)
  const dragRef = useRef({ active: false, x: 0, y: 0, sx: 0, sy: 0, moved: false })

  const [zoom, setZoom] = useState(1)
  const [pan, setPan] = useState({ x: 0, y: 0 })
  const [selId, setSelId] = useState<string | null>(null)
  const [hovId, setHovId] = useState<string | null>(null)

  // Sync external selectedTerritory prop
  const effectiveSelId = selectedTerritory !== undefined ? selectedTerritory : selId
  const handleSelect = (id: string | null) => {
    if (onSelectTerritory) onSelectTerritory(id)
    else setSelId(id)
  }

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

  /* pre-render paths */
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
      const baseColor = ownerColor(a)
      const terrain = getTerrainType(aid, a)
      let displayColor = baseColor
      if (terrain === 'mountain') displayColor = darkenColor(baseColor, 0.3)
      else if (terrain === 'forest') displayColor = baseColor // green overlay applied separately
      return { aid, area: a, color: baseColor, displayColor, terrain, path: pathGen(f) || '' }
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

  /* corps grouping */
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

  /* contested area lookup */
  const contestedMap = useMemo(() => {
    const m: Record<string, ContestedArea> = {}
    for (const ca of contestedAreas) m[ca.areaId] = ca
    return m
  }, [contestedAreas])

  /* visible toasts (fade after 3s) */
  const [visibleToasts, setVisibleToasts] = useState<BattleToast[]>([])
  useEffect(() => {
    if (battleToasts.length === 0) return
    setVisibleToasts(battleToasts)
    const timer = setTimeout(() => setVisibleToasts([]), 3000)
    return () => clearTimeout(timer)
  }, [battleToasts])

  /* selected area */
  const selInfo = useMemo(() => {
    if (!effectiveSelId) return null
    const a = scenarioData?.areas?.[effectiveSelId]
    return { id: effectiveSelId, area: a, corps: corpsByArea[effectiveSelId] || [], name: fmtArea(effectiveSelId, a?.display_name) }
  }, [effectiveSelId, scenarioData, corpsByArea])

  /* ── zoom / pan handlers ── */
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
    <div style={{ height: '100%', display: 'flex', flexDirection: 'column', background: '#0a0a12', fontFamily: 'Cinzel,serif', color: '#f1dfb1' }}>
      {/* ── top power bar ── HIDDEN: App.tsx owns the top bar now ── */}
      <div style={{ display: 'none' }}>
        {Object.entries(POWER_NAMES).map(([pid, name]) => {
          const t = powerStates?.[pid]?.treasury ?? scenarioData?.powers?.[pid]?.starting_treasury ?? 0
          return (
            <div key={pid} style={{ minWidth: 142, height: 36, border: `1px solid ${POWER_COLORS[pid]}`, background: 'rgba(25,16,8,0.92)', display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '0 10px' }}>
              <div>
                <div style={{ color: POWER_COLORS[pid], fontSize: 12, fontWeight: 700, letterSpacing: 1.4 }}>{name}</div>
                <div style={{ color: '#cbb58a', fontSize: 9, letterSpacing: 1.5 }}>TREASURY</div>
              </div>
              <div style={{ color: '#f4df9e', fontSize: 18, fontWeight: 700 }}>{t}</div>
            </div>
          )
        })}
      </div>

      {/* ── map viewport ── */}
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
          style={{ transform: `translate(${pan.x}px,${pan.y}px) scale(${zoom})`, transformOrigin: '0 0', userSelect: 'none' }}
        >
          {/* ocean */}
          <rect width={SVG_W} height={SVG_H} fill={OCEAN} />

          {/* defs: clips, filters, markers, patterns, animations */}
          <defs>
            <clipPath id="lc">
              {landPaths.map((d: string, i: number) => <path key={i} d={d} />)}
            </clipPath>
            <filter id="glow">
              <feGaussianBlur stdDeviation="3" result="blur" />
              <feMerge><feMergeNode in="blur" /><feMergeNode in="SourceGraphic" /></feMerge>
            </filter>
            {/* arrowhead marker per power color */}
            {Object.entries(POWER_COLORS).map(([pid, col]) => (
              <marker key={`ah-${pid}`} id={`arrowhead-${pid}`} markerWidth="10" markerHeight="7" refX="9" refY="3.5" orient="auto">
                <polygon points="0 0, 10 3.5, 0 7" fill={col} />
              </marker>
            ))}
            {/* battle pulse filter */}
            <filter id="battleGlow">
              <feGaussianBlur stdDeviation="4" result="blur" />
              <feMerge><feMergeNode in="blur" /><feMergeNode in="blur" /><feMergeNode in="SourceGraphic" /></feMerge>
            </filter>
            {/* Mountain stipple pattern */}
            <pattern id="mountain-stipple" width="6" height="6" patternUnits="userSpaceOnUse">
              <rect width="6" height="6" fill="none" />
              <circle cx="1" cy="1" r="0.6" fill="rgba(255,255,255,0.25)" />
              <circle cx="4" cy="4" r="0.6" fill="rgba(255,255,255,0.25)" />
            </pattern>
            {/* Coast shimmer gradient */}
            <linearGradient id="coast-shimmer" x1="0%" y1="0%" x2="100%" y2="100%">
              <stop offset="0%" stopColor="rgba(100,160,255,0.08)">
                <animate attributeName="stopColor" values="rgba(100,160,255,0.08);rgba(100,160,255,0.2);rgba(100,160,255,0.08)" dur="3s" repeatCount="indefinite" />
              </stop>
              <stop offset="100%" stopColor="rgba(60,120,220,0.05)">
                <animate attributeName="stopColor" values="rgba(60,120,220,0.05);rgba(60,120,220,0.15);rgba(60,120,220,0.05)" dur="3s" repeatCount="indefinite" />
              </stop>
            </linearGradient>
            {/* Hover brightness filter */}
            <filter id="hover-bright">
              <feComponentTransfer>
                <feFuncR type="linear" slope="1.2" intercept="0.05" />
                <feFuncG type="linear" slope="1.2" intercept="0.05" />
                <feFuncB type="linear" slope="1.2" intercept="0.05" />
              </feComponentTransfer>
            </filter>
            <style>{`
              @keyframes battlePulse {
                0%, 100% { stroke: rgba(255,40,40,0.9); stroke-width: 3; }
                50% { stroke: rgba(255,40,40,0.15); stroke-width: 1.5; }
              }
              .battle-flash { animation: battlePulse 0.8s infinite; fill: none; pointer-events: none; }
              @keyframes arrowDash {
                to { stroke-dashoffset: -24; }
              }
              .attack-arrow-animated {
                stroke-dasharray: 12 6 4 6;
                animation: arrowDash 1.2s linear infinite;
              }
              @keyframes warPulse {
                0%, 100% { opacity: 1; }
                50% { opacity: 0.4; }
              }
            `}</style>
          </defs>

          {/* land base fill */}
          {landPaths.map((d: string, i: number) => (
            <path key={`lb-${i}`} d={d} fill={LAND_BASE} stroke="none" />
          ))}

          {/* territory fills — clipped to land so colours don't bleed into ocean */}
          <g clipPath="url(#lc)">
            {terrData.map((t: any) => {
              const isSelected = effectiveSelId === t.aid
              const isHovered = hovId === t.aid
              return (
                <g key={t.aid}>
                  {/* Base territory fill */}
                  <path
                    d={t.path}
                    fill={t.terrain === 'mountain' ? t.displayColor : t.color}
                    fillOpacity={isSelected ? 0.95 : isHovered ? 0.88 : 0.75}
                    stroke="none"
                    style={{ cursor: 'pointer', transition: 'fill-opacity 100ms' }}
                    filter={isHovered && !isSelected ? 'url(#hover-bright)' : undefined}
                    onMouseEnter={() => setHovId(t.aid)}
                    onMouseLeave={() => setHovId(c => c === t.aid ? null : c)}
                    onClick={e => { e.stopPropagation(); if (!dragRef.current.moved) handleSelect(t.aid) }}
                  />
                  {/* Mountain stipple overlay */}
                  {t.terrain === 'mountain' && (
                    <path d={t.path} fill="url(#mountain-stipple)" fillOpacity={0.7} style={{ pointerEvents: 'none' }} />
                  )}
                  {/* Forest green tint overlay */}
                  {t.terrain === 'forest' && (
                    <path d={t.path} fill="rgba(34,120,34,0.20)" style={{ pointerEvents: 'none' }} />
                  )}
                  {/* Coast shimmer overlay */}
                  {t.terrain === 'coast' && (
                    <path d={t.path} fill="url(#coast-shimmer)" style={{ pointerEvents: 'none' }} />
                  )}
                </g>
              )
            })}
          </g>

          {/* territory borders — clipped to land */}
          <g clipPath="url(#lc)" style={{ pointerEvents: 'none' }}>
            {terrData.map((t: any) => {
              const isSelected = effectiveSelId === t.aid
              const isHovered = hovId === t.aid && !isSelected
              return (
                <path
                  key={`b-${t.aid}`}
                  d={t.path}
                  fill="none"
                  stroke={isSelected ? '#d4af37' : isHovered ? 'rgba(255,255,255,0.85)' : BORDER_CLR}
                  strokeWidth={isSelected ? 2 : isHovered ? 2 : 0.8}
                  style={{ transition: 'stroke 0.15s, stroke-width 0.15s' }}
                />
              )
            })}
          </g>

          {/* coastlines */}
          {landPaths.map((d: string, i: number) => (
            <path key={`co-${i}`} d={d} fill="none" stroke={COAST_CLR} strokeWidth={1.2} style={{ pointerEvents: 'none' }} />
          ))}

          {/* lakes */}
          {lakePaths.map((d: string, i: number) => (
            <path key={`lk-${i}`} d={d} fill={LAKE_CLR} stroke="rgba(100,140,200,0.3)" strokeWidth={0.5} style={{ pointerEvents: 'none' }} />
          ))}

          {/* rivers */}
          {riverPaths.map((d: string, i: number) => (
            <path key={`rv-${i}`} d={d} fill="none" stroke={RIVER_CLR} strokeWidth={0.7} style={{ pointerEvents: 'none' }} />
          ))}

          {/* territory labels */}
          {centers && terrData.map((t: any) => {
            const c = centers[t.aid]
            if (!c) return null
            const p = projection(c as [number, number])
            if (!p) return null
            const lbl = shortLabel(fmtArea(t.aid, t.area?.display_name))
            return (
              <text
                key={`lbl-${t.aid}`}
                x={p[0]} y={p[1]}
                textAnchor="middle" dominantBaseline="central"
                style={{ pointerEvents: 'none', fontFamily: 'Cinzel,serif', fontWeight: 700, fontSize: 7, letterSpacing: 0.5 }}
                fill="rgba(255,255,255,0.9)"
                stroke="rgba(0,0,0,0.7)"
                strokeWidth={2}
                paintOrder="stroke"
              >
                {lbl}
              </text>
            )
          })}

          {/* Capital star markers */}
          {centers && Object.entries(CAPITALS).map(([areaId, power]) => {
            const c = centers[areaId]
            if (!c) return null
            const p = projection(c as [number, number])
            if (!p) return null
            return (
              <text
                key={`cap-${areaId}`}
                x={p[0]} y={p[1] - 10}
                textAnchor="middle" dominantBaseline="central"
                style={{ pointerEvents: 'none', fontSize: 10, fontWeight: 700 }}
                fill={POWER_COLORS[power] || '#d4af37'}
                stroke="rgba(0,0,0,0.6)"
                strokeWidth={1.5}
                paintOrder="stroke"
              >
                ★
              </text>
            )
          })}

          {/* corps markers — shield shapes */}
          {centers && Object.entries(corpsByArea).flatMap(([aid, ac]) => {
            const c = centers[aid]
            if (!c) return []
            const p = projection(c as [number, number])
            if (!p) return []
            return ac.map((corps, i) => {
              const ox = (i % 3) * 16 - 16
              const oy = Math.floor(i / 3) * 18 + 10
              const cx = p[0] + ox
              const cy = p[1] + oy
              const col = POWER_COLORS[corps.owner] || '#f2d89d'
              // Shield polygon points (centered at cx, cy)
              const shieldPoints = `${cx},${cy - 7} ${cx + 6},${cy - 5} ${cx + 6},${cy + 2} ${cx},${cy + 7} ${cx - 6},${cy + 2} ${cx - 6},${cy - 5}`
              return (
                <g key={corps.id} style={{ pointerEvents: 'none' }}>
                  <polygon points={shieldPoints} fill="#0a0a12" opacity={0.85} />
                  <polygon points={shieldPoints} fill={col} stroke="#fff" strokeWidth={0.8} />
                  <text
                    x={cx} y={cy}
                    textAnchor="middle" dominantBaseline="central"
                    style={{ fill: '#fff', fontSize: 5.5, fontWeight: 700, fontFamily: 'sans-serif' }}
                  >
                    {corps.sp}
                  </text>
                </g>
              )
            })
          })}

          {/* ── FRONT LINE LAYERS ── */}

          {/* Contested territory pressure overlay — bicolor split */}
          <g clipPath="url(#lc)" style={{ pointerEvents: 'none' }}>
            {terrData.map((t: any) => {
              const ca = contestedMap[t.aid]
              if (!ca) return null
              const attCol = POWER_COLORS[ca.attacker] || '#888'
              const defCol = POWER_COLORS[ca.defender] || '#888'
              // Split position: 50% at pressure=0, slides with pressure
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
                  <path d={t.path} fill={attCol} fillOpacity={0.5} clipPath={`url(#${clipId}-left)`} />
                  <path d={t.path} fill={defCol} fillOpacity={0.5} clipPath={`url(#${clipId}-right)`} />
                  {/* jagged front line divider */}
                  <path d={t.path} fill="none" stroke="rgba(255,255,255,0.6)" strokeWidth={1.5} strokeDasharray="4 2 1 2" />
                </g>
              )
            })}
          </g>

          {/* Battle flash — pulsing red border on contested areas */}
          <g clipPath="url(#lc)" style={{ pointerEvents: 'none' }}>
            {terrData.map((t: any) => {
              if (!contestedMap[t.aid]) return null
              return (
                <path
                  key={`flash-${t.aid}`}
                  d={t.path}
                  className="battle-flash"
                  filter="url(#battleGlow)"
                />
              )
            })}
          </g>

          {/* Attack arrows — animated pulsing dashes */}
          {centers && attackArrows.map((arrow, i) => {
            const fromC = centers[arrow.fromArea]
            const toC = centers[arrow.toArea]
            if (!fromC || !toC) return null
            const p1 = projection(fromC as [number, number])
            const p2 = projection(toC as [number, number])
            if (!p1 || !p2) return null
            const col = POWER_COLORS[arrow.attacker] || '#fff'
            const thickness = Math.max(1.5, Math.min(6, arrow.strength / 20))
            return (
              <g key={`arrow-${i}`} style={{ pointerEvents: 'none' }}>
                {/* Glow underlay */}
                <line
                  x1={p1[0]} y1={p1[1]}
                  x2={p2[0]} y2={p2[1]}
                  stroke={col}
                  strokeWidth={thickness + 3}
                  strokeOpacity={0.2}
                  strokeLinecap="round"
                />
                {/* Animated dashed line */}
                <line
                  x1={p1[0]} y1={p1[1]}
                  x2={p2[0]} y2={p2[1]}
                  stroke={col}
                  strokeWidth={thickness}
                  strokeOpacity={0.9}
                  markerEnd={`url(#arrowhead-${arrow.attacker})`}
                  className="attack-arrow-animated"
                />
              </g>
            )
          })}
        </svg>

        {/* battle result toasts — bottom right */}
        {visibleToasts.length > 0 && (
          <div style={{ position: 'absolute', bottom: 80, right: 14, display: 'flex', flexDirection: 'column', gap: 6, zIndex: 30 }}>
            {visibleToasts.map((toast, i) => {
              const resultText = toast.result === 'AttackerAdvances' ? `${POWER_NAMES[toast.attacker] || toast.attacker} advances!`
                : toast.result === 'DefenderRoutes' ? `${POWER_NAMES[toast.attacker] || toast.attacker} routs the enemy!`
                : toast.result === 'Stalemate' ? 'Stalemate!'
                : 'Defender holds!'
              return (
                <div key={`toast-${i}`} style={{
                  background: 'rgba(14,10,6,0.92)', border: '1px solid #c8a000',
                  padding: '8px 14px', borderRadius: 4, minWidth: 220,
                  boxShadow: '0 4px 16px rgba(0,0,0,0.5)',
                  animation: 'fadeIn 0.3s ease-out',
                }}>
                  <div style={{ color: '#f3e5bb', fontSize: 13, fontWeight: 700 }}>
                    {'⚔️'} {toast.areaName}: {resultText}
                  </div>
                </div>
              )
            })}
          </div>
        )}

        {/* zoom buttons */}
        <div style={{ position: 'absolute', top: 14, right: 14, display: 'flex', flexDirection: 'column', gap: 8 }}>
          {[
            { label: '+', fn: () => stepZoom(1.25) },
            { label: '\u2212', fn: () => stepZoom(1 / 1.25) },
            { label: 'Reset', fn: resetView },
          ].map(b => (
            <button key={b.label} onClick={b.fn} style={{
              background: 'rgba(18,10,4,0.92)', color: '#f1dfb1', border: '1px solid #7b6338',
              minWidth: 54, height: 38, cursor: 'pointer', fontFamily: 'Cinzel,serif',
              fontSize: b.label === 'Reset' ? 11 : 22, fontWeight: 700, letterSpacing: 1,
            }}>
              {b.label}
            </button>
          ))}
        </div>

        {/* campaign date */}
        <div style={{ position: 'absolute', left: 14, top: 14, background: 'rgba(14,10,6,0.88)', border: '1px solid #5b4527', padding: '10px 12px', minWidth: 180, boxShadow: '0 10px 20px rgba(0,0,0,0.25)' }}>
          <div style={{ color: '#d8bc76', fontSize: 11, letterSpacing: 2 }}>CAMPAIGN DATE</div>
          <div style={{ color: '#f3e5bb', fontSize: 22, fontWeight: 700 }}>{turnDate(scenarioData?.start, currentTurn)}</div>
          <div style={{ color: '#bca47d', fontSize: 11, marginTop: 2 }}>Turn {currentTurn + 1}</div>
        </div>

        {/* area inspector */}
        <div style={{ position: 'absolute', left: 14, bottom: 18, background: 'rgba(14,10,6,0.88)', border: '1px solid #5b4527', padding: '12px 14px', width: 280, boxShadow: '0 10px 20px rgba(0,0,0,0.25)' }}>
          {selInfo ? (
            <>
              <div style={{ color: '#f1dfb1', fontSize: 18, fontWeight: 700 }}>{selInfo.name}</div>
              <div style={{ color: '#bca47d', fontSize: 11, letterSpacing: 1.3, marginTop: 4 }}>OWNER</div>
              <div style={{ color: ownerColor(selInfo.area), fontSize: 14 }}>{ownerLabel(selInfo.area)}</div>
              <div style={{ color: '#bca47d', fontSize: 11, letterSpacing: 1.3, marginTop: 8 }}>TERRAIN</div>
              <div style={{ color: '#efe2bf', fontSize: 14, textTransform: 'capitalize' }}>{(selInfo.area?.terrain || 'unknown').toLowerCase()}</div>
              <div style={{ color: '#bca47d', fontSize: 11, letterSpacing: 1.3, marginTop: 8 }}>UNITS PRESENT</div>
              {selInfo.corps.length > 0
                ? selInfo.corps.map(c => (
                    <div key={c.id} style={{ color: '#efe2bf', fontSize: 13, marginTop: 3 }}>
                      {c.displayName} &middot; {POWER_NAMES[c.owner] || c.owner} &middot; {c.sp} SP
                    </div>
                  ))
                : <div style={{ color: '#8f7b61', fontSize: 13, marginTop: 3 }}>No corps present</div>}
            </>
          ) : (
            <>
              <div style={{ color: '#f1dfb1', fontSize: 16, fontWeight: 700 }}>Map Intelligence</div>
              <div style={{ color: '#bca47d', fontSize: 13, marginTop: 6, lineHeight: 1.5 }}>
                Click a territory to inspect its owner, terrain, and military presence.
              </div>
            </>
          )}
        </div>

        {/* legend */}
        <div style={{ position: 'absolute', right: 14, bottom: 18, background: 'rgba(14,10,6,0.88)', border: '1px solid #5b4527', padding: '10px 12px', width: 200 }}>
          <div style={{ color: '#d8bc76', fontSize: 11, letterSpacing: 2, marginBottom: 8 }}>LAYERS</div>
          <div style={{ color: '#cdb790', fontSize: 12, lineHeight: 1.6 }}>Land &middot; Territories &middot; Rivers &middot; Lakes &middot; Corps &middot; Front Lines</div>
          <div style={{ color: '#8d795a', fontSize: 11, marginTop: 8 }}>Real GeoJSON &middot; Mercator</div>
        </div>
      </div>

      {/* ── bottom bar ── */}
      <div style={{ height: 50, background: 'linear-gradient(180deg,#130d06,#090502)', borderTop: '1px solid #5a4524', display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '0 16px', flexShrink: 0 }}>
        <div style={{ color: '#d7c090', fontSize: 13, letterSpacing: 1.4 }}>GRAND CAMPAIGN 1805 &middot; {turnDate(scenarioData?.start, currentTurn)}</div>
        <button onClick={onEndTurn} style={{
          background: 'linear-gradient(180deg,#8b3a0a,#5a2005)', color: '#f0e0a0',
          border: '1px solid #c8a000', borderRadius: 3, padding: '10px 20px',
          cursor: 'pointer', fontFamily: 'Cinzel,serif', fontSize: 13, fontWeight: 700, letterSpacing: 2,
        }}>
          END TURN
        </button>
      </div>
    </div>
  )
}
