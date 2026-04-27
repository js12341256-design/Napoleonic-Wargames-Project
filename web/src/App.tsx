import React, { useState, useCallback, useEffect, useRef } from 'react'
import type { Marshal, DivisionTemplate, PowerEconomy, GameEvent, PowerPoliticsData, BattleEvent, TerritoryInfo } from './types'
import MapView, { AttackArrow, ContestedArea, BattleToast } from './MapView'
import ClockPanel from './components/ClockPanel'
import MarshalsPanel from './components/MarshalsPanel'
import DivisionDesigner from './components/DivisionDesigner'
import EconomyPanel from './components/EconomyPanel'
import EventPopup from './components/EventPopup'
import FocusTree from './components/FocusTree'
import PoliticsPanel from './components/PoliticsPanel'
import BattleScreen from './components/BattleScreen'
import TerritoryPanel from './components/TerritoryPanel'
import MiniMap from './components/MiniMap'

const POWER_FLAGS: Record<string, string> = {
  FRA: '🇫🇷', GBR: '🇬🇧', AUS: '🦅', PRU: '⚫', RUS: '🐻', OTT: '☪️', SPA: '🇪🇸',
}
const POWER_NAMES: Record<string, string> = {
  FRA: 'France', GBR: 'Britain', AUS: 'Austria', PRU: 'Prussia',
  RUS: 'Russia', OTT: 'Ottoman', SPA: 'Spain',
}
const MONTH_NAMES = [
  'January','February','March','April','May','June',
  'July','August','September','October','November','December',
]
const DAYS_IN_MONTH = [31,28,31,30,31,30,31,31,30,31,30,31]

function formatDate(day: number, month: number, year: number) {
  return `${day} ${MONTH_NAMES[month]} ${year}`
}

const MOCK_MARSHALS: Marshal[] = [
  { id:1, name:'Napoleon Bonaparte', power:'FRA', skill:10, traits:['Tactician','Aggressive','InspirationalLeader'], portraitKey:'napoleon' },
  { id:2, name:'Louis-Nicolas Davout', power:'FRA', skill:9, traits:['DefensiveGenius','Tactician','Logistics'], assignedCorps:3, portraitKey:'davout' },
  { id:3, name:'Michel Ney', power:'FRA', skill:8, traits:['Aggressive','InspirationalLeader'], assignedCorps:6, portraitKey:'ney' },
  { id:4, name:'Joachim Murat', power:'FRA', skill:8, traits:['CavalryMaster','Aggressive'], portraitKey:'murat' },
  { id:5, name:'Duke of Wellington', power:'GBR', skill:9, traits:['DefensiveGenius','Tactician'], portraitKey:'wellington' },
  { id:6, name:'Mikhail Kutuzov', power:'RUS', skill:9, traits:['DefensiveGenius','Logistics'], portraitKey:'kutuzov' },
]

const MOCK_EVENTS: GameEvent[] = [
  {
    id:1, title:'The Emperor Crowns Himself', firesFor:'FRA',
    description:'In the Cathedral of Notre-Dame, Napoleon takes the crown from the Pope\'s hands and places it upon his own head. A new empire is born.',
    options:[
      { label:'Grand Ceremony', effects:['+Morale','-500 Gold'] },
      { label:'Simple Ceremony', effects:['+300 Gold'] },
    ],
  },
  {
    id:6, title:'Trafalgar — The Fleet is Lost', firesFor:'FRA',
    description:'The combined Franco-Spanish fleet has been shattered off Cape Trafalgar. Admiral Villeneuve is captured. France\'s naval ambitions lie broken.',
    options:[
      { label:'Rebuild the Fleet', effects:['-3000 Gold','+Naval Strength'] },
      { label:'Accept Naval Inferiority', effects:['+10000 Manpower','+Army Focus'] },
    ],
  },
]

const MOCK_ATTACK_ARROWS: AttackArrow[] = [
  { fromArea:'ven', toArea:'vie', attacker:'FRA', strength:45000 },
  { fromArea:'bav', toArea:'aus', attacker:'FRA', strength:32000 },
]
const MOCK_CONTESTED: ContestedArea[] = [
  { areaId:'ven', attacker:'FRA', defender:'AUS', pressure:60 },
]

