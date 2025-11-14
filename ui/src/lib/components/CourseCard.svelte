<script lang="ts">
	import type { Course } from '$lib/types';

	let { course }: { course: Course } = $props();

	function getPct(c: Course): number {
		const capacity = Number(c?.capacity) || 0;
		const enrolled = Number(c?.enrolled) || 0;
		return capacity > 0 ? Math.min(100, Math.round((enrolled / capacity) * 100)) : 0;
	}

	function formatTime(raw?: string): string {
		if (!raw) return '';
		const t = String(raw).trim();
		// HHMM e.g. "0930", "1400"
		const hhmm = /^(\d{2})(\d{2})$/;
		const m = t.match(hhmm);
		if (m) {
			let hours = parseInt(m[1], 10);
			const minutes = m[2];
			const suffix = hours >= 12 ? 'PM' : 'AM';
			hours = hours % 12;
			if (hours === 0) hours = 12;
			return `${hours}:${minutes} ${suffix}`;
		}
		// Already formatted, return as-is
		return t;
	}
</script>

<a
	class="group block rounded-none border border-zinc-200 bg-white p-4 shadow-sm transition hover:border-zinc-300 hover:shadow"
	href={course.link}
	target="_blank"
	rel="noreferrer"
>
	<div class="flex items-start justify-between gap-3">
		<div>
			<div class="flex flex-wrap items-center gap-2">
				<span class="bg-zinc-100 px-2 py-0.5 text-xs font-medium text-zinc-700">{course.code}</span>
				{#if course.class_type}
					<span class="bg-blue-50 px-2 py-0.5 text-xs font-medium text-blue-700">{course.class_type}</span>
				{/if}
			</div>
			<h2 class="mt-2 text-lg font-semibold leading-snug">
				{course.title}
				{#if course.subtitle}
					<span class="text-zinc-500"> — {course.subtitle}</span>
				{/if}
                {#if course.special}
                    <span class="text-zinc-500"> — {course.special}</span>
                {/if}
			</h2>
			<p class="mt-1 line-clamp-2 text-sm text-zinc-600">{course.course_description || course.class_description}</p>
			{#if course.days || course.start || course.end}
				<div class="mt-2 flex flex-wrap items-center gap-2 text-sm text-zinc-700">
					{#if course.days}
						<span class="font-medium">{course.days}</span>
					{/if}
					{#if (course.days && (course.start || course.end))}
						<span class="text-zinc-400">•</span>
					{/if}
					{#if course.start || course.end}
						<span>
							{formatTime(course.start)}{(course.start && course.end) ? '–' : ''}{formatTime(course.end)}
						</span>
					{/if}
				</div>
			{/if}
		</div>
		<div class="shrink-0 text-right">
			<div class="text-xs text-zinc-500">Capacity</div>
			<div class="mt-0.5 text-sm font-medium">
				{course.enrolled}/{course.capacity}
			</div>
			{#if course.capacity > 0}
				<div class="mt-1 h-2 w-28 overflow-hidden bg-zinc-100">
					<div
						class="h-full bg-emerald-500 transition-[width]"
						style:width={`${getPct(course)}%`}
					></div>
				</div>
			{/if}
			{#if course.waitlist_max > 0 || course.waitlist > 0}
				<div class="mt-3 text-xs text-zinc-500">Waitlist</div>
				<div class="mt-0.5 text-sm font-medium">{course.waitlist}/{course.waitlist_max}</div>
				{#if course.waitlist_max > 0}
					<div class="mt-1 h-2 w-28 overflow-hidden bg-zinc-100">
						<div
							class="h-full bg-amber-500 transition-[width]"
							style:width={`${Math.min(100, Math.round((course.waitlist / course.waitlist_max) * 100))}%`}
						></div>
					</div>
				{/if}
			{/if}
		</div>
	</div>
</a>


