'use client'

import useSWR from 'swr'
import { api } from '@/lib/api'

export default function SpecsPage() {
  const { data: specs, isLoading } = useSWR('specs', api.specs, { refreshInterval: 5000 })

  if (isLoading) return <p className="text-sm text-gray-400">Loading...</p>

  return (
    <div>
      <h1 className="text-sm font-medium text-gray-500 mb-4">Loaded specifications</h1>
      {(!specs || specs.length === 0) && (
        <p className="text-sm text-gray-400">
          No specs loaded. Make sure the server can find your .proof files.
        </p>
      )}
      <div className="space-y-2">
        {specs?.map(s => (
          <div key={s.name} className="bg-white border border-gray-200 rounded-xl p-4 flex items-center gap-3">
            <span className={[
              'w-2.5 h-2.5 rounded-full flex-shrink-0',
              s.status === 'clean' ? 'bg-green-500' : 'bg-red-500',
            ].join(' ')} />
            <span className="text-sm font-medium text-gray-900 flex-1">{s.name}</span>
            {s.divergences > 0 ? (
              <span className="text-xs px-2 py-0.5 rounded-full bg-red-100 text-red-700">
                {s.divergences} divergence{s.divergences !== 1 ? 's' : ''}
              </span>
            ) : (
              <span className="text-xs px-2 py-0.5 rounded-full bg-green-100 text-green-700">
                Clean
              </span>
            )}
          </div>
        ))}
      </div>
    </div>
  )
}