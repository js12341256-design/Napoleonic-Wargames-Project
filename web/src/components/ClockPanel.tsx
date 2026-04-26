import React, { useEffect, useRef } from 'react'

interface ClockPanelProps {
  date: string
  speed: number // 0-5
  paused: boolean
  onSetSpeed: (s: number) => void
  onTogglePause: () => void
  onTick: () => void
}

const SPEED_INTERVALS: Record<number, number> = {
  1: 1000,
  2: 500,
  3: 250,
  4: 100,
  5: 50,
}

const SPEED_ICONS = ['▶', '▶▶', '▶▶▶', '▶▶▶▶', '▶▶▶▶▶']

export default function ClockPanel({
  date,
  speed,
  paused,
  onSetSpeed,
  onTogglePause,
  onTick,
}: ClockPanelProps) {
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null)

  useEffect(() => {
    if (intervalRef.current) {
      clearInterval(intervalRef.current)
      intervalRef.current = null
    }
    if (!paused && speed >= 1 && speed <= 5) {
      intervalRef.current = setInterval(onTick, SPEED_INTERVALS[speed])
    }
    return () => {
      if (intervalRef.current) clearInterval(intervalRef.current)
    }
  }, [paused, speed, onTick])

  return (
    <div
      style={{
        position: 'absolute',
        top: 56,
        left: '50%',
        transform: 'translateX(-50%)',
        zIndex: 100,
        display: 'flex',
        alignItems: 'center',
        gap: 6,
        background: 'linear-gradient(180deg,#1a1a2e,#111125)',
        border: '1px solid #5a4524',
        borderTop: 'none',
        borderRadius: '0 0 8px 8px',
        padding: '6px 16px',
        boxShadow: '0 4px 16px rgba(0,0,0,0.6)',
        fontFamily: 'Cinzel, serif',
      }}
    >
      {/* date */}
      <span
        style={{
          color: '#d4af37',
          fontSize: 15,
          fontWeight: 700,
          letterSpacing: 1.2,
          marginRight: 12,
          whiteSpace: 'nowrap',
        }}
      >
        {date}
      </span>

      {/* pause button */}
      <button
        onClick={onTogglePause}
        style={{
          background: paused ? 'rgba(180,60,60,0.35)' : 'rgba(60,60,180,0.2)',
          border: `1px solid ${paused ? '#a04040' : '#4a4a8a'}`,
          color: paused ? '#ff6666' : '#8888cc',
          cursor: 'pointer',
          padding: '3px 8px',
          fontSize: 13,
          borderRadius: 3,
          fontFamily: 'monospace',
        }}
      >
        ▐▐
      </button>

      {/* speed buttons */}
      {SPEED_ICONS.map((icon, i) => {
        const s = i + 1
        const active = !paused && speed === s
        return (
          <button
            key={s}
            onClick={() => {
              onSetSpeed(s)
              if (paused) onTogglePause()
            }}
            style={{
              background: active
                ? 'rgba(60,120,255,0.35)'
                : 'rgba(40,40,60,0.5)',
              border: `1px solid ${active ? '#5588ff' : '#3a3a5a'}`,
              color: active ? '#88bbff' : '#666688',
              cursor: 'pointer',
              padding: '3px 6px',
              fontSize: 11,
              borderRadius: 3,
              fontFamily: 'monospace',
              textShadow: active ? '0 0 6px #5588ff' : 'none',
              boxShadow: active ? '0 0 8px rgba(85,136,255,0.4)' : 'none',
            }}
          >
            {icon}
          </button>
        )
      })}
    </div>
  )
}
