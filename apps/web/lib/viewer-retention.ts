import type {
	VideoMetadata,
	VideoViewerSession,
	VideoWatchAlertSettings,
} from "@cap/database/types";

export const RETENTION_BUCKET_COUNT = 20;

export type RetentionBucketMetric = {
	bucketIndex: number;
	startFraction: number;
	endFraction: number;
	watchCount: number;
	retentionPercent: number;
	isHotspot: boolean;
	isDropOff: boolean;
};

export type ComputedRetentionAnalytics = {
	buckets: RetentionBucketMetric[];
	averageCompletionPercent: number;
	hotspotBucketIndex: number | null;
	dropOffBucketIndex: number | null;
	viewerSessions: VideoViewerSession[];
	watchAlertSettings: VideoWatchAlertSettings;
};

export function sanitizeBucketIndices(input: unknown): number[] {
	if (!Array.isArray(input)) return [];
	const unique = new Set<number>();
	for (const item of input) {
		const num = Number(item);
		if (
			Number.isInteger(num) &&
			num >= 0 &&
			num < RETENTION_BUCKET_COUNT
		) {
			unique.add(num);
		}
	}
	return Array.from(unique).sort((a, b) => a - b);
}

export function mergeViewerRetentionSession(params: {
	metadata?: VideoMetadata | null;
	sessionId: string;
	viewerEmail?: string | null;
	viewerName?: string | null;
	location?: string | null;
	watchedBuckets: readonly number[];
	rewatchedBuckets?: readonly number[];
	nowIso?: string;
}): {
	updatedMetadata: VideoMetadata;
	session: VideoViewerSession;
	isNewProspectAlert: boolean;
} {
	const {
		metadata,
		sessionId,
		viewerEmail,
		viewerName,
		location,
		watchedBuckets,
		rewatchedBuckets = [],
		nowIso = new Date().toISOString(),
	} = params;

	const currentBuckets = Array.from(
		{ length: RETENTION_BUCKET_COUNT },
		(_, idx) => Math.max(0, Number(metadata?.retentionBuckets?.[idx] ?? 0)),
	);

	const sessions = [...(metadata?.viewerSessions ?? [])];
	const existingIndex = sessions.findIndex((s) => s.sessionId === sessionId);
	const existingSession =
		existingIndex >= 0 ? sessions[existingIndex] : undefined;

	const prevWatchedSet = new Set<number>(existingSession?.watchedBuckets ?? []);
	const prevRewatchedSet = new Set<number>(
		existingSession?.rewatchedBuckets ?? [],
	);

	const cleanWatched = sanitizeBucketIndices(
		watchedBuckets.length > 0 ? [...watchedBuckets] : [0],
	);
	const cleanRewatched = sanitizeBucketIndices([...rewatchedBuckets]);

	for (const bucket of cleanWatched) {
		if (!prevWatchedSet.has(bucket)) {
			currentBuckets[bucket] = (currentBuckets[bucket] ?? 0) + 1;
			prevWatchedSet.add(bucket);
		}
	}

	for (const bucket of cleanRewatched) {
		if (!prevRewatchedSet.has(bucket)) {
			currentBuckets[bucket] = (currentBuckets[bucket] ?? 0) + 1;
			prevRewatchedSet.add(bucket);
		}
	}

	const mergedWatched = Array.from(prevWatchedSet).sort((a, b) => a - b);
	const mergedRewatched = Array.from(prevRewatchedSet).sort((a, b) => a - b);
	const completionPercent = Math.min(
		100,
		Math.round((mergedWatched.length / RETENTION_BUCKET_COUNT) * 100),
	);

	const updatedSession: VideoViewerSession = {
		sessionId,
		viewerEmail: viewerEmail ?? existingSession?.viewerEmail ?? null,
		viewerName: viewerName ?? existingSession?.viewerName ?? null,
		location: location ?? existingSession?.location ?? null,
		watchedBuckets: mergedWatched,
		rewatchedBuckets: mergedRewatched,
		completionPercent,
		lastWatchedAt: nowIso,
	};

	if (existingIndex >= 0) {
		sessions[existingIndex] = updatedSession;
	} else {
		sessions.unshift(updatedSession);
	}

	const trimmedSessions = sessions.slice(0, 50);
	const alertSettings: VideoWatchAlertSettings =
		metadata?.watchAlertSettings ?? {
			enabled: true,
			notifyOnEachProspectView: true,
			notifyOnHighCompletion: true,
		};

	const isNewProspectAlert =
		alertSettings.enabled &&
		existingIndex === -1 &&
		Boolean(updatedSession.viewerEmail || alertSettings.notifyOnEachProspectView);

	const updatedMetadata: VideoMetadata = {
		...(metadata ?? {}),
		retentionBuckets: currentBuckets,
		viewerSessions: trimmedSessions,
		watchAlertSettings: alertSettings,
	};

	return {
		updatedMetadata,
		session: updatedSession,
		isNewProspectAlert,
	};
}

