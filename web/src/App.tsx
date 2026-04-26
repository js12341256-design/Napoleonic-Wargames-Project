import React, { useState, useCallback } from 'react'
import type { Marshal, DivisionTemplate, PowerEconomy, GameEvent } from './types'
import ClockPanel from './components/ClockPanel'
import MarshalsPanel from './components/MarshalsPanel'
import DivisionDesigner from './components/DivisionDesigner'
import EconomyPanel from './components/EconomyPanel'
import EventPopup from './components/EventPopup'

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

const MOCK_EVENTS: GameEvent[] = [
  {
    id: 1,
    title: 'The Emperor Crowns Himself',
    description:
      'In the Cathedral of Notre-Dame, Napoleon Bonaparte takes the crown from the Pope\'s hands and places it upon his own head. The assembled dignitaries watch in stunned silence as a new era begins for France.',
    firesFor: 'FRA',
    options: [
      { label: 'Grand Ceremony', effects: ['+5 manpower', '-10 treasury'] },
      { label: 'Simple Ceremony', effects: ['+15 treasury'] },
    ],
  },
  {
    id: 6,
    title: 'Trafalgar Aftermath',
    description:
      'The combined Franco-Spanish fleet has been shattered off Cape Trafalgar. Admiral Villeneuve is captured, and French naval power lies broken. The Emperor must decide how to respond.',
    firesFor: 'FRA',
    options: [
      { label: 'Rebuild the Fleet', effects: ['-30 treasury'] },
      { label: 'Accept Naval Inferiority', effects: ['+10 manpower'] },
    ],
  },
  {
    id: 9,
    title: 'Trafalgar Victory',
    description:
      'The Royal Navy has won a decisive victory off Cape Trafalgar. But the triumph is bittersweet \u2014 Admiral Lord Nelson has fallen on the deck of HMS Victory.',
    firesFor: 'GBR',
    options: [
      { label: 'Honor Nelson', effects: ['+5 manpower', '-10 treasury'] },
      { label: 'Focus on Victory', effects: ['+10 treasury'] },
    ],
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
  const [economyOpen, setEconomyOpen] = useState(false)

  // Data
  const [marshals, setMarshals] = useState<Marshal[]>(MOCK_MARSHALS)
  const [templates, setTemplates] = useState<DivisionTemplate[]>([])

  // Economy state — default starting values (France as player)
  const [economies, setEconomies] = useState<Record<string, PowerEconomy>>({
    FRA: { power: 'FRA', treasury: 5000, income_per_day: 120, expenditure_per_day: 80, manpower_pool: 650000, manpower_cap: 650000, manpower_recovery: 8000, factories: 8, war_exhaustion: 0 },
    GBR: { power: 'GBR', treasury: 15000, income_per_day: 80, expenditure_per_day: 60, manpower_pool: 150000, manpower_cap: 150000, manpower_recovery: 2000, factories: 15, war_exhaustion: 0 },
    RUS: { power: 'RUS', treasury: 2000, income_per_day: 60, expenditure_per_day: 45, manpower_pool: 900000, manpower_cap: 900000, manpower_recovery: 10000, factories: 3, war_exhaustion: 0 },
    AUS: { power: 'AUS', treasury: 3000, income_per_day: 70, expenditure_per_day: 50, manpower_pool: 400000, manpower_cap: 400000, manpower_recovery: 5000, factories: 5, war_exhaustion: 0 },
    PRU: { power: 'PRU', treasury: 2500, income_per_day: 50, expenditure_per_day: 35, manpower_pool: 200000, manpower_cap: 200000, manpower_recovery: 3000, factories: 4, war_exhaustion: 0 },
    OTT: { power: 'OTT', treasury: 4000, income_per_day: 55, expenditure_per_day: 40, manpower_pool: 300000, manpower_cap: 300000, manpower_recovery: 4000, factories: 2, war_exhaustion: 0 },
    SPA: { power: 'SPA', treasury: 6000, income_per_day: 65, expenditure_per_day: 45, manpower_pool: 180000, manpower_cap: 180000, manpower_recovery: 2500, factories: 3, war_exhaustion: 0 },
  })
  const playerPower = 'FRA'
  const playerEconomy = economies[playerPower]

  // Events
  const [pendingEvents, setPendingEvents] = useState<GameEvent[]>(MOCK_EVENTS)

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

    // Advance all economies by 1 day per tick
    setEconomies((prev) => {
      const next: Record<string, PowerEconomy> = {}
      for (const [pid, eco] of Object.entries(prev)) {
        const netDaily = eco.income_per_day - eco.expenditure_per_day
        const recovered = Math.floor(eco.manpower_recovery / 30)
        next[pid] = {
          ...eco,
          treasury: eco.treasury + netDaily,
          manpower_pool: Math.min(eco.manpower_cap, eco.manpower_pool + recovered),
        }
      }
      return next
    })
  }, [month])

  const handleRecruit = useCallback(() => {
    setEconomies((prev) => {
      const eco = prev[playerPower]
      if (!eco || eco.manpower_pool < 10_000 || eco.treasury < 500) return prev
      return {
        ...prev,
        [playerPower]: {
          ...eco,
          manpower_pool: eco.manpower_pool - 10_000,
          treasury: eco.treasury - 500,
        },
      }
    })
  }, [playerPower])

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

  const handleResolveEvent = useCallback((eventId: number, _optionIndex: number) => {
    setPendingEvents((prev) => prev.filter((e) => e.id !== eventId))
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

        {/* Economy button */}
        <button
          onClick={() => setEconomyOpen((o) => !o)}
          style={{
            background: economyOpen ? 'rgba(212,175,55,0.2)' : 'rgba(30,25,50,0.8)',
            border: `1px solid ${economyOpen ? '#d4af37' : '#5a4524'}`,
            color: economyOpen ? '#d4af37' : '#aa8844',
            cursor: 'pointer',
            padding: '6px 14px',
            fontSize: 14,
            fontWeight: 700,
            letterSpacing: 1,
            borderRadius: 3,
            fontFamily: 'Cinzel, serif',
          }}
        >
          Economy
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
          treasury={playerEconomy?.treasury}
          incomePerDay={playerEconomy?.income_per_day}
          manpowerPool={playerEconomy?.manpower_pool}
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

      {/* Economy Panel */}
      {playerEconomy && (
        <EconomyPanel
          economy={playerEconomy}
          open={economyOpen}
          onClose={() => setEconomyOpen(false)}
          onRecruit={handleRecruit}
        />
      )}

      {/* Event Popup */}
      <EventPopup events={pendingEvents} onResolve={handleResolveEvent} />
    </div>
  )
}
