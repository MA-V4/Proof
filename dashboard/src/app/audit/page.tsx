'use client'

import { useState } from 'react'
import useSWR from 'swr'
import { api, AuditEntry } from '@/lib/api'

const EVENT_STYLES: Record<string, { color: string; bg: string; label: string; icon: React.ReactNode }> = {
  spec_loaded:          { color: '#3B82F6', bg: '#EFF6FF', label: 'Spec loaded',         icon: <path d="M4 1h8l4 4v10H4V1zM8 1v5h4" stroke="currentColor" strokeWidth="1.4" fill="none" strokeLinecap="round"/> },
  verified:             { color: '#10B981', bg: '#ECFDF5', label: 'Verified',             icon: <path d="M2 8l4 4 8-8" stroke="currentColor" strokeWidth="1.8" fill="none" strokeLinecap="round" strokeLinejoin="round"/> },
  divergence_detected:  { color: '#EF4444', bg: '#FEF2F2', label: 'Divergence detected',  icon: <path d="M8 2L14 13H2L8 2zM8 7v3M8 11.5v.5" stroke="currentColor" strokeWidth="1.4" fill="none" strokeLinecap="round"/> },
  divergence_resolved:  { color: '#10B981', bg: '#ECFDF5', label: 'Divergence resolved',  icon: <path d="M2 8l4 4 8-8" stroke="currentColor" strokeWidth="1.8" fill="none" strokeLinecap="round" strokeLinejoin="round"/> },
  simulation_run:       { color: '#8B5CF6', bg: '#F5F3FF', label: 'Simulation run',       icon: <path d="M3 2l10 6-10 6V2z" stroke="currentColor" strokeWidth="1.4" fill="none" strokeLinecap="round" strokeLinejoin="round"/> },
  spec_signed_off:      { color: '#F59E0B', bg: '#FFFBEB', label: 'Signed off',           icon: <path d="M2 12l3-3 7-7 2 2-7 7-3 3H2v-2z" stroke="currentColor" strokeWidth="1.4" fill="none" strokeLinecap="round"/> },
}

function entryLabel(entry: AuditEntry): string {
  switch (entry.kind) {
    case 'spec_loaded':         return `Spec loaded — ${(entry as any).source}`
    case 'verified':            return `Verified — ${(entry as any).customer_id}`
    case 'divergence_detected': return `Divergence detected — ${(entry as any).divergence_id?.slice(0, 8)}`
    case 'divergence_resolved': return `Divergence resolved — ${(entry as any).divergence_id?.slice(0, 8)}`
    case 'simulation_run':      return `Simulation run — ${(entry as any).verdict}`
    case 'spec_signed_off':     return `Signed off by ${(entry as any).approver}`
    default:                    return 'Event'
  }
}

