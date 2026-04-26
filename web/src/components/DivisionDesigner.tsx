import React, { useState } from 'react'
import type { DivisionTemplate } from '../types'

interface DivisionDesignerProps {
  templates: DivisionTemplate[]
  onSave: (t: DivisionTemplate) => void
  open: boolean
  onClose: () => void
}

type Tactic = DivisionTemplate['tactic']

const TACTIC_DESCRIPTIONS: Record<Tactic, string> = {
  Column: 'Concentrated attack power, weaker defense. +20% attack, -10% defense.',
  Line: 'Balanced formation. Maximizes firepower at range.',
  Square: 'Anti-cavalry formation. -30% attack, +40% defense vs cavalry.',
  SkirmishScreen: 'Light screen formation. +1 speed, -20% attack.',
}

const TACTIC_MOD: Record<Tactic, { atkMul: number; defMul: number; spdBonus: number }> = {
  Column: { atkMul: 1.2, defMul: 0.9, spdBonus: 0 },
  Line: { atkMul: 1.0, defMul: 1.0, spdBonus: 0 },
  Square: { atkMul: 0.7, defMul: 1.4, spdBonus: 0 },
  SkirmishScreen: { atkMul: 0.8, defMul: 0.9, spdBonus: 1 },
}

function calcStats(
  batt: number,
  cav: number,
  art: number,
  tactic: Tactic
) {
  const mod = TACTIC_MOD[tactic]
  const rawAtk = batt * 100 + cav * 80 + art * 150
  const rawDef = batt * 90 + cav * 60 + art * 120
  const attack = Math.round(rawAtk * mod.atkMul)
  const defense = Math.round(rawDef * mod.defMul)
  const speed = 3 - Math.floor(art / 3) + (cav > batt ? 1 : 0) + mod.spdBonus
  const supply = (batt + cav) * 10 + art * 25
  return { attack, defense, speed, supply }
}

function StatBar({ label, value, max, color }: { label: string; value: number; max: number; color: string }) {
  return (
    <div style={{ marginBottom: 6 }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: 11, marginBottom: 2 }}>
        <span style={{ color: '#aaa' }}>{label}</span>
        <span style={{ color, fontWeight: 700 }}>{value}</span>
      </div>
      <div style={{ height: 6, background: '#1a1a2e', borderRadius: 3, overflow: 'hidden' }}>
        <div
          style={{
            height: '100%',
            width: `${Math.min((value / max) * 100, 100)}%`,
            background: color,
            borderRadius: 3,
            transition: 'width 0.2s',
          }}
        />
      </div>
    </div>
  )
}

