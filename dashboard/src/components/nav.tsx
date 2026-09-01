'use client'

import Link from 'next/link'
import { usePathname } from 'next/navigation'
import useSWR from 'swr'
import { api } from '@/lib/api'

export function Nav() {
  const path = usePathname()
  const { data } = useSWR('health', api.health, { refreshInterval: 5000 })

  const links = [
    { href: '/dashboard', label: 'Dashboard' },
    { href: '/specs',     label: 'Specs'     },
    { href: '/simulate',  label: 'Simulate'  },
    { href: '/audit',     label: 'Audit'     },
    { href: '/diff',      label: 'Diff'      },
  ]

  const divCount = data?.divergences ?? 0

  return (
    <nav className="bg-white border-b border-slate-200 sticky top-0 z-50">
      <div className="max-w-6xl mx-auto px-6 h-14 flex items-center gap-8">
        <span className="font-mono font-semibold text-slate-900 text-sm tracking-widest select-none">
          PROOF
        </span>
        <div className="flex items-center gap-0.5">
          {links.map(l => {
            const active = path.startsWith(l.href)
            return (
              <Link
                key={l.href}
                href={l.href}
                className={[
                  'text-sm px-3 py-1.5 rounded-md transition-colors duration-100',
                  active
                    ? 'bg-slate-100 text-slate-900 font-medium'
                    : 'text-slate-500 hover:text-slate-800 hover:bg-slate-50',
                ].join(' ')}
              >
                {l.label}
              </Link>
            )
          })}
        </div>
        <div className="ml-auto">
          {divCount > 0 ? (
            <div className="flex items-center gap-2">
              <span className="relative flex h-2 w-2">
                <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-red-400 opacity-75" />
                <span className="relative inline-flex rounded-full h-2 w-2 bg-red-500" />
              </span>
              <span className="text-xs font-medium text-red-600 tabular">
                {divCount} divergence{divCount !== 1 ? 's' : ''}
              </span>
            </div>
          ) : (
            <div className="flex items-center gap-2">
              <span className="h-2 w-2 rounded-full bg-emerald-500" />
              <span className="text-xs font-medium text-emerald-600">All clean</span>
            </div>
          )}
        </div>
      </div>
    </nav>
  )
}