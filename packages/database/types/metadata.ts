/**
 * Type definitions for JSON metadata fields
 */

/**
 * Video metadata structure
 */
export interface VideoMetadata {
	editProcessing?: {
		token: string;
		startedAt: string;
		ownerId: string;
		bucket: string | null;
		storageIntegrationId: string | null;
		sourceKey: string;
		source: string;
		dispatch: "pending" | "dispatching" | "accepted";
		jobId?: string;
		resultCommitted?: boolean;
		renderedMetadata?: {
			duration: number;
			width: number;
			height: number;
			fps: number;
		};
	};
	completedVideoEdit?: {
		token: string;
		startedAt: string;
		transcriptRemapped: boolean;
	};
	desktopRecordingUpload?: {
		version: 1;
		artifact:
			| { kind: "segments"; manifestSha256: string }
			| {
					kind: "mp4";
					fileSize: number;
					duration: number;
					objectIdentity: string;
			  };
		fileSize: number;
		duration: number;
		hasAudio: boolean;
		fullDecode: true;
		requiredAudioVerified?: boolean;
		objectIdentity: string;
		outputKey?: string;
		outputSha256?: string;
		sourceObjectIdentity?: string;
		sourceProof?: {
			version: 1;
			manifestSha256: string;
			inventorySha256: string;
			sourcePreserved: true;
			videoDuration: number;
			hasAudio: boolean;
			audioVerified: boolean;
		};
	};
	/**
	 * Custom created date that can be edited by the user
	 * This overrides the display of the actual createdAt timestamp
	 */
	customCreatedAt?: string;
	/**
	 * Title of the captured monitor or window
	 */
	sourceName?: string;
	/**
	 * AI generated title for the video
	 */
	aiTitle?: string;
	titleManuallyEdited?: boolean;
	/**
	 * AI generated summary of the content
	 */
	summary?: string;
	summaryManuallyEdited?: boolean;
	/**
	 * Chapter markers generated from the transcript
	 */
	chapters?: { title: string; start: number }[];
	chaptersManuallyEdited?: boolean;
	aiGenerationStatus?:
		| "QUEUED"
		| "PROCESSING"
		| "COMPLETE"
		| "ERROR"
		| "SKIPPED";
	/**
	 * Progress of the provisional live transcription that runs while an
	 * instant-mode recording is still uploading. The transcript content lives
	 * in `transcription.live.json` next to the video; this only gates UI/queue
	 * behavior. Cleared when the canonical transcription completes.
	 */
	liveTranscript?: {
		status: "active" | "complete" | "stopped";
		updatedAt?: string;
	};
	enhancedAudioStatus?: "PROCESSING" | "COMPLETE" | "ERROR" | "SKIPPED";
	agentUpload?: {
		state: "pending" | "accepted" | "rejected";
		rawFileKey?: string;
	};
	sop?: VideoSopGuide;
	bugReport?: VideoBugReportBundle;
	retentionBuckets?: number[];
	viewerSessions?: VideoViewerSession[];
	watchAlertSettings?: VideoWatchAlertSettings;
	ocrText?: string;
}

export interface VideoSopStep {
	stepNumber: number;
	title: string;
	instruction: string;
	timestampSeconds: number;
	formattedTimestamp: string;
	screenshotUrl: string;
	clickTarget?: {
		x: number;
		y: number;
	};
}

export interface VideoSopGuide {
	title: string;
	overview: string;
	steps: VideoSopStep[];
	markdown: string;
	generatedAt: string;
}

export interface CapturedConsoleEntry {
	timestampMs: number;
	level: "log" | "warn" | "error";
	message: string;
	sourceUrl?: string;
}

export interface CapturedNetworkEntry {
	timestampMs: number;
	method: string;
	url: string;
	status: number;
	durationMs: number;
	failed: boolean;
}

export interface VideoBugReportBundle {
	consoleLogs?: CapturedConsoleEntry[];
	networkLogs?: CapturedNetworkEntry[];
	environment?: {
		userAgent?: string;
		url?: string;
		viewport?: string;
		platform?: string;
		recordedAt?: string;
	};
	cursorClicks?: {
		timeMs: number;
		x: number;
		y: number;
		label?: string;
	}[];
}

export interface VideoViewerSession {
	sessionId: string;
	viewerEmail?: string | null;
	viewerName?: string | null;
	location?: string | null;
	watchedBuckets: number[];
	rewatchedBuckets: number[];
	completionPercent: number;
	lastWatchedAt: string;
}

export interface VideoWatchAlertSettings {
	enabled: boolean;
	notifyOnEachProspectView: boolean;
	notifyOnHighCompletion?: boolean;
}

export interface VideoCallToAction {
	label: string;
	url: string;
	headline?: string;
	color?: string;
	showWhilePlaying?: boolean;
}

export type VideoEditRange = {
	start: number;
	end: number;
};

export type VideoEditSpec = {
	version: 1;
	sourceDuration: number;
	keepRanges: VideoEditRange[];
};

/**
 * Space metadata structure
 */
export interface SpaceMetadata {
	[key: string]: never;
}

/**
 * User metadata structure
 */
export interface UserMetadata {
	[key: string]: never;
}
