import type {
	VideoBugReportBundle,
	VideoMetadata,
	VideoSopGuide,
	VideoSopStep,
} from "@cap/database/types";

export type TranscriptCueInput = {
	startSeconds: number;
	endSeconds: number;
	text: string;
};

export type DeveloperBugReport = {
	title: string;
	shareUrl: string;
	reproductionSteps: {
		stepNumber: number;
		timestampSeconds: number;
		formattedTimestamp: string;
		instruction: string;
		deepLink: string;
	}[];
	consoleErrors: {
		formattedTimestamp: string;
		level: "log" | "warn" | "error";
		message: string;
	}[];
	networkFailures: {
		formattedTimestamp: string;
		method: string;
		url: string;
		status: number;
		durationMs: number;
	}[];
	environment: {
		url: string;
		userAgent: string;
		viewport: string;
		platform: string;
	};
	markdown: string;
	linearUrl: string;
	githubUrl: string;
	jiraUrl: string;
};

export function formatSecondsToTimestamp(seconds: number) {
	const clamped = Math.max(0, Math.floor(seconds));
	const mins = Math.floor(clamped / 60);
	const secs = clamped % 60;
	return `${String(mins).padStart(2, "0")}:${String(secs).padStart(2, "0")}`;
}

const ACTION_VERB_REGEX =
	/\b(click|select|open|navigate|go to|press|enter|type|choose|toggle|enable|disable|configure|install|run|save|submit|create|update|delete|copy|paste|upload|download)\b/i;

function summarizeStepTitle(instruction: string, fallbackIndex: number) {
	const cleaned = instruction
		.replace(/^(so|now|next|then|first|after that|here|okay|alright)[,\s]+/i, "")
		.trim();
	if (!cleaned) return `Step ${fallbackIndex}`;
	const words = cleaned.split(/\s+/).slice(0, 7).join(" ");
	const capitalized = words.charAt(0).toUpperCase() + words.slice(1);
	return capitalized.replace(/[.,;:!?]+$/, "");
}

export function buildVideoSopGuide(params: {
	videoId: string;
	title: string;
	shareUrl: string;
	metadata?: VideoMetadata | null;
	cues: readonly TranscriptCueInput[];
}): VideoSopGuide {
	const { videoId, title, shareUrl, metadata, cues } = params;
	const cleanTitle = title.trim() || "Cap Screen Recording";
	const clicks = metadata?.bugReport?.cursorClicks ?? [];
	const chapters = metadata?.chapters ?? [];
	const steps: VideoSopStep[] = [];

	if (clicks.length > 0) {
		let lastStepSec = -10;
		for (const click of clicks) {
			const sec = Math.max(0, Math.floor(click.timeMs / 1000));
			if (sec - lastStepSec < 1.5) continue;
			lastStepSec = sec;
			const matchingText = cues
				.filter(
					(cue) => cue.endSeconds >= sec - 1.5 && cue.startSeconds <= sec + 3.5,
				)
				.map((cue) => cue.text.trim())
				.filter(Boolean)
				.join(" ");
			const stepNumber = steps.length + 1;
			const instruction =
				matchingText ||
				click.label ||
				`Click the target element at (${Math.round(click.x * 100)}%, ${Math.round(click.y * 100)}%).`;
			steps.push({
				stepNumber,
				title: summarizeStepTitle(instruction, stepNumber),
				instruction,
				timestampSeconds: sec,
				formattedTimestamp: formatSecondsToTimestamp(sec),
				screenshotUrl: `${shareUrl}?t=${sec}`,
				clickTarget: { x: click.x, y: click.y },
			});
			if (steps.length >= 16) break;
		}
	}

	if (steps.length === 0 && cues.length > 0) {
		const actionCues = cues.filter((cue) => ACTION_VERB_REGEX.test(cue.text));
		const sourceCues =
			actionCues.length >= 2
				? actionCues
				: cues.filter(
						(_, idx) =>
							idx % Math.max(1, Math.floor(cues.length / 8)) === 0,
					);
		for (const cue of sourceCues.slice(0, 12)) {
			const sec = Math.max(0, Math.floor(cue.startSeconds));
			const stepNumber = steps.length + 1;
			const instruction = cue.text.trim();
			steps.push({
				stepNumber,
				title: summarizeStepTitle(instruction, stepNumber),
				instruction,
				timestampSeconds: sec,
				formattedTimestamp: formatSecondsToTimestamp(sec),
				screenshotUrl: `${shareUrl}?t=${sec}`,
			});
		}
	}

	if (steps.length === 0 && chapters.length > 0) {
		for (const chapter of chapters) {
			const sec = Math.max(0, Math.floor(chapter.start));
			const stepNumber = steps.length + 1;
			steps.push({
				stepNumber,
				title: chapter.title,
				instruction: `Follow the walkthrough for "${chapter.title}".`,
				timestampSeconds: sec,
				formattedTimestamp: formatSecondsToTimestamp(sec),
				screenshotUrl: `${shareUrl}?t=${sec}`,
			});
		}
	}

	if (steps.length === 0) {
		steps.push({
			stepNumber: 1,
			title: "Watch Recording Walkthrough",
			instruction:
				metadata?.summary ||
				`Open the recording for "${cleanTitle}" and follow the demonstrated workflow.`,
			timestampSeconds: 0,
			formattedTimestamp: "00:00",
			screenshotUrl: `${shareUrl}?t=0`,
		});
	}

	const overview =
		metadata?.summary?.trim() ||
		`Step-by-step Standard Operating Procedure generated from "${cleanTitle}" (${videoId}).`;

	const markdownLines = [
		`# ${cleanTitle} — Step-by-Step Guide (SOP)`,
		"",
		`> **Overview**: ${overview}`,
		`> **Recording**: [Watch on Cap](${shareUrl})`,
		"",
		"## Walkthrough Steps",
		"",
	];

	for (const step of steps) {
		const clickNote = step.clickTarget
			? ` _(Click at ${Math.round(step.clickTarget.x * 100)}%, ${Math.round(step.clickTarget.y * 100)}%)_`
			: "";
		markdownLines.push(
			`### ${step.stepNumber}. ${step.title} ([${step.formattedTimestamp}](${step.screenshotUrl}))`,
			`${step.instruction}${clickNote}`,
			"",
			`📸 [View Frame Snapshot at ${step.formattedTimestamp}](${step.screenshotUrl})`,
			"",
		);
	}

	const newline = String.fromCharCode(10);
	return {
		title: `${cleanTitle} — Step-by-Step Guide`,
		overview,
		steps,
		markdown: markdownLines.join(newline),
		generatedAt: new Date().toISOString(),
	};
}

