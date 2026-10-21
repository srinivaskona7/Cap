"use server";

import { db } from "@cap/database";
import { getCurrentUser } from "@cap/database/auth/session";
import { videos, videoViewerGrants } from "@cap/database/schema";
import type {
	VideoMetadata,
	VideoWatchAlertSettings,
} from "@cap/database/types";
import { eq } from "drizzle-orm";
import {
	type ComputedRetentionAnalytics,
	computeRetentionAnalytics,
} from "@/lib/viewer-retention";

export async function getVideoRetentionAnalytics(
	videoId: string,
	fallbackViews = 0,
): Promise<
	ComputedRetentionAnalytics & {
		invitedProspects: { email: string; grantedAt: string }[];
	}
> {
	const [video] = await db()
		.select({
			id: videos.id,
			ownerId: videos.ownerId,
			metadata: videos.metadata,
		})
		.from(videos)
		.where(eq(videos.id, videoId as never))
		.limit(1);

	const metadata = (video?.metadata ?? null) as VideoMetadata | null;
	const computed = computeRetentionAnalytics(metadata, fallbackViews);

	const grants = await db()
		.select({
			email: videoViewerGrants.email,
			createdAt: videoViewerGrants.createdAt,
		})
		.from(videoViewerGrants)
		.where(eq(videoViewerGrants.videoId, videoId as never))
		.catch(() => []);

	return {
		...computed,
		invitedProspects: grants
			.filter((g): g is { email: string; createdAt: Date } => Boolean(g.email))
			.map((g) => ({
				email: g.email,
				grantedAt: g.createdAt.toISOString(),
			})),
	};
}

export async function updateVideoWatchAlertSettings(
	videoId: string,
	settings: VideoWatchAlertSettings,
) {
	const user = await getCurrentUser();
	if (!user) throw new Error("Unauthorized");

	const [video] = await db()
		.select({
			id: videos.id,
			ownerId: videos.ownerId,
			metadata: videos.metadata,
		})
		.from(videos)
		.where(eq(videos.id, videoId as never))
		.limit(1);

	if (!video || video.ownerId !== user.id) {
		throw new Error("Only the video owner can update watch alert settings");
	}

	const currentMeta = (video.metadata ?? {}) as VideoMetadata;
	const updatedMetadata: VideoMetadata = {
		...currentMeta,
		watchAlertSettings: settings,
	};

	await db()
		.update(videos)
		.set({ metadata: updatedMetadata })
		.where(eq(videos.id, video.id));

	return settings;
}
