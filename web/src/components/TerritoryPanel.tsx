import React from 'react'
import type { TerritoryInfo } from '../types'

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

const TERRAIN_BADGES: Record<string, { emoji: string; color: string }> = {
  Plains: { emoji: '🌾', color: '#8bc34a' },
  Mountains: { emoji: '⛰️', color: '#90a4ae' },
  Forest: { emoji: '🌲', color: '#4caf50' },
  Coast: { emoji: '🌊', color: '#29b6f6' },
  River: { emoji: '🏞️', color: '#42a5f5' },
}

interface Props {
  territory: TerritoryInfo
  onClose: () => void
}

export default function TerritoryPanel({ territory, onClose }: Props) {
  const ownerColor = POWER_COLORS[territory.owner] || '#8D6E63'
  const terrBadge = TERRAIN_BADGES[territory.terrain] || { emoji: '?', color: '#888' }

  return (
    <div style={{
      position: 'fixed',
      bottom: 0,
      left: 0,
      right: 0,
      zIndex: 200,
      animation: 'slideUp 0.25s ease-out',
      fontFamily: 'Cinzel, serif',
    }}>
      <div style={{
        maxWidth: 900,
        margin: '0 auto',
        background: 'linear-gradient(180deg, #16132a 0%, #0c0a18 100%)',
        border: '1px solid #3a2f1a',
        borderBottom: 'none',
        borderRadius: '8px 8px 0 0',
        boxShadow: '0 -10px 40px rgba(0,0,0,0.6)',
        padding: '16px 24px 20px',
      }}>
        {/* Header row */}
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 14 }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
            <div style={{ color: '#d4af37', fontSize: 20, fontWeight: 700, letterSpacing: 2 }}>
              {territory.name}
            </div>
            {/* Terrain badge */}
            <span style={{
              background: `${terrBadge.color}22`,
              border: `1px solid ${terrBadge.color}66`,
              color: terrBadge.color,
              padding: '2px 8px',
              borderRadius: 3,
              fontSize: 11,
              fontWeight: 600,
            }}>
              {terrBadge.emoji} {territory.terrain}
            </span>
          </div>
          <button onClick={onClose} style={{
            background: 'none',
            border: '1px solid #3a2f1a',
            color: '#7a6030',
            cursor: 'pointer',
            fontSize: 14,
            padding: '2px 10px',
            borderRadius: 3,
            fontFamily: 'Cinzel, serif',
          }}>
            {'✕'}
          </button>
        </div>

        {/* Content grid */}
        <div style={{ display: 'flex', gap: 24 }}>
          {/* Owner + Corps */}
          <div style={{ flex: 1 }}>
            <div style={{ color: '#bca47d', fontSize: 10, letterSpacing: 1.5, marginBottom: 4 }}>OWNER</div>
            <div style={{ display: 'flex', alignItems: 'center', gap: 6, marginBottom: 12 }}>
              <span style={{ fontSize: 18 }}>{POWER_FLAGS[territory.owner] || '?'}</span>
              <span style={{ color: ownerColor, fontSize: 14, fontWeight: 700 }}>
                {POWER_NAMES[territory.owner] || 'Neutral'}
              </span>
            </div>

            <div style={{ color: '#bca47d', fontSize: 10, letterSpacing: 1.5, marginBottom: 4 }}>STATIONED CORPS</div>
            {territory.corps.length > 0 ? territory.corps.map((c, i) => (
              <div key={i} style={{
                background: 'rgba(0,0,0,0.3)',
                border: '1px solid #2a1f08',
                padding: '6px 10px',
                marginBottom: 4,
                borderRadius: 3,
              }}>
                <div style={{ color: '#e8dcc8', fontSize: 13, fontWeight: 600 }}>{c.name}</div>
                <div style={{ display: 'flex', gap: 12, marginTop: 2 }}>
                  <span style={{ color: '#bca47d', fontSize: 11 }}>Strength: {c.strength.toLocaleString()}</span>
                  {c.marshal && (
                    <span style={{ color: '#d4af37', fontSize: 11 }}>Marshal: {c.marshal}</span>
                  )}
                </div>
              </div>
            )) : (
              <div style={{ color: '#5a4820', fontSize: 12, fontStyle: 'italic' }}>No corps stationed</div>
            )}
          </div>

          {/* Orders */}
          <div style={{ flex: 1 }}>
            <div style={{ color: '#bca47d', fontSize: 10, letterSpacing: 1.5, marginBottom: 6 }}>ISSUE ORDER</div>
            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 6 }}>
              {[
                { label: 'Move', icon: '▶', color: '#3498db' },
                { label: 'Attack', icon: '⚔️', color: '#e74c3c' },
                { label: 'Hold', icon: '🛡️', color: '#f39c12' },
                { label: 'Fortify', icon: '🏰', color: '#27ae60' },
              ].map(btn => (
                <button key={btn.label} style={{
                  background: `${btn.color}18`,
                  border: `1px solid ${btn.color}55`,
                  color: btn.color,
                  padding: '8px 0',
                  borderRadius: 3,
                  cursor: 'pointer',
                  fontFamily: 'Cinzel, serif',
                  fontSize: 11,
                  fontWeight: 700,
                  letterSpacing: 1,
                }}>
                  {btn.icon} {btn.label}
                </button>
              ))}
            </div>

            <div style={{ marginTop: 14 }}>
              <div style={{ color: '#bca47d', fontSize: 10, letterSpacing: 1.5, marginBottom: 4 }}>ECONOMY</div>
              <div style={{ display: 'flex', gap: 16 }}>
                <span style={{ color: '#f0e0a0', fontSize: 13 }}>+{territory.goldPerDay} gold/day</span>
                <span style={{ color: '#8bc34a', fontSize: 13 }}>+{territory.manpowerPerMonth} manpower/mo</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <style>{`
        @keyframes slideUp {
          from { transform: translateY(100%); }
          to { transform: translateY(0); }
        }
      `}</style>
    </div>
  )
}
