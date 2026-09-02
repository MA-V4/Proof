import { NextRequest, NextResponse } from 'next/server'

const BACKEND = process.env.PROOF_API_URL  ?? 'http://localhost:3001'
const API_KEY = process.env.PROOF_API_KEY  ?? ''

export async function GET(req: NextRequest, { params }: { params: { path: string[] } }) {
  return proxy(req, params.path, 'GET')
}

export async function POST(req: NextRequest, { params }: { params: { path: string[] } }) {
  return proxy(req, params.path, 'POST', await req.text())
}

export async function DELETE(req: NextRequest, { params }: { params: { path: string[] } }) {
  return proxy(req, params.path, 'DELETE')
}

async function proxy(req: NextRequest, path: string[], method: string, body?: string) {
  const url      = `${BACKEND}/${path.join('/')}${req.nextUrl.search}`
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    ...(API_KEY ? { 'X-API-Key': API_KEY } : {}),
  }

  const res = await fetch(url, { method, headers, body })
  const text = await res.text()

  return new NextResponse(text, {
    status:  res.status,
    headers: { 'Content-Type': 'application/json' },
  })
}