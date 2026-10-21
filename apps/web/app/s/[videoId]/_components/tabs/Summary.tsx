"use client";

import { Button } from "@cap/ui";
import type { Video } from "@cap/web-domain";
import {
	faRectangleList,
	faRotateRight,
} from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { Bug, Check, Copy, Edit3, ExternalLink, FileText } from "lucide-react";
import { useRef, useState } from "react";
import ReactMarkdown from "react-markdown";
import { toast } from "sonner";
import { generateVideoSopAndBugReport } from "@/actions/videos/generate-sop-and-bug-report";
import { type SummaryEditingState, SummaryEditor } from "./SummaryEditor";

type AiGenerationStatus =
	| "QUEUED"
	| "PROCESSING"
	| "COMPLETE"
	| "ERROR"
	| "SKIPPED";

interface Chapter {
	title: string;
	start: number;
}

interface SummaryProps {
	videoId: Video.VideoId;
	onSeek?: (time: number) => void;
	initialAiData?: {
		title?: string | null;
		summary?: string | null;
		chapters?: Chapter[] | null;
		aiGenerationStatus?: AiGenerationStatus | null;
	};
	aiGenerationEnabled?: boolean;
	isSummaryDisabled?: boolean;
	ownerIsPro: boolean;
	isOwner?: boolean;
	transcriptionStatus?: string | null;
	duration?: number | null;
	onEditingStateChange?: (state: SummaryEditingState) => void;
}

const formatTime = (time: number) => {
	const minutes = Math.floor(time / 60);
	const seconds = Math.floor(time % 60);
	return `${minutes.toString().padStart(2, "0")}:${seconds
		.toString()
		.padStart(2, "0")}`;
};

const SkeletonLoader = () => (
	<div className="p-4 space-y-6 animate-pulse">
		<div>
			<div className="mb-3 w-24 h-6 bg-gray-200 rounded"></div>
			<div className="mb-4 w-32 h-3 bg-gray-100 rounded"></div>
			<div className="space-y-3">
				<div className="w-full h-4 bg-gray-200 rounded"></div>
				<div className="w-5/6 h-4 bg-gray-200 rounded"></div>
				<div className="w-4/5 h-4 bg-gray-200 rounded"></div>
				<div className="w-full h-4 bg-gray-200 rounded"></div>
				<div className="w-3/4 h-4 bg-gray-200 rounded"></div>
			</div>
		</div>

		<div>
			<div className="mb-4 w-24 h-6 bg-gray-200 rounded"></div>
			<div className="space-y-2">
				{[1, 2, 3, 4].map((i) => (
					<div key={i} className="flex items-center p-2">
						<div className="mr-3 w-12 h-4 bg-gray-200 rounded"></div>
						<div className="flex-1 h-4 bg-gray-200 rounded"></div>
					</div>
				))}
			</div>
		</div>
	</div>
);

