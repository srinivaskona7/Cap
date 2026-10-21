export type ExtensionConsoleLog = {
	timestampMs: number;
	level: "log" | "warn" | "error";
	message: string;
	sourceUrl?: string;
};

export type ExtensionNetworkLog = {
	timestampMs: number;
	method: string;
	url: string;
	status: number;
	durationMs: number;
	failed: boolean;
};

export type ExtensionCursorClick = {
	timeMs: number;
	x: number;
	y: number;
	label?: string;
};

export type ExtensionBugReportBundle = {
	consoleLogs: ExtensionConsoleLog[];
	networkLogs: ExtensionNetworkLog[];
	cursorClicks: ExtensionCursorClick[];
	environment: {
		userAgent: string;
		url: string;
		viewport: string;
		platform: string;
		recordedAt: string;
	};
};

const SECRET_REDACT_PATTERN =
	/\b(sk-[a-z0-9_-]{8,}|pk_(?:live|test)_[a-z0-9]{8,}|gh[pousr]_[a-z0-9]{8,}|github_pat_[a-z0-9_]{8,}|xox[baprs]-[a-z0-9-]{8,}|akia[0-9a-z]{12,}|eyj[a-z0-9_-]{10,}\.[a-z0-9_-]{10,}\.[a-z0-9_-]{5,}|bearer\s+[a-z0-9._~+/=-]{8,})/gi;

export function sanitizeDiagnosticString(input: string): string {
	return input.replace(SECRET_REDACT_PATTERN, "[REDACTED_SECRET]");
}

export function sanitizeDiagnosticUrl(rawUrl: string): string {
	try {
		const parsed = new URL(rawUrl, "https://cap.so");
		for (const key of Array.from(parsed.searchParams.keys())) {
			if (/token|key|secret|password|auth|sig|code/i.test(key)) {
				parsed.searchParams.set(key, "REDACTED");
			}
		}
		return parsed.origin === "https://cap.so" && !rawUrl.startsWith("http")
			? `${parsed.pathname}${parsed.search}`
			: parsed.toString();
	} catch {
		return sanitizeDiagnosticString(rawUrl);
	}
}

export class BugReportCollector {
	private startedAtMs: number;
	private consoleLogs: ExtensionConsoleLog[] = [];
	private networkLogs: ExtensionNetworkLog[] = [];
	private cursorClicks: ExtensionCursorClick[] = [];

	constructor(startedAtMs = Date.now()) {
		this.startedAtMs = startedAtMs;
	}

	recordConsole(
		level: "log" | "warn" | "error",
		message: string,
		sourceUrl?: string,
		nowMs = Date.now(),
	) {
		if (this.consoleLogs.length >= 100) return;
		this.consoleLogs.push({
			timestampMs: Math.max(0, nowMs - this.startedAtMs),
			level,
			message: sanitizeDiagnosticString(message).slice(0, 500),
			sourceUrl: sourceUrl ? sanitizeDiagnosticUrl(sourceUrl) : undefined,
		});
	}

	recordNetwork(
		entry: {
			method: string;
			url: string;
			status: number;
			durationMs: number;
			failed?: boolean;
		},
		nowMs = Date.now(),
	) {
		if (this.networkLogs.length >= 100) return;
		const failed = entry.failed ?? (entry.status === 0 || entry.status >= 400);
		this.networkLogs.push({
			timestampMs: Math.max(0, nowMs - this.startedAtMs),
			method: entry.method.toUpperCase(),
			url: sanitizeDiagnosticUrl(entry.url),
			status: entry.status,
			durationMs: Math.max(0, Math.round(entry.durationMs)),
			failed,
		});
	}

	recordClick(
		event: {
			clientX: number;
			clientY: number;
			viewportWidth: number;
			viewportHeight: number;
			label?: string;
		},
		nowMs = Date.now(),
	) {
		if (this.cursorClicks.length >= 100) return;
		const x =
			event.viewportWidth > 0
				? Math.min(1, Math.max(0, event.clientX / event.viewportWidth))
				: 0.5;
		const y =
			event.viewportHeight > 0
				? Math.min(1, Math.max(0, event.clientY / event.viewportHeight))
				: 0.5;
		this.cursorClicks.push({
			timeMs: Math.max(0, nowMs - this.startedAtMs),
			x,
			y,
			label: event.label
				? sanitizeDiagnosticString(event.label).slice(0, 120)
				: undefined,
		});
	}

	buildBundle(env?: {
		userAgent?: string;
		url?: string;
		viewport?: string;
		platform?: string;
	}): ExtensionBugReportBundle {
		return {
			consoleLogs: [...this.consoleLogs],
			networkLogs: [...this.networkLogs],
			cursorClicks: [...this.cursorClicks],
			environment: {
				userAgent:
					env?.userAgent ??
					(typeof navigator !== "undefined" ? navigator.userAgent : "Chrome"),
				url: env?.url
					? sanitizeDiagnosticUrl(env.url)
					: typeof location !== "undefined"
						? sanitizeDiagnosticUrl(location.href)
						: "",
				viewport:
					env?.viewport ??
					(typeof window !== "undefined"
						? `${window.innerWidth}x${window.innerHeight}`
						: "unknown"),
				platform:
					env?.platform ??
					(typeof navigator !== "undefined" ? navigator.platform : "web"),
				recordedAt: new Date(this.startedAtMs).toISOString(),
			},
		};
	}
}
