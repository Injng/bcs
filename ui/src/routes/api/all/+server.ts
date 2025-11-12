import type { RequestHandler } from '@sveltejs/kit';

const DEFAULT_API = 'http://127.0.0.1:8000';

export const GET: RequestHandler = async ({ fetch }) => {
	const base = DEFAULT_API;
	const res = await fetch(`${base}/classes/all`);
	return new Response(res.body, {
		status: res.status,
		headers: { 'content-type': res.headers.get('content-type') || 'application/json' }
	});
};


