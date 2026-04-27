import React from 'react'

const POWER_COLORS: Record<string, string> = {
  FRA: '#1565C0', GBR: '#B71C1C', AUS: '#F9A825', PRU: '#455A64',
  RUS: '#2E7D32', OTT: '#6A1B9A', SPA: '#E65100',
}

// Approximate capital positions on the minimap (normalized 0-1)
const CAPITALS: { power: string; x: number; y: number }[] = [
  { power: 'FRA', x: 0.30, y: 0.52 },  // Paris
  { power: 'GBR', x: 0.26, y: 0.32 },  // London
  { power: 'AUS', x: 0.52, y: 0.50 },  // Vienna
  { power: 'PRU', x: 0.50, y: 0.34 },  // Berlin
  { power: 'RUS', x: 0.72, y: 0.22 },  // St Petersburg
  { power: 'OTT', x: 0.68, y: 0.68 },  // Constantinople
  { power: 'SPA', x: 0.18, y: 0.68 },  // Madrid
]

// Simplified Europe outline points (normalized 0-1)
const EUROPE_OUTLINE = `
  M0.10,0.30 L0.15,0.20 L0.22,0.15 L0.30,0.12 L0.38,0.10 L0.48,0.08
  L0.55,0.10 L0.62,0.08 L0.72,0.10 L0.80,0.12 L0.88,0.15 L0.92,0.20
  L0.95,0.28 L0.93,0.35 L0.90,0.42 L0.88,0.50 L0.85,0.55 L0.80,0.60
  L0.75,0.65 L0.70,0.70 L0.65,0.73 L0.58,0.75 L0.52,0.73 L0.48,0.70
  L0.42,0.72 L0.38,0.75 L0.32,0.78 L0.25,0.80 L0.20,0.78 L0.15,0.75
  L0.12,0.70 L0.10,0.65 L0.08,0.58 L0.06,0.50 L0.05,0.42 L0.07,0.35 Z
`

interface Props {
  /** Pan offset in pixels */
  panX: number
  panY: number
  /** Current zoom level */
  zoom: number
  /** Map SVG dimensions */
  mapWidth: number
  mapHeight: number
  /** Viewport dimensions (container) */
  viewportWidth: number
  viewportHeight: number
  /** Called when user clicks minimap to navigate */
  onNavigate: (panX: number, panY: number) => void
}

const W = 200
const H = 120

export default function MiniMap({ panX, panY, zoom, mapWidth, mapHeight, viewportWidth, viewportHeight, onNavigate }: Props) {
  // Compute viewport rectangle on the minimap
  // The map SVG viewBox is 0..mapWidth x 0..mapHeight, displayed with transform translate(pan) scale(zoom)
  // Visible region in SVG coords: x from -panX/zoom to (-panX + viewportWidth)/zoom
  const visX = -panX / zoom
  const visY = -panY / zoom
  const visW = viewportWidth / zoom
  const visH = viewportHeight / zoom

  // Map those SVG coords to minimap coords
  const rx = (visX / mapWidth) * W
  const ry = (visY / mapHeight) * H
  const rw = (visW / mapWidth) * W
  const rh = (visH / mapHeight) * H

  const handleClick = (e: React.MouseEvent<SVGSVGElement>) => {
    const rect = e.currentTarget.getBoundingClientRect()
    const mx = e.clientX - rect.left
    const my = e.clientY - rect.top

    // Convert minimap coords to SVG coords, then to pan
    const svgX = (mx / W) * mapWidth
    const svgY = (my / H) * mapHeight
    const newPanX = -(svgX * zoom - viewportWidth / 2)
    const newPanY = -(svgY * zoom - viewportHeight / 2)
    onNavigate(newPanX, newPanY)
  }

  // Scale the outline path
  const scaledOutline = EUROPE_OUTLINE.replace(/(\d+\.\d+)/g, (_, num) => num)

  return (
    <div style={{
      position: 'fixed',
      bottom: 68,
      right: 12,
      width: W,
      height: H,
      background: 'rgba(10,8,18,0.92)',
      border: '1px solid #3a2f1a',
      borderRadius: 4,
      boxShadow: '0 4px 16px rgba(0,0,0,0.5)',
      zIndex: 150,
      overflow: 'hidden',
      cursor: 'crosshair',
    }}>
      <svg
        width={W}
        height={H}
        viewBox={`0 0 1 1`}
        preserveAspectRatio="none"
        onClick={handleClick}
        style={{ display: 'block' }}
      >
        {/* Background */}
        <rect width="1" height="1" fill="#0d1526" />

        {/* Simplified Europe outline */}
        <path d={EUROPE_OUTLINE} fill="#1a1a2e" stroke="#3a3a5e" strokeWidth="0.005" />

        {/* Capital dots */}
        {CAPITALS.map(cap => (
          <circle
            key={cap.power}
            cx={cap.x}
            cy={cap.y}
            r={0.018}
            fill={POWER_COLORS[cap.power]}
            stroke="#fff"
            strokeWidth="0.004"
          />
        ))}

        {/* Viewport rectangle */}
        <rect
          x={rx / W}
          y={ry / H}
          width={Math.max(rw / W, 0.05)}
          height={Math.max(rh / H, 0.05)}
          fill="none"
          stroke="#ff4444"
          strokeWidth="0.008"
          opacity={0.9}
        />
      </svg>

      {/* Label */}
      <div style={{
        position: 'absolute',
        top: 2,
        left: 4,
        color: '#5a4820',
        fontSize: 8,
        fontFamily: 'Cinzel, serif',
        letterSpacing: 1,
        pointerEvents: 'none',
      }}>
        MINIMAP
      </div>
    </div>
  )
}
