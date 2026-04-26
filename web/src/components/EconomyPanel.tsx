import React from 'react'
import type { PowerEconomy } from '../types'

interface EconomyPanelProps {
  economy: PowerEconomy
  open: boolean
  onClose: () => void
  onRecruit: () => void
}

function formatNumber(n: number): string {
  if (n >= 1_000_000) return (n / 1_000_000).toFixed(1) + 'M'
  if (n >= 1_000) return Math.floor(n).toLocaleString()
  return String(n)
}

export default function EconomyPanel({
  economy,
  open,
  onClose,
  onRecruit,
}: EconomyPanelProps) {
  const netDaily = economy.income_per_day - economy.expenditure_per_day
  const canRecruit = economy.manpower_pool >= 10_000 && economy.treasury >= 500
  const manpowerPct = economy.manpower_cap > 0
    ? Math.round((economy.manpower_pool / economy.manpower_cap) * 100)
    : 0
  const weColor = economy.war_exhaustion > 50 ? '#ff4444' : '#cc8833'

  // Rough expenditure breakdown (army ~60%, navy ~25%, maintenance ~15%)
  const armyCost = Math.round(economy.expenditure_per_day * 0.6)
  const navyCost = Math.round(economy.expenditure_per_day * 0.25)
  const maintCost = economy.expenditure_per_day - armyCost - navyCost

  return (
    <div
      style={{
        position: 'fixed',
        top: 0,
        right: 0,
        width: 340,
        height: '100vh',
        background: 'linear-gradient(180deg, #0d0d1a 0%, #0a0a14 100%)',
        borderLeft: '1px solid #5a4524',
        zIndex: 200,
        transform: open ? 'translateX(0)' : 'translateX(100%)',
        transition: 'transform 0.3s ease',
        fontFamily: 'Cinzel, serif',
        display: 'flex',
        flexDirection: 'column',
        overflow: 'hidden',
      }}
    >
      {/* Header */}
      <div
        style={{
          padding: '14px 16px',
          borderBottom: '1px solid #5a4524',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          background: 'rgba(212,175,55,0.06)',
        }}
      >
        <span style={{ color: '#d4af37', fontSize: 15, fontWeight: 700, letterSpacing: 1.2 }}>
          ECONOMY
        </span>
        <button
          onClick={onClose}
          style={{
            background: 'none',
            border: '1px solid #5a4524',
            color: '#aa8844',
            cursor: 'pointer',
            padding: '2px 10px',
            fontSize: 14,
            borderRadius: 3,
            fontFamily: 'Cinzel, serif',
          }}
        >
          X
        </button>
      </div>

      {/* Content */}
      <div style={{ padding: '16px', flex: 1, overflowY: 'auto' }}>
        {/* Treasury */}
        <Section label="Treasury">
          <div style={{ display: 'flex', alignItems: 'baseline', gap: 8 }}>
            <span style={{ color: '#d4af37', fontSize: 22, fontWeight: 700 }}>
              {formatNumber(economy.treasury)} gold
            </span>
            <span style={{ color: netDaily >= 0 ? '#44cc44' : '#ff4444', fontSize: 13 }}>
              {netDaily >= 0 ? '+' : ''}{netDaily}/day
            </span>
          </div>
        </Section>

        {/* Manpower */}
        <Section label="Manpower">
          <div style={{ color: '#c8c0b0', fontSize: 14, marginBottom: 6 }}>
            {formatNumber(economy.manpower_pool)} / {formatNumber(economy.manpower_cap)}
          </div>
          <ProgressBar pct={manpowerPct} color="#4488cc" />
          <div style={{ color: '#668899', fontSize: 11, marginTop: 4 }}>
            +{formatNumber(economy.manpower_recovery)}/month recovery
          </div>
        </Section>

        {/* War Exhaustion */}
        <Section label="War Exhaustion">
          <ProgressBar pct={economy.war_exhaustion} color={weColor} />
          <div style={{ color: weColor, fontSize: 12, marginTop: 4 }}>
            {economy.war_exhaustion}%
          </div>
        </Section>

        {/* Factories */}
        <Section label="Factories">
          <div style={{ color: '#b0a890', fontSize: 16 }}>
            {economy.factories} factories
          </div>
        </Section>

        {/* Expenditure Breakdown */}
        <Section label="Expenditure">
          <div style={{ color: '#ff6655', fontSize: 12 }}>
            <div>Army: {armyCost}/day</div>
            <div>Navy: {navyCost}/day</div>
            <div>Maintenance: {maintCost}/day</div>
          </div>
          <div style={{ color: '#ff4444', fontSize: 13, marginTop: 4, fontWeight: 700 }}>
            Total: {economy.expenditure_per_day}/day
          </div>
        </Section>

        {/* Recruit Corps */}
        <div style={{ marginTop: 20 }}>
          <button
            onClick={onRecruit}
            disabled={!canRecruit}
            style={{
              width: '100%',
              padding: '10px 0',
              background: canRecruit
                ? 'linear-gradient(180deg, #2a4a2a, #1a3a1a)'
                : 'rgba(40,40,40,0.5)',
              border: `1px solid ${canRecruit ? '#44aa44' : '#333'}`,
              color: canRecruit ? '#66dd66' : '#555',
              cursor: canRecruit ? 'pointer' : 'not-allowed',
              fontSize: 13,
              fontWeight: 700,
              letterSpacing: 1,
              borderRadius: 4,
              fontFamily: 'Cinzel, serif',
            }}
          >
            Recruit Corps
          </button>
          <div style={{ color: '#666', fontSize: 10, marginTop: 4, textAlign: 'center' }}>
            Cost: 10,000 manpower + 500 gold
          </div>
        </div>
      </div>
    </div>
  )
}

function Section({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div style={{ marginBottom: 18 }}>
      <div
        style={{
          color: '#8a7a5a',
          fontSize: 10,
          fontWeight: 700,
          letterSpacing: 1.5,
          textTransform: 'uppercase',
          marginBottom: 6,
        }}
      >
        {label}
      </div>
      {children}
    </div>
  )
}

function ProgressBar({ pct, color }: { pct: number; color: string }) {
  return (
    <div
      style={{
        width: '100%',
        height: 10,
        background: 'rgba(255,255,255,0.06)',
        borderRadius: 3,
        overflow: 'hidden',
        border: '1px solid rgba(255,255,255,0.08)',
      }}
    >
      <div
        style={{
          width: `${Math.min(100, Math.max(0, pct))}%`,
          height: '100%',
          background: color,
          borderRadius: 3,
          transition: 'width 0.3s ease',
        }}
      />
    </div>
  )
}
