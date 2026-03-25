import { NextRequest, NextResponse } from "next/server";

export async function GET(
    request: NextRequest,
    { params }: { params: Promise<{ id: string }> }
) {
    const { id } = await params;
    
    // In a real app, this would call the backend:
    // const res = await fetch(`${process.env.BACKEND_URL}/calls/${id}/odds`);
    // return res;

    // For now, return mock odds based on ID or random
    const mockOdds: Record<string, any> = {
        "1": { yes: 1.5, no: 2.8, totalPool: 23500 },
        "2": { yes: 2.1, no: 1.9, totalPool: 37000 },
        "3": { yes: 1.2, no: 5.4, totalPool: 27000 },
    };

    const odds = mockOdds[id] || { yes: 2.0, no: 2.0, totalPool: 0 };

    return NextResponse.json(odds);
}
