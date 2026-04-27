import React from 'react'
import type { Order } from '../hooks/useOrders'

const ORDER_META: Record<Order['type'], { icon: string; color: string; bg: string }> = {
  Attack:    { icon: '⚔️',  color: '#ff4444', bg: 'rgba(255,68,68,0.15)' },
  Move:      { icon: '🚶',  color: '#4488ff', bg: 'rgba(68,136,255,0.15)' },
  Hold:      { icon: '🛡️',  color: '#44cc44', bg: 'rgba(68,204,68,0.15)' },
  Fortify:   { icon: '🏰',  color: '#999',    bg: 'rgba(153,153,153,0.15)' },
  Diplomacy: { icon: '🤝',  color: '#bb66ff', bg: 'rgba(187,102,255,0.15)' },
}

interface OrderPanelProps {
  orders: Order[]
  onRemoveOrder: (id: string) => void
  onExecuteOrders: () => void
  onClearOrders: () => void
}

function fmtTerritory(id: string) {
  return id.replace(/^AREA_/, '').replace(/_/g, ' ').replace(/\b\w/g, c => c.toUpperCase())
}

export default function OrderPanel({ orders, onRemoveOrder, onExecuteOrders, onClearOrders }: OrderPanelProps) {
  const visibleOrders = orders.slice(-5)
  return (
    <div style={{
      position: 'absolute',
      bottom: 60,
      left: '50%',
      transform: 'translateX(-50%)',
      width: 320,
      background: 'rgba(10,8,18,0.95)',
      border: '1px solid #5b4527',
      borderRadius: 4,
      boxShadow: '0 8px 32px rgba(0,0,0,0.7)',
      fontFamily: 'Cinzel, serif',
      zIndex: 60,
      overflow: 'hidden',
    }}>
      {/* Header */}
      <div style={{
        display: 'flex', alignItems: 'center', justifyContent: 'space-between',
        padding: '8px 12px',
        background: 'linear-gradient(180deg, rgba(40,30,10,0.9), rgba(20,15,5,0.9))',
        borderBottom: '1px solid #3a2f1a',
      }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
          <span style={{ color: '#d4af37', fontSize: 13, fontWeight: 700, letterSpacing: 1.5 }}>ORDERS</span>
          {orders.length > 0 && (
            <span style={{
              background: 'rgba(212,175,55,0.2)', border: '1px solid #d4af37',
              color: '#d4af37', fontSize: 10, fontWeight: 700,
              padding: '1px 7px', borderRadius: 10, letterSpacing: 0.5,
            }}>
              {orders.length} queued
            </span>
          )}
        </div>
        {orders.length > 0 && (
          <button onClick={onClearOrders} style={{
            background: 'none', border: 'none', color: '#7a6030', cursor: 'pointer',
            fontSize: 10, fontFamily: 'Cinzel, serif', letterSpacing: 1,
          }}>
            CLEAR
          </button>
        )}
      </div>

      {/* Order list */}
      <div style={{ maxHeight: 200, overflowY: 'auto', padding: '4px 0' }}>
        {orders.length === 0 ? (
          <div style={{ color: '#5a4820', fontSize: 12, textAlign: 'center', padding: '16px 12px', lineHeight: 1.5 }}>
            Select a territory to issue orders.
          </div>
        ) : (
          visibleOrders.map(order => {
            const meta = ORDER_META[order.type]
            return (
              <div key={order.id} style={{
                display: 'flex', alignItems: 'center', gap: 8,
                padding: '6px 12px',
                borderBottom: '1px solid rgba(90,69,36,0.3)',
                transition: 'background 0.15s',
              }}>
                <span style={{ fontSize: 14, flexShrink: 0 }}>{meta.icon}</span>
                <div style={{ flex: 1, minWidth: 0 }}>
                  <div style={{ color: '#e8dcc8', fontSize: 11, fontWeight: 700, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                    {fmtTerritory(order.fromTerritory)}
                    {order.toTerritory && <span style={{ color: '#7a6030' }}> → </span>}
                    {order.toTerritory && fmtTerritory(order.toTerritory)}
                  </div>
                </div>
                <span style={{
                  background: meta.bg, color: meta.color,
                  fontSize: 9, fontWeight: 700, letterSpacing: 0.8,
                  padding: '2px 6px', borderRadius: 2, flexShrink: 0,
                }}>
                  {order.type.toUpperCase()}
                </span>
                <button onClick={() => onRemoveOrder(order.id)} style={{
                  background: 'none', border: 'none', color: '#7a4030',
                  cursor: 'pointer', fontSize: 14, padding: '0 2px', flexShrink: 0,
                  lineHeight: 1,
                }}>
                  ✕
                </button>
              </div>
            )
          })
        )}
        {orders.length > 5 && (
          <div style={{ color: '#5a4820', fontSize: 10, textAlign: 'center', padding: '4px' }}>
            +{orders.length - 5} more orders
          </div>
        )}
      </div>

      {/* Execute button */}
      {orders.length > 0 && (
        <div style={{ padding: '8px 12px', borderTop: '1px solid #3a2f1a' }}>
          <button onClick={onExecuteOrders} style={{
            width: '100%',
            background: 'linear-gradient(180deg, #b8860b, #8b6508)',
            color: '#fff8e1',
            border: '1px solid #d4af37',
            borderRadius: 3,
            padding: '10px 0',
            cursor: 'pointer',
            fontFamily: 'Cinzel, serif',
            fontSize: 13,
            fontWeight: 700,
            letterSpacing: 2,
            boxShadow: '0 2px 8px rgba(212,175,55,0.3)',
            transition: 'all 0.15s',
          }}>
            EXECUTE ORDERS
          </button>
        </div>
      )}
    </div>
  )
}
