"use client";

import { ExternalLink, Search, Sparkles, X } from "lucide-react";
import Link from "next/link";
import { type FormEvent, useState } from "react";
import { toast } from "sonner";
import { askWorkspaceVideos } from "@/actions/videos/ask-workspace";
import type { WorkspaceSearchResponse } from "@/lib/workspace-semantic-search";

export function AskWorkspaceBar() {
	const [query, setQuery] = useState("");
	const [isSearching, setIsSearching] = useState(false);
	const [result, setResult] = useState<WorkspaceSearchResponse | null>(null);

	const handleSubmit = async (event: FormEvent) => {
		event.preventDefault();
		const trimmed = query.trim();
		if (!trimmed) return;
		setIsSearching(true);
		try {
			const response = await askWorkspaceVideos(trimmed);
			setResult(response);
		} catch (error) {
			toast.error(
				error instanceof Error
					? error.message
					: "Failed to search workspace videos",
			);
		} finally {
			setIsSearching(false);
		}
	};

	return (
		<div className="mb-8 w-full space-y-3">
			<form
				onSubmit={(e) => void handleSubmit(e)}
				className="flex items-center gap-2 rounded-2xl border border-gray-4 bg-gray-2 px-3.5 py-2 transition focus-within:border-blue-500/60 focus-within:bg-gray-1"
			>
				<Sparkles className="size-4 shrink-0 text-blue-500" />
				<input
					type="search"
					value={query}
					onChange={(e) => setQuery(e.target.value)}
					placeholder="Ask My Team's Videos — Search spoken words, SOP steps, chapters & on-screen OCR..."
					className="min-w-0 flex-1 bg-transparent text-sm text-gray-12 placeholder:text-gray-9 outline-none"
				/>
				{query && (
					<button
						type="button"
						onClick={() => {
							setQuery("");
							setResult(null);
						}}
						aria-label="Clear workspace search"
						className="inline-flex size-6 items-center justify-center rounded-full text-gray-9 hover:bg-gray-3 hover:text-gray-12"
					>
						<X className="size-3.5" />
					</button>
				)}
				<button
					type="submit"
					disabled={isSearching || !query.trim()}
					className="inline-flex h-8 shrink-0 items-center gap-1.5 rounded-xl bg-gray-12 px-3.5 text-xs font-medium text-gray-1 transition hover:opacity-90 disabled:opacity-40"
				>
					<Search className="size-3.5" />
					{isSearching ? "Searching..." : "Ask Videos"}
				</button>
			</form>

			{result && (
				<div className="rounded-2xl border border-gray-4 bg-gray-2 p-4 space-y-3">
					<div className="flex items-center justify-between gap-2">
						<div className="flex items-center gap-1.5">
							<Sparkles className="size-3.5 text-blue-500" />
							<span className="text-xs font-semibold text-gray-12">
								Workspace AI Search Results ({result.hits.length})
							</span>
						</div>
						<button
							type="button"
							onClick={() => setResult(null)}
							className="text-xs text-gray-10 hover:text-gray-12"
						>
							Dismiss
						</button>
					</div>

					{result.hits.length === 0 ? (
						<p className="text-xs text-gray-10">{result.synthesizedAnswer}</p>
					) : (
						<div className="grid grid-cols-1 gap-2.5 md:grid-cols-2">
							{result.hits.map((hit) => (
								<div
									key={hit.videoId}
									className="rounded-xl border border-gray-4 bg-gray-1 p-3 space-y-2"
								>
									<div className="flex items-center justify-between gap-2">
										<Link
											href={`/s/${hit.videoId}`}
											className="text-sm font-semibold text-gray-12 hover:text-blue-600 truncate"
										>
											{hit.title}
										</Link>
										<Link
											href={`/s/${hit.videoId}`}
											className="inline-flex items-center gap-1 text-[11px] font-medium text-blue-600 shrink-0"
										>
											Open
											<ExternalLink className="size-3" />
										</Link>
									</div>
									{hit.snippets.length > 0 ? (
										<div className="space-y-1.5">
											{hit.snippets.slice(0, 2).map((snippet, idx) => (
												<Link
													key={`${hit.videoId}-${idx}`}
													href={`/s/${hit.videoId}?t=${snippet.timestampSeconds}`}
													className="flex items-start gap-2 rounded-lg bg-gray-2 p-2 text-xs transition hover:bg-gray-3"
												>
													<span className="shrink-0 rounded bg-blue-500/10 px-1.5 py-0.5 font-mono text-[10px] font-semibold text-blue-600">
														{snippet.formattedTimestamp}
													</span>
													<span className="text-gray-11 line-clamp-2">
														{snippet.text}
													</span>
												</Link>
											))}
										</div>
									) : (
										hit.summary && (
											<p className="text-xs text-gray-10 line-clamp-2">
												{hit.summary}
											</p>
										)
									)}
								</div>
							))}
						</div>
					)}
				</div>
			)}
		</div>
	);
}
