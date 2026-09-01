'use client'

import { useState } from 'react'

const BASE = process.env.NEXT_PUBLIC_API_URL ?? 'http://localhost:3001'

type Flag = {
  rule: string
  severity: 'Info' | 'Review' | 'Block'
  description: string
  action: string
  notice_days: number | null
}

type Report = {
  spec_name: string
  customers_total: number
  customers_worse: number
  customers_better: number
  customers_neutral: number
  daily_delta: string
  monthly_delta: string
  avg_delta_worse: string | null
  regulatory_flags: Flag[]
  verdict: 'DeployClean' | 'DeployWithReview' | 'DoNotDeploy'
}

const EXAMPLE_V2 = `product SavingsAccount {
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

const EXAMPLE_PORTFOLIO = `[
  {"customer_id":"C-001","event_type":{"type":"daily_accrual"},"balance":"9840"},
  {"customer_id":"C-002","event_type":{"type":"daily_accrual"},"balance":"12000"},
  {"customer_id":"C-003","event_type":{"type":"daily_accrual"},"balance":"500"},
  {"customer_id":"C-004","event_type":{"type":"daily_accrual"},"balance":"1500"},
  {"customer_id":"C-005","event_type":{"type":"daily_accrual"},"balance":"2000"},
  {"customer_id":"C-006","event_type":{"type":"daily_accrual"},"balance":"3000"},
  {"customer_id":"C-007","event_type":{"type":"daily_accrual"},"balance":"15000"},
  {"customer_id":"C-008","event_type":{"type":"daily_accrual"},"balance":"800"},
  {"customer_id":"C-009","event_type":{"type":"daily_accrual"},"balance":"1200"},
  {"customer_id":"C-010","event_type":{"type":"daily_accrual"},"balance":"2400"}
]`

export default function SimulatePage() {
  const [specName,      setSpecName]      = useState('SavingsAccount')
  const [newSpecText,   setNewSpecText]   = useState(EXAMPLE_V2)
  const [portfolioText, setPortfolioText] = useState(EXAMPLE_PORTFOLIO)
  const [report,        setReport]        = useState<Report | null>(null)
  const [loading,       setLoading]       = useState(false)
  const [error,         setError]         = useState<string | null>(null)

  async function runSim() {
    setLoading(true)
    setError(null)
    setReport(null)
    try {
      const portfolio = JSON.parse(portfolioText)
      const res = await fetch(`${BASE}/simulate`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ old_spec_name: specName, new_spec_text: newSpecText, portfolio }),
      })
      if (!res.ok) {
        const err = await res.json()
        throw new Error(err.error ?? 'Simulation failed')
      }
      setReport(await res.json())
    } catch (e: any) {
      setError(e.message)
    } finally {
      setLoading(false)
    }
  }

  const verdictStyle = {
    DeployClean:       'bg-green-100 text-green-700',
    DeployWithReview:  'bg-yellow-100 text-yellow-800',
    DoNotDeploy:       'bg-red-100 text-red-700',
  }

  const verdictLabel = {
    DeployClean:       'Deploy - no concerns',
    DeployWithReview:  'Deploy with review',
    DoNotDeploy:       'Do not deploy',
  }

  return (
    <div>
      <h1 className="text-sm font-medium text-gray-500 mb-4">Portfolio simulation</h1>

      <div className="grid grid-cols-2 gap-3 mb-3">
        <div>
          <label className="text-xs text-gray-500 block mb-1">Old spec (currently loaded)</label>
          <input
            value={specName}
            onChange={e => setSpecName(e.target.value)}
            className="w-full text-sm border border-gray-200 rounded-lg px-3 py-2 font-mono"
          />
        </div>
        <div className="flex items-end">
          <button
            onClick={runSim}
            disabled={loading}
            className="w-full text-sm bg-gray-900 text-white rounded-lg px-4 py-2 hover:bg-gray-700 disabled:opacity-50 transition-colors"
          >
            {loading ? 'Simulating...' : 'Run simulation'}
          </button>
        </div>
      </div>

      <div className="grid grid-cols-2 gap-3 mb-4">
        <div>
          <label className="text-xs text-gray-500 block mb-1">New spec (.proof)</label>
          <textarea
            value={newSpecText}
            onChange={e => setNewSpecText(e.target.value)}
            className="w-full h-64 text-xs border border-gray-200 rounded-lg px-3 py-2 font-mono resize-none"
          />
        </div>
        <div>
          <label className="text-xs text-gray-500 block mb-1">Portfolio (JSON array of events)</label>
          <textarea
            value={portfolioText}
            onChange={e => setPortfolioText(e.target.value)}
            className="w-full h-64 text-xs border border-gray-200 rounded-lg px-3 py-2 font-mono resize-none"
          />
        </div>
      </div>

      {error && (
        <div className="bg-red-50 border border-red-200 rounded-lg px-4 py-3 mb-4 text-sm text-red-700">
          {error}
        </div>
      )}

      {report && (
        <div>
          {/* Before/after */}
          <div className="grid grid-cols-2 gap-3 mb-3">
            <div className="bg-gray-50 border border-gray-200 rounded-xl p-4">
              <div className="text-xs font-medium text-gray-500 mb-3">Current - {report.spec_name}</div>
              <div className="space-y-1">
                {[
                  ['Total customers', report.customers_total],
                  ['Daily interest paid', '(baseline)'],
                ].map(([k, v]) => (
                  <div key={String(k)} className="flex justify-between text-sm">
                    <span className="text-gray-500">{k}</span>
                    <span className="font-medium">{v}</span>
                  </div>
                ))}
              </div>
            </div>
            <div className="bg-yellow-50 border border-yellow-200 rounded-xl p-4">
              <div className="text-xs font-medium text-yellow-700 mb-3">Proposed</div>
              <div className="space-y-1">
                {[
                  ['Customers worse off',  report.customers_worse],
                  ['Customers better off', report.customers_better],
                  ['Customers neutral',    report.customers_neutral],
                  ['Monthly aggregate',    `£${report.monthly_delta}`],
                ].map(([k, v]) => (
                  <div key={String(k)} className="flex justify-between text-sm">
                    <span className="text-gray-500">{k}</span>
                    <span className={`font-medium ${Number(report.customers_worse) > 0 && k === 'Customers worse off' ? 'text-red-600' : ''}`}>{v}</span>
                  </div>
                ))}
              </div>
            </div>
          </div>

          {/* Regulatory flags */}
          {report.regulatory_flags.length > 0 && (
            <div className="bg-white border border-gray-200 rounded-xl p-4 mb-3">
              <div className="text-xs font-medium text-gray-500 mb-3">Regulatory flags</div>
              {report.regulatory_flags.map((f, i) => (
                <div key={i} className="flex gap-3 items-start py-2 border-b border-gray-100 last:border-0">
                  <span className={`text-xs mt-0.5 ${f.severity === 'Block' ? 'text-red-600' : f.severity === 'Review' ? 'text-yellow-600' : 'text-blue-600'}`}>
                    {f.severity === 'Block' ? '✗' : f.severity === 'Review' ? '⚠' : 'i'}
                  </span>
                  <div>
                    <div className="text-sm font-medium text-gray-900">{f.rule}</div>
                    <div className="text-xs text-gray-500">{f.description}</div>
                    <div className="text-xs text-gray-500">{f.action}</div>
                    {f.notice_days && <div className="text-xs text-gray-400">Notice: {f.notice_days} days</div>}
                  </div>
                </div>
              ))}
            </div>
          )}

          {/* Verdict */}
          <div className={`rounded-xl px-4 py-3 flex items-center justify-between ${verdictStyle[report.verdict]}`}>
            <span className="text-sm font-medium">{verdictLabel[report.verdict]}</span>
            {report.avg_delta_worse && (
              <span className="text-xs opacity-75">avg customer impact: £{report.avg_delta_worse}/day</span>
            )}
          </div>
        </div>
      )}
    </div>
  )
}