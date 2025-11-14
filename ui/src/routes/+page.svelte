<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import type { Course } from "$lib/types";
  import { normalizeCourse } from "$lib/types";
  import SearchIcon from "lucide-svelte/icons/search";
  import LoaderIcon from "lucide-svelte/icons/loader-2";
  import FilterIcon from "lucide-svelte/icons/filter";
  import ChevronDown from "lucide-svelte/icons/chevron-down";
  import PlusIcon from "lucide-svelte/icons/plus";
  import TrashIcon from "lucide-svelte/icons/trash-2";
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

  // Filters
  let filtersOpen = false;
  // Static general requirements aligned with backend enum serialization
  let requirementsOptions: string[] = [
    "the American Cultures requirement",
    "the American Hist & Institutions requirement",
    "the Entry Level Writing requirement",
    "the Reading and Composition A requirement",
    "the Reading and Composition B requirement",
    "Arts & Literature, L&S Breadth",
    "Biological Science, L&S Breadth",
    "Historical Studies, L&S Breadth",
    "International Studies, L&S Breadth",
    "Philosophy & Values, L&S Breadth",
    "Physical Science, L&S Breadth",
    "Social & Behavioral Sciences, L&S Breadth",
  ];
  let selectedRequirements = new Set<string>();
  let requirementsOr = true; // true = OR (any), false = AND (all)
  // Time range filters (server expects times: Vec<(Time, Time)>)
  const timeStepOptions: string[] = (() => {
    const out: string[] = [];
    for (let h = 0; h < 24; h++) {
      for (let m = 0; m < 60; m += 30) {
        const hh = String(h).padStart(2, "0");
        const mm = String(m).padStart(2, "0");
        out.push(`${hh}:${mm}`);
      }
    }
    return out;
  })();
  type TimeRange = { id: number; start: string; end: string };
  let nextTimeRangeId = 1;
  let timeRanges: TimeRange[] = [];

  // Day-of-week filter (server expects concatenated codes like "MoWeFr")
  type DayOption = { label: string; code: "Mo" | "Tu" | "We" | "Th" | "Fr" | "Sa" | "Su" };
  const dayOptions: DayOption[] = [
    { label: "Monday", code: "Mo" },
    { label: "Tuesday", code: "Tu" },
    { label: "Wednesday", code: "We" },
    { label: "Thursday", code: "Th" },
    { label: "Friday", code: "Fr" },
    { label: "Saturday", code: "Sa" },
    { label: "Sunday", code: "Su" }
  ];
  const dayOrder: Array<DayOption["code"]> = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"];
  let selectedDays = new Set<DayOption["code"]>();

  function toggleDay(code: DayOption["code"]) {
    const next = new Set(selectedDays);
    if (next.has(code)) next.delete(code);
    else next.add(code);
    selectedDays = next;
    // Trigger search
    if (inflightAbort) inflightAbort.abort();
    results = [];
    hasMore = false;
    error = null;
    debounceTimer && clearTimeout(debounceTimer);
    doSearch();
  }

  function clearDays() {
    if (selectedDays.size === 0) return;
    selectedDays = new Set();
    if (inflightAbort) inflightAbort.abort();
    results = [];
    hasMore = false;
    error = null;
    debounceTimer && clearTimeout(debounceTimer);
    doSearch();
  }

  function buildDaysString(): string {
    return dayOrder.filter((c) => selectedDays.has(c)).join("");
  }

  function toggleFilters() {
    filtersOpen = !filtersOpen;
  }

  function toggleRequirement(req: string) {
    if (selectedRequirements.has(req)) {
      selectedRequirements.delete(req);
    } else {
      selectedRequirements.add(req);
    }
    // Reset and re-search with new filters
    if (inflightAbort) inflightAbort.abort();
    results = [];
    hasMore = false;
    error = null;
    debounceTimer && clearTimeout(debounceTimer);
    doSearch();
  }

  function addTimeRange() {
    timeRanges = [...timeRanges, { id: nextTimeRangeId++, start: "08:00", end: "12:00" }];
    // Trigger search
    if (inflightAbort) inflightAbort.abort();
    results = [];
    hasMore = false;
    error = null;
    debounceTimer && clearTimeout(debounceTimer);
    doSearch();
  }

  function removeTimeRange(id: number) {
    timeRanges = timeRanges.filter((r) => r.id !== id);
    // Trigger search
    if (inflightAbort) inflightAbort.abort();
    results = [];
    hasMore = false;
    error = null;
    debounceTimer && clearTimeout(debounceTimer);
    doSearch();
  }

  function updateTimeRange(id: number, which: "start" | "end", value: string) {
    timeRanges = timeRanges.map((r) => (r.id === id ? { ...r, [which]: value } as TimeRange : r));
    // Trigger search
    if (inflightAbort) inflightAbort.abort();
    results = [];
    hasMore = false;
    error = null;
    debounceTimer && clearTimeout(debounceTimer);
    doSearch();
  }

  function setRequirementsOr(value: boolean) {
    if (requirementsOr === value) return;
    requirementsOr = value;
    // Reset and re-search with new logic
    if (inflightAbort) inflightAbort.abort();
    results = [];
    hasMore = false;
    error = null;
    debounceTimer && clearTimeout(debounceTimer);
    doSearch();
  }

  function clearRequirements() {
    selectedRequirements = new Set();
    // Reset and re-search with cleared filters
    if (inflightAbort) inflightAbort.abort();
    results = [];
    hasMore = false;
    error = null;
    debounceTimer && clearTimeout(debounceTimer);
    doSearch();
  }

  // No dynamic option building; static list above

  function buildFiltersPayload() {
    const payload: Record<string, unknown> = { requirements_or: requirementsOr };
    if (selectedRequirements.size > 0) {
      payload.requirements = Array.from(selectedRequirements);
    }
    const daysString = buildDaysString();
    if (daysString.length > 0) {
      payload.days = daysString;
    }
    if (daysString.length === 0) {
      payload.days = null;
    }
    const validRanges: Array<[string, string]> = timeRanges
      .filter((r) => r.start < r.end) // "HH:MM" lexicographic works
      .map((r) => [r.start, r.end]);
    if (validRanges.length > 0) {
      payload.times = validRanges;
    }
    return payload;
  }

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
				body: JSON.stringify({ keywords: query, offset: 0, filters: buildFiltersPayload() }),
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
				body: JSON.stringify({ keywords: currentQuery, offset: currentOffset, filters: buildFiltersPayload() }),
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

    <div class="mb-3 flex items-center justify-between">
      <button
        class="inline-flex items-center gap-2 border border-zinc-300 bg-white px-3 py-2 text-sm transition hover:border-zinc-400"
        on:click={toggleFilters}
        aria-expanded={filtersOpen}
        aria-controls="filters"
      >
        <FilterIcon class="size-4 text-zinc-500" />
        <span>Filters</span>
        <ChevronDown class="size-4 text-zinc-500" />
      </button>
      {#if selectedRequirements.size > 0}
        <button
          class="text-xs text-zinc-600 underline underline-offset-4 hover:text-zinc-800"
          on:click={clearRequirements}
        >
          Clear filters
        </button>
      {/if}
    </div>

    {#if filtersOpen}
      <div id="filters" class="mb-6 border border-zinc-200 bg-zinc-50 p-3">
        <div>
          <div class="mb-2 flex items-center justify-between">
            <div class="text-sm font-medium text-zinc-800">Requirements</div>
            <div class="flex items-center gap-2">
              <span class="text-xs text-zinc-600">Match</span>
              <div class="inline-flex border border-zinc-300 text-xs">
                <button
                  class={"px-2 py-1 " + (requirementsOr ? "bg-zinc-200 text-zinc-800" : "bg-white text-zinc-700")}
                  on:click={() => setRequirementsOr(true)}
                  type="button"
                  aria-pressed={requirementsOr}
                >
                  Any
                </button>
                <button
                  class={"px-2 py-1 border-l border-zinc-300 " + (!requirementsOr ? "bg-zinc-200 text-zinc-800" : "bg-white text-zinc-700")}
                  on:click={() => setRequirementsOr(false)}
                  type="button"
                  aria-pressed={!requirementsOr}
                >
                  All
                </button>
              </div>
            </div>
          </div>
          {#if requirementsOptions.length === 0}
            <div class="text-xs text-zinc-600">No requirement data.</div>
          {:else}
            <div class="grid grid-cols-1 gap-2 sm:grid-cols-2">
              {#each requirementsOptions as req}
                <label class="flex cursor-pointer items-start gap-2 text-sm">
                  <input
                    type="checkbox"
                    class="mt-0.5"
                    checked={selectedRequirements.has(req)}
                    on:change={() => toggleRequirement(req)}
                  />
                  <span class="text-zinc-700">{req}</span>
                </label>
              {/each}
            </div>
          {/if}
        </div>
        <div class="mt-4">
          <div class="mb-2 flex items-center justify-between">
            <div class="text-sm font-medium text-zinc-800">Days</div>
            {#if selectedDays.size > 0}
              <button
                class="text-xs text-zinc-600 underline underline-offset-4 hover:text-zinc-800"
                on:click={clearDays}
                type="button"
              >
                Clear days
              </button>
            {/if}
          </div>
          <div class="grid grid-cols-7 gap-1">
            {#each dayOptions as d}
              <button
                type="button"
                class={"inline-flex items-center justify-center border border-zinc-300 px-2 py-2 text-sm transition " + (selectedDays.has(d.code) ? "bg-zinc-600 text-white" : "bg-white text-zinc-700 hover:border-zinc-400")}
                aria-pressed={selectedDays.has(d.code)}
                on:click={() => toggleDay(d.code)}
              >
                {d.label}
              </button>
            {/each}
          </div>
        </div>
        <div class="mt-4">
          <div class="mb-2 flex items-center justify-between">
            <div class="text-sm font-medium text-zinc-800">Time ranges</div>
            <button
              class="inline-flex items-center gap-1 border border-zinc-300 bg-white px-2 py-1 text-xs transition hover:border-zinc-400"
              type="button"
              on:click={addTimeRange}
            >
              <PlusIcon class="size-3.5 text-zinc-600" />
              Add range
            </button>
          </div>
          <div class="space-y-2">
            {#each timeRanges as r (r.id)}
              <div class="flex items-center gap-2">
                <select
                  class="border border-zinc-300 bg-white px-2 py-1 text-sm"
                  bind:value={r.start}
                  on:change={(e) => updateTimeRange(r.id, "start", (e.target as HTMLSelectElement).value)}
                >
                  {#each timeStepOptions as opt}
                    <option value={opt}>{opt}</option>
                  {/each}
                </select>
                <span class="text-zinc-500">to</span>
                <select
                  class="border border-zinc-300 bg-white px-2 py-1 text-sm"
                  bind:value={r.end}
                  on:change={(e) => updateTimeRange(r.id, "end", (e.target as HTMLSelectElement).value)}
                >
                  {#each timeStepOptions as opt}
                    <option value={opt}>{opt}</option>
                  {/each}
                </select>
                <button
                  class="ml-1 inline-flex items-center gap-1 border border-zinc-300 bg-white px-2 py-1 text-xs transition hover:border-zinc-400"
                  type="button"
                  on:click={() => removeTimeRange(r.id)}
                  aria-label="Remove time range"
                >
                  <TrashIcon class="size-3.5 text-zinc-600" />
                  Remove
                </button>
              </div>
            {/each}
          </div>
        </div>
      </div>
    {/if}

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
					<span class="select-none">{loadingMore ? "Loading more…" : ""}</span>
				</div>
			{:else}
				<div class="py-6 text-center text-sm text-zinc-500">No more results</div>
			{/if}
    {/if}
  </div>
</div>
