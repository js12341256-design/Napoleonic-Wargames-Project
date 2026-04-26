import React, { useMemo, useState } from 'react'
import { AREA_COORDS } from './MapData'
import AreaPolygon from './components/AreaPolygon'

interface GameMapProps {
  areas: Record<string, any>
  corps: Record<string, any>
  fleets: Record<string, any>
  powers: Record<string, any>
  powerStates: Record<string, any>
  currentTurn: number
  onAreaClick: (areaId: string) => void
  onEndTurn: () => void
  selectedArea: string | null
}

const OWNER_COLORS: Record<string, string> = {
  FRA: '#1a3a6a',
  GBR: '#8b0000',
  AUS: '#c8a000',
  PRU: '#2c3e50',
  RUS: '#27ae60',
  SPA: '#8e44ad',
  OTT: '#e67e22',
  MINOR: '#8b7355',
  UNOWNED: '#a89070',
}

function getOwnerKey(area: any) {
  const owner = area?.owner
  if (!owner) return 'UNOWNED'
  if (owner.kind === 'POWER') return owner.power || 'UNOWNED'
  if (owner.kind === 'MINOR') return 'MINOR'
  return 'UNOWNED'
}

function getOwnerLabel(area: any, powers: Record<string, any>) {
  const owner = area?.owner
  if (!owner) return 'Unowned'
  if (owner.kind === 'POWER') return powers[owner.power]?.display_name || owner.power
  if (owner.kind === 'MINOR') return owner.minor?.replace('MINOR_', '').replace(/_/g, ' ') || 'Minor Power'
  return 'Unowned'
}

function prettyName(areaId: string, area: any) {
  return area?.display_name || areaId.replace('AREA_', '').replace(/_/g, ' ')
}

function shortLabel(name: string) {
  if (name.length <= 12) return name
  return name
    .replace('Saint', 'St.')
    .replace('Petersburg', 'Pete.')
    .replace('Colonies', 'Cols.')
    .replace('Swedish', 'Swed.')
}

function corpsStrength(corps: any) {
  return Number(corps?.infantry_sp || 0) + Number(corps?.cavalry_sp || 0) + Number(corps?.artillery_sp || 0)
}

