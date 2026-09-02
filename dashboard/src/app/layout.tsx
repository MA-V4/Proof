import type { Metadata } from 'next'
import { Inter } from 'next/font/google'
import './globals.css'
import { Sidebar } from '@/components/sidebar'

const inter = Inter({ subsets: ['latin'], variable: '--font-inter' })

export const metadata: Metadata = {
  title: 'PROOF',
  description: 'Financial logic, verified pure.',
}

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en" className={inter.variable}>
      <body className="font-sans bg-slate-50 antialiased">
        <div className="flex h-screen overflow-hidden">
          <Sidebar />
          <div className="flex-1 flex flex-col overflow-hidden">
            <div className="flex-1 overflow-auto">
              {children}
            </div>
            <footer className="bg-slate-900 border-t border-slate-800 px-8 h-12 flex items-center justify-between flex-shrink-0">
              <div className="flex items-center gap-2 text-xs text-slate-400">
                <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
                  <path d="M7 1L12.2 4v6L7 13 1.8 10V4L7 1z" stroke="#3B82F6" strokeWidth="1.2"/>
                </svg>
                <span>PROOF is continuously verifying your financial logic in real-time</span>
              </div>
              <div className="flex items-center gap-6 text-xs text-slate-500">
                <span>Audit ready</span>
                <span className="text-slate-700">·</span>
                <span>FCA compliant</span>
                <span className="text-slate-700">·</span>
                <span>Immutable</span>
              </div>
              <button className="text-xs bg-blue-600 hover:bg-blue-700 text-white px-4 py-1.5 rounded-lg font-medium transition-colors flex items-center gap-2">
                Export audit package
                <svg width="12" height="12" viewBox="0 0 12 12" fill="currentColor">
                  <path d="M6 1v7M3 5l3 3 3-3M1 10h10" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" fill="none"/>
                </svg>
              </button>
            </footer>
          </div>
        </div>
      </body>
    </html>
  )
}