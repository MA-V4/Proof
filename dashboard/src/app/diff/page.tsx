'use client'

import { useState } from 'react'
import { api, DiffItem } from '@/lib/api'

const OLD_EXAMPLE = `product SavingsAccount {
  jurisdiction: UK
  regulator:    FCA
  category:     deposit

  interest {
    base_rate: 4.50%

    tiers {
      when balance >= 10_000  rate: base_rate + 1.00%
      when balance >= 1_000   rate: base_rate + 0.50%
      otherwise               rate: base_rate
    }

    accrual {
      frequency:  daily
      basis:      ACT/365
      compound:   annually
    }
  }

  protection {
    scheme: FSCS
    limit:  GBP 85_000
  }

  obligations {
    cooling_off:          14 days
    rate_change_notice:   14 days
    annual_summary:       required
  }
}`

const NEW_EXAMPLE = `product SavingsAccount {
  jurisdiction: UK
  regulator:    FCA
  category:     deposit

  interest {
    base_rate: 4.50%

    tiers {
      when balance >= 10_000  rate: base_rate + 1.00%
      when balance >= 2_500   rate: base_rate + 0.50%
      otherwise               rate: base_rate
    }

    accrual {
      frequency:  daily
      basis:      ACT/365
      compound:   annually
    }
  }

  protection {
    scheme: FSCS
    limit:  GBP 85_000
  }

  obligations {
    cooling_off:          14 days
    rate_change_notice:   14 days
    annual_summary:       required
  }
}`

function diffLabel(item: DiffItem): { label: string; old?: string; new?: string } {
  switch (item.type) {
    case 'base_rate_changed':
      return { label: 'Base rate changed', old: `${item.old}%`, new: `${item.new}%` }
    case 'tier_threshold_changed':
      return { label: `Tier ${item.tier_index + 1} threshold`, old: `£${item.old_threshold}`, new: `£${item.new_threshold}` }
    case 'tier_rate_changed':
      return { label: `Tier ${item.tier_index + 1} rate`, old: item.old_rate, new: item.new_rate }
    case 'tier_added':
      return { label: `Tier ${item.tier_index + 1} added` }
    case 'tier_removed':
      return { label: `Tier ${item.tier_index + 1} removed` }
    case 'promotional_rate_changed':
      return { label: 'Promotional rate', old: item.old, new: item.new }
    case 'obligation_changed':
      return { label: `Obligation: ${item.field}`, old: item.old, new: item.new }
    default:
      return { label: 'Unknown change' }
  }
}

export default function DiffPage() {
  const [oldText, setOldText] = useState(OLD_EXAMPLE)
  const [newText, setNewText] = useState(NEW_EXAMPLE)
  const [items,   setItems]   = useState<DiffItem[] | null>(null)
  const [loading, setLoading] = useState(false)
  const [error,   setError]   = useState<string | null>(null)

  async function runDiff() {
    setLoading(true); setError(null); setItems(null)
    try {
      const result = await api.diff({ old_spec_text: oldText, new_spec_text: newText })
      setItems(result)
    } catch (e: any) {
      setError(e.message)
    } finally {
      setLoading(false)
    }
  }

  return (
    <div>
      <h1 className="text-sm font-medium text-gray-500 mb-4">Specification diff</h1>

      <div className="grid grid-cols-2 gap-3 mb-3">
        <div>
          <label className="text-xs text-gray-500 block mb-1">Old spec</label>
          <textarea
            value={oldText}
            onChange={e => setOldText(e.target.value)}
            className="w-full h-64 text-xs border border-gray-200 rounded-lg px-3 py-2 font-mono resize-none"
          />
        </div>
        <div>
          <label className="text-xs text-gray-500 block mb-1">New spec</label>
          <textarea
            value={newText}
            onChange={e => setNewText(e.target.value)}
            className="w-full h-64 text-xs border border-gray-200 rounded-lg px-3 py-2 font-mono resize-none"
          />
        </div>
      </div>

      <button
        onClick={runDiff}
        disabled={loading}
        className="w-full text-sm bg-gray-900 text-white rounded-lg px-4 py-2 hover:bg-gray-700 disabled:opacity-50 transition-colors mb-4"
      >
        {loading ? 'Diffing...' : 'Run diff'}
      </button>

      {error && (
        <div className="bg-red-50 border border-red-200 rounded-lg px-4 py-3 mb-4 text-sm text-red-700">
          {error}
        </div>
      )}

      {items !== null && items.length === 0 && (
        <div className="bg-green-50 border border-green-200 rounded-lg px-4 py-3 text-sm text-green-700">
          No differences — specs are functionally identical.
        </div>
      )}

      {items && items.length > 0 && (
        <div className="bg-white border border-gray-200 rounded-xl divide-y divide-gray-100">
          {items.map((item, i) => {
            const { label, old, new: n } = diffLabel(item)
            return (
              <div key={i} className="px-4 py-3 flex items-center gap-4">
                <span className="w-2 h-2 rounded-full bg-yellow-400 flex-shrink-0" />
                <span className="text-sm text-gray-700 flex-1">{label}</span>
                {old && n && (
                  <div className="flex items-center gap-2 text-xs font-mono">
                    <span className="text-red-600 bg-red-50 px-2 py-0.5 rounded">{old}</span>
                    <span className="text-gray-400">→</span>
                    <span className="text-green-600 bg-green-50 px-2 py-0.5 rounded">{n}</span>
                  </div>
                )}
              </div>
            )
          })}
        </div>
      )}
    </div>
  )
}