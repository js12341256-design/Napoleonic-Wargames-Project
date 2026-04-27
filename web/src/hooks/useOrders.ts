import { useState, useCallback } from 'react'

export interface Order {
  id: string
  type: 'Attack' | 'Move' | 'Hold' | 'Fortify' | 'Diplomacy'
  fromTerritory: string
  toTerritory?: string
  corpsId?: number
  marshalId?: number
}

let _nextId = 1
function genId() { return `order-${_nextId++}` }

export function useOrders() {
  const [orders, setOrders] = useState<Order[]>([])

  const addOrder = useCallback((order: Omit<Order, 'id'>) => {
    setOrders(prev => [...prev, { ...order, id: genId() }])
  }, [])

  const removeOrder = useCallback((id: string) => {
    setOrders(prev => prev.filter(o => o.id !== id))
  }, [])

  const clearOrders = useCallback(() => {
    setOrders([])
  }, [])

  const executeOrders = useCallback(() => {
    const summary = orders.map(o => {
      const dest = o.toTerritory ? ` → ${o.toTerritory}` : ''
      console.log(`[ORDER] ${o.type}: ${o.fromTerritory}${dest}`)
      return `${o.type}: ${o.fromTerritory}${dest}`
    })
    setOrders([])
    return summary
  }, [orders])

  return { orders, addOrder, removeOrder, clearOrders, executeOrders }
}
