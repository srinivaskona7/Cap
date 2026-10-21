"use server";

import { db } from "@cap/database";
import { getCurrentUser } from "@cap/database/auth/session";
import { integrationInstallations, videos } from "@cap/database/schema";
import type { VideoMetadata } from "@cap/database/types";
import { buildEnv, serverEnv } from "@cap/env";
import { S3Buckets } from "@cap/web-backend";
import { and, eq } from "drizzle-orm";
import { Effect, Option } from "effect";
import { runPromise } from "@/lib/server";
import {
	buildDeveloperBugReport,
	buildVideoSopGuide,
	type DeveloperBugReport,
	type TranscriptCueInput,
} from "@/lib/video-sop-and-bug-report";

function parseVttTimeSeconds(raw: string): number | null {
	const match = raw
		.trim()
		.match(/^(?:(\d{1,2}):)?(\d{2}):(\d{2})[.,](\d{3})$/);
	if (!match) return null;
	const hours = match[1] ? Number(match[1]) : 0;
	const minutes = Number(match[2]);
	const seconds = Number(match[3]);
	const ms = Number(match[4]);
	return hours * 3600 + minutes * 60 + seconds + ms / 1000;
}

function parseVttToCues(vtt: string): TranscriptCueInput[] {
	const cr = String.fromCharCode(13);
	const lf = String.fromCharCode(10);
	const normalized = vtt.split(`${cr}${lf}`).join(lf);
	const blocks = normalized.split(`${lf}${lf}`);
	const cues: TranscriptCueInput[] = [];
	for (const block of blocks) {
		const lines = block
			.split(lf)
			.map((l) => l.trim())
			.filter(Boolean);
		const arrowIndex = lines.findIndex((line) => line.includes("-->"));
		if (arrowIndex === -1) continue;
		const timingLine = lines[arrowIndex] ?? "";
		const [startRaw = "", endPart = ""] = timingLine.split("-->");
		const endRaw = endPart.trim().split(/\s+/)[0] ?? "";
		const startSeconds = parseVttTimeSeconds(startRaw);
		const endSeconds = parseVttTimeSeconds(endRaw);
		if (startSeconds === null || endSeconds === null) continue;
		const text = lines
			.slice(arrowIndex + 1)
			.join(" ")
			.replace(/<[^>]+>/g, "")
			.trim();
		if (text) {
			cues.push({ startSeconds, endSeconds, text });
		}
	}
	return cues;
}

export async function generateVideoSopAndBugReport(videoId: string): Promise<{
	sop: ReturnType<typeof buildVideoSopGuide>;
	bugReport: DeveloperBugReport;
	connectedProviders: ("linear" | "github" | "jira")[];
}> {
	const rows = await db()
		.select()
		.from(videos)
		.where(eq(videos.id, videoId as never))
		.limit(1);
	const video = rows[0];
	if (!video) {
		throw new Error("Video not found");
	}

	let cues: TranscriptCueInput[] = [];
	try {
		const maybeVtt = await runPromise(
			Effect.gen(function* () {
				const s3Buckets = yield* S3Buckets;
				const [bucket] = yield* s3Buckets.getBucketAccess(
					Option.fromNullable(video.bucket),
				);
				return yield* bucket.getObject(
					`${video.ownerId}/${video.id}/transcription.vtt`,
				);
			}),
		).catch(() => Option.none<string>());
		if (Option.isSome(maybeVtt)) {
			cues = parseVttToCues(maybeVtt.value);
		}
	} catch {
		cues = [];
	}

	const appUrl =
		buildEnv.NEXT_PUBLIC_WEB_URL ||
		serverEnv().WEB_URL ||
		"https://cap.link";
	const shareUrl = `${appUrl.replace(/\/$/, "")}/s/${video.id}`;
	const metadata = (video.metadata ?? null) as VideoMetadata | null;
	const title = metadata?.aiTitle || video.name || "Cap Recording";

	const sop = buildVideoSopGuide({
		videoId: video.id,
		title,
		shareUrl,
		metadata,
		cues,
	});

	const bugReport = buildDeveloperBugReport({
		videoId: video.id,
		title,
		shareUrl,
		metadata: { ...metadata, sop },
		cues,
		bundle: metadata?.bugReport ?? null,
	});

	const user = await getCurrentUser().catch(() => null);
	if (user && user.id === video.ownerId) {
		const updatedMetadata: VideoMetadata = {
			...(metadata ?? {}),
			sop,
		};
		await db()
			.update(videos)
			.set({ metadata: updatedMetadata })
			.where(eq(videos.id, video.id));
	}

	const installations = await db()
		.select({ provider: integrationInstallations.provider })
		.from(integrationInstallations)
		.where(
			and(
				eq(integrationInstallations.orgId, video.orgId),
				eq(integrationInstallations.status, "active"),
			),
		)
		.catch(() => []);

	const connectedProviders = installations
		.map((row) => row.provider)
		.filter(
			(p): p is "linear" | "github" | "jira" =>
				p === "linear" || p === "github" || p === "jira",
		);

	return {
		sop,
		bugReport,
		connectedProviders,
	};
}
