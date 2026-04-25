import React, { useState } from 'react'

// WASM module will be loaded dynamically when wasm-pack output is available
// For now, scaffold the React shell

interface GameState {
  turn: number
  powers: string[]
  status: 'idle' | 'loading' | 'playing'
}

export default function App() {
  const [game, setGame] = useState<GameState>({
    turn: 0,
    powers: [],
    status: 'idle',
  })

  return (
    <div
      style={{
        fontFamily: 'monospace',
        background: '#1a1a2e',
        color: '#eee',
        minHeight: '100vh',
        padding: '2rem',
      }}
    >
      <h1>Grand Campaign 1805</h1>
      <p>Turn: {game.turn}</p>
      <p>Status: {game.status}</p>
      {game.status === 'idle' && (
        <button
          onClick={() => setGame((g) => ({ ...g, status: 'loading' }))}
          style={{ padding: '0.5rem 1rem', cursor: 'pointer' }}
        >
          Load Scenario
        </button>
      )}
      {game.powers.length > 0 && (
        <div>
          <h2>Powers</h2>
          <ul>{game.powers.map((power) => <li key={power}>{power}</li>)}</ul>
        </div>
      )}
      <p style={{ color: '#888', fontSize: '0.8rem' }}>
        Phase 15 skeleton — full PixiJS map rendering in Phase 19
      </p>
    </div>
  )
}
