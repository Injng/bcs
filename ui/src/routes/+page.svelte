<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import type { Course } from '$lib/types';
	import { normalizeCourse } from '$lib/types';
	import SearchIcon from 'lucide-svelte/icons/search';
	import LoaderIcon from 'lucide-svelte/icons/loader-2';
	import CourseCard from '$lib/components/CourseCard.svelte';

	let query = '';
	let results: Course[] = [];
	let loading = false;
	let error: string | null = null;

	let debounceTimer: ReturnType<typeof setTimeout> | null = null;
	let inflightAbort: AbortController | null = null;

	function handleInput(event: Event) {
		const value = (event.target as HTMLInputElement).value;
		query = value;
		triggerSearch();
	}

	function triggerSearch() {
		if (debounceTimer) clearTimeout(debounceTimer);
		debounceTimer = setTimeout(doSearch, 250);
	}

	async function doSearch() {
		if (!query.trim()) {
			await fetchAll();
			return;
		}

		if (inflightAbort) inflightAbort.abort();
		inflightAbort = new AbortController();

		loading = true;
		error = null;

		try {
			const res = await fetch('/api/search', {
				method: 'POST',
				headers: { 'content-type': 'application/json' },
				body: JSON.stringify({ keywords: query }),
				signal: inflightAbort.signal
			});
			if (!res.ok) throw new Error(`Request failed: ${res.status}`);
			const data: Course[] = await res.json();
			results = data.map(normalizeCourse);
		} catch (e) {
			if ((e as Error).name !== 'AbortError') {
				error = 'Something went wrong. Please try again.';
				results = [];
			}
		} finally {
			loading = false;
		}
	}

	async function fetchAll() {
		if (inflightAbort) inflightAbort.abort();
		inflightAbort = new AbortController();
		loading = true;
		error = null;
		try {
			const res = await fetch('/api/all', { signal: inflightAbort.signal });
			if (!res.ok) throw new Error(`Request failed: ${res.status}`);
			const data: Course[] = await res.json();
			results = data.map(normalizeCourse);
		} catch (e) {
			if ((e as Error).name !== 'AbortError') {
				error = 'Something went wrong. Please try again.';
				results = [];
			}
		} finally {
			loading = false;
		}
	}

	onMount(() => {
		fetchAll();
	});

	onDestroy(() => {
		if (debounceTimer) clearTimeout(debounceTimer);
		if (inflightAbort) inflightAbort.abort();
	});
</script>

<div class="min-h-screen bg-zinc-50 text-zinc-900">
	<div class="mx-auto max-w-5xl px-4 py-10">
		<div class="mb-8">
			<h1 class="text-2xl font-semibold tracking-tight">Search Courses</h1>
			<p class="mt-1 text-sm text-zinc-600">Type a keyword: code, title, subtitle, or description.</p>
		</div>

		<div class="relative">
			<input
				type="search"
				placeholder="e.g. CS 61A, Algorithms, AI..."
				class="w-full rounded-none border border-zinc-300 bg-white px-4 py-3 pr-11 text-[15px] outline-none ring-4 ring-transparent transition focus:border-zinc-400 focus:ring-blue-100"
				on:input={handleInput}
				autocomplete="off"
				spellcheck="false"
				aria-label="Search courses"
			/>
			<div class="pointer-events-none absolute inset-y-0 right-3 flex items-center">
				{#if loading}
					<LoaderIcon class="size-5 animate-spin text-zinc-400" />
				{:else}
					<SearchIcon class="size-5 text-zinc-400" />
				{/if}
			</div>
		</div>

		{#if error}
			<div class="mt-4 rounded-none border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-800">{error}</div>
		{/if}

		{#if !loading && !error && query.trim().length > 0 && results.length === 0}
			<p class="mt-6 text-sm text-zinc-600">No results for “{query}”.</p>
		{/if}

		{#if results.length > 0}
			<div class="mt-6 grid grid-cols-1 gap-4">
				{#each results as course}
					<CourseCard {course} />
				{/each}
			</div>
		{/if}
	</div>
</div>