export default function AuditPage() {
  const { data: entries, isLoading, mutate } = useSWR('audit', api.audit, { refreshInterval: 5000 })
  const { data: specs }                       = useSWR('specs', api.specs)
  const [approver,   setApprover]   = useState('')
  const [specName,   setSpecName]   = useState('SavingsAccount')
  const [signing,    setSigning]    = useState(false)
  const [exporting,  setExporting]  = useState(false)

  async function handleSignOff() {
    if (!approver.trim()) return
    setSigning(true)
    try { await api.signOff(specName, approver); setApprover(''); mutate() }
    finally { setSigning(false) }
  }

  async function handleExport() {
    setExporting(true)
    try {
      const pack = await api.exportFca(specName)
      const blob = new Blob([JSON.stringify(pack, null, 2)], { type: 'application/json' })
      const url  = URL.createObjectURL(blob)
      const a    = document.createElement('a')
      a.href     = url
      a.download = `fca-audit-${specName}-${new Date().toISOString().slice(0, 10)}.json`
      a.click()
      URL.revokeObjectURL(url)
    } finally { setExporting(false) }
  }

  return (
    <div className="p-8 min-h-full">
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-xl font-bold text-slate-900">Audit trail</h1>
          <p className="text-sm text-slate-500 mt-0.5">
            {entries?.length ?? 0} entries · Immutable · FCA compliant
          </p>
        </div>
        <button
          onClick={handleExport}
          disabled={exporting}
          className="flex items-center gap-2 bg-slate-900 hover:bg-slate-800 text-white text-sm font-semibold px-4 py-2.5 rounded-xl transition-colors disabled:opacity-50 shadow-sm"
        >
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round">
            <path d="M7 1v8M4 6l3 3 3-3M1 11h12"/>
          </svg>
          {exporting ? 'Exporting...' : 'Export FCA pack'}
        </button>
      </div>

      {/* Sign-off card */}
      <div className="bg-white border border-slate-200 rounded-2xl p-5 mb-5 shadow-sm">
        <div className="text-xs font-semibold text-slate-500 uppercase tracking-widest mb-3">Compliance sign-off</div>
        <div className="flex items-center gap-3">
          <div className="flex-1">
            <label className="text-xs text-slate-500 block mb-1.5">Specification</label>
            <select
              value={specName}
              onChange={e => setSpecName(e.target.value)}
              className="w-full text-sm border border-slate-200 rounded-xl px-3 py-2.5 bg-slate-50 text-slate-900 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            >
              {specs?.map(s => <option key={s.name}>{s.name}</option>)}
            </select>
          </div>
          <div className="flex-1">
            <label className="text-xs text-slate-500 block mb-1.5">Your name</label>
            <input
              value={approver}
              onChange={e => setApprover(e.target.value)}
              placeholder="Full name"
              className="w-full text-sm border border-slate-200 rounded-xl px-3 py-2.5 bg-slate-50 text-slate-900 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            />
          </div>
          <div className="pt-5">
            <button
              onClick={handleSignOff}
              disabled={signing || !approver.trim()}
              className="text-sm font-semibold text-blue-600 border border-blue-200 bg-blue-50 hover:bg-blue-100 rounded-xl px-5 py-2.5 transition-colors disabled:opacity-40"
            >
              {signing ? 'Signing...' : 'Sign off'}
            </button>
          </div>
        </div>
      </div>

      {/* Audit log */}
      {isLoading ? (
        <p className="text-sm text-slate-400 text-center py-12">Loading...</p>
      ) : !entries?.length ? (
        <div className="bg-white border border-slate-200 rounded-2xl p-12 text-center shadow-sm">
          <p className="text-sm text-slate-400">No audit entries yet. Send events to the server to start building the trail.</p>
        </div>
      ) : (
        <div className="bg-white border border-slate-200 rounded-2xl shadow-sm overflow-hidden">
          <div className="divide-y divide-slate-50">
            {entries.map(entry => {
              const style = EVENT_STYLES[entry.kind] ?? EVENT_STYLES['verified']
              return (
                <div key={entry.id} className="flex items-center gap-4 px-5 py-3.5 hover:bg-slate-50 transition-colors">
                  <div
                    className="w-8 h-8 rounded-lg flex items-center justify-center flex-shrink-0"
                    style={{ background: style.bg }}
                  >
                    <svg width="14" height="14" viewBox="0 0 14 14" style={{ color: style.color }}>
                      {style.icon}
                    </svg>
                  </div>
                  <div className="flex-1 min-w-0">
                    <div className="text-sm font-medium text-slate-900 truncate">
                      {entryLabel(entry)}
                    </div>
                    <div className="flex items-center gap-2 mt-0.5">
                      <span className="text-xs font-mono text-slate-400">{entry.spec_hash.slice(0, 8)}</span>
                      <span className="text-slate-300">·</span>
                      <span className="text-xs text-slate-400">{entry.spec_name}</span>
                      <span className="text-slate-300">·</span>
                      <span className="text-xs text-slate-400">by {entry.actor}</span>
                    </div>
                  </div>
                  <div className="text-xs font-mono text-slate-400 flex-shrink-0">
                    {new Date(entry.timestamp).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' })}
                  </div>
                  <div
                    className="text-xs font-semibold px-2.5 py-1 rounded-full border flex-shrink-0"
                    style={{ color: style.color, background: style.bg, borderColor: `${style.color}22` }}
                  >
                    {style.label}
                  </div>
                </div>
              )
            })}
          </div>
        </div>
      )}
    </div>
  )
}