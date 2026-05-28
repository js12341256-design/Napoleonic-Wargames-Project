import React from 'react'
import type { AreaCoords } from '../MapData'

interface AreaPolygonProps {
  areaId: string
  coords: AreaCoords
  ownerColor: string
  isSelected: boolean
  isHovered: boolean
  onClick: () => void
  onMouseEnter: () => void
  onMouseLeave: () => void
}

function lighten(hex: string, amount: number) {
  const normalized = hex.replace('#', '')
  if (normalized.length !== 6) return hex

  const value = Number.parseInt(normalized, 16)
  const r = Math.min(255, Math.max(0, (value >> 16) + amount))
  const g = Math.min(255, Math.max(0, ((value >> 8) & 0xff) + amount))
  const b = Math.min(255, Math.max(0, (value & 0xff) + amount))

  return `#${[r, g, b].map(part => part.toString(16).padStart(2, '0')).join('')}`
}

// Omniatlas-style: bright yellow selection, dark thin borders

export default function AreaPolygon({
  areaId,
  coords,
  ownerColor,
  isSelected,
  isHovered,
  onClick,
  onMouseEnter,
  onMouseLeave,
}: AreaPolygonProps) {
  const fill = isHovered ? lighten(ownerColor, 24) : ownerColor
  const points = coords.polygon.map(([x, y]) => `${x},${y}`).join(' ')

  return (
    <g>
      <polygon
        data-area-id={areaId}
        points={points}
        fill={fill}
        stroke={isSelected ? '#FFD700' : '#2a2a2a'}
        strokeWidth={isSelected ? 2 : 0.5}
        strokeOpacity={isSelected ? 1 : 0.7}
        style={{ cursor: 'pointer', transition: 'fill 100ms ease, stroke 100ms ease' }}
        onClick={onClick}
        onMouseEnter={onMouseEnter}
        onMouseLeave={onMouseLeave}
      />
      {isSelected && (
        <polygon
          points={points}
          fill="rgba(255,215,0,0.15)"
          stroke="none"
          style={{ pointerEvents: 'none' }}
        />
      )}
    </g>
  )
}
