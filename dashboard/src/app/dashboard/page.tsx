'use client'

import useSWR from 'swr'
import { useState } from 'react'
import { api, Divergence } from '@/lib/api'

function MetricCard({ label, value, sub, danger }: {
  label: string; value: string | number; sub: string; danger?: boolean
}) {
  return (
    <div className="bg-white border border-slate-200 rounded-lg p-4 shadow-sm">
      <div className="text-xs font-semibold text-slate-400 uppercase tracking-widest mb-2">
        {label}
      </div>
      <div className={`text-3xl font-semibold tabular ${danger ? 'text-red-600' : 'text-slate-900'}`}>
        {value}
      </div>
      <div className="text-xs text-slate-400 mt-1">{sub}</div>
    </div>
  )
}

export default function DashboardPage() {
  const { data: health }                   = useSWR('health',  api.health,          { refreshInterval: 5000 })
  const { data: events }                   = useSWR('recent',  api.recentEvents,    { refreshInterval: 3000 })
  const { data: divs, mutate: mutateDivs } = useSWR('allDivs', api.allDivergences, { refreshInterval: 3000 })
  const [selected, setSelected]            = useState<Divergence | null>(null)

  const activeDivs    = divs    ?? []
  const recentEvents  = events  ?? []

  async function handleResolve(d: Divergence) {
    await api.resolve(d.spec_name, d.id)
    mutateDivs()
    if (selected?.id === d.id) setSelected(null)
  }

  return (
    <div className="space-y-5">
      {activeDivs.length > 0 && (
        <div className="bg-red-50 border border-red-200 rounded-lg px-4 py-3 flex items-start gap-3">
          <div className="mt-0.5 text-red-500">
            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
              <path d="M8 1a7 7 0 100 14A7 7 0 008 1zM7.25 4.75a.75.75 0 011.5 0v3.5a.75.75 0 01-1.5 0v-3.5zm.75 7a.75.75 0 110-1.5.75.75 0 010 1.5z"/>
            </svg>
          </div>
          <div>
            <p className="text-sm font-semibold text-red-700">
              {activeDivs.length} divergence{activeDivs.length !== 1 ? 's' : ''} detected
            </p>
            <p className="text-xs text-red-500 mt-0.5">
              {[...new Set(activeDivs.map(d => d.spec_name))].join(', ')}
            </p>
          </div>
        </div>
      )}

      <div className="grid grid-cols-4 gap-3">
        <MetricCard
          label="Events verified"
          value={(health?.events_verified ?? 0).toLocaleString()}
          sub="total"
        />
        <MetricCard
          label="Divergences"
          value={health?.divergences ?? 0}
          sub="active"
          danger={(health?.divergences ?? 0) > 0}
        />
        <MetricCard
          label="Specifications"
          value={health?.specs ?? 0}
          sub="loaded"
        />
        <MetricCard
          label="Server"
          value={health?.status === 'ok' ? 'Online' : 'Offline'}
          sub="status"
        />
      </div>

      <div className="grid grid-cols-2 gap-4">
        <div className="bg-white border border-slate-200 rounded-lg shadow-sm overflow-hidden">
          <div className="px-4 py-3 border-b border-slate-100">
            <h2 className="text-xs font-semibold text-slate-500 uppercase tracking-widest">
              Verification feed
            </h2>
          </div>
          <div>
            {recentEvents.length === 0 ? (
              <p className="text-sm text-slate-400 px-4 py-6 text-center">
                No events yet. Run a verify command.
              </p>
            ) : (
              recentEvents.map((e, i) => {
                const div = activeDivs.find(d =>
                  d.customer_id === e.customer_id && d.spec_name === e.spec_name
                )
                return (
                  <div
                    key={i}
                    onClick={() => div && setSelected(div)}
                    className={[
                      'flex items-center gap-3 px-4 py-2.5 border-b border-slate-50 last:border-0 transition-colors',
                      div ? 'cursor-pointer hover:bg-slate-50' : '',
                      selected?.id === div?.id ? 'bg-slate-50' : '',
                    ].join(' ')}
                  >
                    <span className={[
                      'w-5 h-5 rounded-full flex items-center justify-center text-xs flex-shrink-0 font-semibold',
                      e.ok
                        ? 'bg-emerald-50 text-emerald-600'
                        : 'bg-red-50 text-red-600',
                    ].join(' ')}>
                      {e.ok ? '✓' : '✗'}
                    </span>
                    <span className="flex-1 text-sm text-slate-700 truncate">
                      <span className="font-medium">{e.spec_name}</span>
                      <span className="text-slate-400 mx-1.5">·</span>
                      <span className="font-mono text-xs">{e.event_type}</span>
                    </span>
                    <span className="text-xs font-mono text-slate-400 flex-shrink-0">
                      {e.customer_id}
                    </span>
                  </div>
                )
              })
            )}
          </div>
        </div>

        <div className="bg-white border border-slate-200 rounded-lg shadow-sm overflow-hidden">
          <div className="px-4 py-3 border-b border-slate-100">
            <h2 className="text-xs font-semibold text-slate-500 uppercase tracking-widest">
              {selected ? 'Divergence detail' : 'Select a divergence'}
            </h2>
          </div>
          <div className="p-4">
            {!selected && activeDivs.length > 0 && (
              <div className="space-y-2">
                {activeDivs.map(d => (
                  <button
                    key={d.id}
                    onClick={() => setSelected(d)}
                    className="w-full flex items-center gap-3 p-3 rounded-md border border-red-100 bg-red-50 hover:bg-red-100 transition-colors text-left"
                  >
                    <span className="w-2 h-2 rounded-full bg-red-500 flex-shrink-0" />
                    <span className="text-sm font-semibold text-red-700">{d.spec_name}</span>
                    <span className="text-xs text-slate-500">{d.customer_id}</span>
                    <span className="text-xs font-mono text-slate-400 ml-auto">£{d.balance}</span>
                  </button>
                ))}
              </div>
            )}

            {!selected && activeDivs.length === 0 && (
              <p className="text-sm text-slate-400 text-center py-6">
                No active divergences.
              </p>
            )}

            {selected && (
              <div className="space-y-4">
                <div className="flex items-center gap-2">
                  <span className="text-sm font-semibold text-slate-900">
                    {selected.spec_name}
                  </span>
                  <span className="text-xs px-2 py-0.5 rounded-full bg-red-50 text-red-600 font-medium border border-red-100">
                    Divergence
                  </span>
                </div>

                <div>
                  <div className="text-xs font-semibold text-slate-400 uppercase tracking-widest mb-2">
                    Context
                  </div>
                  {[
                    ['customer', selected.customer_id],
                    ['balance',  `£${selected.balance}`],
                    ['event',    selected.event_type],
                  ].map(([k, v]) => (
                    <div key={k} className="flex justify-between py-1.5 border-b border-slate-50 last:border-0">
                      <span className="text-xs font-mono text-slate-400">{k}</span>
                      <span className="text-xs font-medium text-slate-900">{v}</span>
                    </div>
                  ))}
                </div>

                <div>
                  <div className="text-xs font-semibold text-slate-400 uppercase tracking-widest mb-2">
                    Field mismatches
                  </div>
                  {selected.diffs.map(d => (
                    <div key={d.field} className="flex items-center justify-between py-1.5 border-b border-slate-50 last:border-0">
                      <span className="text-xs font-mono text-slate-400">{d.field}</span>
                      <div className="flex items-center gap-2 text-xs">
                        <span className="text-emerald-600 font-medium">{d.spec_value}</span>
                        <span className="text-slate-300">→</span>
                        <span className="text-red-600 font-medium">{d.system_value}</span>
                        {d.delta && (
                          <span className="text-red-500 font-semibold tabular">{d.delta}</span>
                        )}
                      </div>
                    </div>
                  ))}
                </div>

                <div className="flex gap-2 pt-1">
                  <button
                    onClick={() => handleResolve(selected)}
                    className="text-xs text-slate-500 hover:text-slate-900 border border-slate-200 rounded-md px-3 py-1.5 transition-colors hover:bg-slate-50"
                  >
                    Mark resolved
                  </button>
                  <button
                    onClick={() => setSelected(null)}
                    className="text-xs text-slate-400 hover:text-slate-600 px-3 py-1.5 transition-colors"
                  >
                    Back
                  </button>
                </div>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  )
}