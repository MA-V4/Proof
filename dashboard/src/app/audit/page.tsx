'use client'

import useSWR from 'swr'
import { api, AuditEntry } from '@/lib/api'

function kindLabel(entry: AuditEntry): { label: string; ok: boolean | null } {
  switch (entry.kind) {
    case 'spec_loaded':         return { label: `Spec loaded — ${entry.source}`,             ok: null }
    case 'verified':            return { label: `Verified — ${entry.customer_id}`,            ok: entry.ok }
    case 'divergence_detected': return { label: `Divergence detected — ${entry.divergence_id.slice(0,8)}`, ok: false }
    case 'divergence_resolved': return { label: `Divergence resolved — ${entry.divergence_id.slice(0,8)}`, ok: true }
    case 'simulation_run':      return { label: `Simulation run — verdict: ${entry.verdict}`, ok: null }
    case 'spec_signed_off':     return { label: `Signed off by ${entry.approver}`,            ok: true }
    default:                    return { label: 'Unknown event',                              ok: null }
  }
}

function dot(ok: boolean | null) {
  if (ok === null)  return <span className="w-2 h-2 rounded-full bg-gray-300 flex-shrink-0 mt-1" />
  if (ok)           return <span className="w-2 h-2 rounded-full bg-green-500 flex-shrink-0 mt-1" />
  return               <span className="w-2 h-2 rounded-full bg-red-500 flex-shrink-0 mt-1" />
}

export default function AuditPage() {
  const { data: entries, isLoading } = useSWR('audit', api.audit, { refreshInterval: 5000 })

  if (isLoading) return <p className="text-sm text-gray-400">Loading...</p>

  return (
    <div>
      <div className="flex items-center justify-between mb-4">
        <h1 className="text-sm font-medium text-gray-500">Audit trail</h1>
        <span className="text-xs text-gray-400">{entries?.length ?? 0} entries</span>
      </div>

      {(!entries || entries.length === 0) && (
        <p className="text-sm text-gray-400">No audit entries yet — send some events to the server.</p>
      )}

      <div className="bg-white border border-gray-200 rounded-xl divide-y divide-gray-100">
        {entries?.map(entry => {
          const { label, ok } = kindLabel(entry)
          return (
            <div key={entry.id} className="flex items-start gap-3 px-4 py-3">
              {dot(ok)}
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2">
                  <span className="text-sm text-gray-900">{label}</span>
                  <span className="text-xs text-gray-400">{entry.spec_name}</span>
                </div>
                <div className="flex items-center gap-3 mt-0.5">
                  <span className="text-xs font-mono text-gray-400">{entry.spec_hash.slice(0,8)}</span>
                  <span className="text-xs text-gray-400">
                    {new Date(entry.timestamp).toLocaleTimeString()}
                  </span>
                </div>
              </div>
            </div>
          )
        })}
      </div>
    </div>
  )
}