import React, { useState, useMemo, useCallback } from 'react'
import type { Focus, FocusTreeData, FocusEffect } from '../types'

// ── Constants ──

const POWER_TABS: { id: string; label: string; flag: string }[] = [
  { id: 'FRA', label: 'France', flag: '🇫🇷' },
  { id: 'GBR', label: 'Britain', flag: '🇬🇧' },
]

const NODE_W = 180
const NODE_H = 72
const GAP_X = 200
const GAP_Y = 120
const PAD_X = 60
const PAD_Y = 80

function effectSummary(effects: FocusEffect[]): string {
  const parts: string[] = []
  for (const e of effects) {
    if ('ManpowerBonus' in e && e.ManpowerBonus != null) parts.push(`+${(e.ManpowerBonus / 1000).toFixed(0)}k manpower`)
    if ('AttackBonus' in e && e.AttackBonus != null) parts.push(`+${e.AttackBonus}% attack`)
    if ('DefenseBonus' in e && e.DefenseBonus != null) parts.push(`+${e.DefenseBonus}% defense`)
    if ('SupplyRangeBonus' in e && e.SupplyRangeBonus != null) parts.push(`+${e.SupplyRangeBonus}% supply`)
    if ('TreasuryBonus' in e && e.TreasuryBonus != null) parts.push(`+${e.TreasuryBonus} treasury`)
    if ('NavalBonus' in e && e.NavalBonus != null) parts.push(`+${e.NavalBonus}% naval`)
    if ('ResearchBonus' in e && e.ResearchBonus != null) parts.push(`+${e.ResearchBonus}% research`)
    if ('UnlockUnit' in e && e.UnlockUnit != null) parts.push(`Unlock: ${e.UnlockUnit}`)
    if ('DiplomaticInfluence' in e && e.DiplomaticInfluence != null) {
      const [target, val] = e.DiplomaticInfluence
      parts.push(`${val > 0 ? '+' : ''}${val} influence (${target})`)
    }
  }
  return parts.join(', ')
}

// ── France mock data (matches Rust) ──

