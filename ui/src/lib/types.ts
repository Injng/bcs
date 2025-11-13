export interface Course {
	code: string;
	'class-type'?: string;
	class_type?: string;
	count: string;
	title: string;
	subtitle: string;
	special: string;
	link: string;
	instructor: string;
	days: string;
	start: string;
	end: string;
	location: string;
	id: string;
	units: string;
	mode: string;
	'course-description'?: string;
	course_description?: string;
	'class-description'?: string;
	class_description?: string;
	capacity: number;
	enrolled: number;
	waitlist: number;
	'waitlist-max'?: number;
	waitlist_max: number;
	requirements?: string[];
	seats: Record<string, number>;
}

export function normalizeCourse(raw: Course): Course {
	// Normalize snake/kebab to camel where convenient for UI
	return {
		...raw,
		class_type: (raw.class_type ?? (raw as any)['class-type']) as string | undefined,
		course_description: (raw.course_description ?? (raw as any)['course-description']) as
			| string
			| undefined,
		class_description: (raw.class_description ?? (raw as any)['class-description']) as
			| string
			| undefined,
		waitlist_max: (raw.waitlist_max ?? (raw as any)['waitlist-max']) as number,
	};
}


