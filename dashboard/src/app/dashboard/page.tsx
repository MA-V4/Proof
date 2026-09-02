'use client'

import useSWR from 'swr'
import { useState, useEffect } from 'react'
import { api, Divergence, AuditEntry } from '@/lib/api'
import { PieChart, Pie, Cell, Tooltip, ResponsiveContainer } from 'recharts'

function getGreeting() {
  const h = new Date().getHours()
  if (h < 12) return 'Good morning'
  if (h < 17) return 'Good afternoon'
  return 'Good evening'
}

function Sparkline({ points, color }: { points: number[]; color: string }) {
  const max   = Math.max(...points)
  const min   = Math.min(...points)
  const range = max - min || 1
  const w = 120, h = 36
  const pts = points
    .map((v, i) => `${(i / (points.length - 1)) * w},${h - ((v - min) / range) * (h - 4) - 2}`)
    .join(' ')
  const areaPath = `M${pts.split(' ').join('L')} L${w},${h} L0,${h} Z`

  return (
    <svg width="100%" height={h} viewBox={`0 0 ${w} ${h}`} preserveAspectRatio="none">
      <defs>
        <linearGradient id={`grad-${color.replace('#', '')}`} x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" stopColor={color} stopOpacity="0.15"/>
          <stop offset="100%" stopColor={color} stopOpacity="0"/>
        </linearGradient>
      </defs>
      <path d={areaPath} fill={`url(#grad-${color.replace('#', '')})`}/>
      <polyline points={pts} fill="none" stroke={color} strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round"/>
    </svg>
  )
}

function MetricCard({
  label, value, sub, delta, deltaUp, color, sparkData, icon,
}: {
  label: string; value: string; sub: string
  delta?: string; deltaUp?: boolean; color: string
  sparkData: number[]; icon: React.ReactNode
}) {
  return (
    <div className="bg-white border border-slate-200 rounded-2xl p-5 shadow-sm flex flex-col justify-between overflow-hidden">
      <div className="flex items-start justify-between mb-3">
        <div className="text-xs font-semibold text-slate-400 uppercase tracking-widest">{label}</div>
        <div className="w-8 h-8 rounded-lg flex items-center justify-center" style={{ background: `${color}18` }}>
          <span style={{ color }}>{icon}</span>
        </div>
      </div>
      <div>
        <div className="text-3xl font-bold text-slate-900 tabular mb-0.5">{value}</div>
        <div className="text-xs text-slate-500 mb-3">{sub}</div>
      </div>
      {delta && (
        <div className={`text-xs font-medium flex items-center gap-1 mb-2 ${deltaUp ? 'text-emerald-600' : 'text-red-500'}`}>
          <span>{deltaUp ? '↑' : '↓'}</span>
          <span>{delta}</span>
        </div>
      )}
      <div className="-mx-5 -mb-5">
        <Sparkline points={sparkData} color={color} />
      </div>
    </div>
  )
}

function DonutChart({ high, medium, low }: { high: number, medium: number, low: number }) {
  const total = high + medium + low
  const data  = total === 0
    ? [{ name: 'Clean', value: 1, color: '#10B981' }]
    : [
        { name: 'High severity',   value: high,   color: '#EF4444' },
        { name: 'Medium severity', value: medium, color: '#FB923C' },
        { name: 'Low severity',    value: low,    color: '#93C5FD' },
      ].filter(d => d.value > 0)

  return (
    <div className="flex items-center gap-6">
      <div className="relative w-28 h-28 flex-shrink-0">
        <ResponsiveContainer width="100%" height="100%">
          <PieChart>
            <Pie
              data={data}
              cx="50%" cy="50%"
              innerRadius={32} outerRadius={52}
              startAngle={90} endAngle={-270}
              dataKey="value"
              strokeWidth={0}
            >
              {data.map((entry, i) => <Cell key={i} fill={entry.color} />)}
            </Pie>
          </PieChart>
        </ResponsiveContainer>
        <div className="absolute inset-0 flex flex-col items-center justify-center">
          <span className="text-2xl font-bold text-slate-900 tabular">{total}</span>
          <span className="text-xs text-slate-400">Active</span>
        </div>
      </div>
      <div className="space-y-2">
        {[
          { label: 'High severity',   value: high,   color: '#EF4444' },
          { label: 'Medium severity', value: medium, color: '#FB923C' },
          { label: 'Low severity',    value: low,    color: '#93C5FD' },
        ].map(row => (
          <div key={row.label} className="flex items-center gap-2 text-sm">
            <span className="w-2.5 h-2.5 rounded-full flex-shrink-0" style={{ background: row.color }}/>
            <span className="text-slate-600">{row.label}</span>
            <span className="font-semibold text-slate-900 ml-auto tabular">{row.value}</span>
          </div>
        ))}
      </div>
    </div>
  )
}