function franceFocuses(): Focus[] {
  return [
    { id: 1, name: 'Grande Armée Reform', description: 'Reorganize the French army into a modern instrument of war.', power: 'FRA', cost_days: 70, prerequisites: [], effects: [{ AttackBonus: 10 }], x: 0, y: 0, icon: '⚔️', category: 'military' },
    { id: 2, name: 'Corps System', description: 'Adopt the corps d\'armée system for independent combined-arms formations.', power: 'FRA', cost_days: 70, prerequisites: [1], effects: [{ SupplyRangeBonus: 25 }], x: -1, y: 1, icon: '⚔️', category: 'military' },
    { id: 3, name: 'Imperial Guard', description: 'Expand the elite Imperial Guard into a full corps of veterans.', power: 'FRA', cost_days: 140, prerequisites: [2], effects: [{ UnlockUnit: 'Imperial Guard' }], x: -1, y: 2, icon: '👑', category: 'military' },
    { id: 4, name: 'Light Infantry Doctrine', description: 'Train voltigeur and chasseur companies in skirmish warfare.', power: 'FRA', cost_days: 70, prerequisites: [1], effects: [{ DefenseBonus: 10 }], x: 1, y: 1, icon: '⚔️', category: 'military' },
    { id: 5, name: 'Continental System', description: 'Impose an economic blockade on Britain across Europe.', power: 'FRA', cost_days: 70, prerequisites: [], effects: [{ TreasuryBonus: 200 }], x: 3, y: 0, icon: '💰', category: 'economic' },
    { id: 6, name: 'Economic Dominance', description: 'Establish French commercial supremacy over continental markets.', power: 'FRA', cost_days: 140, prerequisites: [5], effects: [{ TreasuryBonus: 400 }], x: 3, y: 1, icon: '💰', category: 'economic' },
    { id: 7, name: 'European Hegemony', description: 'France dominates the continent — all nations bend the knee.', power: 'FRA', cost_days: 210, prerequisites: [6], effects: [{ DiplomaticInfluence: ['ALL', 50] }, { TreasuryBonus: 500 }], x: 3, y: 2, icon: '👑', category: 'economic' },
    { id: 8, name: 'Blockade Britain', description: 'Enforce the Continental System and strangle British trade.', power: 'FRA', cost_days: 105, prerequisites: [5], effects: [{ NavalBonus: 10 }, { DiplomaticInfluence: ['GBR', -30] }], x: 5, y: 1, icon: '⚓', category: 'economic' },
    { id: 9, name: 'Napoleonic Code', description: 'Codify civil law across the Empire, modernizing governance.', power: 'FRA', cost_days: 70, prerequisites: [], effects: [{ ResearchBonus: 15 }], x: 7, y: 0, icon: '🏛️', category: 'political' },
    { id: 10, name: 'Administrative Reform', description: 'Rationalize the prefectural system and tax collection.', power: 'FRA', cost_days: 70, prerequisites: [9], effects: [{ TreasuryBonus: 150 }], x: 7, y: 1, icon: '🏛️', category: 'political' },
    { id: 11, name: 'Centralized State', description: 'Complete centralization of the French administrative apparatus.', power: 'FRA', cost_days: 140, prerequisites: [10], effects: [{ ResearchBonus: 20 }, { TreasuryBonus: 250 }], x: 7, y: 2, icon: '🏛️', category: 'political' },
    { id: 12, name: 'Conscription Levée', description: 'Institute mass conscription under the Jourdan Law.', power: 'FRA', cost_days: 35, prerequisites: [], effects: [{ ManpowerBonus: 50000 }], x: 10, y: 0, icon: '⚔️', category: 'military' },
    { id: 13, name: 'Mass Mobilization', description: 'Call up the reserves and expand the training depots.', power: 'FRA', cost_days: 70, prerequisites: [12], effects: [{ ManpowerBonus: 100000 }], x: 10, y: 1, icon: '⚔️', category: 'military' },
    { id: 14, name: 'Grand Armée 600K', description: 'The Grande Armée reaches its full strength of 600,000 men.', power: 'FRA', cost_days: 140, prerequisites: [13], effects: [{ ManpowerBonus: 200000 }, { AttackBonus: 5 }], x: 10, y: 2, icon: '👑', category: 'military' },
  ]
}

