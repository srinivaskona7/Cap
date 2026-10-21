import { describe, expect, it } from "vitest";
import {
	createEditTranscript,
	detectPiiWordIndices,
	isPiiToken,
	planPiiCuts,
	planSilenceCuts,
} from "@/lib/edit-transcript";
import {
	buildDeveloperBugReport,
	buildVideoSopGuide,
} from "@/lib/video-sop-and-bug-report";
import {
	computeRetentionAnalytics,
	mergeViewerRetentionSession,
} from "@/lib/viewer-retention";
import { runWorkspaceSemanticSearch } from "@/lib/workspace-semantic-search";

describe("Tier 1 & Tier 2 Max/Enterprise Features", () => {
	it("plans one-click silence cuts while preserving breath padding", () => {
		const transcript = createEditTranscript(
			{
				speech_model_used: "universal-2",
				words: [
					{ text: "Hello", start: 1800, end: 2200 },
					{ text: "team", start: 2250, end: 2600 },
					{ text: "welcome", start: 4500, end: 5000 },
				],
			},
			7000,
		);

		const silencePlan = planSilenceCuts(transcript.words, 7000, {
			minSilenceMs: 1200,
			breathPaddingMs: 200,
		});

		expect(silencePlan.silenceCount).toBe(3);
		expect(silencePlan.ranges).toEqual([
			{ startMs: 0, endMs: 1600, safe: true, reason: null },
			{ startMs: 2800, endMs: 4300, safe: true, reason: null },
			{ startMs: 5200, endMs: 7000, safe: true, reason: null },
		]);
		expect(silencePlan.totalSavedMs).toBe(4900);
	});

	it("detects and plans cuts for spoken PII, emails, API keys, and passwords", () => {
		expect(isPiiToken("alice@acme.io")).toBe(true);
		expect(isPiiToken("sk-proj-1234567890abcdef")).toBe(true);
		expect(isPiiToken("ghp_1234567890abcdef")).toBe(true);
		expect(isPiiToken("123-45-6789")).toBe(true);
		expect(isPiiToken("dashboard")).toBe(false);

		const transcript = createEditTranscript(
			{
				speech_model_used: "universal-2",
				words: [
					{ text: "Email", start: 100, end: 400 },
					{ text: "alice@acme.io", start: 500, end: 1100 },
					{ text: "and", start: 1200, end: 1400 },
					{ text: "password", start: 1500, end: 1900 },
					{ text: "is", start: 1950, end: 2100 },
					{ text: "hunter42", start: 2150, end: 2700 },
					{ text: "done", start: 2900, end: 3400 },
				],
			},
			4000,
		);

		const indices = Array.from(detectPiiWordIndices(transcript.words));
		expect(indices).toEqual([1, 5]);

		const piiPlan = planPiiCuts(transcript.words, 4000);
		expect(piiPlan.piiCount).toBe(2);
		expect(piiPlan.ranges).toHaveLength(2);
	});

	it("generates a step-by-step SOP guide from cursor clicks and transcript cues", () => {
		const sop = buildVideoSopGuide({
			videoId: "vid_123",
			title: "Deploy Production API",
			shareUrl: "https://cap.so/s/vid_123",
			metadata: {
				summary: "How to deploy the API from the dashboard.",
				bugReport: {
					cursorClicks: [
						{ timeMs: 4000, x: 0.25, y: 0.15 },
						{ timeMs: 12000, x: 0.8, y: 0.6 },
					],
				},
			},
			cues: [
				{
					startSeconds: 3.5,
					endSeconds: 6.0,
					text: "Click on the Environments tab in the top bar.",
				},
				{
					startSeconds: 11.5,
					endSeconds: 14.5,
					text: "Press the Deploy Production button to start rollout.",
				},
			],
		});

		expect(sop.steps).toHaveLength(2);
		expect(sop.steps[0]?.formattedTimestamp).toBe("00:04");
		expect(sop.steps[0]?.instruction).toContain("Click on the Environments tab");
		expect(sop.steps[1]?.formattedTimestamp).toBe("00:12");
		expect(sop.markdown).toContain("# Deploy Production API — Step-by-Step Guide (SOP)");
		expect(sop.markdown).toContain("https://cap.so/s/vid_123?t=12");
	});

	it("builds a developer bug report bundle with Linear, GitHub, and Jira 1-click links", () => {
		const bug = buildDeveloperBugReport({
			videoId: "vid_bug",
			title: "Checkout 500 Error",
			shareUrl: "https://cap.so/s/vid_bug",
			metadata: {
				summary: "Submitting checkout throws a 500 error.",
				bugReport: {
					consoleLogs: [
						{
							timestampMs: 8000,
							level: "error",
							message: "Unhandled rejection: Stripe token missing",
						},
					],
					networkLogs: [
						{
							timestampMs: 8200,
							method: "POST",
							url: "/api/checkout",
							status: 500,
							durationMs: 142,
							failed: true,
						},
					],
					environment: {
						url: "https://app.example.com/checkout",
						userAgent: "Chrome 134",
						viewport: "1440x900",
						platform: "macOS",
					},
				},
			},
			cues: [
				{
					startSeconds: 2,
					endSeconds: 9,
					text: "Click submit order on the checkout page.",
				},
			],
		});

		expect(bug.title).toBe("[Bug] Checkout 500 Error");
		expect(bug.consoleErrors).toHaveLength(1);
		expect(bug.networkFailures).toHaveLength(1);
		expect(bug.markdown).toContain("Unhandled rejection: Stripe token missing");
		expect(bug.markdown).toContain("/api/checkout");
		expect(bug.linearUrl).toContain("https://linear.app/new?");
		expect(bug.githubUrl).toContain("https://github.com/issues/new?");
	});

	it("tracks viewer retention heatmaps and per-prospect watch sessions", () => {
		const first = mergeViewerRetentionSession({
			metadata: null,
			sessionId: "sess_1",
			viewerEmail: "cto@prospect.io",
			viewerName: "cto@prospect.io",
			location: "San Francisco, US",
			watchedBuckets: [0, 1, 2, 3, 4],
			rewatchedBuckets: [2],
			nowIso: "2026-10-20T12:00:00.000Z",
		});

		expect(first.isNewProspectAlert).toBe(true);
		expect(first.session.completionPercent).toBe(25);
		expect(first.updatedMetadata.retentionBuckets?.[2]).toBe(2);

		const analytics = computeRetentionAnalytics(first.updatedMetadata, 1);
		expect(analytics.hotspotBucketIndex).toBe(2);
		expect(analytics.viewerSessions[0]?.viewerEmail).toBe("cto@prospect.io");
		expect(analytics.averageCompletionPercent).toBe(25);
	});

	it("searches across workspace videos by spoken transcript, SOP steps, and OCR text", () => {
		const searchResult = runWorkspaceSemanticSearch(
			[
				{
					id: "vid_auth",
					name: "Q4 Architecture Review",
					ownerId: "u1",
					orgId: "org1",
					createdAt: "2026-10-20T00:00:00.000Z",
					shareUrl: "https://cap.so/s/vid_auth",
					metadata: {
						summary: "Walkthrough of WorkOS SAML SSO configuration.",
						ocrText: "WORKOS_CLIENT_ID and SAML_METADATA_URL in settings",
					},
					cues: [
						{
							startMs: 45_000,
							endMs: 52_000,
							text: "Here we paste the WorkOS SAML metadata URL into the organization portal.",
						},
					],
				},
				{
					id: "vid_design",
					name: "Landing Page Polish",
					ownerId: "u2",
					orgId: "org1",
					createdAt: "2026-10-19T00:00:00.000Z",
					shareUrl: "https://cap.so/s/vid_design",
					metadata: {
						summary: "Typography and button hover states.",
					},
					cues: [],
				},
			],
			"WorkOS SAML metadata",
		);

		expect(searchResult.hits).toHaveLength(1);
		expect(searchResult.hits[0]?.videoId).toBe("vid_auth");
		expect(
			searchResult.hits[0]?.snippets.some(
				(s) => s.source === "transcript" && s.timestampSeconds === 45,
			),
		).toBe(true);
		expect(searchResult.synthesizedAnswer).toContain("Q4 Architecture Review");
	});
});