export default function DivisionDesigner({
  templates,
  onSave,
  open,
  onClose,
}: DivisionDesignerProps) {
  const [name, setName] = useState('')
  const [battalions, setBattalions] = useState(6)
  const [cavalry, setCavalry] = useState(2)
  const [artillery, setArtillery] = useState(1)
  const [tactic, setTactic] = useState<Tactic>('Line')
  const [showForm, setShowForm] = useState(false)
  const [nextId, setNextId] = useState(100)

  if (!open) return null

  const preview = calcStats(battalions, cavalry, artillery, tactic)

  return (
    <div
      style={{
        position: 'fixed',
        inset: 0,
        zIndex: 300,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        background: 'rgba(0,0,0,0.7)',
      }}
      onClick={(e) => {
        if (e.target === e.currentTarget) onClose()
      }}
    >
      <div
        style={{
          width: 560,
          maxHeight: '85vh',
          overflowY: 'auto',
          background: 'linear-gradient(180deg,#0d1a0d,#0a120a)',
          border: '2px solid #2d5016',
          borderRadius: 6,
          boxShadow: '0 8px 32px rgba(0,0,0,0.8)',
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
            padding: '14px 20px',
            borderBottom: '1px solid #2d5016',
            background: 'rgba(45,80,22,0.2)',
          }}
        >
          <span style={{ color: '#8bc34a', fontSize: 16, fontWeight: 700, letterSpacing: 1.5 }}>
            DIVISION DESIGNER
          </span>
          <button
            onClick={onClose}
            style={{
              background: 'none',
              border: '1px solid #2d5016',
              color: '#8bc34a',
              cursor: 'pointer',
              fontSize: 14,
              padding: '2px 8px',
              borderRadius: 3,
            }}
          >
            ✕
          </button>
        </div>

        {/* existing templates */}
        <div style={{ padding: '12px 20px' }}>
          <div style={{ fontSize: 12, color: '#888', marginBottom: 8, letterSpacing: 1 }}>
            EXISTING TEMPLATES
          </div>
          {templates.length === 0 && (
            <div style={{ color: '#555', fontSize: 12, fontStyle: 'italic' }}>No templates yet.</div>
          )}
          {templates.map((t) => {
            const s = calcStats(t.battalions, t.cavalrySquadrons, t.artilleryBatteries, t.tactic)
            return (
              <div
                key={t.id}
                style={{
                  padding: '8px 12px',
                  marginBottom: 4,
                  background: 'rgba(20,30,15,0.7)',
                  border: '1px solid #2a3a20',
                  borderRadius: 3,
                  display: 'flex',
                  justifyContent: 'space-between',
                  alignItems: 'center',
                }}
              >
                <div>
                  <div style={{ fontWeight: 700, fontSize: 13 }}>{t.name}</div>
                  <div style={{ fontSize: 10, color: '#888' }}>
                    {t.battalions}Bn {t.cavalrySquadrons}Cav {t.artilleryBatteries}Art — {t.tactic}
                  </div>
                </div>
                <div style={{ textAlign: 'right', fontSize: 11 }}>
                  <span style={{ color: '#cc4444' }}>ATK {s.attack}</span>{' '}
                  <span style={{ color: '#4488cc' }}>DEF {s.defense}</span>{' '}
                  <span style={{ color: '#88cc44' }}>SPD {s.speed}</span>{' '}
                  <span style={{ color: '#ccaa44' }}>SUP {s.supply}</span>
                </div>
              </div>
            )
          })}

          {/* new template toggle */}
          {!showForm && (
            <button
              onClick={() => setShowForm(true)}
              style={{
                marginTop: 10,
                background: 'rgba(45,80,22,0.3)',
                border: '1px solid #2d5016',
                color: '#8bc34a',
                cursor: 'pointer',
                padding: '8px 16px',
                fontSize: 12,
                borderRadius: 3,
                fontWeight: 700,
                letterSpacing: 1,
                width: '100%',
              }}
            >
              + NEW TEMPLATE
            </button>
          )}
        </div>

        {/* new template form */}
        {showForm && (
          <div
            style={{
              padding: '16px 20px',
              borderTop: '1px solid #2d5016',
              background: 'rgba(45,80,22,0.08)',
            }}
          >
            <div style={{ fontSize: 12, color: '#8bc34a', marginBottom: 10, letterSpacing: 1, fontWeight: 700 }}>
              NEW TEMPLATE
            </div>

            {/* name */}
            <div style={{ marginBottom: 12 }}>
              <label style={{ fontSize: 11, color: '#888', display: 'block', marginBottom: 3 }}>Name</label>
              <input
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder="e.g. Ligne d'Infanterie"
                style={{
                  width: '100%',
                  background: '#0d1a0d',
                  border: '1px solid #2a3a20',
                  color: '#e8dcc8',
                  padding: '6px 10px',
                  fontSize: 13,
                  borderRadius: 3,
                  boxSizing: 'border-box',
                }}
              />
            </div>

            {/* sliders */}
            <SliderRow label="Infantry Battalions" value={battalions} min={1} max={12} onChange={setBattalions} color="#cc8844" />
            <SliderRow label="Cavalry Squadrons" value={cavalry} min={0} max={6} onChange={setCavalry} color="#88cc44" />
            <SliderRow label="Artillery Batteries" value={artillery} min={0} max={4} onChange={setArtillery} color="#cc4444" />

            {/* tactic */}
            <div style={{ marginBottom: 12 }}>
              <label style={{ fontSize: 11, color: '#888', display: 'block', marginBottom: 4 }}>Tactic</label>
              <div style={{ display: 'flex', gap: 6, flexWrap: 'wrap' }}>
                {(Object.keys(TACTIC_DESCRIPTIONS) as Tactic[]).map((t) => (
                  <button
                    key={t}
                    onClick={() => setTactic(t)}
                    title={TACTIC_DESCRIPTIONS[t]}
                    style={{
                      background: tactic === t ? 'rgba(45,80,22,0.5)' : 'rgba(20,30,15,0.5)',
                      border: `1px solid ${tactic === t ? '#8bc34a' : '#2a3a20'}`,
                      color: tactic === t ? '#8bc34a' : '#888',
                      cursor: 'pointer',
                      padding: '4px 10px',
                      fontSize: 11,
                      borderRadius: 3,
                      fontWeight: tactic === t ? 700 : 400,
                    }}
                  >
                    {t}
                  </button>
                ))}
              </div>
              <div style={{ fontSize: 10, color: '#666', marginTop: 4, fontStyle: 'italic' }}>
                {TACTIC_DESCRIPTIONS[tactic]}
              </div>
            </div>

            {/* live stat preview */}
            <div
              style={{
                padding: '12px',
                background: 'rgba(10,18,10,0.8)',
                border: '1px solid #2a3a20',
                borderRadius: 4,
                marginBottom: 12,
              }}
            >
              <div style={{ fontSize: 11, color: '#8bc34a', marginBottom: 8, fontWeight: 700 }}>STAT PREVIEW</div>
              <StatBar label="Attack" value={preview.attack} max={2500} color="#cc4444" />
              <StatBar label="Defense" value={preview.defense} max={2000} color="#4488cc" />
              <StatBar label="Speed" value={preview.speed} max={6} color="#88cc44" />
              <StatBar label="Supply Cost" value={preview.supply} max={200} color="#ccaa44" />
            </div>

            {/* save */}
            <div style={{ display: 'flex', gap: 8 }}>
              <button
                onClick={() => {
                  if (!name.trim()) return
                  onSave({
                    id: nextId,
                    name: name.trim(),
                    power: 'FRA',
                    battalions,
                    cavalrySquadrons: cavalry,
                    artilleryBatteries: artillery,
                    tactic,
                  })
                  setNextId((n) => n + 1)
                  setName('')
                  setBattalions(6)
                  setCavalry(2)
                  setArtillery(1)
                  setTactic('Line')
                  setShowForm(false)
                }}
                style={{
                  flex: 1,
                  background: 'rgba(45,80,22,0.4)',
                  border: '1px solid #2d5016',
                  color: '#8bc34a',
                  cursor: 'pointer',
                  padding: '8px',
                  fontSize: 12,
                  borderRadius: 3,
                  fontWeight: 700,
                }}
              >
                SAVE TEMPLATE
              </button>
              <button
                onClick={() => setShowForm(false)}
                style={{
                  background: 'rgba(60,20,20,0.3)',
                  border: '1px solid #5a2020',
                  color: '#aa6666',
                  cursor: 'pointer',
                  padding: '8px 16px',
                  fontSize: 12,
                  borderRadius: 3,
                }}
              >
                Cancel
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  )
}

function SliderRow({
  label,
  value,
  min,
  max,
  onChange,
  color,
}: {
  label: string
  value: number
  min: number
  max: number
  onChange: (v: number) => void
  color: string
}) {
  return (
    <div style={{ marginBottom: 10 }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: 11, marginBottom: 2 }}>
        <span style={{ color: '#888' }}>{label}</span>
        <span style={{ color, fontWeight: 700 }}>{value}</span>
      </div>
      <input
        type="range"
        min={min}
        max={max}
        value={value}
        onChange={(e) => onChange(parseInt(e.target.value, 10))}
        style={{ width: '100%', accentColor: color }}
      />
    </div>
  )
}