function britainFocuses(): Focus[] {
  return [
    { id: 101, name: 'Naval Supremacy', description: 'The Royal Navy must command the seas absolutely.', power: 'GBR', cost_days: 70, prerequisites: [], effects: [{ NavalBonus: 20 }], x: 0, y: 0, icon: '⚓', category: 'naval' },
    { id: 102, name: 'Ship of the Line Program', description: 'Expand the fleet with 74-gun ships of the line.', power: 'GBR', cost_days: 140, prerequisites: [101], effects: [{ NavalBonus: 30 }, { UnlockUnit: 'First Rate Ship' }], x: -1, y: 1, icon: '⚓', category: 'naval' },
    { id: 103, name: 'Rule Britannia', description: 'Britannia rules the waves — total naval dominance achieved.', power: 'GBR', cost_days: 210, prerequisites: [102], effects: [{ NavalBonus: 50 }, { DiplomaticInfluence: ['ALL', 25] }], x: -1, y: 2, icon: '👑', category: 'naval' },
    { id: 104, name: 'Blockade France', description: 'Enforce a naval blockade of French and allied ports.', power: 'GBR', cost_days: 105, prerequisites: [101], effects: [{ NavalBonus: 15 }, { DiplomaticInfluence: ['FRA', -30] }], x: 1, y: 1, icon: '⚓', category: 'naval' },
    { id: 105, name: 'Coalition Building', description: 'Forge alliances against France on the continent.', power: 'GBR', cost_days: 70, prerequisites: [], effects: [{ DiplomaticInfluence: ['ALL', 15] }], x: 3, y: 0, icon: '🏛️', category: 'political' },
    { id: 106, name: 'Subsidize Allies', description: 'Use British gold to fund continental armies against Napoleon.', power: 'GBR', cost_days: 105, prerequisites: [105], effects: [{ DiplomaticInfluence: ['AUS', 25] }, { DiplomaticInfluence: ['PRU', 25] }, { DiplomaticInfluence: ['RUS', 25] }], x: 3, y: 1, icon: '💰', category: 'political' },
    { id: 107, name: 'Grand Coalition', description: 'The Third Coalition is formed — all Europe stands against France.', power: 'GBR', cost_days: 140, prerequisites: [106], effects: [{ DiplomaticInfluence: ['ALL', 40] }, { AttackBonus: 5 }], x: 3, y: 2, icon: '👑', category: 'political' },
    { id: 108, name: 'Industrial Revolution', description: 'Harness the power of industry to fuel the war effort.', power: 'GBR', cost_days: 70, prerequisites: [], effects: [{ TreasuryBonus: 300 }], x: 6, y: 0, icon: '💰', category: 'economic' },
    { id: 109, name: 'Steam Power', description: 'Apply Watt\'s steam engine to manufacturing and transport.', power: 'GBR', cost_days: 140, prerequisites: [108], effects: [{ ResearchBonus: 25 }, { TreasuryBonus: 200 }], x: 6, y: 1, icon: '💰', category: 'economic' },
    { id: 110, name: 'Factory System', description: 'Industrialize arms production with the factory system.', power: 'GBR', cost_days: 140, prerequisites: [109], effects: [{ TreasuryBonus: 400 }, { ManpowerBonus: 30000 }], x: 6, y: 2, icon: '💰', category: 'economic' },
    { id: 111, name: "Wellington's Army", description: 'Reform the British Army under Sir Arthur Wellesley.', power: 'GBR', cost_days: 105, prerequisites: [], effects: [{ DefenseBonus: 15 }, { AttackBonus: 10 }], x: 9, y: 0, icon: '⚔️', category: 'military' },
    { id: 112, name: 'Peninsula Campaign', description: 'Launch a major campaign in Iberia to bleed France dry.', power: 'GBR', cost_days: 140, prerequisites: [111], effects: [{ AttackBonus: 15 }, { DiplomaticInfluence: ['SPA', 30] }], x: 9, y: 1, icon: '⚔️', category: 'military' },
  ]
}

function getDefaultTree(powerId: string): FocusTreeData {
  const focuses = powerId === 'FRA' ? franceFocuses() : britainFocuses()
  const focusMap: Record<string, Focus> = {}
  for (const f of focuses) {
    focusMap[String(f.id)] = f
  }
  return { power: powerId, focuses: focusMap, completed: [], in_progress: null }
}

// ── Node position helpers ──

interface NodePos {
  cx: number
  cy: number
  focus: Focus
}

function layoutNodes(focuses: Focus[]): { nodes: NodePos[]; svgW: number; svgH: number } {
  // find min/max grid coords
  let minX = Infinity, maxX = -Infinity, maxY = -Infinity
  for (const f of focuses) {
    if (f.x < minX) minX = f.x
    if (f.x > maxX) maxX = f.x
    if (f.y > maxY) maxY = f.y
  }
  const nodes: NodePos[] = focuses.map(f => ({
    cx: PAD_X + (f.x - minX) * GAP_X,
    cy: PAD_Y + f.y * GAP_Y,
    focus: f,
  }))
  const svgW = PAD_X * 2 + (maxX - minX) * GAP_X + NODE_W
  const svgH = PAD_Y * 2 + maxY * GAP_Y + NODE_H
  return { nodes, svgW, svgH }
}

// ── Component ──

interface FocusTreeProps {
  open: boolean
  onClose: () => void
}

type NodeState = 'locked' | 'available' | 'in_progress' | 'completed'

