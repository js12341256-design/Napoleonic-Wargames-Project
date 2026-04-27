import React from 'react'
import type { PowerPoliticsData, Faction, StabilityEffects } from '../types'

interface Props {
  politics: PowerPoliticsData
  open: boolean
  onClose: () => void
}

const FACTION_COLORS: Record<Faction, string> = {
  Military: '#cc4444',
  Nobility: '#d4af37',
  Clergy: '#9944cc',
  Merchants: '#44aa44',
  Peasantry: '#8B6914',
  Revolutionaries: '#e44',
}

const ALL_FACTIONS: Faction[] = ['Military', 'Nobility', 'Clergy', 'Merchants', 'Peasantry', 'Revolutionaries']

const GOVERNMENT_LABELS: Record<string, string> = {
  Empire: 'Empire',
  AbsoluteMonarchy: 'Absolute Monarchy',
  ConstitutionalMonarchy: 'Constitutional Monarchy',
  Republic: 'Republic',
}

function stabilityEffects(stability: number): StabilityEffects {
  switch (stability) {
    case 3: return { income_modifier: 15, manpower_modifier: 10, revolt_chance: 0, civil_war_risk: false }
    case 2: return { income_modifier: 10, manpower_modifier: 5, revolt_chance: 0, civil_war_risk: false }
    case 1: return { income_modifier: 5, manpower_modifier: 0, revolt_chance: 0, civil_war_risk: false }
    case 0: return { income_modifier: 0, manpower_modifier: 0, revolt_chance: 0, civil_war_risk: false }
    case -1: return { income_modifier: -10, manpower_modifier: 0, revolt_chance: 5, civil_war_risk: false }
    case -2: return { income_modifier: -20, manpower_modifier: 0, revolt_chance: 15, civil_war_risk: false }
    default: return { income_modifier: -30, manpower_modifier: 0, revolt_chance: 30, civil_war_risk: true }
  }
}

function stabilityColor(val: number): string {
  if (val > 0) return ['#66bb6a', '#43a047', '#2e7d32'][val - 1] || '#2e7d32'
  if (val === 0) return '#888'
  return ['#ef5350', '#d32f2f', '#b71c1c'][(-val) - 1] || '#b71c1c'
}

function stabilityLabel(val: number): string {
  if (val > 0) return '+' + val
  return String(val)
}

