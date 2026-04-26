import React, { useState } from 'react'
import type { Marshal, MarshalTrait } from '../types'

interface MarshalsPanelProps {
  marshals: Marshal[]
  onAssign: (marshalId: number, corpsId: number) => void
  open: boolean
  onClose: () => void
}

const TRAIT_COLORS: Record<MarshalTrait, string> = {
  DefensiveGenius: '#4488cc',
  Aggressive: '#cc4444',
  CavalryMaster: '#44aa44',
  Tactician: '#d4af37',
  Logistics: '#888888',
  Siege: '#8B4513',
  NavalCommander: '#228B8B',
  InspirationalLeader: '#9944cc',
}

function StarRating({ skill }: { skill: number }) {
  const full = Math.round(skill / 2)
  const empty = 5 - full
  return (
    <span style={{ color: '#d4af37', fontSize: 14, letterSpacing: 1 }}>
      {'★'.repeat(full)}
      {'☆'.repeat(empty)}
    </span>
  )
}

function TraitBadge({ trait }: { trait: MarshalTrait }) {
  return (
    <span
      style={{
        display: 'inline-block',
        background: TRAIT_COLORS[trait] + '33',
        color: TRAIT_COLORS[trait],
        border: `1px solid ${TRAIT_COLORS[trait]}66`,
        borderRadius: 3,
        padding: '1px 6px',
        fontSize: 10,
        fontWeight: 600,
        marginRight: 4,
        marginBottom: 2,
        letterSpacing: 0.5,
      }}
    >
      {trait}
    </span>
  )
}

export default function MarshalsPanel({
  marshals,
  onAssign,
  open,
  onClose,
}: MarshalsPanelProps) {
  const [selectedId, setSelectedId] = useState<number | null>(null)
  const [assignCorps, setAssignCorps] = useState('')

  const selected = marshals.find((m) => m.id === selectedId) ?? null

  return (
    <div
      style={{
        position: 'fixed',
        top: 0,
        right: 0,
        width: 360,
        height: '100vh',
        background: 'linear-gradient(180deg,#0d0d2a,#0a0a1e)',
        borderLeft: '2px solid #5a4524',
        boxShadow: '-4px 0 24px rgba(0,0,0,0.7)',
        transform: open ? 'translateX(0)' : 'translateX(100%)',
        transition: 'transform 0.3s ease',
        zIndex: 200,
        display: 'flex',
        flexDirection: 'column',
        fontFamily: 'Cinzel, serif',
        color: '#e8dcc8',
      }}
    >
      {/* header */}
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          padding: '14px 16px',
          borderBottom: '1px solid #5a4524',
          background: 'rgba(30,20,10,0.6)',
        }}
      >
        <span style={{ color: '#d4af37', fontSize: 16, fontWeight: 700, letterSpacing: 1.5 }}>
          MARSHALS
        </span>
        <button
          onClick={onClose}
          style={{
            background: 'none',
            border: '1px solid #5a4524',
            color: '#aa8844',
            cursor: 'pointer',
            fontSize: 14,
            padding: '2px 8px',
            borderRadius: 3,
          }}
        >
          ✕
        </button>
      </div>

      {/* list */}
      <div style={{ flex: 1, overflowY: 'auto', padding: '8px 12px' }}>
        {marshals.map((m) => (
          <div
            key={m.id}
            onClick={() => setSelectedId(selectedId === m.id ? null : m.id)}
            style={{
              padding: '10px 12px',
              marginBottom: 6,
              background:
                selectedId === m.id
                  ? 'rgba(212,175,55,0.12)'
                  : 'rgba(20,18,40,0.7)',
              border: `1px solid ${selectedId === m.id ? '#d4af37' : '#2a2a4a'}`,
              borderRadius: 4,
              cursor: 'pointer',
              transition: 'background 0.15s',
            }}
          >
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 4 }}>
              <span style={{ fontWeight: 700, fontSize: 14 }}>{m.name}</span>
              <StarRating skill={m.skill} />
            </div>
            <div style={{ display: 'flex', flexWrap: 'wrap' }}>
              {m.traits.map((t) => (
                <TraitBadge key={t} trait={t} />
              ))}
            </div>
          </div>
        ))}
      </div>

      {/* detail card */}
      {selected && (
        <div
          style={{
            borderTop: '1px solid #5a4524',
            padding: '14px 16px',
            background: 'rgba(15,12,30,0.9)',
          }}
        >
          <div style={{ color: '#d4af37', fontSize: 15, fontWeight: 700, marginBottom: 6 }}>
            {selected.name}
          </div>
          <div style={{ fontSize: 12, marginBottom: 4 }}>
            Skill: <StarRating skill={selected.skill} />
          </div>
          <div style={{ fontSize: 12, marginBottom: 6 }}>
            Assigned Corps: {selected.assignedCorps != null ? `Corps ${selected.assignedCorps}` : 'None'}
          </div>
          <div style={{ marginBottom: 8 }}>
            <div style={{ fontSize: 11, color: '#888', marginBottom: 3 }}>Traits:</div>
            {selected.traits.map((t) => (
              <TraitBadge key={t} trait={t} />
            ))}
          </div>
          <div style={{ fontSize: 12, color: '#aaa', marginBottom: 8 }}>
            Stat Bonuses: +{selected.skill * 2}% attack, +{selected.skill}% defense
          </div>
          {/* assign form */}
          <div style={{ display: 'flex', gap: 6 }}>
            <input
              type="number"
              placeholder="Corps ID"
              value={assignCorps}
              onChange={(e) => setAssignCorps(e.target.value)}
              style={{
                flex: 1,
                background: '#1a1a2e',
                border: '1px solid #3a3a5a',
                color: '#e8dcc8',
                padding: '4px 8px',
                fontSize: 12,
                borderRadius: 3,
              }}
            />
            <button
              onClick={() => {
                const cid = parseInt(assignCorps, 10)
                if (!isNaN(cid)) {
                  onAssign(selected.id, cid)
                  setAssignCorps('')
                }
              }}
              style={{
                background: 'rgba(212,175,55,0.2)',
                border: '1px solid #d4af37',
                color: '#d4af37',
                cursor: 'pointer',
                padding: '4px 12px',
                fontSize: 12,
                borderRadius: 3,
                fontWeight: 600,
              }}
            >
              Assign
            </button>
          </div>
        </div>
      )}
    </div>
  )
}
