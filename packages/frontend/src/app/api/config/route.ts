import { NextResponse } from 'next/server'

export async function GET() {
  // Return a mock config. In production this should proxy the real backend.
  const config = {
    feePercent: 1.5,
  }

  return NextResponse.json(config)
}
