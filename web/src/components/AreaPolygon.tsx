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
        stroke={isSelected ? '#f7e7a1' : '#5a4a3a'}
        strokeWidth={isSelected ? 4 : 1.5}
        style={{ cursor: 'pointer', transition: 'fill 120ms ease, stroke 120ms ease' }}
        onClick={onClick}
        onMouseEnter={onMouseEnter}
        onMouseLeave={onMouseLeave}
      />
      <polygon
        points={points}
        fill="none"
        stroke={isSelected ? 'rgba(255, 248, 200, 0.95)' : 'rgba(255,255,255,0.08)'}
        strokeWidth={isSelected ? 1.5 : 0.5}
        style={{ pointerEvents: 'none' }}
      />
    </g>
  )
}
