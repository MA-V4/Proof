'use client'
import useSWR from 'swr'
import { useState } from 'react'
import { api, Divergence, RecentEvent } from '@/lib/api'

export default function DashboardPage() {
  const { data: health }  = useSWR('health',  api.health,       { refreshInterval: 5000 })
  const { data: events, mutate: mutateEvents }  = useSWR('recent', api.recentEvents, { refreshInterval: 3000 })
  const { data: divs, mutate: mutateDivs }    = useSWR('allDivs', api.allDivergences, { refreshInterval: 3000 })
  const [selected, setSelected] = useState<Divergence | null>(null)

  const activeDivs = divs ?? []
  const recentEvents = events ?? []

  async function handleResolve(d: Divergence) {
    await api.resolve(d.spec_name, d.id)
    mutateDivs()
    if (selected?.id === d.id) setSelected(null)
  }

  return (
    <div>
      {/* Alert banner */}
      {activeDivs.length > 0 && (
        <div className="bg-red-50 border border-red-200 rounded-lg px-4 py-3 mb-4 flex items-start gap-3">
          <span className="text-red-600 text-base mt-0.5">⚠</span>
          <div>
            <p className="text-sm font-medium text-red-700">
              {activeDivs.length} divergence{activeDivs.length !== 1 ? 's' : ''} detected
            </p>
            <p className="text-xs text-gray-500 mt-0.5">
              {[...new Set(activeDivs.map(d => d.spec_name))].join(', ')}
            </p>
          </div>
        </div>
      )}

      {/* Metric cards */}
      <div className="grid grid-cols-4 gap-3 mb-4">
        {[
          { label: 'Events verified', value: health?.events_verified?.toLocaleString() ?? '—', sub: 'total' },
          { label: 'Divergences',     value: health?.divergences ?? '—', sub: 'active', red: (health?.divergences ?? 0) > 0 },
          { label: 'Specifications',  value: health?.specs ?? '—', sub: 'loaded' },
          { label: 'Server',          value: health?.status === 'ok' ? 'Online' : 'Offline', sub: 'status' },
        ].map(c => (
          <div key={c.label} className="bg-white border border-gray-200 rounded-lg p-3">
            <div className="text-xs text-gray-500 mb-1">{c.label}</div>
            <div className={`text-2xl font-medium ${c.red ? 'text-red-600' : 'text-gray-900'}`}>
              {String(c.value)}
            </div>
            <div className="text-xs text-gray-400 mt-0.5">{c.sub}</div>
          </div>
        ))}
      </div>

      {/* Main columns */}
      <div className="grid grid-cols-2 gap-3">
        {/* Feed */}
        <div className="bg-white border border-gray-200 rounded-xl p-4">
          <div className="text-xs font-medium text-gray-500 mb-3">Recent verification events</div>
          {recentEvents.length === 0 && (
            <p className="text-xs text-gray-400">No events yet — run a verify command.</p>
          )}
          <div className="space-y-0">
            {recentEvents.map((e, i) => {
              const div = activeDivs.find(d => d.customer_id === e.customer_id && d.spec_name === e.spec_name)
              return (
                <div
                  key={i}
                  onClick={() => div && setSelected(div)}
                  className={[
                    'flex items-center gap-2 py-1.5 px-1.5 rounded text-sm border-b border-gray-100 last:border-0',
                    div ? 'cursor-pointer hover:bg-gray-50' : '',
                    selected?.id === div?.id ? 'bg-gray-50' : '',
                  ].join(' ')}
                >
                  <span className={[
                    'w-4 h-4 rounded-full flex items-center justify-center text-xs flex-shrink-0',
                    e.ok ? 'bg-green-100 text-green-600' : 'bg-red-100 text-red-600',
                  ].join(' ')}>
                    {e.ok ? '✓' : '✗'}
                  </span>
                  <span className="flex-1 text-gray-800 truncate">
                    {e.spec_name} · {e.event_type}
                  </span>
                  <span className="text-xs text-gray-400 flex-shrink-0">{e.customer_id}</span>
                </div>
              )
            })}
          </div>
        </div>

        {/* Detail */}
        <div className="bg-white border border-gray-200 rounded-xl p-4">
          <div className="text-xs font-medium text-gray-500 mb-3">
            {selected ? 'Divergence detail' : 'Select a divergence from the feed'}
          </div>
          {!selected && activeDivs.length > 0 && (
            <div className="space-y-2">
              {activeDivs.map(d => (
                <div
                  key={d.id}
                  onClick={() => setSelected(d)}
                  className="flex items-center gap-2 p-2 rounded border border-red-100 bg-red-50 cursor-pointer hover:bg-red-100 transition-colors"
                >
                  <span className="text-xs font-medium text-red-700">{d.spec_name}</span>
                  <span className="text-xs text-gray-500">{d.customer_id}</span>
                  <span className="text-xs text-gray-400 ml-auto">£{d.balance}</span>
                </div>
              ))}
            </div>
          )}
          {selected && (
            <div>
              <div className="flex items-center gap-2 mb-4">
                <span className="text-sm font-medium text-gray-900">{selected.spec_name}</span>
                <span className="text-xs px-2 py-0.5 rounded-full bg-red-100 text-red-700">Divergence</span>
              </div>

              {/* Context */}
              <div className="mb-4">
                <div className="text-xs font-medium text-gray-500 mb-1">Context</div>
                {[
                  ['customer_id', selected.customer_id],
                  ['balance',     `£${selected.balance}`],
                  ['event',       selected.event_type],
                ].map(([k, v]) => (
                  <div key={k} className="flex justify-between py-1 border-b border-gray-100 last:border-0">
                    <span className="text-xs font-mono text-gray-500">{k}</span>
                    <span className="text-xs font-medium text-gray-900">{v}</span>
                  </div>
                ))}
              </div>

              {/* Diffs */}
              <div className="mb-4">
                <div className="text-xs font-medium text-gray-500 mb-1">Field mismatches</div>
                {selected.diffs.map(d => (
                  <div key={d.field} className="flex justify-between py-1 border-b border-gray-100 last:border-0">
                    <span className="text-xs font-mono text-gray-500">{d.field}</span>
                    <div className="flex items-center gap-2 text-xs">
                      <span className="text-green-600">{d.spec_value}</span>
                      <span className="text-gray-300">→</span>
                      <span className="text-red-600">{d.system_value}</span>
                      {d.delta && <span className="text-red-500 font-medium">{d.delta}</span>}
                    </div>
                  </div>
                ))}
              </div>

              <button
                onClick={() => handleResolve(selected)}
                className="text-xs text-gray-500 hover:text-gray-900 border border-gray-200 rounded px-3 py-1.5 transition-colors"
              >
                Mark resolved
              </button>
            </div>
          )}
        </div>
      </div>
    </div>
  )
}