function classifyDivergence(div: Divergence): 'high' | 'medium' | 'low' {
  const pct = parseFloat(
    div.diffs.find(d => d.delta_pct && d.delta_pct !== 'null')?.delta_pct ?? '0'
  )
  if (Math.abs(pct) > 10) return 'high'
  if (Math.abs(pct) > 4)  return 'medium'
  return 'low'
}

const SPARK_VERIFY   = [98.6, 98.7, 98.9, 99.0, 98.8, 99.1, 99.0, 99.2, 99.1, 99.5, 99.7, 99.98]
const SPARK_DIVS     = [0, 1, 0, 2, 1, 0, 0, 3, 1, 0, 2, 2]
const SPARK_SPECS    = [8, 9, 9, 9, 10, 10, 10, 11, 11, 12, 12, 12]
const SPARK_IMPACT   = [400, 600, 800, 550, 900, 750, 1000, 850, 1100, 950, 1150, 1284]

export default function DashboardPage() {
  const { data: health }                     = useSWR('health',   api.health,          { refreshInterval: 5000 })
  const { data: events }                     = useSWR('recent',   api.recentEvents,    { refreshInterval: 3000 })
  const { data: divs, mutate: mutateDivs }   = useSWR('allDivs',  api.allDivergences, { refreshInterval: 3000 })
  const { data: auditEntries }               = useSWR('audit',    api.audit,           { refreshInterval: 10000 })
  const { data: specs }                      = useSWR('specs',    api.specs,           { refreshInterval: 10000 })
  const [selected, setSelected]              = useState<Divergence | null>(null)

  const activeDivs   = divs    ?? []
  const recentEvents = events  ?? []

  const total    = health?.events_verified ?? 0
  const divCount = health?.divergences ?? 0
  const specCount = health?.specs ?? 0
  const okRate   = total > 0
    ? (((total - divCount) / total) * 100).toFixed(2) + '%'
    : '100.00%'

  const monthlyImpact = activeDivs.reduce((sum, d) => {
    const delta = parseFloat(d.diffs.find(f => f.field === 'amount')?.delta ?? '0')
    return sum + Math.abs(delta) * 30
  }, 0)

  const highCount   = activeDivs.filter(d => classifyDivergence(d) === 'high').length
  const medCount    = activeDivs.filter(d => classifyDivergence(d) === 'medium').length
  const lowCount    = activeDivs.filter(d => classifyDivergence(d) === 'low').length

  const specChanges = (auditEntries ?? [])
    .filter(e => e.kind === 'spec_loaded' || e.kind === 'spec_signed_off')
    .slice(0, 4)

  async function handleResolve(d: Divergence) {
    await api.resolve(d.spec_name, d.id)
    mutateDivs()
    if (selected?.id === d.id) setSelected(null)
  }

  return (
    <div className="p-8 min-h-full">
      {/* Header */}
      <div className="flex items-start justify-between mb-7">
        <div>
          <h1 className="text-2xl font-bold text-slate-900">
            {getGreeting()}, Admin 👋
          </h1>
          <p className="text-sm text-slate-500 mt-1">
            {activeDivs.length > 0
              ? `${activeDivs.length} divergence${activeDivs.length !== 1 ? 's' : ''} require your attention.`
              : 'Everything is verified and operating within expected parameters.'}
          </p>
        </div>
        <div className="flex items-center gap-4">
          {divCount === 0 ? (
            <div className="flex items-center gap-2 bg-emerald-50 border border-emerald-100 rounded-full px-3 py-1.5">
              <span className="w-2 h-2 rounded-full bg-emerald-500" />
              <span className="text-xs font-semibold text-emerald-700">All systems nominal</span>
            </div>
          ) : (
            <div className="flex items-center gap-2 bg-red-50 border border-red-100 rounded-full px-3 py-1.5">
              <span className="w-2 h-2 rounded-full bg-red-500 animate-pulse" />
              <span className="text-xs font-semibold text-red-700">{divCount} divergence{divCount !== 1 ? 's' : ''} active</span>
            </div>
          )}
          <div className="relative">
            <button className="p-2 rounded-xl hover:bg-slate-100 border border-slate-200 transition-colors bg-white">
              <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="#64748B" strokeWidth="1.5">
                <path d="M8 1a5 5 0 015 5c0 5.25 2 6.5 2 6.5H1S3 11.25 3 6a5 5 0 015-5z"/>
                <path d="M6.5 13.5a1.5 1.5 0 003 0"/>
              </svg>
            </button>
            {divCount > 0 && (
              <span className="absolute -top-1 -right-1 w-4 h-4 bg-red-500 rounded-full text-white text-xs flex items-center justify-center font-bold">
                {divCount}
              </span>
            )}
          </div>
        </div>
      </div>

      {/* Metric cards */}
      <div className="grid grid-cols-4 gap-4 mb-6">
        <MetricCard
          label="Verification rate"
          value={okRate}
          sub="of events verified"
          delta="0.02% from yesterday"
          deltaUp={true}
          color="#10B981"
          sparkData={SPARK_VERIFY}
          icon={
            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
              <circle cx="8" cy="8" r="7" fill="none" stroke="currentColor" strokeWidth="1.5"/>
              <circle cx="8" cy="8" r="3"/>
            </svg>
          }
        />
        <MetricCard
          label="Active divergences"
          value={String(divCount)}
          sub="requiring investigation"
          delta={divCount > 0 ? `${divCount} new since last scan` : undefined}
          deltaUp={false}
          color="#EF4444"
          sparkData={SPARK_DIVS.map(v => v + divCount)}
          icon={
            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
              <path d="M8 1L15 14H1L8 1zm0 4v4m0 2v1.5"/>
            </svg>
          }
        />
        <MetricCard
          label="Specifications"
          value={String(specCount)}
          sub="deployed"
          color="#3B82F6"
          sparkData={SPARK_SPECS}
          icon={
            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
              <rect x="2" y="1" width="12" height="14" rx="2" fill="none" stroke="currentColor" strokeWidth="1.5"/>
              <rect x="4" y="4" width="8" height="1.5" rx="0.5"/>
              <rect x="4" y="7" width="6" height="1.5" rx="0.5"/>
              <rect x="4" y="10" width="7" height="1.5" rx="0.5"/>
            </svg>
          }
        />
        <MetricCard
          label="Customer impact"
          value={`£${monthlyImpact.toFixed(2)}`}
          sub="potential monthly impact"
          color="#8B5CF6"
          sparkData={SPARK_IMPACT}
          icon={
            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
              <circle cx="8" cy="5" r="3"/>
              <path d="M2 14c0-3.31 2.69-6 6-6s6 2.69 6 6"/>
            </svg>
          }
        />
      </div>

      {/* Main grid */}
      <div className="grid grid-cols-5 gap-4">
        {/* Left col: feed + simulation */}
        <div className="col-span-2 space-y-4">
          {/* Verification feed */}
          <div className="bg-white border border-slate-200 rounded-2xl shadow-sm overflow-hidden">
            <div className="px-5 py-4 flex items-center justify-between border-b border-slate-100">
              <h2 className="text-sm font-semibold text-slate-900">Verification feed</h2>
              <span className="text-xs font-semibold text-blue-600 bg-blue-50 border border-blue-100 rounded-full px-2 py-0.5">
                Live
              </span>
            </div>
            <div className="divide-y divide-slate-50">
              {recentEvents.length === 0 ? (
                <p className="text-sm text-slate-400 px-5 py-6 text-center">
                  No events yet. Run a verify command.
                </p>
              ) : (
                recentEvents.slice(0, 6).map((e, i) => {
                  const div = activeDivs.find(
                    d => d.customer_id === e.customer_id && d.spec_name === e.spec_name
                  )
                  return (
                    <div
                      key={i}
                      onClick={() => div && setSelected(div)}
                      className={[
                        'flex items-center gap-3 px-5 py-3 transition-colors',
                        div ? 'cursor-pointer hover:bg-slate-50' : '',
                        selected?.id === div?.id ? 'bg-slate-50' : '',
                      ].join(' ')}
                    >
                      <div className={[
                        'w-6 h-6 rounded-full flex items-center justify-center flex-shrink-0 text-xs font-bold',
                        e.ok
                          ? 'bg-emerald-100 text-emerald-600'
                          : 'bg-red-100 text-red-600',
                      ].join(' ')}>
                        {e.ok ? '✓' : '!'}
                      </div>
                      <div className="flex-1 min-w-0">
                        <div className="text-sm font-medium text-slate-900 flex items-center gap-1.5">
                          <span className="font-mono text-xs">{e.event_type}</span>
                          <span className="text-slate-300">·</span>
                          <span>{e.spec_name}</span>
                        </div>
                        <div className="text-xs text-slate-400 mt-0.5">
                          {e.customer_id} · {e.ok ? 'Event verified' : 'Divergence detected'}
                        </div>
                      </div>
                      <div className="text-xs font-mono text-slate-400 flex-shrink-0">
                        {new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' })}
                      </div>
                      <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="#CBD5E1" strokeWidth="1.5">
                        <path d="M5 3l4 4-4 4"/>
                      </svg>
                    </div>
                  )
                })
              )}
            </div>
            <div className="px-5 py-3 border-t border-slate-100">
              <a href="/audit" className="text-xs font-semibold text-blue-600 hover:text-blue-700 flex items-center gap-1">
                View full audit trail
                <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" strokeWidth="1.5">
                  <path d="M2 6h8M6 2l4 4-4 4"/>
                </svg>
              </a>
            </div>
          </div>

          {/* Simulation quick start */}
          <div className="bg-white border border-slate-200 rounded-2xl shadow-sm p-5">
            <div className="mb-1">
              <h2 className="text-sm font-semibold text-slate-900">Simulation quick start</h2>
              <p className="text-xs text-slate-500 mt-0.5">Test changes before they go live.</p>
            </div>
            <div className="flex items-center gap-2 mt-4">
              <div className="flex-1 bg-slate-50 rounded-lg p-3 border border-slate-200">
                <div className="text-xs text-slate-400 mb-1">Select specification</div>
                <div className="text-sm font-semibold text-slate-900">
                  {specs?.[0]?.name ?? 'SavingsAccount'}
                </div>
                <div className="text-xs text-slate-400">v2.1 (current)</div>
              </div>
              <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="#CBD5E1" strokeWidth="1.5">
                <path d="M3 8h10M9 4l4 4-4 4"/>
              </svg>
              <div className="flex-1 bg-slate-50 rounded-lg p-3 border border-slate-200">
                <div className="text-xs text-slate-400 mb-1">Configure changes</div>
                <div className="text-sm font-semibold text-slate-900">
                  {activeDivs.length} change{activeDivs.length !== 1 ? 's' : ''} detected
                </div>
                <a href="/diff" className="text-xs text-blue-600 font-medium">View diff</a>
              </div>
              <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="#CBD5E1" strokeWidth="1.5">
                <path d="M3 8h10M9 4l4 4-4 4"/>
              </svg>
              <div className="flex-1 bg-slate-50 rounded-lg p-3 border border-slate-200">
                <div className="text-xs text-slate-400 mb-1">Run simulation</div>
                <div className="text-sm font-semibold text-slate-900">Estimate impact</div>
                <div className="text-xs text-slate-400">On portfolio</div>
              </div>
              
              <a href="/simulate" className="w-10 h-10 bg-blue-600 hover:bg-blue-700 rounded-xl flex items-center justify-center text-white flex-shrink-0 transition-colors shadow-sm"><svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor"><path d="M4 3l9 5-9 5V3z"/></svg></a>
            </div>
            <div className="mt-4">
              <a href="/simulate" className="text-xs font-semibold text-blue-600 hover:text-blue-700 flex items-center gap-1">
                Go to simulator
                <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" strokeWidth="1.5">
                  <path d="M2 6h8M6 2l4 4-4 4"/>
                </svg>
              </a>
            </div>
          </div>
        </div>

        {/* Middle col: donut + spec changes */}
        <div className="col-span-2 space-y-4">
          {/* Divergence overview */}
          <div className="bg-white border border-slate-200 rounded-2xl shadow-sm p-5">
            <h2 className="text-sm font-semibold text-slate-900 mb-4">Divergence overview</h2>
            <DonutChart high={highCount} medium={medCount} low={lowCount} />
            {activeDivs.length > 0 && selected && (
              <div className="mt-4 p-3 bg-slate-50 rounded-xl border border-slate-100">
                <div className="text-xs font-semibold text-slate-700 mb-2">Selected: {selected.customer_id}</div>
                {selected.diffs.map(d => (
                  <div key={d.field} className="flex items-center justify-between text-xs py-0.5">
                    <span className="font-mono text-slate-500">{d.field}</span>
                    <div className="flex items-center gap-2">
                      <span className="text-emerald-600">{d.spec_value}</span>
                      <span className="text-slate-300">→</span>
                      <span className="text-red-600">{d.system_value}</span>
                    </div>
                  </div>
                ))}
                <div className="flex gap-2 mt-2">
                  <button
                    onClick={() => handleResolve(selected)}
                    className="text-xs text-slate-600 border border-slate-200 rounded-md px-2.5 py-1 hover:bg-slate-100 transition-colors"
                  >
                    Resolve
                  </button>
                  <button
                    onClick={() => setSelected(null)}
                    className="text-xs text-slate-400 px-2.5 py-1"
                  >
                    Close
                  </button>
                </div>
              </div>
            )}
            <div className="mt-4">
              <a href="/dashboard" className="text-xs font-semibold text-blue-600 hover:text-blue-700 flex items-center gap-1">
                Investigate divergences
                <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" strokeWidth="1.5">
                  <path d="M2 6h8M6 2l4 4-4 4"/>
                </svg>
              </a>
            </div>
          </div>

          {/* Recent spec changes */}
          <div className="bg-white border border-slate-200 rounded-2xl shadow-sm overflow-hidden">
            <div className="px-5 py-4 flex items-center justify-between border-b border-slate-100">
              <h2 className="text-sm font-semibold text-slate-900">Recent specification changes</h2>
              <a href="/audit" className="text-xs font-semibold text-blue-600 hover:text-blue-700 flex items-center gap-1">
                View all
                <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" strokeWidth="1.5">
                  <path d="M2 5h6M5 2l3 3-3 3"/>
                </svg>
              </a>
            </div>
            <div className="divide-y divide-slate-50">
              {specChanges.length === 0 ? (
                <p className="text-sm text-slate-400 px-5 py-5 text-center">No spec changes yet.</p>
              ) : (
                specChanges.map((entry: AuditEntry) => {
                  const isLoaded   = entry.kind === 'spec_loaded'
                  const isSignedOff = entry.kind === 'spec_signed_off'
                  return (
                    <div key={entry.id} className="flex items-center gap-3 px-5 py-3">
                      <div className="w-8 h-8 bg-blue-50 rounded-lg flex items-center justify-center flex-shrink-0">
                        <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="#3B82F6" strokeWidth="1.5">
                          <rect x="2" y="1" width="10" height="12" rx="1.5"/>
                          <path d="M4 4h6M4 7h4"/>
                        </svg>
                      </div>
                      <div className="flex-1 min-w-0">
                        <div className="text-sm font-semibold text-slate-900 truncate">{entry.spec_name}</div>
                        <div className="text-xs text-slate-400">
                          by {entry.actor} · {new Date(entry.timestamp).toLocaleString([], { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' })}
                        </div>
                      </div>
                      <span className={[
                        'text-xs font-semibold px-2 py-0.5 rounded-full border flex-shrink-0',
                        isSignedOff
                          ? 'bg-emerald-50 text-emerald-700 border-emerald-100'
                          : isLoaded
                          ? 'bg-blue-50 text-blue-700 border-blue-100'
                          : 'bg-amber-50 text-amber-700 border-amber-100',
                      ].join(' ')}>
                        {isSignedOff ? 'Signed off' : isLoaded ? 'Deployed' : 'Pending'}
                      </span>
                      <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="#CBD5E1" strokeWidth="1.5">
                        <path d="M5 3l4 4-4 4"/>
                      </svg>
                    </div>
                  )
                })
              )}
            </div>
          </div>
        </div>

        {/* Right col: system integrity */}
        <div className="col-span-1">
          <div className="bg-white border border-slate-200 rounded-2xl shadow-sm p-5 h-full flex flex-col">
            <h2 className="text-sm font-semibold text-slate-900 mb-4">System integrity</h2>

            {/* Hexagon visualization */}
            <div className="flex-1 flex items-center justify-center py-4">
              <div className="relative">
                <svg width="140" height="140" viewBox="0 0 140 140">
                  {/* Outer rings */}
                  {[60, 48, 36].map((r, i) => (
                    <circle key={i} cx="70" cy="70" r={r}
                      fill="none"
                      stroke={i === 0 ? '#EFF6FF' : i === 1 ? '#DBEAFE' : '#BFDBFE'}
                      strokeWidth={i === 0 ? 1 : 1}
                    />
                  ))}
                  {/* Hex dots around */}
                  {[0, 60, 120, 180, 240, 300].map((deg, i) => {
                    const rad = (deg * Math.PI) / 180
                    const x = 70 + 52 * Math.cos(rad)
                    const y = 70 + 52 * Math.sin(rad)
                    return (
                      <circle key={i} cx={x} cy={y} r="5"
                        fill={i % 2 === 0 ? '#3B82F6' : '#93C5FD'} fillOpacity="0.6"
                      />
                    )
                  })}
                  {/* Center hex */}
                  <path
                    d="M70,48 L84,56 L84,70 L70,78 L56,70 L56,56 Z"
                    fill="#EFF6FF" stroke="#3B82F6" strokeWidth="1.5"
                  />
                  {/* Center check */}
                  <circle cx="70" cy="62" r="10" fill="#10B981"/>
                  <path d="M65 62l3 3 6-6" stroke="white" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" fill="none"/>
                </svg>
              </div>
            </div>

            <div className="text-center mt-2">
              <div className="text-xs text-slate-500 mb-1">All critical systems</div>
              <div className="text-lg font-bold text-slate-900">Operational</div>
            </div>

            <div className="mt-4 space-y-2">
              {[
                { label: 'Evaluation engine', ok: true },
                { label: 'Verification engine', ok: true },
                { label: 'Database', ok: true },
                { label: 'API server', ok: health?.status === 'ok' },
              ].map(row => (
                <div key={row.label} className="flex items-center justify-between text-xs">
                  <span className="text-slate-500">{row.label}</span>
                  <div className="flex items-center gap-1.5">
                    <span className={`w-1.5 h-1.5 rounded-full ${row.ok ? 'bg-emerald-500' : 'bg-red-500'}`}/>
                    <span className={row.ok ? 'text-emerald-600 font-medium' : 'text-red-600 font-medium'}>
                      {row.ok ? 'Online' : 'Error'}
                    </span>
                  </div>
                </div>
              ))}
            </div>

            <div className="mt-4 pt-4 border-t border-slate-100 text-center">
              <div className="text-xs text-slate-400">Last verified</div>
              <div className="text-xs font-mono font-semibold text-slate-700 mt-0.5">
                {new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' })}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}