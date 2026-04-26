import React, { useState, useCallback } from 'react'
import type { Marshal, DivisionTemplate } from './types'
import ClockPanel from './components/ClockPanel'
import MarshalsPanel from './components/MarshalsPanel'
import DivisionDesigner from './components/DivisionDesigner'
import FocusTree from './components/FocusTree'

const POWER_FLAGS: Record<string, string> = {
  FRA: '🇫🇷',
  GBR: '🇬🇧',
  AUS: '🦅',
  PRU: '⚫',
  RUS: '🐻',
  OTT: '☪️',
  SPA: '🇪🇸',
}

const POWER_NAMES: Record<string, string> = {
  FRA: 'France',
  GBR: 'Britain',
  AUS: 'Austria',
  PRU: 'Prussia',
  RUS: 'Russia',
  OTT: 'Ottoman',
  SPA: 'Spain',
}

const MONTH_NAMES = [
  'January', 'February', 'March', 'April', 'May', 'June',
  'July', 'August', 'September', 'October', 'November', 'December',
]

const DAYS_IN_MONTH = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]

function formatDate(day: number, month: number, year: number): string {
  return `${day} ${MONTH_NAMES[month]} ${year}`
}

const MOCK_MARSHALS: Marshal[] = [
  {
    id: 1,
    name: 'Michel Ney',
    power: 'FRA',
    skill: 8,
    traits: ['Aggressive', 'CavalryMaster', 'InspirationalLeader'],
    assignedCorps: 6,
    portraitKey: 'ney',
  },
  {
    id: 2,
    name: 'Louis-Nicolas Davout',
    power: 'FRA',
    skill: 9,
    traits: ['DefensiveGenius', 'Tactician', 'Logistics'],
    assignedCorps: 3,
    portraitKey: 'davout',
  },
  {
    id: 3,
    name: 'Jean Lannes',
    power: 'FRA',
    skill: 8,
    traits: ['Aggressive', 'Siege', 'InspirationalLeader'],
    portraitKey: 'lannes',
  },
  {
    id: 4,
    name: 'André Masséna',
    power: 'FRA',
    skill: 7,
    traits: ['DefensiveGenius', 'Logistics'],
    portraitKey: 'massena',
  },
]

interface GameState {
  turn: number
  powers: string[]
  status: 'idle' | 'loading' | 'playing'
}

