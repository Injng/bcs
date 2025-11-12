<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import type { Course } from "$lib/types";
  import { normalizeCourse } from "$lib/types";
  import SearchIcon from "lucide-svelte/icons/search";
  import LoaderIcon from "lucide-svelte/icons/loader-2";
  import CourseCard from "$lib/components/CourseCard.svelte";

  let query = "";
  let results: Course[] = [];
  let loading = false;
	let loadingMore = false;
  let error: string | null = null;
	let hasMore = false;

  let debounceTimer: ReturnType<typeof setTimeout> | null = null;
  let inflightAbort: AbortController | null = null;
	let sentinelEl: HTMLElement | null = null;
	let observer: IntersectionObserver | null = null;
	let prevSentinelEl: HTMLElement | null = null;

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
		// If query is empty, reset UI and skip request
		if (query.trim().length === 0) {
			if (inflightAbort) inflightAbort.abort();
			results = [];
			error = null;
			loading = false;
			hasMore = false;
			return;
		}

    if (inflightAbort) inflightAbort.abort();
    inflightAbort = new AbortController();

    loading = true;
    error = null;

    try {
      const res = await fetch("/api/search", {
        method: "POST",
        headers: { "content-type": "application/json" },
				body: JSON.stringify({ keywords: query, offset: 0 }),
        signal: inflightAbort.signal,
      });
      if (!res.ok) throw new Error(`Request failed: ${res.status}`);
			const data: Course[] = await res.json();
			results = data.map(normalizeCourse);
			hasMore = data.length > 0;
    } catch (e) {
      if ((e as Error).name !== "AbortError") {
        error = "Something went wrong. Please try again.";
        results = [];
				hasMore = false;
      }
    } finally {
      loading = false;
    }
  }

	async function loadMore() {
		// Guards
		if (loading || loadingMore || !hasMore) return;
		if (query.trim().length === 0) return;

		loadingMore = true;
		try {
			const currentOffset = results.length;
			const currentQuery = query;
			const res = await fetch("/api/search", {
				method: "POST",
				headers: { "content-type": "application/json" },
				body: JSON.stringify({ keywords: currentQuery, offset: currentOffset }),
			});
			if (!res.ok) throw new Error(`Request failed: ${res.status}`);
			const data: Course[] = await res.json();
			// If user changed the query mid-flight, drop these results
			if (currentQuery !== query) return;
			if (data.length === 0) {
				hasMore = false;
				return;
			}
			const mapped = data.map(normalizeCourse);
			results = [...results, ...mapped];
		} catch (_e) {
		} finally {
			loadingMore = false;
		}
	}

	onMount(() => {
		observer = new IntersectionObserver(
			(entries) => {
				for (const entry of entries) {
					if (entry.isIntersecting) {
						loadMore();
					}
				}
			},
			{
				root: null,
				rootMargin: "0px",
				threshold: 0.1,
			}
		);

		// Observe if sentinel is already rendered
		if (sentinelEl) observer.observe(sentinelEl);
	});

	// Re-bind observer if sentinel element changes
	$: {
		if (observer && sentinelEl !== prevSentinelEl) {
			if (prevSentinelEl) observer.unobserve(prevSentinelEl);
			if (sentinelEl) observer.observe(sentinelEl);
			prevSentinelEl = sentinelEl;
		}
	}

  onDestroy(() => {
    if (debounceTimer) clearTimeout(debounceTimer);
    if (inflightAbort) inflightAbort.abort();
		if (observer) {
			observer.disconnect();
			observer = null;
		}
  });
</script>

<div class="min-h-screen bg-zinc-50 text-zinc-900">
  <div class="mx-auto max-w-5xl px-4 py-10">
    <div class="mb-8">
      <h1 class="text-2xl font-semibold tracking-tight">
        (Better) Berkeley Course Search
      </h1>
      <p class="mt-1 text-sm text-zinc-600">
        Type a keyword: code, title, subtitle, or description.
      </p>
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
      <div
        class="pointer-events-none absolute inset-y-0 right-3 flex items-center"
      >
        {#if loading}
          <LoaderIcon class="size-5 animate-spin text-zinc-400" />
        {:else}
          <SearchIcon class="size-5 text-zinc-400" />
        {/if}
      </div>
    </div>

    {#if error}
      <div
        class="mt-4 rounded-none border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-800"
      >
        {error}
      </div>
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
			<!-- Infinite scroll sentinel -->
			{#if hasMore}
				<div
					class="flex items-center justify-center py-6 text-sm text-zinc-500"
					bind:this={sentinelEl}
				>
					{#if loadingMore}
						<LoaderIcon class="mr-2 size-4 animate-spin text-zinc-400" />
					{/if}
					<span class="select-none">{loadingMore ? "Loading more…" : "Scroll to load more"}</span>
				</div>
			{:else}
				<div class="py-6 text-center text-sm text-zinc-500">No more results</div>
			{/if}
    {/if}
  </div>
</div>
