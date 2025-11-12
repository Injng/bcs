import type { RequestHandler } from '@sveltejs/kit';

const DEFAULT_API = 'http://127.0.0.1:8000';

export const POST: RequestHandler = async ({ request, fetch }) => {
	const body = await request.json().catch(() => ({ keywords: '' }));
	const base = DEFAULT_API;
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