export default function App() {
  const [game, setGame] = useState<GameState>({
    turn: 0,
    powers: Object.keys(POWER_NAMES),
    status: 'playing',
  })

  // Clock state
  const [day, setDay] = useState(1)
  const [month, setMonth] = useState(0) // 0-indexed
  const [year, setYear] = useState(1805)
  const [speed, setSpeed] = useState(1)
  const [paused, setPaused] = useState(true)

  // Panel toggles
  const [marshalsOpen, setMarshalsOpen] = useState(false)
  const [divisionsOpen, setDivisionsOpen] = useState(false)
  const [focusOpen, setFocusOpen] = useState(false)

  // Data
  const [marshals, setMarshals] = useState<Marshal[]>(MOCK_MARSHALS)
  const [templates, setTemplates] = useState<DivisionTemplate[]>([])

  const handleTick = useCallback(() => {
    setDay((d) => {
      let newDay = d + 1
      setMonth((m) => {
        let newMonth = m
        setYear((y) => {
          let newYear = y
          if (newDay > DAYS_IN_MONTH[newMonth]) {
            newDay = 1
            newMonth = newMonth + 1
            if (newMonth > 11) {
              newMonth = 0
              newYear = y + 1
            }
            // Need to set month since we changed it
            setMonth(newMonth)
            setYear(newYear)
          }
          return newYear
        })
        return m // month set inside year setter if needed
      })
      if (newDay > DAYS_IN_MONTH[month]) {
        return 1
      }
      return newDay
    })
  }, [month])

  const handleAssign = useCallback((marshalId: number, corpsId: number) => {
    setMarshals((prev) =>
      prev.map((m) =>
        m.id === marshalId ? { ...m, assignedCorps: corpsId } : m
      )
    )
  }, [])

  const handleSaveTemplate = useCallback((t: DivisionTemplate) => {
    setTemplates((prev) => [...prev, t])
  }, [])

  return (
    <div
      style={{
        fontFamily: 'Cinzel, serif',
        background: '#0a0a12',
        color: '#e8dcc8',
        minHeight: '100vh',
        display: 'flex',
        flexDirection: 'column',
      }}
    >
      {/* ── Top bar (HoI4 style) ── */}
      <div
        style={{
          height: 54,
          background: 'linear-gradient(180deg,#0d0d1a,#080810)',
          borderBottom: '1px solid #5a4524',
          display: 'flex',
          alignItems: 'center',
          padding: '0 12px',
          gap: 10,
          flexShrink: 0,
          overflowX: 'auto',
          zIndex: 50,
        }}
      >
        {/* Power cards */}
        {game.powers.map((pid) => (
          <div
            key={pid}
            style={{
              minWidth: 130,
              height: 36,
              border: '1px solid #5a4524',
              background: 'rgba(15,12,25,0.92)',
              display: 'flex',
              alignItems: 'center',
              padding: '0 10px',
              gap: 6,
              borderRadius: 2,
            }}
          >
            <span style={{ fontSize: 16 }}>{POWER_FLAGS[pid] ?? ''}</span>
            <span
              style={{
                color: '#d4af37',
                fontSize: 12,
                fontWeight: 700,
                letterSpacing: 1.2,
              }}
            >
              {POWER_NAMES[pid]}
            </span>
          </div>
        ))}

        {/* spacer */}
        <div style={{ flex: 1 }} />

        {/* Focus button */}
        <button
          onClick={() => setFocusOpen(true)}
          style={{
            background: focusOpen ? 'rgba(212,175,55,0.2)' : 'rgba(30,25,50,0.8)',
            border: `1px solid ${focusOpen ? '#d4af37' : '#5a4524'}`,
            color: focusOpen ? '#d4af37' : '#aa8844',
            cursor: 'pointer',
            padding: '6px 14px',
            fontSize: 12,
            fontWeight: 700,
            letterSpacing: 1,
            borderRadius: 3,
            fontFamily: 'Cinzel, serif',
          }}
        >
          Focus
        </button>

        {/* Marshals button */}
        <button
          onClick={() => setMarshalsOpen((o) => !o)}
          style={{
            background: marshalsOpen ? 'rgba(212,175,55,0.2)' : 'rgba(30,25,50,0.8)',
            border: `1px solid ${marshalsOpen ? '#d4af37' : '#5a4524'}`,
            color: marshalsOpen ? '#d4af37' : '#aa8844',
            cursor: 'pointer',
            padding: '6px 14px',
            fontSize: 12,
            fontWeight: 700,
            letterSpacing: 1,
            borderRadius: 3,
            fontFamily: 'Cinzel, serif',
          }}
        >
          Marshals
        </button>

        {/* Divisions button */}
        <button
          onClick={() => setDivisionsOpen(true)}
          style={{
            background: 'rgba(30,25,50,0.8)',
            border: '1px solid #5a4524',
            color: '#aa8844',
            cursor: 'pointer',
            padding: '6px 14px',
            fontSize: 12,
            fontWeight: 700,
            letterSpacing: 1,
            borderRadius: 3,
            fontFamily: 'Cinzel, serif',
          }}
        >
          Divisions
        </button>
      </div>

      {/* ── Main content area ── */}
      <div style={{ flex: 1, position: 'relative' }}>
        {/* Clock Panel */}
        <ClockPanel
          date={formatDate(day, month, year)}
          speed={speed}
          paused={paused}
          onSetSpeed={setSpeed}
          onTogglePause={() => setPaused((p) => !p)}
          onTick={handleTick}
        />

        {/* Placeholder for map */}
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            height: 'calc(100vh - 54px)',
            color: '#444',
            fontSize: 14,
            fontStyle: 'italic',
          }}
        >
          Map viewport — awaiting WASM integration
        </div>
      </div>

      {/* Marshals Panel */}
      <MarshalsPanel
        marshals={marshals}
        onAssign={handleAssign}
        open={marshalsOpen}
        onClose={() => setMarshalsOpen(false)}
      />

      {/* Division Designer */}
      <DivisionDesigner
        templates={templates}
        onSave={handleSaveTemplate}
        open={divisionsOpen}
        onClose={() => setDivisionsOpen(false)}
      />

      {/* Focus Tree */}
      <FocusTree
        open={focusOpen}
        onClose={() => setFocusOpen(false)}
      />
    </div>
  )
}