export default function App() {
  // Clock
  const [day, setDay] = useState(1)
  const [month, setMonth] = useState(0)
  const [year, setYear] = useState(1805)
  const [speed, setSpeed] = useState(1)
  const [paused, setPaused] = useState(true)
  const monthRef = useRef(month)
  useEffect(() => { monthRef.current = month }, [month])

  // Panels
  const [marshalsOpen, setMarshalsOpen] = useState(false)
  const [divisionsOpen, setDivisionsOpen] = useState(false)
  const [economyOpen, setEconomyOpen] = useState(false)
  const [focusOpen, setFocusOpen] = useState(false)
  const [politicsOpen, setPoliticsOpen] = useState(false)

  // Data
  const [marshals, setMarshals] = useState<Marshal[]>(MOCK_MARSHALS)
  const [templates, setTemplates] = useState<DivisionTemplate[]>([])
  const [pendingEvents, setPendingEvents] = useState<GameEvent[]>([])
  useEffect(() => { const t = setTimeout(() => setPendingEvents(MOCK_EVENTS), 8000); return () => clearTimeout(t) }, [])
  const [battleToasts, setBattleToasts] = useState<BattleToast[]>([
    { area:'ven', areaName:'Austerlitz', attacker:'FRA', result:'AttackerAdvances', timestamp: Date.now() },
  ])
  const [turn, setTurn] = useState(0)

  // Combat & territory state
  const [battleEvent, setBattleEvent] = useState<BattleEvent | null>(null)
  const [selectedTerritory, setSelectedTerritory] = useState<TerritoryInfo | null>(null)

  const playerPower = 'FRA'

  const [playerPolitics] = useState<PowerPoliticsData>({
    power: 'FRA',
    legitimacy: 85,
    stability: 2,
    government: 'Empire',
    ruling_faction: 'Military',
    faction_support: { Military: 60, Merchants: 40, Clergy: 30 },
    puppets: [],
    overlord: null,
  })

  // Economy
  const [economies, setEconomies] = useState<Record<string,PowerEconomy>>({
    FRA: { power:'FRA', treasury:5000,  income_per_day:120, expenditure_per_day:80,  manpower_pool:650000, manpower_cap:650000, manpower_recovery:8000,  factories:8,  war_exhaustion:0 },
    GBR: { power:'GBR', treasury:15000, income_per_day:80,  expenditure_per_day:60,  manpower_pool:150000, manpower_cap:150000, manpower_recovery:2000,  factories:15, war_exhaustion:0 },
    RUS: { power:'RUS', treasury:2000,  income_per_day:60,  expenditure_per_day:45,  manpower_pool:900000, manpower_cap:900000, manpower_recovery:10000, factories:3,  war_exhaustion:0 },
    AUS: { power:'AUS', treasury:3000,  income_per_day:70,  expenditure_per_day:50,  manpower_pool:400000, manpower_cap:400000, manpower_recovery:5000,  factories:5,  war_exhaustion:0 },
    PRU: { power:'PRU', treasury:2500,  income_per_day:50,  expenditure_per_day:35,  manpower_pool:200000, manpower_cap:200000, manpower_recovery:3000,  factories:4,  war_exhaustion:0 },
    OTT: { power:'OTT', treasury:4000,  income_per_day:55,  expenditure_per_day:40,  manpower_pool:300000, manpower_cap:300000, manpower_recovery:4000,  factories:2,  war_exhaustion:0 },
    SPA: { power:'SPA', treasury:6000,  income_per_day:65,  expenditure_per_day:45,  manpower_pool:180000, manpower_cap:180000, manpower_recovery:2500,  factories:3,  war_exhaustion:0 },
  })
  const playerEconomy = economies[playerPower]

  const handleTick = useCallback(() => {
    setDay(d => {
      const m = monthRef.current
      const limit = DAYS_IN_MONTH[m]
      if (d + 1 > limit) {
        setMonth(pm => {
          const nm = (pm + 1) % 12
          if (nm === 0) setYear(y => y + 1)
          monthRef.current = nm
          return nm
        })
        return 1
      }
      return d + 1
    })
    setEconomies(prev => {
      const next: Record<string,PowerEconomy> = {}
      for (const [pid, eco] of Object.entries(prev)) {
        const net = eco.income_per_day - eco.expenditure_per_day
        const mp = Math.floor(eco.manpower_recovery / 30)
        next[pid] = { ...eco, treasury: eco.treasury + net, manpower_pool: Math.min(eco.manpower_cap, eco.manpower_pool + mp) }
      }
      return next
    })
  }, [])

  const handleRecruit = useCallback(() => {
    setEconomies(prev => {
      const eco = prev[playerPower]
      if (!eco || eco.manpower_pool < 10000 || eco.treasury < 500) return prev
      return { ...prev, [playerPower]: { ...eco, manpower_pool: eco.manpower_pool - 10000, treasury: eco.treasury - 500 } }
    })
  }, [])

  const handleAssign = useCallback((marshalId: number, corpsId: number) => {
    setMarshals(prev => prev.map(m => m.id === marshalId ? { ...m, assignedCorps: corpsId } : m))
  }, [])

  const handleSaveTemplate = useCallback((t: DivisionTemplate) => {
    setTemplates(prev => [...prev, t])
  }, [])

  const handleResolveEvent = useCallback((eventId: number, _idx: number) => {
    setPendingEvents(prev => prev.filter(e => e.id !== eventId))
  }, [])

  const handleEndTurn = useCallback(() => {
    setTurn(t => t + 1)
    // Mock: trigger a battle between France and Austria
    setBattleEvent({
      territory: 'Austerlitz',
      attacker: { power: 'FRA', commander: 'Napoleon Bonaparte', strength: 73000, tactic: 'Column' },
      defender: { power: 'AUS', commander: 'Archduke Charles', strength: 85000, tactic: 'Line' },
      outcome: 'attacker_advances',
      attackerCasualties: 9200,
      defenderCasualties: 27000,
    })
  }, [])

  const btnStyle = (active: boolean): React.CSSProperties => ({
    background: active ? 'rgba(212,175,55,0.18)' : 'rgba(20,18,35,0.9)',
    border: `1px solid ${active ? '#d4af37' : '#3a2f1a'}`,
    color: active ? '#d4af37' : '#7a6030',
    cursor: 'pointer',
    padding: '5px 13px',
    fontSize: 11,
    fontWeight: 700,
    letterSpacing: 1,
    borderRadius: 2,
    fontFamily: 'Cinzel, serif',
    transition: 'all 0.15s',
  })

  return (
    <div style={{ fontFamily:'Cinzel, serif', background:'#080810', color:'#e8dcc8', minHeight:'100vh', display:'flex', flexDirection:'column', overflow:'hidden' }}>

      {/* ── TOP BAR ── */}
      <div style={{
        height: 48,
        background: 'linear-gradient(180deg,#0e0c1c 0%,#07060f 100%)',
        borderBottom: '1px solid #2a1f08',
        display: 'flex',
        alignItems: 'center',
        padding: '0 10px',
        gap: 6,
        flexShrink: 0,
        zIndex: 100,
        boxShadow: '0 2px 12px rgba(0,0,0,0.7)',
      }}>
        {/* Power flags row */}
        {Object.keys(POWER_NAMES).map(pid => (
          <div key={pid} style={{
            height: 34,
            border: pid === playerPower ? '1px solid #d4af37' : '1px solid #2a1f08',
            background: pid === playerPower ? 'rgba(212,175,55,0.1)' : 'rgba(10,8,20,0.8)',
            display: 'flex', alignItems: 'center', padding: '0 8px', gap: 5, borderRadius: 2,
          }}>
            <span style={{ fontSize: 14 }}>{POWER_FLAGS[pid]}</span>
            <span style={{ color: pid === playerPower ? '#d4af37' : '#5a4820', fontSize: 10, fontWeight: 700, letterSpacing: 1 }}>
              {POWER_NAMES[pid]}
            </span>
          </div>
        ))}

        <div style={{ flex: 1 }} />

        {/* Action buttons */}
        <button style={btnStyle(politicsOpen)} onClick={() => setPoliticsOpen(o => !o)}>⚖️ Politics</button>
        <button style={btnStyle(focusOpen)} onClick={() => setFocusOpen(o => !o)}>🎯 Focus</button>
        <button style={btnStyle(economyOpen)} onClick={() => setEconomyOpen(o => !o)}>💰 Economy</button>
        <button style={btnStyle(marshalsOpen)} onClick={() => setMarshalsOpen(o => !o)}>⚔️ Marshals</button>
        <button style={btnStyle(divisionsOpen)} onClick={() => setDivisionsOpen(true)}>🪖 Divisions</button>
      </div>

      {/* ── MAIN: MAP fills remaining space ── */}
      <div style={{ flex: 1, position: 'relative', overflow: 'hidden' }}>
        <MapView
          scenarioData={null}
          powerStates={{}}
          currentTurn={turn}
          onEndTurn={handleEndTurn}
          attackArrows={MOCK_ATTACK_ARROWS}
          contestedAreas={MOCK_CONTESTED}
          battleToasts={battleToasts}
        />

        {/* Clock anchored top-center over map */}
        <div style={{ position: 'absolute', top: 10, left: '50%', transform: 'translateX(-50%)', zIndex: 50 }}>
          <ClockPanel
            date={formatDate(day, month, year)}
            speed={speed}
            paused={paused}
            onSetSpeed={setSpeed}
            onTogglePause={() => setPaused(p => !p)}
            onTick={handleTick}
            treasury={playerEconomy?.treasury}
            incomePerDay={playerEconomy?.income_per_day}
            manpowerPool={playerEconomy?.manpower_pool}
          />
        </div>
      </div>

      {/* ── OVERLAYS ── */}
      <MarshalsPanel marshals={marshals} onAssign={handleAssign} open={marshalsOpen} onClose={() => setMarshalsOpen(false)} />
      <DivisionDesigner templates={templates} onSave={handleSaveTemplate} open={divisionsOpen} onClose={() => setDivisionsOpen(false)} />
      {playerEconomy && <EconomyPanel economy={playerEconomy} open={economyOpen} onClose={() => setEconomyOpen(false)} onRecruit={handleRecruit} />}
      <PoliticsPanel politics={playerPolitics} open={politicsOpen} onClose={() => setPoliticsOpen(false)} />
      <FocusTree open={focusOpen} onClose={() => setFocusOpen(false)} />
      <EventPopup events={pendingEvents} onResolve={handleResolveEvent} />

      {/* Battle screen modal */}
      {battleEvent && <BattleScreen battle={battleEvent} onClose={() => setBattleEvent(null)} />}

      {/* Territory panel */}
      {selectedTerritory && <TerritoryPanel territory={selectedTerritory} onClose={() => setSelectedTerritory(null)} />}

      {/* Minimap */}
      <MiniMap
        panX={0} panY={0} zoom={1}
        mapWidth={1600} mapHeight={1000}
        viewportWidth={1200} viewportHeight={700}
        onNavigate={() => {}}
      />

      {/* Mock: Trigger Battle button */}
      <button
        onClick={() => setBattleEvent({
          territory: 'Ulm',
          attacker: { power: 'FRA', commander: 'Michel Ney', strength: 45000, tactic: 'Column' },
          defender: { power: 'AUS', commander: 'Karl Mack', strength: 32000, tactic: 'Square' },
          outcome: 'attacker_advances',
          attackerCasualties: 3800,
          defenderCasualties: 18000,
        })}
        style={{
          position: 'fixed',
          bottom: 12,
          left: 12,
          background: 'rgba(20,18,35,0.9)',
          border: '1px solid #3a2f1a',
          color: '#7a6030',
          cursor: 'pointer',
          padding: '4px 10px',
          fontSize: 10,
          fontFamily: 'Cinzel, serif',
          borderRadius: 3,
          zIndex: 100,
        }}
      >
        Trigger Battle
      </button>

      {/* Mock: territory click */}
      <button
        onClick={() => setSelectedTerritory({
          id: 'vie',
          name: 'Vienna',
          owner: 'AUS',
          terrain: 'Plains',
          corps: [
            { name: 'I Corps', strength: 28000, marshal: 'Archduke Charles' },
            { name: 'II Corps', strength: 22000 },
          ],
          goldPerDay: 15,
          manpowerPerMonth: 3000,
        })}
        style={{
          position: 'fixed',
          bottom: 12,
          left: 120,
          background: 'rgba(20,18,35,0.9)',
          border: '1px solid #3a2f1a',
          color: '#7a6030',
          cursor: 'pointer',
          padding: '4px 10px',
          fontSize: 10,
          fontFamily: 'Cinzel, serif',
          borderRadius: 3,
          zIndex: 100,
        }}
      >
        Territory Info
      </button>
    </div>
  )
}
