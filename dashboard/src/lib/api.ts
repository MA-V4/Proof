const BASE = '/api/proxy'
const API_KEY = process.env.API_KEY  ?? ''

function authHeaders(extra: Record<string, string> = {}): Record<string, string> {
  return {
    ...(API_KEY ? { 'X-API-Key': API_KEY } : {}),
    ...extra,
  }
}

const get = async (path: string) => {
  const r    = await fetch(`${BASE}${path}`)
  const text = await r.text()
  if (!text) throw new Error(`Empty response (HTTP ${r.status})`)
  const data = JSON.parse(text)
  if (!r.ok)  throw new Error(data.error ?? `HTTP ${r.status}`)
  return data
}

const post = async (path: string, body: unknown) => {
  const r    = await fetch(`${BASE}${path}`, {
    method:  'POST',
    headers: { 'Content-Type': 'application/json' },
    body:    JSON.stringify(body),
  })
  const text = await r.text()
  if (!text) throw new Error(`Empty response (HTTP ${r.status})`)
  const data = JSON.parse(text)
  if (!r.ok)  throw new Error(data.error ?? `HTTP ${r.status}`)
  return data
}

const del = (path: string) =>
  fetch(`${BASE}${path}`, { method: 'DELETE' }).then(() => {})

export type SpecSummary = {
  name:        string
  divergences: number
  status:      'clean' | 'divergence'
}

export type FieldDiff = {
  field:        string
  spec_value:   string
  system_value: string
  delta:        string | null
  delta_pct:    string | null
}

export type Output = {
  applied_tier: string | null
  rate_applied: string | null
  amount:       string | null
  reasoning:    string[]
}

export type Divergence = {
  id:            string
  detected_at:   string
  customer_id:   string
  spec_name:     string
  event_type:    string
  balance:       string
  spec_output:   Output
  system_output: Output
  diffs:         FieldDiff[]
}

export type RecentEvent = {
  customer_id: string
  spec_name:   string
  event_type:  string
  ok:          boolean
  timestamp:   string
}

export type Health = {
  status:          string
  specs:           number
  events_verified: number
  divergences:     number
}

export type AuditEntryKind =
  | { kind: 'spec_loaded';         source: string }
  | { kind: 'verified';            customer_id: string; ok: boolean }
  | { kind: 'divergence_detected'; divergence_id: string }
  | { kind: 'divergence_resolved'; divergence_id: string }
  | { kind: 'simulation_run';      verdict: string }
  | { kind: 'spec_signed_off';     approver: string }

export type AuditEntry = {
  id:        string
  timestamp: string
  spec_name: string
  spec_hash: string
  actor:     string
} & AuditEntryKind

export type DiffItem =
  | { type: 'base_rate_changed';        old: string; new: string; delta: string }
  | { type: 'tier_threshold_changed';   tier_index: number; old_threshold: string; new_threshold: string }
  | { type: 'tier_rate_changed';        tier_index: number; old_rate: string; new_rate: string }
  | { type: 'tier_added';               tier_index: number }
  | { type: 'tier_removed';             tier_index: number }
  | { type: 'promotional_rate_changed'; old: string; new: string }
  | { type: 'obligation_changed';       field: string; old: string; new: string }

export const api = {
  health:       (): Promise<Health>        => get('/health'),
  specs:        (): Promise<SpecSummary[]> => get('/specs'),
  recentEvents: (): Promise<RecentEvent[]> => get('/events/recent'),
  audit:        (): Promise<AuditEntry[]>  => get('/audit'),
  specAudit:    (name: string): Promise<AuditEntry[]>  => get(`/specs/${name}/audit`),
  divergences:  (name: string): Promise<Divergence[]>  => get(`/specs/${name}/divergences`),
  exportFca:    (name: string): Promise<unknown>       => get(`/specs/${name}/audit/export`),

  allDivergences: async (): Promise<Divergence[]> => {
    const specs: SpecSummary[] = await get('/specs')
    const all = await Promise.all(
      specs.filter(s => s.divergences > 0).map(s => get(`/specs/${s.name}/divergences`))
    )
    return all.flat()
  },

  resolve: (specName: string, id: string): Promise<void> =>
    del(`/specs/${specName}/divergences/${id}`),

  signOff: (specName: string, approver: string): Promise<void> =>
    post(`/specs/${specName}/signoff`, { approver }),

  diff: (body: { old_spec_text: string; new_spec_text: string }): Promise<DiffItem[]> =>
    post('/diff', body),
}