import type { VideoMetadata } from "@cap/database/types";
import { formatSecondsToTimestamp } from "./video-sop-and-bug-report";

export type SearchableTranscriptCue = {
	startMs: number;
	endMs: number;
	text: string;
};

export type SearchableWorkspaceVideo = {
	id: string;
	name: string;
	ownerId: string;
	orgId: string;
	duration?: number | null;
	createdAt: string;
	shareUrl: string;
	spaceNames?: string[];
	metadata?: VideoMetadata | null;
	cues?: readonly SearchableTranscriptCue[];
};

export type WorkspaceVideoMatchSnippet = {
	source: "transcript" | "summary" | "chapter" | "ocr" | "sop";
	text: string;
	timestampSeconds: number;
	formattedTimestamp: string;
	deepLink: string;
};

export type WorkspaceSearchHit = {
	videoId: string;
	title: string;
	shareUrl: string;
	createdAt: string;
	spaceNames: string[];
	score: number;
	summary: string | null;
	snippets: WorkspaceVideoMatchSnippet[];
};

export type WorkspaceSearchResponse = {
	query: string;
	synthesizedAnswer: string;
	hits: WorkspaceSearchHit[];
};

const STOP_WORDS = new Set([
	"a",
	"an",
	"and",
	"are",
	"as",
	"at",
	"be",
	"by",
	"did",
	"do",
	"does",
	"for",
	"from",
	"how",
	"i",
	"in",
	"is",
	"it",
	"my",
	"of",
	"on",
	"or",
	"our",
	"team",
	"that",
	"the",
	"this",
	"to",
	"video",
	"videos",
	"was",
	"we",
	"what",
	"when",
	"where",
	"which",
	"who",
	"why",
	"with",
]);

export function tokenizeSearchQuery(query: string): string[] {
	const cleaned = query
		.toLowerCase()
		.replace(/[^\p{L}\p{N}\s_-]+/gu, " ")
		.split(/\s+/)
		.map((token) => token.trim())
		.filter((token) => token.length >= 2 && !STOP_WORDS.has(token));
	if (cleaned.length > 0) {
		return Array.from(new Set(cleaned));
	}
	return query
		.toLowerCase()
		.split(/\s+/)
		.map((t) => t.trim())
		.filter(Boolean);
}

function scoreTextField(
	text: string,
	tokens: readonly string[],
	fullPhrase: string,
): number {
	if (!text) return 0;
	const lower = text.toLowerCase();
	let score = 0;
	if (fullPhrase.length >= 3 && lower.includes(fullPhrase)) {
		score += 4;
	}
	for (const token of tokens) {
		if (lower.includes(token)) {
			score += 1.5;
		}
	}
	return score;
}