export default function FocusTree({ open, onClose }: FocusTreeProps) {
  const [powerId, setPowerId] = useState('FRA')
  const [trees, setTrees] = useState<Record<string, FocusTreeData>>(() => ({
    FRA: getDefaultTree('FRA'),
    GBR: getDefaultTree('GBR'),
  }))
  const [hoveredId, setHoveredId] = useState<number | null>(null)

  const tree = trees[powerId]
  const focuses = useMemo(() => Object.values(tree.focuses), [tree.focuses])
  const { nodes, svgW, svgH } = useMemo(() => layoutNodes(focuses), [focuses])

  const completedSet = useMemo(() => new Set(tree.completed), [tree.completed])

  const getNodeState = useCallback((f: Focus): NodeState => {
    if (completedSet.has(f.id)) return 'completed'
    if (tree.in_progress && tree.in_progress[0] === f.id) return 'in_progress'
    const prereqsMet = f.prerequisites.every(p => completedSet.has(p))
    if (prereqsMet && !tree.in_progress) return 'available'
    return 'locked'
  }, [completedSet, tree.in_progress])

  const handleClick = useCallback((f: Focus) => {
    const state = getNodeState(f)
    if (state !== 'available') return
    setTrees(prev => {
      const t = { ...prev[powerId] }
      t.in_progress = [f.id, f.cost_days]
      return { ...prev, [powerId]: t }
    })
  }, [getNodeState, powerId])

  // Advance day (simulate — in real game this is called by clock tick)
  const handleAdvanceDay = useCallback(() => {
    setTrees(prev => {
      const t = { ...prev[powerId] }
      if (!t.in_progress) return prev
      const [fid, remaining] = t.in_progress
      if (remaining <= 1) {
        t.completed = [...t.completed, fid]
        t.in_progress = null
      } else {
        t.in_progress = [fid, remaining - 1]
      }
      return { ...prev, [powerId]: t }
    })
  }, [powerId])

  // Quick complete for testing
  const handleComplete = useCallback(() => {
    setTrees(prev => {
      const t = { ...prev[powerId] }
      if (!t.in_progress) return prev
      const [fid] = t.in_progress
      t.completed = [...t.completed, fid]
      t.in_progress = null
      return { ...prev, [powerId]: t }
    })
  }, [powerId])

  if (!open) return null

  // Build lookup for line drawing
  const posById = new Map<number, NodePos>()
  for (const n of nodes) posById.set(n.focus.id, n)

  // Category colors for subtle tinting
  const catColor: Record<string, string> = {
    military: '#c44', economic: '#c90', political: '#58c', naval: '#2aa',
  }

  return (
    <div
      onClick={(e) => { if (e.target === e.currentTarget) onClose() }}
      style={{
        position: 'fixed', inset: 0, zIndex: 400,
        background: 'rgba(4,4,12,0.95)',
        display: 'flex', flexDirection: 'column',
        fontFamily: 'Cinzel, serif',
      }}
    >
      {/* ── Header bar ── */}
      <div style={{
        height: 52, flexShrink: 0,
        background: 'linear-gradient(180deg,#12111e,#0a0914)',
        borderBottom: '1px solid #5a4524',
        display: 'flex', alignItems: 'center', padding: '0 16px', gap: 12,
      }}>
        <span style={{ color: '#d4af37', fontSize: 16, fontWeight: 700, letterSpacing: 2 }}>
          NATIONAL FOCUS
        </span>

        {/* Power tabs */}
        <div style={{ display: 'flex', gap: 4, marginLeft: 24 }}>
          {POWER_TABS.map(p => (
            <button
              key={p.id}
              onClick={() => setPowerId(p.id)}
              style={{
                background: powerId === p.id ? 'rgba(212,175,55,0.2)' : 'rgba(30,25,50,0.6)',
                border: `1px solid ${powerId === p.id ? '#d4af37' : '#3a3344'}`,
                color: powerId === p.id ? '#d4af37' : '#887766',
                cursor: 'pointer', padding: '5px 14px', fontSize: 12,
                fontWeight: 700, letterSpacing: 1, borderRadius: 3,
                fontFamily: 'Cinzel, serif',
              }}
            >
              {p.flag} {p.label}
            </button>
          ))}
        </div>

        <div style={{ flex: 1 }} />

        {/* Progress controls */}
        {tree.in_progress && (
          <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
            <button onClick={handleAdvanceDay} style={smallBtnStyle}>+1 Day</button>
            <button onClick={handleComplete} style={smallBtnStyle}>Complete</button>
          </div>
        )}

        {/* Close */}
        <button
          onClick={onClose}
          style={{
            background: 'none', border: 'none', color: '#aa8844',
            fontSize: 22, cursor: 'pointer', padding: '2px 8px',
          }}
        >
          ✕
        </button>
      </div>

      {/* ── In-progress bar ── */}
      {tree.in_progress && (() => {
        const [fid, remaining] = tree.in_progress
        const f = tree.focuses[String(fid)]
        if (!f) return null
        const pct = ((f.cost_days - remaining) / f.cost_days) * 100
        return (
          <div style={{
            height: 36, flexShrink: 0,
            background: '#0e0d18', borderBottom: '1px solid #3a3344',
            display: 'flex', alignItems: 'center', padding: '0 16px', gap: 12,
          }}>
            <span style={{ color: '#d4af37', fontSize: 12, fontWeight: 700 }}>
              {f.icon} {f.name}
            </span>
            <div style={{
              flex: 1, maxWidth: 400, height: 14, background: '#1a1824',
              borderRadius: 3, border: '1px solid #3a3344', overflow: 'hidden',
            }}>
              <div style={{
                width: `${pct}%`, height: '100%',
                background: 'linear-gradient(90deg,#d4af37,#b8942e)',
                transition: 'width 0.3s',
              }} />
            </div>
            <span style={{ color: '#aa8844', fontSize: 11 }}>
              {remaining} days remaining
            </span>
          </div>
        )
      })()}

      {/* ── SVG tree ── */}
      <div style={{ flex: 1, overflow: 'auto', padding: 8 }}>
        <svg width={svgW} height={svgH} style={{ display: 'block', margin: '0 auto' }}>
          {/* Dependency lines */}
          {nodes.map(n => n.focus.prerequisites.map(preId => {
            const parent = posById.get(preId)
            if (!parent) return null
            return (
              <line
                key={`${preId}-${n.focus.id}`}
                x1={parent.cx + NODE_W / 2} y1={parent.cy + NODE_H}
                x2={n.cx + NODE_W / 2} y2={n.cy}
                stroke={completedSet.has(preId) ? '#5a4524' : '#2a2434'}
                strokeWidth={2}
              />
            )
          }))}

          {/* Focus nodes */}
          {nodes.map(n => {
            const f = n.focus
            const state = getNodeState(f)
            const hovered = hoveredId === f.id

            let fill = '#14121f'
            let stroke = '#2a2434'
            let opacity = 0.5
            let textColor = '#555'

            if (state === 'completed') {
              fill = '#0f1f12'
              stroke = '#2a6e2a'
              opacity = 1
              textColor = '#8bc88b'
            } else if (state === 'in_progress') {
              fill = '#1f1a0c'
              stroke = '#d4af37'
              opacity = 1
              textColor = '#d4af37'
            } else if (state === 'available') {
              fill = '#18162a'
              stroke = '#8a7544'
              opacity = 1
              textColor = '#e8dcc8'
            } else {
              // locked
              fill = '#0e0d16'
              stroke = '#2a2434'
              opacity = 0.55
              textColor = '#554e44'
            }

            if (hovered && state === 'available') {
              stroke = '#d4af37'
              fill = '#1e1a2f'
            }

            return (
              <g
                key={f.id}
                style={{ cursor: state === 'available' ? 'pointer' : 'default' }}
                onClick={() => handleClick(f)}
                onMouseEnter={() => setHoveredId(f.id)}
                onMouseLeave={() => setHoveredId(null)}
              >
                {/* Glow for in-progress */}
                {state === 'in_progress' && (
                  <rect
                    x={n.cx - 3} y={n.cy - 3}
                    width={NODE_W + 6} height={NODE_H + 6}
                    rx={6} fill="none" stroke="#d4af37" strokeWidth={2}
                    opacity={0.6}
                  >
                    <animate attributeName="opacity" values="0.3;0.8;0.3" dur="2s" repeatCount="indefinite" />
                  </rect>
                )}

                {/* Node box */}
                <rect
                  x={n.cx} y={n.cy}
                  width={NODE_W} height={NODE_H}
                  rx={4} fill={fill} stroke={stroke}
                  strokeWidth={hovered && state === 'available' ? 2 : 1.5}
                  opacity={opacity}
                />

                {/* Category accent line */}
                <rect
                  x={n.cx} y={n.cy}
                  width={4} height={NODE_H}
                  rx={2} fill={catColor[f.category] || '#555'}
                  opacity={opacity * 0.7}
                />

                {/* Icon */}
                <text
                  x={n.cx + 18} y={n.cy + 22}
                  fontSize={16} textAnchor="middle"
                  opacity={opacity}
                >
                  {f.icon}
                </text>

                {/* Name */}
                <text
                  x={n.cx + 32} y={n.cy + 20}
                  fontSize={10} fontWeight={700}
                  fill={textColor} fontFamily="Cinzel, serif"
                  opacity={opacity}
                >
                  {f.name.length > 20 ? f.name.slice(0, 19) + '…' : f.name}
                </text>

                {/* Cost */}
                <text
                  x={n.cx + 32} y={n.cy + 34}
                  fontSize={9} fill={state === 'in_progress' ? '#d4af37' : '#776655'}
                  fontFamily="Cinzel, serif"
                  opacity={opacity}
                >
                  {f.cost_days} days
                </text>

                {/* Effect summary */}
                <text
                  x={n.cx + 32} y={n.cy + 48}
                  fontSize={8} fill="#665544"
                  fontFamily="sans-serif"
                  opacity={opacity}
                >
                  {effectSummary(f.effects).slice(0, 30)}
                </text>

                {/* Completed checkmark */}
                {state === 'completed' && (
                  <text
                    x={n.cx + NODE_W - 16} y={n.cy + 18}
                    fontSize={16} fill="#2a6e2a"
                    textAnchor="middle"
                  >
                    ✓
                  </text>
                )}

                {/* Tooltip on hover */}
                {hovered && (
                  <foreignObject
                    x={n.cx} y={n.cy + NODE_H + 6}
                    width={240} height={100}
                    style={{ pointerEvents: 'none' }}
                  >
                    <div style={{
                      background: 'rgba(10,8,20,0.95)',
                      border: '1px solid #5a4524',
                      borderRadius: 4, padding: '8px 10px',
                      color: '#e8dcc8', fontSize: 10,
                      fontFamily: 'sans-serif', lineHeight: 1.5,
                    }}>
                      <div style={{ fontWeight: 700, marginBottom: 4, fontFamily: 'Cinzel, serif' }}>
                        {f.icon} {f.name}
                      </div>
                      <div style={{ color: '#998877', marginBottom: 4 }}>{f.description}</div>
                      <div style={{ color: '#d4af37' }}>{effectSummary(f.effects)}</div>
                    </div>
                  </foreignObject>
                )}
              </g>
            )
          })}
        </svg>
      </div>
    </div>
  )
}

const smallBtnStyle: React.CSSProperties = {
  background: 'rgba(212,175,55,0.15)',
  border: '1px solid #5a4524',
  color: '#d4af37',
  cursor: 'pointer',
  padding: '3px 10px',
  fontSize: 11,
  fontWeight: 700,
  borderRadius: 3,
  fontFamily: 'Cinzel, serif',
}
