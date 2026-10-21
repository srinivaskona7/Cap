"use client";

import { Bell, Check, Copy, Flame } from "lucide-react";
import { use, useEffect, useMemo, useState } from "react";
import { toast } from "sonner";
import { getVideoAnalytics } from "@/actions/videos/get-analytics";
import {
	getVideoRetentionAnalytics,
	updateVideoWatchAlertSettings,
} from "@/actions/videos/retention-analytics";
import { CapCardAnalytics } from "@/app/(org)/dashboard/caps/components/CapCard/CapCardAnalytics";
import type { CommentType } from "../../../Share";

const Analytics = (props: {
	videoId: string;
	views: MaybePromise<number>;
	comments: CommentType[];
	isLoadingAnalytics: boolean;
	isOwner: boolean;
	onSeek?: (time: number) => void;
}) => {
	const [views, setViews] = useState(
		props.views instanceof Promise ? use(props.views) : props.views,
	);
	const [showRetention, setShowRetention] = useState(false);
	const [retention, setRetention] = useState<Awaited<
		ReturnType<typeof getVideoRetentionAnalytics>
	> | null>(null);
	const [prospectEmail, setProspectEmail] = useState("");
	const [copiedProspect, setCopiedProspect] = useState(false);

	useEffect(() => {
		const fetchAnalytics = async () => {
			try {
				const result = await getVideoAnalytics(props.videoId);
				setViews(result.count);
				if (props.isOwner) {
					const ret = await getVideoRetentionAnalytics(
						props.videoId,
						result.count,
					);
					setRetention(ret);
				}
			} catch (error) {
				console.error("Error fetching analytics:", error);
			}
		};

		void fetchAnalytics();
	}, [props.isOwner, props.videoId]);

	const totalComments = useMemo(
		() => props.comments.filter((c) => c.type === "text").length,
		[props.comments],
	);

	const totalReactions = useMemo(
		() => props.comments.filter((c) => c.type === "emoji").length,
		[props.comments],
	);

	const copyProspectLink = async () => {
		const email = prospectEmail.trim();
		const baseUrl =
			typeof window !== "undefined"
				? `${window.location.origin}/s/${props.videoId}`
				: `https://cap.link/s/${props.videoId}`;
		const url = email
			? `${baseUrl}?prospect=${encodeURIComponent(email)}`
			: baseUrl;
		await navigator.clipboard.writeText(url);
		setCopiedProspect(true);
		toast.success(
			email
				? `Tracked link for ${email} copied to clipboard`
				: "Share link copied to clipboard",
		);
		window.setTimeout(() => setCopiedProspect(false), 2000);
	};

	const toggleWatchAlerts = async () => {
		if (!retention) return;
		const nextSettings = {
			...retention.watchAlertSettings,
			enabled: !retention.watchAlertSettings.enabled,
		};
		try {
			await updateVideoWatchAlertSettings(props.videoId, nextSettings);
			setRetention({ ...retention, watchAlertSettings: nextSettings });
			toast.success(
				nextSettings.enabled
					? "Per-prospect watch alerts enabled"
					: "Per-prospect watch alerts paused",
			);
		} catch {
			toast.error("Failed to update watch alert settings");
		}
	};

	return (
		<div className="w-full space-y-3">
			<div className="flex items-center justify-between gap-2">
				<CapCardAnalytics
					isLoadingAnalytics={props.isLoadingAnalytics}
					capId={props.videoId}
					displayCount={views}
					totalComments={totalComments}
					totalReactions={totalReactions}
					isOwner={props.isOwner}
				/>
				{props.isOwner && (
					<button
						type="button"
						onClick={() => setShowRetention((prev) => !prev)}
						className="inline-flex items-center gap-1.5 rounded-full border border-gray-4 bg-gray-2 px-2.5 py-1 text-[11px] font-medium text-gray-12 transition hover:bg-gray-3"
					>
						<Flame className="size-3 text-amber-500" />
						{showRetention ? "Hide Heatmap" : "Retention & Watch Alerts"}
					</button>
				)}
			</div>

			{props.isOwner && showRetention && retention && (
				<div className="rounded-xl border border-gray-4 bg-gray-2 p-3 space-y-3">
					<div className="flex items-center justify-between gap-2">
						<div>
							<p className="text-xs font-semibold text-gray-12">
								Viewer Retention Heatmap
							</p>
							<p className="text-[11px] text-gray-10">
								Avg. completion {retention.averageCompletionPercent}% · Click a
								bar to jump to segment
							</p>
						</div>
						<button
							type="button"
							onClick={() => void toggleWatchAlerts()}
							className={[
								"inline-flex items-center gap-1 rounded-full border px-2.5 py-1 text-[10px] font-medium transition",
								retention.watchAlertSettings.enabled
									? "border-blue-500/30 bg-blue-500/10 text-blue-600"
									: "border-gray-4 bg-gray-1 text-gray-10",
							].join(" ")}
						>
							<Bell className="size-3" />
							{retention.watchAlertSettings.enabled
								? "Watch Alerts On"
								: "Watch Alerts Off"}
						</button>
					</div>

					<div className="flex h-16 items-end gap-1 rounded-lg bg-gray-1 p-2 border border-gray-3">
						{retention.buckets.map((bucket) => {
							const heightPct = Math.max(12, bucket.retentionPercent);
							const barColor = bucket.isHotspot
								? "bg-purple-500"
								: bucket.isDropOff
									? "bg-amber-500"
									: "bg-blue-500/80";
							return (
								<button
									type="button"
									key={bucket.bucketIndex}
									onClick={() => {
										if (props.onSeek) {
											props.onSeek(Math.round(bucket.startFraction * 60));
										}
									}}
									title={`Segment ${bucket.bucketIndex + 1}/20: ${bucket.retentionPercent}% retention (${bucket.watchCount} views)${bucket.isHotspot ? " · Re-watched Hotspot" : ""}${bucket.isDropOff ? " · Biggest Drop-off" : ""}`}
									className="group flex h-full flex-1 items-end"
								>
									<span
										style={{ height: `${heightPct}%` }}
										className={`w-full rounded-sm transition group-hover:opacity-100 opacity-85 ${barColor}`}
									/>
								</button>
							);
						})}
					</div>

					<div className="flex items-center gap-1.5">
						<input
							type="email"
							value={prospectEmail}
							onChange={(e) => setProspectEmail(e.target.value)}
							placeholder="lead@company.com (tracked prospect link)"
							className="h-7 min-w-0 flex-1 rounded-lg border border-gray-4 bg-gray-1 px-2.5 text-xs text-gray-12 outline-none focus:border-blue-500"
						/>
						<button
							type="button"
							onClick={() => void copyProspectLink()}
							className="inline-flex h-7 shrink-0 items-center gap-1 rounded-lg border border-gray-5 bg-gray-1 px-2.5 text-[11px] font-medium text-gray-12 transition hover:bg-gray-3"
						>
							{copiedProspect ? (
								<Check className="size-3 text-green-600" />
							) : (
								<Copy className="size-3" />
							)}
							Copy Tracked Link
						</button>
					</div>

					{retention.viewerSessions.length > 0 && (
						<div className="space-y-1.5 pt-1 border-t border-gray-3">
							<p className="text-[11px] font-semibold text-gray-11">
								Recent Prospect & Viewer Sessions
							</p>
							<div className="max-h-32 overflow-y-auto space-y-1">
								{retention.viewerSessions.slice(0, 8).map((session) => (
									<div
										key={session.sessionId}
										className="flex items-center justify-between gap-2 rounded-md bg-gray-1 px-2.5 py-1.5 text-[11px]"
									>
										<span className="font-medium text-gray-12 truncate">
											{session.viewerEmail ||
												session.viewerName ||
												"Anonymous Viewer"}
										</span>
										<span className="shrink-0 font-mono text-[10px] text-gray-10">
											{session.completionPercent}% watched
											{session.rewatchedBuckets.length > 0
												? ` · ${session.rewatchedBuckets.length} re-watched`
												: ""}
										</span>
									</div>
								))}
							</div>
						</div>
					)}
				</div>
			)}
		</div>
	);
};

export default Analytics;
