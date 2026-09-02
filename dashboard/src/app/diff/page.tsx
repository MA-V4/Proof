'use client'

import { useState } from 'react'
import { api, DiffItem } from '@/lib/api'

const OLD_DEFAULT = `product SavingsAccount {
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

const NEW_DEFAULT = `product SavingsAccount {
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

function diffLabel(item: DiffItem): { label: string; old?: string; new?: string; impact?: string } {
  switch (item.type) {
    case 'base_rate_changed':
      return { label: 'Base rate changed', old: `${item.old}%`, new: `${item.new}%`, impact: `Delta: ${item.delta}%` }
    case 'tier_threshold_changed':
      return {
        label: `Tier ${item.tier_index + 1} threshold`,
        old: `£${item.old_threshold}`,
        new: `£${item.new_threshold}`,
        impact: item.tier_index === 1 ? 'Customers with balances between old and new threshold lose tier rate' : undefined,
      }
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
      return { label: 'Change detected' }
  }
}

export default function DiffPage() {
  const [oldText, setOldText] = useState(OLD_DEFAULT)
  const [newText, setNewText] = useState(NEW_DEFAULT)
  const [items,   setItems]   = useState<DiffItem[] | null>(null)
  const [loading, setLoading] = useState(false)
  const [error,   setError]   = useState<string | null>(null)

  async function runDiff() {
    setLoading(true); setError(null); setItems(null)
    try {
      setItems(await api.diff({ old_spec_text: oldText, new_spec_text: newText }))
    } catch (e: any) {
      setError(e.message)
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="p-8 min-h-full flex flex-col">
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-xl font-bold text-slate-900">Specification diff</h1>
          <p className="text-sm text-slate-500 mt-0.5">Compare two spec versions to identify financial logic changes.</p>
        </div>
        <button
          onClick={runDiff}
          disabled={loading}
          className="flex items-center gap-2 bg-slate-900 hover:bg-slate-800 text-white text-sm font-semibold px-5 py-2.5 rounded-xl transition-colors disabled:opacity-50 shadow-sm"
        >
          {loading ? (
            <>
              <svg className="animate-spin w-4 h-4" viewBox="0 0 24 24" fill="none">
                <circle cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="3" strokeOpacity="0.25"/>
                <path d="M12 2a10 10 0 0110 10" stroke="currentColor" strokeWidth="3" strokeLinecap="round"/>
              </svg>
              Running diff...
            </>
          ) : (
            <>
              <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round">
                <path d="M1 4h5M1 9h5M8 2l4 5-4 5"/>
              </svg>
              Run diff
            </>
          )}
        </button>
      </div>

      {/* Editors */}
      <div className="grid grid-cols-2 gap-4 mb-4">
        {[
          { label: 'Old spec', value: oldText, onChange: setOldText, tag: 'Current' },
          { label: 'New spec', value: newText, onChange: setNewText, tag: 'Proposed' },
        ].map(({ label, value, onChange, tag }) => (
          <div key={label} className="bg-white border border-slate-200 rounded-2xl shadow-sm overflow-hidden">
            <div className="flex items-center justify-between px-4 py-3 border-b border-slate-100">
              <span className="text-sm font-semibold text-slate-900">{label}</span>
              <span className="text-xs font-semibold text-slate-500 bg-slate-100 px-2.5 py-1 rounded-full">{tag}</span>
            </div>
            <textarea
              value={value}
              onChange={e => onChange(e.target.value)}
              className="w-full h-72 px-4 py-3 font-mono text-xs text-slate-700 bg-slate-50 resize-none focus:outline-none focus:bg-white transition-colors leading-relaxed"
              spellCheck={false}
            />
          </div>
        ))}
      </div>

      {/* Error */}
      {error && (
        <div className="bg-red-50 border border-red-200 rounded-xl px-4 py-3 text-sm text-red-700 mb-4">
          {error}
        </div>
      )}

      {/* Results */}
      {items !== null && (
        <div className="bg-white border border-slate-200 rounded-2xl shadow-sm overflow-hidden">
          {items.length === 0 ? (
            <div className="flex items-center gap-3 px-5 py-5">
              <div className="w-8 h-8 bg-emerald-50 rounded-lg flex items-center justify-center flex-shrink-0">
                <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="#10B981" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
                  <path d="M2 7l4 4 6-6"/>
                </svg>
              </div>
              <div>
                <div className="text-sm font-semibold text-slate-900">Specs are identical</div>
                <div className="text-xs text-slate-500">No financial logic differences detected.</div>
              </div>
            </div>
          ) : (
            <>
              <div className="px-5 py-3 border-b border-slate-100 flex items-center justify-between">
                <span className="text-sm font-semibold text-slate-900">
                  {items.length} change{items.length !== 1 ? 's' : ''} detected
                </span>
                <span className="text-xs text-amber-600 font-semibold bg-amber-50 border border-amber-100 px-2.5 py-1 rounded-full">
                  Requires review
                </span>
              </div>
              <div className="divide-y divide-slate-50">
                {items.map((item, i) => {
                  const { label, old: o, new: n, impact } = diffLabel(item)
                  return (
                    <div key={i} className="px-5 py-4 flex items-start gap-4">
                      <div className="w-8 h-8 bg-amber-50 rounded-lg flex items-center justify-center flex-shrink-0 mt-0.5">
                        <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="#F59E0B" strokeWidth="1.5" strokeLinecap="round">
                          <path d="M1 4h5M1 9h5M9 2l4 5-4 5"/>
                        </svg>
                      </div>
                      <div className="flex-1">
                        <div className="text-sm font-semibold text-slate-900 mb-1.5">{label}</div>
                        {o && n && (
                          <div className="flex items-center gap-2 text-sm">
                            <span className="font-mono bg-red-50 text-red-700 border border-red-100 px-2.5 py-1 rounded-lg">{o}</span>
                            <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="#CBD5E1" strokeWidth="1.5" strokeLinecap="round">
                              <path d="M3 8h10M9 4l4 4-4 4"/>
                            </svg>
                            <span className="font-mono bg-emerald-50 text-emerald-700 border border-emerald-100 px-2.5 py-1 rounded-lg">{n}</span>
                          </div>
                        )}
                        {impact && (
                          <p className="text-xs text-slate-500 mt-1.5">{impact}</p>
                        )}
                      </div>
                    </div>
                  )
                })}
              </div>
              <div className="px-5 py-4 border-t border-slate-100 bg-slate-50 flex items-center justify-between">
                <p className="text-xs text-slate-500">
                  Run a simulation to see the customer impact of these changes before deploying.
                </p>
                
                  href="/simulate"
                  className="text-xs font-semibold text-blue-600 hover:text-blue-700 flex items-center gap-1"
                <a>
                  Go to simulator
                  <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round">
                    <path d="M2 6h8M6 2l4 4-4 4"/>
                  </svg>
                </a>
              </div>
            </>
          )}
        </div>
      )}
    </div>
  )
}