export default function GameMap({
  areas,
  corps,
  fleets,
  powers,
  powerStates,
  currentTurn,
  onAreaClick,
  onEndTurn,
  selectedArea,
}: GameMapProps) {
  const [hoveredArea, setHoveredArea] = useState<string | null>(null)

  const corpsByArea = useMemo(() => {
    const grouped: Record<string, any[]> = {}
    for (const corpsData of Object.values(corps || {})) {
      const areaId = (corpsData as any).area
      if (!areaId) continue
      grouped[areaId] ||= []
      grouped[areaId].push(corpsData)
    }
    return grouped
  }, [corps])

  const fleetsByArea = useMemo(() => {
    const grouped: Record<string, any[]> = {}
    for (const fleetData of Object.values(fleets || {})) {
      const areaId = (fleetData as any).at_port
      if (!areaId) continue
      grouped[areaId] ||= []
      grouped[areaId].push(fleetData)
    }
    return grouped
  }, [fleets])

  const selectedAreaData = selectedArea ? areas[selectedArea] : null
  const selectedAreaCorps = selectedArea ? corpsByArea[selectedArea] || [] : []
  const selectedAreaFleets = selectedArea ? fleetsByArea[selectedArea] || [] : []

  return (
    <div style={{ display: 'flex', height: '100vh', background: '#1a1209', color: '#f3e7d1', fontFamily: 'Cinzel, serif', position: 'relative', overflow: 'hidden' }}>
      <div
        style={{
          position: 'absolute',
          top: 0,
          left: 0,
          right: 0,
          height: 64,
          background: 'linear-gradient(180deg, rgba(12,8,4,0.95), rgba(33,22,12,0.82))',
          display: 'flex',
          alignItems: 'center',
          padding: '0 1rem',
          gap: '0.75rem',
          zIndex: 10,
          borderBottom: '1px solid rgba(224, 193, 139, 0.25)',
          overflowX: 'auto',
        }}
      >
        {Object.entries(powers).map(([powerId, power]) => {
          const state = powerStates[powerId] || {}
          const color = OWNER_COLORS[powerId] || (power as any).color_hex || OWNER_COLORS.MINOR
          return (
            <div
              key={powerId}
              style={{
                minWidth: 150,
                padding: '0.4rem 0.65rem',
                background: 'rgba(250, 239, 209, 0.08)',
                border: `1px solid ${color}`,
                borderRadius: 10,
                boxShadow: 'inset 0 0 0 1px rgba(255,255,255,0.05)',
              }}
            >
              <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 4 }}>
                <span style={{ width: 12, height: 12, borderRadius: 999, background: color, display: 'inline-block' }} />
                <strong style={{ fontSize: 13 }}>{(power as any).display_name || powerId}</strong>
              </div>
              <div style={{ fontSize: 11, opacity: 0.88 }}>Treasury {state.treasury ?? 0} · Prestige {state.prestige ?? 0}</div>
            </div>
          )
        })}
      </div>

      <div style={{ flex: 1, paddingTop: 64, paddingBottom: 58, paddingRight: 300 }}>
        <svg viewBox="0 0 1200 900" style={{ width: '100%', height: '100%', display: 'block' }}>
          <defs>
            <filter id="paperTexture">
              <feTurbulence type="fractalNoise" baseFrequency="0.8" numOctaves="2" stitchTiles="stitch" />
              <feColorMatrix type="saturate" values="0" />
              <feComponentTransfer>
                <feFuncA type="table" tableValues="0 0.08" />
              </feComponentTransfer>
            </filter>
          </defs>

          <rect width="1200" height="900" fill="#d4b896" />
          <rect width="1200" height="900" fill="#ead8b0" filter="url(#paperTexture)" opacity="0.6" />
          <rect width="1200" height="900" fill="#4a7fa8" opacity="0.22" />
          <ellipse cx="620" cy="420" rx="500" ry="310" fill="#ccb084" opacity="0.55" />
          <ellipse cx="920" cy="590" rx="320" ry="190" fill="#c4a678" opacity="0.35" />

          <g opacity="0.12" stroke="#6f5a43" strokeWidth="2" fill="none">
            <path d="M 110 170 C 180 130, 270 120, 360 165" />
            <path d="M 385 545 C 490 565, 620 550, 760 520" />
            <path d="M 650 150 C 720 170, 790 180, 855 165" />
          </g>

          {Object.entries(AREA_COORDS).map(([areaId, coords]) => {
            const area = areas[areaId]
            const ownerKey = getOwnerKey(area)
            const ownerColor = OWNER_COLORS[ownerKey] || OWNER_COLORS.MINOR
            return (
              <AreaPolygon
                key={areaId}
                areaId={areaId}
                coords={coords}
                ownerColor={ownerColor}
                isSelected={selectedArea === areaId}
                isHovered={hoveredArea === areaId}
                onClick={() => onAreaClick(areaId)}
                onMouseEnter={() => setHoveredArea(areaId)}
                onMouseLeave={() => setHoveredArea(current => (current === areaId ? null : current))}
              />
            )
          })}

          {Object.entries(AREA_COORDS).map(([areaId, coords]) => {
            const area = areas[areaId]
            return (
              <text
                key={`${areaId}-label`}
                x={coords.labelPos[0]}
                y={coords.labelPos[1]}
                textAnchor="middle"
                style={{ fill: '#2a2118', fontSize: 15, fontWeight: 700, letterSpacing: 0.5, pointerEvents: 'none' }}
              >
                {shortLabel(prettyName(areaId, area))}
              </text>
            )
          })}

          {Object.entries(corpsByArea).flatMap(([areaId, areaCorps]) => {
            const coords = AREA_COORDS[areaId]
            if (!coords) return []
            return areaCorps.map((corpsData, index) => {
              const ownerColor = OWNER_COLORS[(corpsData as any).owner] || OWNER_COLORS.MINOR
              const x = coords.center[0] - 14 + index * 18
              const y = coords.center[1] + 2 + (index % 2) * 16
              return (
                <g key={`${areaId}-${index}`}>
                  <path d={`M ${x} ${y} l 12 -8 l 12 8 l 0 14 l -12 8 l -12 -8 z`} fill={ownerColor} stroke="#f6ead0" strokeWidth="1.2" />
                  <text x={x + 12} y={y + 13} textAnchor="middle" style={{ fill: '#fdf6e3', fontSize: 10, fontWeight: 700 }}>
                    {corpsStrength(corpsData)}
                  </text>
                </g>
              )
            })
          })}

          {Object.entries(fleetsByArea).flatMap(([areaId, areaFleets]) => {
            const coords = AREA_COORDS[areaId]
            if (!coords) return []
            return areaFleets.map((fleetData, index) => {
              const ownerColor = OWNER_COLORS[(fleetData as any).owner] || OWNER_COLORS.MINOR
              const x = coords.center[0] + 18 + index * 14
              const y = coords.center[1] - 8
              return (
                <g key={`${areaId}-fleet-${index}`} transform={`translate(${x} ${y})`}>
                  <text x="0" y="0" style={{ fill: ownerColor, fontSize: 20, fontWeight: 700 }}>⚓</text>
                </g>
              )
            })
          })}

          <g transform="translate(960 110)">
            <circle cx="0" cy="0" r="46" fill="rgba(90,74,58,0.12)" stroke="#6a563f" strokeWidth="2" />
            <path d="M 0 -34 L 8 0 L 0 34 L -8 0 Z" fill="#6b4f2e" stroke="#f2dfb4" />
            <path d="M -34 0 L 0 8 L 34 0 L 0 -8 Z" fill="#6b4f2e" stroke="#f2dfb4" />
            <text x="0" y="-48" textAnchor="middle" style={{ fill: '#4b3a28', fontSize: 16, fontWeight: 700 }}>N</text>
          </g>

          <g transform="translate(150 95)">
            <rect width="260" height="72" rx="12" fill="rgba(84,61,39,0.75)" stroke="#e6c78f" strokeWidth="2" />
            <text x="130" y="30" textAnchor="middle" style={{ fill: '#f1dfb4', fontSize: 26, fontFamily: 'Cinzel Decorative, Cinzel, serif' }}>
              Grand Campaign
            </text>
            <text x="130" y="54" textAnchor="middle" style={{ fill: '#f7ead0', fontSize: 20, letterSpacing: 2 }}>
              1805
            </text>
          </g>
        </svg>
      </div>

      <aside
        style={{
          position: 'absolute',
          top: 64,
          right: 0,
          bottom: 58,
          width: 300,
          background: 'linear-gradient(180deg, rgba(33,20,10,0.93), rgba(57,38,20,0.9))',
          borderLeft: '1px solid rgba(224, 193, 139, 0.2)',
          padding: '1rem',
          overflowY: 'auto',
        }}
      >
        <h2 style={{ marginTop: 0, marginBottom: 10, fontSize: 24, color: '#e7c88d', fontFamily: 'Cinzel Decorative, Cinzel, serif' }}>Gazetteer</h2>
        {selectedArea && selectedAreaData ? (
          <>
            <div style={{ fontSize: 22, marginBottom: 8 }}>{prettyName(selectedArea, selectedAreaData)}</div>
            <div style={{ fontSize: 13, lineHeight: 1.7, color: '#eadfc8' }}>
              <div><strong>Owner:</strong> {getOwnerLabel(selectedAreaData, powers)}</div>
              <div><strong>Terrain:</strong> {selectedAreaData.terrain || 'Unknown'}</div>
              <div><strong>Fortifications:</strong> {selectedAreaData.fort_level ?? 0}</div>
              <div><strong>Port:</strong> {selectedAreaData.port ? 'Yes' : 'No'}</div>
              <div><strong>Capital:</strong> {selectedAreaData.capital_of || '—'}</div>
            </div>

            <div style={{ marginTop: 16 }}>
              <strong style={{ color: '#e7c88d' }}>Corps Present</strong>
              {selectedAreaCorps.length ? selectedAreaCorps.map((corpsData, index) => (
                <div key={index} style={{ marginTop: 8, padding: '0.55rem', border: '1px solid rgba(231,200,141,0.2)', borderRadius: 8, background: 'rgba(255,255,255,0.04)' }}>
                  <div style={{ fontSize: 14 }}>{(corpsData as any).display_name}</div>
                  <div style={{ fontSize: 12, opacity: 0.85 }}>
                    {(corpsData as any).owner} · Strength {corpsStrength(corpsData)} · Morale {Math.round(Number((corpsData as any).morale_q4 || 0) / 100)}
                  </div>
                </div>
              )) : <div style={{ marginTop: 8, fontSize: 13, opacity: 0.8 }}>No corps stationed here.</div>}
            </div>

            <div style={{ marginTop: 16 }}>
              <strong style={{ color: '#e7c88d' }}>Fleets in Port</strong>
              {selectedAreaFleets.length ? selectedAreaFleets.map((fleetData, index) => (
                <div key={index} style={{ marginTop: 8, padding: '0.55rem', border: '1px solid rgba(231,200,141,0.2)', borderRadius: 8, background: 'rgba(255,255,255,0.04)' }}>
                  <div style={{ fontSize: 14 }}>{(fleetData as any).display_name}</div>
                  <div style={{ fontSize: 12, opacity: 0.85 }}>
                    {(fleetData as any).owner} · SOL {(fleetData as any).ships_of_the_line ?? 0} · Frigates {(fleetData as any).frigates ?? 0}
                  </div>
                </div>
              )) : <div style={{ marginTop: 8, fontSize: 13, opacity: 0.8 }}>No fleet anchored here.</div>}
            </div>
          </>
        ) : (
          <div style={{ fontSize: 14, lineHeight: 1.7, color: '#eadfc8' }}>
            Select a province on the map to inspect its owner, terrain, and forces.
          </div>
        )}
      </aside>

      <div
        style={{
          position: 'absolute',
          left: 0,
          right: 0,
          bottom: 0,
          height: 58,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          padding: '0 1rem',
          background: 'linear-gradient(180deg, rgba(26,18,9,0.85), rgba(7,5,3,0.95))',
          borderTop: '1px solid rgba(224, 193, 139, 0.2)',
        }}
      >
        <div style={{ fontSize: 16, color: '#e8d3aa' }}>Turn {currentTurn} · Year of the Third Coalition</div>
        <button
          onClick={onEndTurn}
          style={{
            padding: '0.65rem 1.25rem',
            background: 'linear-gradient(180deg, #94713b, #6e5329)',
            color: '#fff7e5',
            border: '1px solid #e7c88d',
            borderRadius: 10,
            cursor: 'pointer',
            fontFamily: 'Cinzel, serif',
            fontWeight: 700,
            letterSpacing: 0.5,
          }}
        >
          End Turn → Economic Phase
        </button>
      </div>
    </div>
  )
}
