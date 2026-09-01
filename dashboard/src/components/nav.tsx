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
    { href: '/specs',     label: 'Specs' },
    { href: '/simulate',  label: 'Simulate'},
    { href: '/audit',     label: 'Audit'},
    { href: '/diff',      label: 'Diff',      stub: true },
  ]

  return (
    <nav className="flex items-center border-b border-gray-200 px-6 py-3 gap-1 bg-white">
      <span className="font-mono text-sm font-medium tracking-widest text-gray-900 mr-4">
        PROOF
      </span>
      {links.map(l => (
        <Link
          key={l.href}
          href={l.href}
          className={[
            'text-sm px-3 py-1.5 rounded transition-colors',
            path.startsWith(l.href)
              ? 'bg-gray-100 text-gray-900'
              : l.stub
              ? 'text-gray-400 cursor-not-allowed pointer-events-none'
              : 'text-gray-500 hover:text-gray-900',
          ].join(' ')}
        >
          {l.label}
        </Link>
      ))}
      {data && data.divergences > 0 && (
        <div className="ml-auto flex items-center gap-2 text-xs text-red-600 font-medium">
          <span className="w-2 h-2 rounded-full bg-red-600 animate-pulse" />
          {data.divergences} divergence{data.divergences !== 1 ? 's' : ''}
        </div>
      )}
      {data && data.divergences === 0 && (
        <div className="ml-auto flex items-center gap-2 text-xs text-green-600">
          <span className="w-2 h-2 rounded-full bg-green-600" />
          All clean
        </div>
      )}
    </nav>
  )
}