export function computeRetentionAnalytics(
	metadata?: VideoMetadata | null,
	fallbackTotalViews = 0,
): ComputedRetentionAnalytics {
	const rawBuckets = Array.from(
		{ length: RETENTION_BUCKET_COUNT },
		(_, idx) => Math.max(0, Number(metadata?.retentionBuckets?.[idx] ?? 0)),
	);
	const hasRecordedBuckets = rawBuckets.some((v) => v > 0);
	const baseline = Math.max(
		1,
		rawBuckets[0] || 0,
		...rawBuckets,
		fallbackTotalViews,
	);

	let hotspotBucketIndex: number | null = null;
	let maxRetentionDelta = 0;
	let dropOffBucketIndex: number | null = null;
	let maxDropDelta = 0;

	const buckets: RetentionBucketMetric[] = rawBuckets.map((count, idx) => {
		const effectiveCount = hasRecordedBuckets
			? count
			: fallbackTotalViews > 0
				? Math.max(
						1,
						Math.round(
							fallbackTotalViews * Math.pow(0.94, idx),
						),
					)
				: 0;
		const retentionPercent =
			effectiveCount > 0
				? Math.min(100, Math.round((effectiveCount / baseline) * 100))
				: 0;

		if (idx > 0 && hasRecordedBuckets) {
			const prevCount = rawBuckets[idx - 1] ?? 0;
			const rise = count - prevCount;
			if (rise > maxRetentionDelta) {
				maxRetentionDelta = rise;
				hotspotBucketIndex = idx;
			}
			const drop = prevCount - count;
			if (drop > maxDropDelta) {
				maxDropDelta = drop;
				dropOffBucketIndex = idx;
			}
		}

		return {
			bucketIndex: idx,
			startFraction: idx / RETENTION_BUCKET_COUNT,
			endFraction: (idx + 1) / RETENTION_BUCKET_COUNT,
			watchCount: effectiveCount,
			retentionPercent,
			isHotspot: false,
			isDropOff: false,
		};
	});

	if (hotspotBucketIndex !== null && buckets[hotspotBucketIndex]) {
		buckets[hotspotBucketIndex].isHotspot = true;
	}
	if (dropOffBucketIndex !== null && buckets[dropOffBucketIndex]) {
		buckets[dropOffBucketIndex].isDropOff = true;
	}

	const sessions = metadata?.viewerSessions ?? [];
	const averageCompletionPercent =
		sessions.length > 0
			? Math.round(
					sessions.reduce((acc, s) => acc + s.completionPercent, 0) /
						sessions.length,
				)
			: buckets.length > 0 && buckets.some((b) => b.retentionPercent > 0)
				? Math.round(
						buckets.reduce((acc, b) => acc + b.retentionPercent, 0) /
							buckets.length,
					)
				: 0;

	return {
		buckets,
		averageCompletionPercent,
		hotspotBucketIndex,
		dropOffBucketIndex,
		viewerSessions: sessions,
		watchAlertSettings: metadata?.watchAlertSettings ?? {
			enabled: true,
			notifyOnEachProspectView: true,
			notifyOnHighCompletion: true,
		},
	};
}
