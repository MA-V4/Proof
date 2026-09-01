const BASE = process.env.NEXT_PUBLIC_API_URL ?? 'http://localhost:3001'

export type SpecSummary = {
  name: string
  divergences: number
  status: 'clean' | 'divergence'
}

export type FieldDiff = {
  field: string
  spec_value: string
  system_value: string
  delta: string | null
  delta_pct: string | null
}

export type Output = {
  applied_tier: string | null
  rate_applied: string | null
  amount: string | null
  reasoning: string[]
}

export type Divergence = {
  id: string
  detected_at: string
  customer_id: string
  spec_name: string
  event_type: string
  balance: string
  spec_output: Output
  system_output: Output
  diffs: FieldDiff[]
}

export type RecentEvent = {
  customer_id: string
  spec_name: string
  event_type: string
  ok: boolean
  timestamp: string
}

export type Health = {
  status: string
  specs: number
  events_verified: number
  divergences: number
}

const get = (path: string) => fetch(`${BASE}${path}`).then(r => r.json())

export const api = {
  health:      (): Promise<Health>          => get('/health'),
  specs:       (): Promise<SpecSummary[]>   => get('/specs'),
  divergences: (name: string): Promise<Divergence[]> => get(`/specs/${name}/divergences`),
  allDivergences: async (): Promise<Divergence[]> => {
    const specs: SpecSummary[] = await get('/specs')
    const divsBySpec = await Promise.all(
      specs.filter(s => s.divergences > 0).map(s => get(`/specs/${s.name}/divergences`))
    )
    return divsBySpec.flat()
  },
  recentEvents: (): Promise<RecentEvent[]> => get('/events/recent'),
  resolve: (specName: string, id: string): Promise<void> =>
    fetch(`${BASE}/specs/${specName}/divergences/${id}`, { method: 'DELETE' }).then(() => {}),
}