import type { RequestHandler } from '@sveltejs/kit';
import { API_BASE } from '$env/static/private';

export const POST: RequestHandler = async ({ request, fetch }) => {
  const body = await request.json().catch(() => ({ keywords: '' }));
  const base = API_BASE;
  const res = await fetch(`${base}/classes/search`, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(body)
  });
  return new Response(res.body, {
    status: res.status,
    headers: { 'content-type': res.headers.get('content-type') || 'application/json' }
  });
};


