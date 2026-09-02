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

const NEW_SPEC_DEFAULT = `product SavingsAccount {
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

const PORTFOLIO_DEFAULT = `[
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

const VERDICT_CONFIG = {
  DeployClean: {
    label: 'Deploy — no concerns',
    color: '#10B981',
    bg: '#ECFDF5',
    border: '#A7F3D0',
  },
  DeployWithReview: {
    label: 'Deploy with review',
    color: '#F59E0B',
    bg: '#FFFBEB',
    border: '#FDE68A',
  },
  DoNotDeploy: {
    label: 'Do not deploy',
    color: '#EF4444',
    bg: '#FEF2F2',
    border: '#FECACA',
  },
} as const

export default function SimulatePage() {
  const [specName, setSpecName] = useState('SavingsAccount')
  const [newSpecText, setNewSpecText] = useState(NEW_SPEC_DEFAULT)
  const [portfolioText, setPortfolioText] = useState(PORTFOLIO_DEFAULT)
  const [report, setReport] = useState<Report | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  async function runSim() {
    setLoading(true)
    setError(null)
    setReport(null)

    try {
      let portfolio: unknown

      try {
        portfolio = JSON.parse(portfolioText)
      } catch {
        throw new Error('Portfolio must contain valid JSON.')
      }

      if (!Array.isArray(portfolio)) {
        throw new Error('Portfolio must be a JSON array.')
      }

      const res = await fetch(`${BASE}/simulate`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          old_spec_name: specName,
          new_spec_text: newSpecText,
          portfolio,
        }),
      })

      const text = await res.text()

      if (!text) {
        throw new Error(`Empty response (HTTP ${res.status})`)
      }

      let data: Report | { error?: string }

      try {
        data = JSON.parse(text)
      } catch {
        throw new Error(`Invalid JSON response (HTTP ${res.status})`)
      }

      if (!res.ok) {
        throw new Error(
          'error' in data && data.error
            ? data.error
            : `HTTP ${res.status}`,
        )
      }

      setReport(data as Report)
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : 'Something went wrong.')
    } finally {
      setLoading(false)
    }
  }

  const verdict = report ? VERDICT_CONFIG[report.verdict] : null

  return (
    <div className="p-8 min-h-full flex flex-col">
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-xl font-bold text-slate-900">
            Portfolio simulation
          </h1>
          <p className="text-sm text-slate-500 mt-0.5">
            Replay your portfolio through a proposed spec change before
            deploying.
          </p>
        </div>

        <button
          onClick={runSim}
          disabled={loading}
          className="flex items-center gap-2 bg-slate-900 hover:bg-slate-800 text-white text-sm font-semibold px-5 py-2.5 rounded-xl transition-colors disabled:opacity-50 shadow-sm"
        >
          {loading ? (
            <>
              <svg
                className="animate-spin w-4 h-4"
                viewBox="0 0 24 24"
                fill="none"
              >
                <circle
                  cx="12"
                  cy="12"
                  r="10"
                  stroke="currentColor"
                  strokeWidth="3"
                  strokeOpacity="0.25"
                />
                <path
                  d="M12 2a10 10 0 0110 10"
                  stroke="currentColor"
                  strokeWidth="3"
                  strokeLinecap="round"
                />
              </svg>
              Simulating...
            </>
          ) : (
            <>
              <svg
                width="14"
                height="14"
                viewBox="0 0 14 14"
                fill="currentColor"
              >
                <path d="M3 2l9 5-9 5V2z" />
              </svg>
              Run simulation
            </>
          )}
        </button>
      </div>

      {/* Spec name */}
      <div className="mb-4">
        <label className="text-xs font-semibold text-slate-500 uppercase tracking-widest block mb-2">
          Current specification (old version)
        </label>

        <input
          value={specName}
          onChange={(e) => setSpecName(e.target.value)}
          className="w-full max-w-xs text-sm border border-slate-200 rounded-xl px-3 py-2.5 bg-white text-slate-900 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
        />
      </div>

      {/* Editors */}
      <div className="grid grid-cols-2 gap-4 mb-4">
        <div className="bg-white border border-slate-200 rounded-2xl shadow-sm overflow-hidden">
          <div className="flex items-center justify-between px-4 py-3 border-b border-slate-100">
            <span className="text-sm font-semibold text-slate-900">
              New spec (.proof)
            </span>

            <span className="text-xs font-semibold text-blue-600 bg-blue-50 border border-blue-100 px-2.5 py-1 rounded-full">
              Proposed
            </span>
          </div>

          <textarea
            value={newSpecText}
            onChange={(e) => setNewSpecText(e.target.value)}
            className="w-full h-72 px-4 py-3 font-mono text-xs text-slate-700 bg-slate-50 resize-none focus:outline-none focus:bg-white transition-colors leading-relaxed"
            spellCheck={false}
          />
        </div>

        <div className="bg-white border border-slate-200 rounded-2xl shadow-sm overflow-hidden">
          <div className="flex items-center justify-between px-4 py-3 border-b border-slate-100">
            <span className="text-sm font-semibold text-slate-900">
              Portfolio
            </span>

            <span className="text-xs font-semibold text-slate-500 bg-slate-100 px-2.5 py-1 rounded-full">
              JSON array of events
            </span>
          </div>

          <textarea
            value={portfolioText}
            onChange={(e) => setPortfolioText(e.target.value)}
            className="w-full h-72 px-4 py-3 font-mono text-xs text-slate-700 bg-slate-50 resize-none focus:outline-none focus:bg-white transition-colors leading-relaxed"
            spellCheck={false}
          />
        </div>
      </div>

      {/* Error */}
      {error && (
        <div className="bg-red-50 border border-red-200 rounded-xl px-4 py-3 text-sm text-red-700 mb-4">
          {error}
        </div>
      )}

      {/* Results */}
      {report && (
        <div className="space-y-4">
          {/* Impact cards */}
          <div className="grid grid-cols-4 gap-3">
            {[
              {
                label: 'Total customers',
                value: report.customers_total,
                color: '#3B82F6',
              },
              {
                label: 'Worse off',
                value: report.customers_worse,
                color: '#EF4444',
              },
              {
                label: 'Better off',
                value: report.customers_better,
                color: '#10B981',
              },
              {
                label: 'Unaffected',
                value: report.customers_neutral,
                color: '#94A3B8',
              },
            ].map(({ label, value, color }) => (
              <div
                key={label}
                className="bg-white border border-slate-200 rounded-xl p-4 shadow-sm"
              >
                <div className="text-xs font-semibold text-slate-400 uppercase tracking-widest mb-1">
                  {label}
                </div>

                <div
                  className="text-2xl font-bold tabular"
                  style={{ color }}
                >
                  {value}
                </div>
              </div>
            ))}
          </div>

          {/* Financial impact + regulatory flags */}
          <div className="grid grid-cols-2 gap-4">
            <div className="bg-white border border-slate-200 rounded-2xl p-5 shadow-sm">
              <div className="text-xs font-semibold text-slate-500 uppercase tracking-widest mb-3">
                Financial impact
              </div>

              <div className="space-y-2">
                {[
                  {
                    label: 'Daily aggregate',
                    value: `£${report.daily_delta}`,
                  },
                  {
                    label: 'Monthly aggregate',
                    value: `£${report.monthly_delta}`,
                  },
                  ...(report.avg_delta_worse
                    ? [
                        {
                          label: 'Avg impact per affected customer',
                          value: `£${report.avg_delta_worse}/day`,
                        },
                      ]
                    : []),
                ].map(({ label, value }) => (
                  <div
                    key={label}
                    className="flex items-center justify-between py-1.5 border-b border-slate-50 last:border-0"
                  >
                    <span className="text-sm text-slate-500">{label}</span>
                    <span className="text-sm font-semibold text-slate-900 tabular">
                      {value}
                    </span>
                  </div>
                ))}
              </div>
            </div>

            {report.regulatory_flags.length > 0 ? (
              <div className="bg-white border border-slate-200 rounded-2xl p-5 shadow-sm">
                <div className="text-xs font-semibold text-slate-500 uppercase tracking-widest mb-3">
                  Regulatory flags
                </div>

                <div className="space-y-3">
                  {report.regulatory_flags.map((flag, i) => {
                    const isBlock = flag.severity === 'Block'
                    const isReview = flag.severity === 'Review'

                    return (
                      <div key={`${flag.rule}-${i}`} className="flex gap-3 items-start">
                        <div
                          className={`w-6 h-6 rounded-lg flex items-center justify-center flex-shrink-0 mt-0.5 ${
                            isBlock
                              ? 'bg-red-50'
                              : isReview
                                ? 'bg-amber-50'
                                : 'bg-blue-50'
                          }`}
                        >
                          <span
                            className={`text-xs font-bold ${
                              isBlock
                                ? 'text-red-600'
                                : isReview
                                  ? 'text-amber-600'
                                  : 'text-blue-600'
                            }`}
                          >
                            {isBlock ? '✗' : isReview ? '!' : 'i'}
                          </span>
                        </div>

                        <div>
                          <div className="text-sm font-semibold text-slate-900">
                            {flag.rule}
                          </div>

                          <div className="text-xs text-slate-500 mt-0.5">
                            {flag.description}
                          </div>

                          <div className="text-xs text-slate-500">
                            {flag.action}
                          </div>

                          {flag.notice_days !== null && (
                            <div className="text-xs font-semibold text-amber-600 mt-1">
                              {flag.notice_days}-day notice required
                            </div>
                          )}
                        </div>
                      </div>
                    )
                  })}
                </div>
              </div>
            ) : (
              <div className="bg-white border border-slate-200 rounded-2xl p-5 shadow-sm flex items-center gap-3">
                <div className="w-10 h-10 bg-emerald-50 rounded-xl flex items-center justify-center flex-shrink-0">
                  <svg
                    width="18"
                    height="18"
                    viewBox="0 0 18 18"
                    fill="none"
                    stroke="#10B981"
                    strokeWidth="2"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                  >
                    <path d="M3 9l5 5 7-7" />
                  </svg>
                </div>

                <div>
                  <div className="text-sm font-semibold text-slate-900">
                    No regulatory flags
                  </div>

                  <div className="text-xs text-slate-500 mt-0.5">
                    This change does not trigger any FCA obligations.
                  </div>
                </div>
              </div>
            )}
          </div>

          {/* Verdict */}
          {verdict && (
            <div
              className="rounded-2xl px-6 py-4 flex items-center justify-between border shadow-sm"
              style={{
                backgroundColor: verdict.bg,
                borderColor: verdict.border,
              }}
            >
              <div>
                <div
                  className="text-sm font-bold"
                  style={{ color: verdict.color }}
                >
                  {verdict.label}
                </div>

                {report.avg_delta_worse && (
                  <div
                    className="text-xs mt-0.5"
                    style={{
                      color: verdict.color,
                      opacity: 0.8,
                    }}
                  >
                    Average customer impact: £{report.avg_delta_worse}/day
                  </div>
                )}
              </div>

              <a
                href="/audit"
                className="text-xs font-semibold px-4 py-2 rounded-lg transition-colors"
                style={{
                  backgroundColor: verdict.color,
                  color: 'white',
                }}
              >
                Sign off in audit trail
              </a>
            </div>
          )}
        </div>
      )}
    </div>
  )
}