export function scoreWorkspaceVideoDocument(
	video: SearchableWorkspaceVideo,
	rawQuery: string,
): WorkspaceSearchHit | null {
	const fullPhrase = rawQuery.trim().toLowerCase();
	const tokens = tokenizeSearchQuery(rawQuery);
	if (tokens.length === 0 && !fullPhrase) return null;

	const title = video.metadata?.aiTitle?.trim() || video.name || "Untitled Cap";
	const summary = video.metadata?.summary?.trim() || null;
	const chapters = video.metadata?.chapters ?? [];
	const ocrText = video.metadata?.ocrText?.trim() || "";
	const sopSteps = video.metadata?.sop?.steps ?? [];
	const cues = video.cues ?? [];

	let totalScore = scoreTextField(title, tokens, fullPhrase) * 4.0;
	const snippets: WorkspaceVideoMatchSnippet[] = [];

	if (summary) {
		const summaryScore = scoreTextField(summary, tokens, fullPhrase) * 2.5;
		if (summaryScore > 0) {
			totalScore += summaryScore;
			snippets.push({
				source: "summary",
				text: summary.slice(0, 240),
				timestampSeconds: 0,
				formattedTimestamp: "00:00",
				deepLink: video.shareUrl,
			});
		}
	}

	for (const chapter of chapters) {
		const chScore = scoreTextField(chapter.title, tokens, fullPhrase) * 2.5;
		if (chScore > 0) {
			totalScore += chScore;
			const sec = Math.max(0, Math.floor(chapter.start));
			snippets.push({
				source: "chapter",
				text: chapter.title,
				timestampSeconds: sec,
				formattedTimestamp: formatSecondsToTimestamp(sec),
				deepLink: `${video.shareUrl}?t=${sec}`,
			});
		}
	}

	for (const step of sopSteps) {
		const stepText = `${step.title}: ${step.instruction}`;
		const stepScore = scoreTextField(stepText, tokens, fullPhrase) * 2.2;
		if (stepScore > 0) {
			totalScore += stepScore;
			snippets.push({
				source: "sop",
				text: stepText.slice(0, 240),
				timestampSeconds: step.timestampSeconds,
				formattedTimestamp: step.formattedTimestamp,
				deepLink: `${video.shareUrl}?t=${step.timestampSeconds}`,
			});
		}
	}

	if (ocrText) {
		const ocrScore = scoreTextField(ocrText, tokens, fullPhrase) * 2.2;
		if (ocrScore > 0) {
			totalScore += ocrScore;
			snippets.push({
				source: "ocr",
				text: ocrText.slice(0, 240),
				timestampSeconds: 0,
				formattedTimestamp: "00:00",
				deepLink: video.shareUrl,
			});
		}
	}

	for (const cue of cues) {
		const cueScore = scoreTextField(cue.text, tokens, fullPhrase) * 3.0;
		if (cueScore > 0) {
			totalScore += cueScore;
			const sec = Math.max(0, Math.floor(cue.startMs / 1000));
			if (snippets.length < 6) {
				snippets.push({
					source: "transcript",
					text: cue.text.slice(0, 240),
					timestampSeconds: sec,
					formattedTimestamp: formatSecondsToTimestamp(sec),
					deepLink: `${video.shareUrl}?t=${sec}`,
				});
			}
		}
	}

	if (totalScore <= 0) return null;

	return {
		videoId: video.id,
		title,
		shareUrl: video.shareUrl,
		createdAt: video.createdAt,
		spaceNames: video.spaceNames ?? [],
		score: Math.round(totalScore * 10) / 10,
		summary,
		snippets: snippets.slice(0, 5),
	};
}

export function runWorkspaceSemanticSearch(
	videos: readonly SearchableWorkspaceVideo[],
	rawQuery: string,
	limit = 10,
): WorkspaceSearchResponse {
	const hits = videos
		.map((v) => scoreWorkspaceVideoDocument(v, rawQuery))
		.filter((hit): hit is WorkspaceSearchHit => hit !== null)
		.sort((a, b) => b.score - a.score)
		.slice(0, Math.max(1, limit));

	if (hits.length === 0) {
		return {
			query: rawQuery,
			synthesizedAnswer: `No matching recordings were found across your workspace for "${rawQuery}".`,
			hits: [],
		};
	}

	const citationLines = hits.slice(0, 4).map((hit) => {
		const topSnippet = hit.snippets[0];
		if (topSnippet) {
			return `- **${hit.title}** ([${topSnippet.formattedTimestamp}](${topSnippet.deepLink})) [${topSnippet.source}]: "${topSnippet.text}"`;
		}
		return `- **${hit.title}** ([Watch](${hit.shareUrl})): ${hit.summary ?? "Matching workspace recording"}`;
	});

	const newline = String.fromCharCode(10);
	const synthesizedAnswer = [
		`Found ${hits.length} relevant recording${hits.length === 1 ? "" : "s"} for "${rawQuery}":`,
		...citationLines,
	].join(newline);

	return {
		query: rawQuery,
		synthesizedAnswer,
		hits,
	};
}
