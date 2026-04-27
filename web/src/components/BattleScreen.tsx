import React, { useState, useEffect } from 'react'
import type { BattleEvent } from '../types'

const POWER_FLAGS: Record<string, string> = {
  FRA: '🇫🇷', GBR: '🇬🇧', AUS: '🦅', PRU: '⚫', RUS: '🐻', OTT: '☪️', SPA: '🇪🇸',
}
const POWER_NAMES: Record<string, string> = {
  FRA: 'France', GBR: 'Britain', AUS: 'Austria', PRU: 'Prussia',
  RUS: 'Russia', OTT: 'Ottoman', SPA: 'Spain',
}
const POWER_COLORS: Record<string, string> = {
  FRA: '#1565C0', GBR: '#B71C1C', AUS: '#F9A825', PRU: '#455A64',
  RUS: '#2E7D32', OTT: '#6A1B9A', SPA: '#E65100',
}

const TACTIC_COLORS: Record<string, string> = {
  Column: '#e67e22', Line: '#3498db', Square: '#27ae60', SkirmishScreen: '#9b59b6',
}

interface Props {
  battle: BattleEvent
  onClose: () => void
}

export default function BattleScreen({ battle, onClose }: Props) {
  const [phase, setPhase] = useState<'fighting' | 'resolved'>('fighting')
  const [drainPct, setDrainPct] = useState(0)

  useEffect(() => {
    // Start drain animation after mount
    const t1 = requestAnimationFrame(() => setDrainPct(1))
    const t2 = setTimeout(() => setPhase('resolved'), 2200)
    return () => { cancelAnimationFrame(t1); clearTimeout(t2) }
  }, [])

  const attColor = POWER_COLORS[battle.attacker.power] || '#888'
  const defColor = POWER_COLORS[battle.defender.power] || '#888'

  const attStrengthAfter = battle.attacker.strength - battle.attackerCasualties
  const defStrengthAfter = battle.defender.strength - battle.defenderCasualties
  const attPct = drainPct === 0 ? 1 : attStrengthAfter / battle.attacker.strength
  const defPct = drainPct === 0 ? 1 : defStrengthAfter / battle.defender.strength

  const outcomeLabel =
    battle.outcome === 'attacker_advances' ? `${POWER_NAMES[battle.attacker.power] || 'ATTACKER'} ADVANCES` :
    battle.outcome === 'stalemate' ? 'STALEMATE' :
    'REPELLED'
  const outcomeBg =
    battle.outcome === 'attacker_advances' ? 'rgba(39,174,96,0.85)' :
    battle.outcome === 'stalemate' ? 'rgba(241,196,15,0.85)' :
    'rgba(192,57,43,0.85)'

  const sidePanel = (side: 'attacker' | 'defender') => {
    const data = battle[side]
    const color = side === 'attacker' ? attColor : defColor
    const pct = side === 'attacker' ? attPct : defPct
    const casualties = side === 'attacker' ? battle.attackerCasualties : battle.defenderCasualties
    const isLeft = side === 'attacker'

    return (
      <div style={{
        flex: 1,
        display: 'flex',
        flexDirection: 'column',
        alignItems: isLeft ? 'flex-end' : 'flex-start',
        padding: '20px 24px',
        gap: 12,
      }}>
        {/* Power identity */}
        <div style={{ display: 'flex', alignItems: 'center', gap: 10, flexDirection: isLeft ? 'row' : 'row-reverse' }}>
          <span style={{ fontSize: 32 }}>{POWER_FLAGS[data.power] || '?'}</span>
          <div style={{ textAlign: isLeft ? 'right' : 'left' }}>
            <div style={{ color, fontSize: 18, fontWeight: 700, letterSpacing: 2 }}>
              {POWER_NAMES[data.power] || data.power}
            </div>
            <div style={{ color: '#bca47d', fontSize: 10, letterSpacing: 1.5, textTransform: 'uppercase' }}>
              {side}
            </div>
          </div>
        </div>

        {/* Commander */}
        <div style={{ color: '#e8dcc8', fontSize: 13, fontWeight: 600 }}>
          {data.commander || 'No Commander'}
        </div>

        {/* Tactic badge */}
        <div style={{
          display: 'inline-block',
          background: TACTIC_COLORS[data.tactic] || '#666',
          color: '#fff',
          padding: '3px 10px',
          borderRadius: 3,
          fontSize: 11,
          fontWeight: 700,
          letterSpacing: 1,
          alignSelf: isLeft ? 'flex-end' : 'flex-start',
        }}>
          {data.tactic}
        </div>

        {/* Strength bar */}
        <div style={{ width: '100%' }}>
          <div style={{ color: '#bca47d', fontSize: 10, letterSpacing: 1.5, marginBottom: 4 }}>STRENGTH</div>
          <div style={{
            width: '100%',
            height: 22,
            background: 'rgba(0,0,0,0.4)',
            borderRadius: 3,
            overflow: 'hidden',
            border: `1px solid ${color}44`,
          }}>
            <div style={{
              width: `${pct * 100}%`,
              height: '100%',
              background: `linear-gradient(90deg, ${color}, ${color}aa)`,
              transition: 'width 2s ease-out',
              borderRadius: 2,
            }} />
          </div>
          <div style={{ color: '#e8dcc8', fontSize: 12, marginTop: 3 }}>
            {drainPct === 0 ? data.strength.toLocaleString() : `${Math.round(data.strength * pct).toLocaleString()} / ${data.strength.toLocaleString()}`}
          </div>
        </div>

        {/* Casualties */}
        <div>
          <div style={{ color: '#bca47d', fontSize: 10, letterSpacing: 1.5 }}>CASUALTIES</div>
          <div style={{ color: '#ff6b6b', fontSize: 20, fontWeight: 700 }}>
            {phase === 'fighting' ? '...' : casualties.toLocaleString()}
          </div>
        </div>
      </div>
    )
  }

  return (
    <div style={{
      position: 'fixed',
      inset: 0,
      background: 'rgba(0,0,0,0.82)',
      zIndex: 1000,
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
      fontFamily: 'Cinzel, serif',
    }}>
      <div style={{
        width: 680,
        background: 'linear-gradient(180deg, #12101e 0%, #0a0810 100%)',
        border: '1px solid #3a2f1a',
        borderRadius: 6,
        boxShadow: '0 20px 60px rgba(0,0,0,0.8)',
        overflow: 'hidden',
      }}>
        {/* Header */}
        <div style={{
          textAlign: 'center',
          padding: '14px 0 10px',
          borderBottom: '1px solid #2a1f08',
          background: 'linear-gradient(180deg, #1a1530 0%, #12101e 100%)',
        }}>
          <div style={{ fontSize: 28 }}>{'⚔️'}</div>
          <div style={{ color: '#d4af37', fontSize: 14, fontWeight: 700, letterSpacing: 3, marginTop: 4 }}>
            BATTLE OF {battle.territory.toUpperCase()}
          </div>
        </div>

        {/* Two sides */}
        <div style={{ display: 'flex' }}>
          {sidePanel('attacker')}
          {/* Center divider */}
          <div style={{
            width: 2,
            background: 'linear-gradient(180deg, transparent, #d4af3744, transparent)',
            alignSelf: 'stretch',
          }} />
          {sidePanel('defender')}
        </div>

        {/* Outcome banner */}
        {phase === 'resolved' && (
          <div style={{
            textAlign: 'center',
            padding: '14px 0',
            background: outcomeBg,
            color: '#fff',
            fontSize: 16,
            fontWeight: 700,
            letterSpacing: 3,
            animation: 'slideDown 0.3s ease-out',
          }}>
            {outcomeLabel}
          </div>
        )}

        {/* Close button */}
        {phase === 'resolved' && (
          <div style={{ textAlign: 'center', padding: '14px 0', borderTop: '1px solid #2a1f08' }}>
            <button
              onClick={onClose}
              style={{
                background: 'linear-gradient(180deg,#8b3a0a,#5a2005)',
                color: '#f0e0a0',
                border: '1px solid #c8a000',
                borderRadius: 3,
                padding: '8px 28px',
                cursor: 'pointer',
                fontFamily: 'Cinzel, serif',
                fontSize: 12,
                fontWeight: 700,
                letterSpacing: 2,
              }}
            >
              CLOSE
            </button>
          </div>
        )}
      </div>

      <style>{`
        @keyframes slideDown {
          from { transform: translateY(-10px); opacity: 0; }
          to { transform: translateY(0); opacity: 1; }
        }
      `}</style>
    </div>
  )
}
