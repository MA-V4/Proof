'use client'

import useSWR from 'swr'
import { api } from '@/lib/api'

export default function SpecsPage() {
  const { data: specs, isLoading } = useSWR('specs', api.specs, { refreshInterval: 5000 })

  if (isLoading) {
    return <p className="text-sm text-slate-400">Loading...</p>
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h1 className="text-xs font-semibold text-slate-500 uppercase tracking-widest">
          Specifications
        </h1>
        <span className="text-xs text-slate-400">{specs?.length ?? 0} loaded</span>
      </div>

      {(!specs || specs.length === 0) && (
        <div className="bg-white border border-slate-200 rounded-lg shadow-sm p-8 text-center">
          <p className="text-sm text-slate-400">
            No specs loaded. Place .proof files in your PROOF_SPECS_DIR.
          </p>
        </div>
      )}

      <div className="bg-white border border-slate-200 rounded-lg shadow-sm overflow-hidden">
        {specs?.map((s, i) => (
          <div
            key={s.name}
            className={[
              'flex items-center gap-4 px-5 py-4',
              i < (specs.length - 1) ? 'border-b border-slate-100' : '',
            ].join(' ')}
          >
            <span className={[
              'w-2.5 h-2.5 rounded-full flex-shrink-0',
              s.status === 'clean' ? 'bg-emerald-500' : 'bg-red-500',
            ].join(' ')} />
            <span className="text-sm font-semibold text-slate-900 flex-1 font-mono">
              {s.name}
            </span>
            <span className="text-xs text-slate-400">
              {s.status === 'clean' ? 'Verified' : `${s.divergences} divergence${s.divergences !== 1 ? 's' : ''}`}
            </span>
            <span className={[
              'text-xs px-2.5 py-1 rounded-full font-medium border',
              s.status === 'clean'
                ? 'bg-emerald-50 text-emerald-700 border-emerald-100'
                : 'bg-red-50 text-red-700 border-red-100',
            ].join(' ')}>
              {s.status === 'clean' ? 'Clean' : 'Action required'}
            </span>
          </div>
        ))}
      </div>
    </div>
  )
}