import React, { useState, useEffect } from 'react'
import type { GameEvent } from '../types'

const EVENT_PORTRAITS: Record<string, string> = {
  FRA: '\u{1F451}', // crown
  GBR: '\u{1F3F4}', // flag
  RUS: '\u{1F43B}', // bear
  AUS: '\u{1F985}', // eagle
  PRU: '\u{2694}\uFE0F', // swords
}

const POWER_COLORS: Record<string, string> = {
  FRA: '#1565C0',
  GBR: '#B71C1C',
  RUS: '#2E7D32',
  AUS: '#F9A825',
  PRU: '#455A64',
}

interface EventPopupProps {
  events: GameEvent[]
  onResolve: (eventId: number, optionIndex: number) => void
}

export default function EventPopup({ events, onResolve }: EventPopupProps) {
  const [visible, setVisible] = useState(false)
  const [currentIndex, setCurrentIndex] = useState(0)

  useEffect(() => {
    if (events.length > 0) {
      setVisible(false)
      setCurrentIndex(0)
      // Trigger fade-in on next frame
      const raf = requestAnimationFrame(() => setVisible(true))
      return () => cancelAnimationFrame(raf)
    }
    setVisible(false)
  }, [events])

  if (events.length === 0) return null

  const event = events[currentIndex]
  if (!event) return null

  const portrait = EVENT_PORTRAITS[event.firesFor] || '\u{2694}\uFE0F'
  const accentColor = POWER_COLORS[event.firesFor] || '#d4af37'

  const handleOption = (optionIndex: number) => {
    onResolve(event.id, optionIndex)
    if (currentIndex < events.length - 1) {
      setCurrentIndex((i) => i + 1)
    }
  }

  return (
    <div
      style={{
        position: 'fixed',
        inset: 0,
        zIndex: 1000,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        background: 'rgba(0, 0, 0, 0.75)',
        opacity: visible ? 1 : 0,
        pointerEvents: visible ? 'auto' : 'none',
        transition: 'opacity 0.4s ease-in-out',
        fontFamily: 'Cinzel, serif',
      }}
    >
      <div
        style={{
          width: 520,
          maxWidth: '90vw',
          background: 'linear-gradient(180deg, #1a1a2e 0%, #0d0d1a 100%)',
          border: `2px solid ${accentColor}`,
          borderRadius: 6,
          boxShadow: `0 0 40px ${accentColor}33, 0 8px 32px rgba(0,0,0,0.7)`,
          overflow: 'hidden',
          transform: visible ? 'scale(1)' : 'scale(0.95)',
          transition: 'transform 0.4s ease-in-out',
        }}
      >
        {/* Header with counter */}
        {events.length > 1 && (
          <div
            style={{
              padding: '6px 16px',
              background: 'rgba(0,0,0,0.4)',
              color: '#888',
              fontSize: 11,
              letterSpacing: 1.5,
              textAlign: 'right',
              borderBottom: '1px solid #333',
            }}
          >
            {currentIndex + 1} of {events.length}
          </div>
        )}

        {/* Portrait area */}
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            height: 120,
            background: `radial-gradient(ellipse at center, ${accentColor}22 0%, transparent 70%)`,
            fontSize: 64,
            userSelect: 'none',
          }}
        >
          {portrait}
        </div>

        {/* Title */}
        <div
          style={{
            padding: '12px 24px 4px',
            textAlign: 'center',
          }}
        >
          <h2
            style={{
              margin: 0,
              color: '#d4af37',
              fontSize: 20,
              fontWeight: 700,
              letterSpacing: 1.5,
              textShadow: '0 2px 8px rgba(212,175,55,0.3)',
            }}
          >
            {event.title}
          </h2>
        </div>

        {/* Description */}
        <div
          style={{
            padding: '8px 24px 16px',
            textAlign: 'center',
          }}
        >
          <p
            style={{
              margin: 0,
              color: '#c8bfa8',
              fontSize: 13,
              fontStyle: 'italic',
              lineHeight: 1.6,
              fontFamily: 'Georgia, serif',
            }}
          >
            {event.description}
          </p>
        </div>

        {/* Divider */}
        <div
          style={{
            height: 1,
            margin: '0 24px',
            background: `linear-gradient(90deg, transparent, ${accentColor}66, transparent)`,
          }}
        />

        {/* Options */}
        <div style={{ padding: '16px 24px 20px' }}>
          {event.options.map((option, idx) => (
            <button
              key={idx}
              onClick={() => handleOption(idx)}
              style={{
                display: 'block',
                width: '100%',
                padding: '10px 16px',
                marginBottom: idx < event.options.length - 1 ? 8 : 0,
                background: 'rgba(30, 25, 50, 0.8)',
                border: '1px solid #5a4524',
                borderRadius: 4,
                cursor: 'pointer',
                textAlign: 'left',
                fontFamily: 'Cinzel, serif',
                transition: 'all 0.15s ease',
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.background = 'rgba(212, 175, 55, 0.15)'
                e.currentTarget.style.borderColor = '#d4af37'
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.background = 'rgba(30, 25, 50, 0.8)'
                e.currentTarget.style.borderColor = '#5a4524'
              }}
            >
              <div
                style={{
                  color: '#e8dcc8',
                  fontSize: 14,
                  fontWeight: 700,
                  marginBottom: 4,
                }}
              >
                {option.label}
              </div>
              <div
                style={{
                  color: '#8a7a60',
                  fontSize: 11,
                  fontStyle: 'italic',
                }}
              >
                {option.effects.join(' \u2022 ')}
              </div>
            </button>
          ))}
        </div>
      </div>
    </div>
  )
}
