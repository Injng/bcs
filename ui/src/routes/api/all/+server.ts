import type { RequestHandler } from '@sveltejs/kit';
import { API_BASE } from '$env/static/private';

export const GET: RequestHandler = async ({ fetch }) => {
  const base = API_BASE;
  const res = await fetch(`${base}/classes/all`);
  return new Response(res.body, {
    status: res.status,
    headers: { 'content-type': res.headers.get('content-type') || 'application/json' }
  });
};