export function buildDeveloperBugReport(params: {
	videoId: string;
	title: string;
	shareUrl: string;
	metadata?: VideoMetadata | null;
	cues: readonly TranscriptCueInput[];
	bundle?: VideoBugReportBundle | null;
}): DeveloperBugReport {
	const { videoId, title, shareUrl, metadata, cues, bundle } = params;
	const sop =
		metadata?.sop ??
		buildVideoSopGuide({
			videoId,
			title,
			shareUrl,
			metadata,
			cues,
		});

	const effectiveBundle = bundle ?? metadata?.bugReport;
	const reproductionSteps = sop.steps.map((step) => ({
		stepNumber: step.stepNumber,
		timestampSeconds: step.timestampSeconds,
		formattedTimestamp: step.formattedTimestamp,
		instruction: step.instruction,
		deepLink: `${shareUrl}?t=${step.timestampSeconds}`,
	}));

	const consoleErrors = (effectiveBundle?.consoleLogs ?? []).map((entry) => ({
		formattedTimestamp: formatSecondsToTimestamp(entry.timestampMs / 1000),
		level: entry.level,
		message: entry.message,
	}));

	const networkFailures = (effectiveBundle?.networkLogs ?? [])
		.filter((entry) => entry.failed || entry.status >= 400)
		.map((entry) => ({
			formattedTimestamp: formatSecondsToTimestamp(entry.timestampMs / 1000),
			method: entry.method,
			url: entry.url,
			status: entry.status,
			durationMs: entry.durationMs,
		}));

	const environment = {
		url: effectiveBundle?.environment?.url ?? shareUrl,
		userAgent:
			effectiveBundle?.environment?.userAgent ?? "Cap Desktop / Web Recorder",
		viewport: effectiveBundle?.environment?.viewport ?? "Auto",
		platform: effectiveBundle?.environment?.platform ?? "Cross-platform",
	};

	const bugTitle = `[Bug] ${title.trim() || "Issue captured in Cap"}`;
	const mdSections: string[] = [
		"## Bug Summary",
		metadata?.summary?.trim() ||
			`Issue recorded via Cap: [${title.trim() || videoId}](${shareUrl})`,
		"",
		`**Video Recording**: [Watch in Cap](${shareUrl})`,
		"",
		"## Steps to Reproduce",
		...reproductionSteps.map(
			(step) =>
				`${step.stepNumber}. [${step.formattedTimestamp}](${step.deepLink}) — ${step.instruction}`,
		),
		"",
	];

	if (consoleErrors.length > 0) {
		mdSections.push(
			"## Console Logs & Errors",
			"```text",
			...consoleErrors.map(
				(log) =>
					`[${log.formattedTimestamp}] [${log.level.toUpperCase()}] ${log.message}`,
			),
			"```",
			"",
		);
	}

	if (networkFailures.length > 0) {
		mdSections.push(
			"## Failed Network Requests",
			"| Time | Method | Status | Duration | URL |",
			"| --- | --- | --- | --- | --- |",
			...networkFailures.map(
				(req) =>
					`| ${req.formattedTimestamp} | ${req.method} | ${req.status} | ${req.durationMs}ms | ${req.url} |`,
			),
			"",
		);
	}

	mdSections.push(
		"## Environment",
		`- **Page / Source**: ${environment.url}`,
		`- **Platform**: ${environment.platform}`,
		`- **Viewport**: ${environment.viewport}`,
		`- **User Agent**: ${environment.userAgent}`,
	);

	const newline = String.fromCharCode(10);
	const markdown = mdSections.join(newline);
	const encodedTitle = encodeURIComponent(bugTitle);
	const encodedBody = encodeURIComponent(markdown);

	return {
		title: bugTitle,
		shareUrl,
		reproductionSteps,
		consoleErrors,
		networkFailures,
		environment,
		markdown,
		linearUrl: `https://linear.app/new?title=${encodedTitle}&description=${encodedBody}`,
		githubUrl: `https://github.com/issues/new?title=${encodedTitle}&body=${encodedBody}`,
		jiraUrl: `https://id.atlassian.com/login?continue=${encodeURIComponent(`https://atlassian.net/secure/CreateIssue!default.jspa?summary=${bugTitle}`)}`,
	};
}
