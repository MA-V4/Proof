'use client'

import Link from 'next/link'
import { usePathname } from 'next/navigation'
import { useState, useEffect } from 'react'
import useSWR from 'swr'
import { api } from '@/lib/api'

const Logo = () => (
  <svg width="30" height="30" viewBox="0 0 30 30" fill="none">
    <path d="M15 2L26.26 8.5v13L15 28 3.74 21.5v-13L15 2z" fill="#3B82F6" fillOpacity="0.15" stroke="#3B82F6" strokeWidth="1.5"/>
    <path d="M15 8l8 4.5v9L15 26l-8-4.5v-9L15 8z" fill="#3B82F6" fillOpacity="0.25"/>
    <circle cx="15" cy="15" r="3" fill="#3B82F6"/>
  </svg>
)

function NavIcon({ type }: { type: string }) {
  const cls = "w-4 h-4"
  if (type === 'dashboard') return (
    <svg className={cls} viewBox="0 0 16 16" fill="currentColor">
      <rect x="1" y="1" width="6" height="6" rx="1.5"/>
      <rect x="9" y="1" width="6" height="6" rx="1.5"/>
      <rect x="1" y="9" width="6" height="6" rx="1.5"/>
      <rect x="9" y="9" width="6" height="6" rx="1.5"/>
    </svg>
  )
  if (type === 'specs') return (
    <svg className={cls} viewBox="0 0 16 16" fill="currentColor">
      <rect x="1" y="2" width="14" height="2" rx="1"/>
      <rect x="1" y="7" width="10" height="2" rx="1"/>
      <rect x="1" y="12" width="12" height="2" rx="1"/>
    </svg>
  )
  if (type === 'simulate') return (
    <svg className={cls} viewBox="0 0 16 16" fill="currentColor">
      <path d="M8 1a7 7 0 100 14A7 7 0 008 1zM6.5 5.5l5 2.5-5 2.5V5.5z"/>
    </svg>
  )
  if (type === 'audit') return (
    <svg className={cls} viewBox="0 0 16 16" fill="currentColor">
      <path d="M3 1h10a1 1 0 011 1v12a1 1 0 01-1 1H3a1 1 0 01-1-1V2a1 1 0 011-1zm1 3v1h8V4H4zm0 3v1h8V7H4zm0 3v1h5v-1H4z"/>
    </svg>
  )
  if (type === 'diff') return (
    <svg className={cls} viewBox="0 0 16 16" fill="currentColor">
      <path d="M2 3h5v2H2V3zm7 0h5v2H9V3zM2 7h5v2H2V7zm9 1V6l3 2-3 2V8zm-2 3H2v2h7v-2zm2 1v-2l3 2-3 2v-2z"/>
    </svg>
  )
  return null
}

const navItems = [
  { href: '/dashboard', label: 'Dashboard',      icon: 'dashboard' },
  { href: '/specs',     label: 'Specifications', icon: 'specs'     },
  { href: '/simulate',  label: 'Simulate',       icon: 'simulate'  },
  { href: '/audit',     label: 'Audit trail',    icon: 'audit'     },
  { href: '/diff',      label: 'Diff engine',    icon: 'diff'      },
]

export function Sidebar() {
  const path = usePathname()
  const { data: health } = useSWR('health', api.health, { refreshInterval: 5000 })

  const [time, setTime] = useState('')

  useEffect(() => {
    const fmt = () =>
      new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' })
    setTime(fmt())
    const id = setInterval(() => setTime(fmt()), 1000)
    return () => clearInterval(id)
  }, [])

  const total  = health?.events_verified ?? 0
  const divs   = health?.divergences ?? 0
  const okRate = total > 0
    ? (((total - divs) / total) * 100).toFixed(2)
    : '100.00'

  return (
    <aside className="w-56 bg-white border-r border-slate-200 flex flex-col h-screen flex-shrink-0">
      <div className="px-5 py-4 flex items-center gap-2.5 border-b border-slate-100">
        <Logo />
        <span className="font-bold text-slate-900 text-base tracking-wider">PROOF</span>
      </div>

      <nav className="flex-1 px-3 py-4 space-y-0.5 overflow-y-auto">
        {navItems.map(item => {
          const active = path.startsWith(item.href)
          return (
            <Link
              key={item.href}
              href={item.href}
              className={[
                'flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm transition-all duration-100',
                active
                  ? 'bg-blue-50 text-blue-700 font-semibold'
                  : 'text-slate-500 hover:bg-slate-50 hover:text-slate-800',
              ].join(' ')}
            >
              <span className={active ? 'text-blue-600' : 'text-slate-400'}>
                <NavIcon type={item.icon} />
              </span>
              {item.label}
            </Link>
          )
        })}
      </nav>

      <div className="mx-3 mb-3 bg-slate-900 rounded-xl p-4">
        <div className="text-xs font-semibold text-slate-500 uppercase tracking-widest mb-2">
          System integrity
        </div>
        <div className="text-2xl font-bold text-emerald-400 tabular mb-1">{okRate}%</div>
        <svg width="100%" height="28" viewBox="0 0 140 28" preserveAspectRatio="none" className="mb-2">
          <path
            d="M0,20 C10,18 20,22 30,16 C40,10 50,14 60,12 C70,10 80,8 90,10 C100,12 110,6 120,8 C130,10 135,7 140,6"
            fill="none" stroke="#34D399" strokeWidth="1.8" strokeLinecap="round"
          />
        </svg>
        <div className="text-xs text-slate-400">All systems verified</div>
        <div className="text-xs text-slate-600 mt-0.5">
          {time} · Live
        </div>
      </div>

      <div className="px-4 py-3 border-t border-slate-100 flex items-center gap-3">
        <div className="w-8 h-8 rounded-full bg-blue-600 flex items-center justify-center text-white text-xs font-bold flex-shrink-0">
          A
        </div>
        <div className="min-w-0">
          <div className="text-sm font-semibold text-slate-900 truncate">ADMIN</div>
          <div className="text-xs text-slate-500">Administrator</div>
        </div>
      </div>
    </aside>
  )
}