export const Summary: React.FC<SummaryProps> = ({
	videoId,
	onSeek,
	initialAiData,
	isSummaryDisabled = false,
	aiGenerationEnabled = false,
	ownerIsPro,
	isOwner = false,
	transcriptionStatus,
	duration,
	onEditingStateChange,
}) => {
	const [isEditing, setIsEditing] = useState(false);
	const [saved, setSaved] = useState(false);
	const editButtonRef = useRef<HTMLButtonElement>(null);
	const [isRetrying, setIsRetrying] = useState(false);
	const [retryError, setRetryError] = useState<string | null>(null);
	const [sopAndBugData, setSopAndBugData] = useState<Awaited<
		ReturnType<typeof generateVideoSopAndBugReport>
	> | null>(null);
	const [isGeneratingSop, setIsGeneratingSop] = useState(false);
	const [copiedTarget, setCopiedTarget] = useState<"sop" | "bug" | null>(null);

	const handleGenerateSopAndBug = async () => {
		setIsGeneratingSop(true);
		try {
			const data = await generateVideoSopAndBugReport(videoId);
			setSopAndBugData(data);
			toast.success("Step-by-Step SOP & Bug Report Bundle generated.");
		} catch (err) {
			toast.error(
				err instanceof Error ? err.message : "Failed to generate SOP & Bug Report",
			);
		} finally {
			setIsGeneratingSop(false);
		}
	};

	const copyMarkdown = async (type: "sop" | "bug", content: string) => {
		await navigator.clipboard.writeText(content);
		setCopiedTarget(type);
		toast.success(
			type === "sop"
				? "SOP Guide copied for Notion / Confluence / Markdown"
				: "Developer Bug Report copied to clipboard",
		);
		window.setTimeout(() => setCopiedTarget(null), 2000);
	};

	const aiData = initialAiData || null;
	const aiGenerationStatus = aiData?.aiGenerationStatus;

	const isLoading =
		aiGenerationEnabled &&
		(aiGenerationStatus === "QUEUED" || aiGenerationStatus === "PROCESSING") &&
		!aiData?.summary &&
		!aiData?.chapters?.length;

	const canRetry =
		isOwner &&
		(aiGenerationStatus === "ERROR" || aiGenerationStatus === "SKIPPED");
	const canEdit =
		isOwner &&
		transcriptionStatus !== "PROCESSING" &&
		aiGenerationStatus !== "QUEUED" &&
		aiGenerationStatus !== "PROCESSING";
	const editButton = canEdit ? (
		<Button
			ref={editButtonRef}
			type="button"
			variant="gray"
			size="sm"
			aria-label="Edit summary and chapters"
			onClick={() => {
				setSaved(false);
				setIsEditing(true);
			}}
		>
			<Edit3 className="mr-1.5 size-3.5" /> Edit
		</Button>
	) : null;

	const handleSeek = (time: number) => {
		if (onSeek) {
			onSeek(time);
		}
	};

	const handleRetry = async () => {
		setIsRetrying(true);
		setRetryError(null);

		try {
			const response = await fetch(`/api/videos/${videoId}/retry-ai`, {
				method: "POST",
			});

			if (!response.ok) {
				const data = await response.json();
				throw new Error(data.error || "Failed to retry AI generation");
			}

			window.location.reload();
		} catch (error) {
			setRetryError(
				error instanceof Error
					? error.message
					: "Failed to retry AI generation",
			);
		} finally {
			setIsRetrying(false);
		}
	};

	if (!ownerIsPro) {
		return (
			<div className="flex flex-col justify-center items-center p-8 h-full text-center">
				<div className="space-y-4 max-w-sm">
					<div className="p-6 bg-gradient-to-br from-blue-50 to-purple-50 rounded-lg border border-blue-100">
						<div className="mb-3 text-blue-600">
							<svg
								aria-hidden="true"
								xmlns="http://www.w3.org/2000/svg"
								className="mx-auto w-12 h-12"
								fill="none"
								viewBox="0 0 24 24"
								stroke="currentColor"
							>
								<path
									strokeLinecap="round"
									strokeLinejoin="round"
									strokeWidth={1.5}
									d="M9.813 15.904L9 18.75l-.813-2.846a4.5 4.5 0 00-3.09-3.09L2.25 12l2.846-.813a4.5 4.5 0 003.09-3.09L9 5.25l.813 2.846a4.5 4.5 0 003.09 3.09L15.75 12l-2.846.813a4.5 4.5 0 00-3.09 3.09zM18.259 8.715L18 9.75l-.259-1.035a3.375 3.375 0 00-2.455-2.456L14.25 6l1.036-.259a3.375 3.375 0 002.455-2.456L18 2.25l.259 1.035a3.375 3.375 0 002.456 2.456L21.75 6l-1.035.259a3.375 3.375 0 00-2.456 2.456z"
								/>
							</svg>
						</div>
						<h3 className="mb-2 text-lg font-semibold text-gray-900">
							Unlock Cap AI
						</h3>
						<p className="mb-4 text-sm leading-relaxed text-gray-600">
							Upgrade to Cap Pro to access AI-powered features including
							automatic titles, video summaries, and intelligent chapter
							generation.
						</p>
						<Button
							href="/pricing"
							variant="primary"
							size="sm"
							className="mx-auto"
						>
							Upgrade to Cap Pro
						</Button>
					</div>
				</div>
			</div>
		);
	}

	if (isSummaryDisabled) return null;

	if (isEditing && isOwner) {
		return (
			<SummaryEditor
				videoId={videoId}
				initialContent={{
					summary: aiData?.summary || "",
					chapters: aiData?.chapters ?? [],
				}}
				duration={duration}
				onEditingStateChange={onEditingStateChange}
				onClose={(didSave) => {
					setIsEditing(false);
					setSaved(didSave);
					requestAnimationFrame(() => editButtonRef.current?.focus());
				}}
			/>
		);
	}

	if (isLoading) {
		return (
			<div className="flex flex-col h-full">
				<div className="overflow-y-auto flex-1">
					<SkeletonLoader />
				</div>
			</div>
		);
	}

	if (!aiData?.summary && !aiData?.chapters?.length) {
		return (
			<div className="flex flex-col justify-center items-center p-8 h-full text-center">
				<FontAwesomeIcon
					icon={faRectangleList}
					className="mb-4 text-gray-12 size-8"
				/>
				<div className="space-y-1">
					<h3 className="text-base font-medium text-gray-12">
						{aiGenerationStatus === "ERROR"
							? "AI generation failed"
							: aiGenerationStatus === "SKIPPED"
								? "AI generation skipped"
								: "No summary available"}
					</h3>
					<p className="text-sm text-gray-10">
						{aiGenerationStatus === "ERROR"
							? "There was an error generating the AI summary."
							: aiGenerationStatus === "SKIPPED"
								? "The video was too short or had no speech detected."
								: "AI summary has not been generated for this video yet."}
					</p>
					{canEdit && (
						<div className="flex justify-center pt-4">{editButton}</div>
					)}
					{canRetry && (
						<div className="pt-4">
							<Button
								variant="gray"
								size="sm"
								onClick={handleRetry}
								disabled={isRetrying}
							>
								<FontAwesomeIcon
									icon={faRotateRight}
									className={`mr-2 ${isRetrying ? "animate-spin" : ""}`}
								/>
								{isRetrying ? "Retrying..." : "Retry AI Generation"}
							</Button>
							{retryError && (
								<p className="mt-2 text-sm text-red-500">{retryError}</p>
							)}
						</div>
					)}
				</div>
			</div>
		);
	}

	return (
		<div className="flex flex-col h-full">
			<div className="overflow-y-auto flex-1">
				<div className="p-4 space-y-6">
					{canEdit && (
						<div className="flex items-center justify-between gap-2">
							<output className="text-xs text-gray-10">
								{saved ? "Changes saved" : "Summary and chapters"}
							</output>
							{editButton}
						</div>
					)}
					{aiData?.summary && (
						<div>
							<h3 className="text-lg font-medium">Summary</h3>
							<div className="text-sm text-gray-12 prose prose-sm prose-gray max-w-none prose-p:my-2 prose-ul:my-2 prose-li:my-0 prose-strong:text-gray-12">
								<ReactMarkdown>{aiData.summary}</ReactMarkdown>
							</div>
						</div>
					)}

					{aiData?.chapters && aiData.chapters.length > 0 && (
						<div className={aiData?.summary ? "mt-6" : ""}>
							<h3 className="mb-2 text-lg font-medium">Chapters</h3>
							<div className="divide-y">
								{aiData.chapters.map((chapter) => (
									<button
										type="button"
										key={chapter.start}
										className="flex items-center w-full p-2 text-left rounded transition-colors hover:bg-gray-3 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-9"
										onClick={() => handleSeek(chapter.start)}
									>
										<span className="w-16 text-xs text-gray-500">
											{formatTime(chapter.start)}
										</span>
										<span className="ml-2 text-sm">{chapter.title}</span>
									</button>
								))}
							</div>
						</div>
					)}

					<div className="pt-4 border-t border-gray-4 space-y-4">
						<div className="flex flex-wrap items-center justify-between gap-2">
							<div>
								<h3 className="text-sm font-semibold text-gray-12">
									Step-by-Step SOP & Dev Bug Bundle
								</h3>
								<p className="text-xs text-gray-10">
									Turn this recording into a Notion/Markdown SOP or a 1-click
									Linear/GitHub/Jira bug report.
								</p>
							</div>
							<Button
								type="button"
								variant="gray"
								size="sm"
								disabled={isGeneratingSop}
								onClick={() => void handleGenerateSopAndBug()}
							>
								<FileText className="mr-1.5 size-3.5" />
								{isGeneratingSop
									? "Generating..."
									: sopAndBugData
										? "Refresh SOP & Bug Bundle"
										: "Generate SOP & Bug Bundle"}
							</Button>
						</div>

						{sopAndBugData && (
							<div className="space-y-4">
								<div className="rounded-xl border border-gray-4 bg-gray-2 p-3.5 space-y-3">
									<div className="flex items-center justify-between gap-2">
										<span className="text-xs font-semibold text-gray-12">
											Step-by-Step Guide ({sopAndBugData.sop.steps.length} steps)
										</span>
										<button
											type="button"
											onClick={() =>
												void copyMarkdown("sop", sopAndBugData.sop.markdown)
											}
											className="inline-flex items-center gap-1 rounded-md border border-gray-5 bg-gray-1 px-2.5 py-1 text-xs font-medium text-gray-12 transition hover:bg-gray-3"
										>
											{copiedTarget === "sop" ? (
												<Check className="size-3 text-green-600" />
											) : (
												<Copy className="size-3" />
											)}
											Copy Markdown / Notion
										</button>
									</div>
									<div className="space-y-2">
										{sopAndBugData.sop.steps.map((step) => (
											<button
												type="button"
												key={step.stepNumber}
												onClick={() => handleSeek(step.timestampSeconds)}
												className="flex w-full items-start gap-2.5 rounded-lg bg-gray-1 p-2.5 text-left border border-gray-3 transition hover:border-gray-5"
											>
												<span className="mt-0.5 inline-flex size-5 shrink-0 items-center justify-center rounded-full bg-blue-500/10 text-[11px] font-semibold text-blue-600">
													{step.stepNumber}
												</span>
												<div className="min-w-0 flex-1">
													<div className="flex items-center justify-between gap-2">
														<span className="text-xs font-semibold text-gray-12 truncate">
															{step.title}
														</span>
														<span className="font-mono text-[10px] text-gray-9 shrink-0">
															{step.formattedTimestamp}
														</span>
													</div>
													<p className="mt-0.5 text-xs text-gray-11 line-clamp-2">
														{step.instruction}
													</p>
												</div>
											</button>
										))}
									</div>
								</div>

								<div className="rounded-xl border border-gray-4 bg-gray-2 p-3.5 space-y-3">
									<div className="flex items-center justify-between gap-2">
										<div className="flex items-center gap-1.5">
											<Bug className="size-3.5 text-gray-11" />
											<span className="text-xs font-semibold text-gray-12">
												1-Click Issue Export (Linear / GitHub / Jira)
											</span>
										</div>
										<button
											type="button"
											onClick={() =>
												void copyMarkdown(
													"bug",
													sopAndBugData.bugReport.markdown,
												)
											}
											className="inline-flex items-center gap-1 rounded-md border border-gray-5 bg-gray-1 px-2.5 py-1 text-xs font-medium text-gray-12 transition hover:bg-gray-3"
										>
											{copiedTarget === "bug" ? (
												<Check className="size-3 text-green-600" />
											) : (
												<Copy className="size-3" />
											)}
											Copy Bug Bundle
										</button>
									</div>
									<div className="flex flex-wrap items-center gap-2">
										<a
											href={sopAndBugData.bugReport.linearUrl}
											target="_blank"
											rel="noopener noreferrer"
											className="inline-flex items-center gap-1.5 rounded-lg border border-gray-4 bg-gray-1 px-3 py-1.5 text-xs font-medium text-gray-12 transition hover:bg-gray-3"
										>
											Create Linear Issue
											<ExternalLink className="size-3 text-gray-9" />
										</a>
										<a
											href={sopAndBugData.bugReport.githubUrl}
											target="_blank"
											rel="noopener noreferrer"
											className="inline-flex items-center gap-1.5 rounded-lg border border-gray-4 bg-gray-1 px-3 py-1.5 text-xs font-medium text-gray-12 transition hover:bg-gray-3"
										>
											Create GitHub Issue
											<ExternalLink className="size-3 text-gray-9" />
										</a>
										<a
											href={sopAndBugData.bugReport.jiraUrl}
											target="_blank"
											rel="noopener noreferrer"
											onClick={() => {
												void navigator.clipboard.writeText(
													sopAndBugData.bugReport.markdown,
												);
											}}
											className="inline-flex items-center gap-1.5 rounded-lg border border-gray-4 bg-gray-1 px-3 py-1.5 text-xs font-medium text-gray-12 transition hover:bg-gray-3"
										>
											Create Jira Issue
											<ExternalLink className="size-3 text-gray-9" />
										</a>
									</div>
								</div>
							</div>
						)}
					</div>
				</div>
			</div>
		</div>
	);
};