export default function PoliticsPanel({ politics, open, onClose }: Props) {
  const effects = stabilityEffects(politics.stability)

  return (
    <div style={{
      position: 'fixed', top: 0, right: 0, width: 370, height: '100vh',
      background: 'linear-gradient(180deg, #0e0c1c 0%, #070610 100%)',
      borderLeft: '1px solid #2a1f08',
      zIndex: 210,
      display: 'flex', flexDirection: 'column',
      transform: open ? 'translateX(0)' : 'translateX(100%)',
      transition: 'transform 0.3s ease',
      fontFamily: 'Cinzel, serif',
      color: '#e8dcc8',
    }}>
      {/* Header */}
      <div style={{
        padding: '14px 16px', display: 'flex', alignItems: 'center', justifyContent: 'space-between',
        borderBottom: '1px solid #5a4524',
      }}>
        <span style={{ color: '#d4af37', fontWeight: 700, fontSize: 14, letterSpacing: 2, textTransform: 'uppercase' }}>
          Politics
        </span>
        <button onClick={onClose} style={{
          background: 'none', border: '1px solid rgba(212,175,55,0.3)', color: '#d4af37',
          cursor: 'pointer', padding: '2px 8px', fontSize: 14, borderRadius: 2,
        }}>X</button>
      </div>

      {/* Content */}
      <div style={{ flex: 1, overflowY: 'auto', padding: '12px 16px' }}>

        {/* Government badge */}
        <div style={{
          background: 'rgba(212,175,55,0.08)', border: '1px solid #3a2f1a', borderRadius: 3,
          padding: '8px 12px', marginBottom: 14, textAlign: 'center',
        }}>
          <div style={{ fontSize: 10, color: '#7a6030', marginBottom: 4, letterSpacing: 1 }}>GOVERNMENT</div>
          <div style={{ fontSize: 14, color: '#d4af37', fontWeight: 700 }}>
            {GOVERNMENT_LABELS[politics.government] || politics.government}
          </div>
        </div>

        {/* Legitimacy bar */}
        <div style={{ marginBottom: 14 }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: 4 }}>
            <span style={{ fontSize: 10, color: '#7a6030', letterSpacing: 1 }}>LEGITIMACY</span>
            <span style={{ fontSize: 12, color: '#d4af37', fontWeight: 700 }}>{politics.legitimacy}/100</span>
          </div>
          <div style={{ height: 10, background: '#1a1510', borderRadius: 2, border: '1px solid #3a2f1a', overflow: 'hidden' }}>
            <div
              style={{
                height: '100%', borderRadius: 2,
                width: `${politics.legitimacy}%`,
                background: `linear-gradient(90deg, #b8860b, #d4af37)`,
                transition: 'width 0.3s',
              }}
              title="Low legitimacy increases revolt risk and reduces national cohesion"
            />
          </div>
          <div style={{ fontSize: 9, color: '#5a4820', marginTop: 3, fontStyle: 'italic' }}>
            Low legitimacy increases revolt risk and reduces national cohesion
          </div>
        </div>

        {/* Stability indicator */}
        <div style={{ marginBottom: 14 }}>
          <div style={{ fontSize: 10, color: '#7a6030', letterSpacing: 1, marginBottom: 6 }}>STABILITY</div>
          <div style={{ display: 'flex', gap: 4, alignItems: 'center', justifyContent: 'center' }}>
            {[-3, -2, -1, 0, 1, 2, 3].map(val => {
              const active = val === politics.stability
              const inRange = (val <= politics.stability && val >= 0) || (val >= politics.stability && val <= 0)
              return (
                <div key={val} style={{
                  width: 36, height: 28,
                  display: 'flex', alignItems: 'center', justifyContent: 'center',
                  borderRadius: 3,
                  fontSize: 11, fontWeight: 700,
                  border: active ? `2px solid ${stabilityColor(val)}` : '1px solid #2a2218',
                  background: inRange || active ? `${stabilityColor(val)}22` : 'rgba(10,8,20,0.6)',
                  color: active ? stabilityColor(val) : inRange ? `${stabilityColor(val)}88` : '#3a3020',
                  transition: 'all 0.2s',
                }}>
                  {stabilityLabel(val)}
                </div>
              )
            })}
          </div>
        </div>

        {/* Ruling faction */}
        <div style={{
          background: 'rgba(212,175,55,0.05)', border: '1px solid #2a2218', borderRadius: 3,
          padding: '8px 12px', marginBottom: 14, display: 'flex', justifyContent: 'space-between', alignItems: 'center',
        }}>
          <span style={{ fontSize: 10, color: '#7a6030', letterSpacing: 1 }}>RULING FACTION</span>
          <span style={{
            fontSize: 12, fontWeight: 700, padding: '2px 10px', borderRadius: 2,
            background: `${FACTION_COLORS[politics.ruling_faction]}22`,
            color: FACTION_COLORS[politics.ruling_faction],
            border: `1px solid ${FACTION_COLORS[politics.ruling_faction]}44`,
          }}>
            {politics.ruling_faction}
          </span>
        </div>

        {/* Faction popularity bars */}
        <div style={{ marginBottom: 14 }}>
          <div style={{ fontSize: 10, color: '#7a6030', letterSpacing: 1, marginBottom: 8 }}>FACTION SUPPORT</div>
          {ALL_FACTIONS.map(faction => {
            const support = politics.faction_support[faction] || 0
            const color = FACTION_COLORS[faction]
            return (
              <div key={faction} style={{ marginBottom: 6 }}>
                <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: 2 }}>
                  <span style={{ fontSize: 10, color: support > 0 ? color : '#3a3020' }}>{faction}</span>
                  <span style={{ fontSize: 10, color: support > 0 ? '#e8dcc8' : '#3a3020', fontWeight: 600 }}>{support}%</span>
                </div>
                <div style={{ height: 6, background: '#1a1510', borderRadius: 2, overflow: 'hidden' }}>
                  <div style={{
                    height: '100%', borderRadius: 2,
                    width: `${support}%`,
                    background: color,
                    opacity: support > 0 ? 0.8 : 0.2,
                    transition: 'width 0.3s',
                  }} />
                </div>
              </div>
            )
          })}
        </div>

        {/* Stability effects box */}
        <div style={{
          background: 'rgba(20,18,35,0.8)', border: '1px solid #2a2218', borderRadius: 3,
          padding: '10px 12px',
        }}>
          <div style={{ fontSize: 10, color: '#7a6030', letterSpacing: 1, marginBottom: 8 }}>STABILITY EFFECTS</div>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
            <EffectRow
              label="Income"
              value={effects.income_modifier}
              suffix="%"
            />
            <EffectRow
              label="Manpower Recovery"
              value={effects.manpower_modifier}
              suffix="%"
            />
            {effects.revolt_chance > 0 && (
              <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                <span style={{ fontSize: 10, color: '#aa6040' }}>Revolt Chance</span>
                <span style={{ fontSize: 11, color: '#ef5350', fontWeight: 700 }}>{effects.revolt_chance}%/month</span>
              </div>
            )}
            {effects.civil_war_risk && (
              <div style={{ fontSize: 10, color: '#ef5350', fontWeight: 700, textAlign: 'center', marginTop: 4, padding: '4px', background: 'rgba(239,83,80,0.1)', borderRadius: 2 }}>
                CIVIL WAR RISK
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  )
}

function EffectRow({ label, value, suffix }: { label: string; value: number; suffix: string }) {
  const color = value > 0 ? '#66bb6a' : value < 0 ? '#ef5350' : '#888'
  const sign = value > 0 ? '+' : ''
  return (
    <div style={{ display: 'flex', justifyContent: 'space-between' }}>
      <span style={{ fontSize: 10, color: '#7a6030' }}>{label}</span>
      <span style={{ fontSize: 11, color, fontWeight: 700 }}>{sign}{value}{suffix}</span>
    </div>
  )
}
