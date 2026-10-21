import { useEffect, useRef, useState } from "react";

type HighlightRect = {
	top: number;
	left: number;
	width: number;
	height: number;
};

type BlurOverlayProps = {
	active: boolean;
	onDone: () => void;
};

const OVERLAY_ROOT_ID = "cap-extension-recorder-overlay";

const SENSITIVE_INPUT_SELECTOR = [
	'input[type="password"]',
	'input[type="email"]',
	'input[name*="secret" i]',
	'input[name*="token" i]',
	'input[name*="api_key" i]',
	'input[name*="apikey" i]',
	'input[autocomplete*="cc-"]',
	"[data-sensitive]",
].join(",");

const SENSITIVE_TEXT_REGEX =
	/\b([a-z0-9._%+-]+@[a-z0-9.-]+\.[a-z]{2,}|sk-[a-z0-9_-]{12,}|pk_(?:live|test)_[a-z0-9]{12,}|gh[pousr]_[a-z0-9]{12,}|akia[0-9a-z]{12,})\b/i;

export function autoBlurSensitiveDomElements(root: ParentNode = document) {
	let count = 0;
	const applyBlur = (el: HTMLElement) => {
		if (el.dataset.capBlurred === "true" || el.closest(`#${OVERLAY_ROOT_ID}`)) {
			return;
		}
		el.dataset.capOrigFilter = el.style.filter || "";
		el.style.setProperty("filter", "blur(12px)", "important");
		el.style.setProperty("user-select", "none", "important");
		el.dataset.capBlurred = "true";
		count++;
	};

	for (const el of Array.from(
		root.querySelectorAll<HTMLElement>(SENSITIVE_INPUT_SELECTOR),
	)) {
		applyBlur(el);
	}

	for (const el of Array.from(
		root.querySelectorAll<HTMLElement>("code, pre, span, td, p"),
	)) {
		if (el.children.length === 0 && el.textContent) {
			if (SENSITIVE_TEXT_REGEX.test(el.textContent)) {
				applyBlur(el);
			}
		}
	}
	return count;
}

export function BlurOverlay({ active, onDone }: BlurOverlayProps) {
	const [highlightRect, setHighlightRect] = useState<HighlightRect | null>(
		null,
	);
	const rafRef = useRef<number | null>(null);

	useEffect(() => {
		if (!active) {
			setHighlightRect(null);
			return;
		}
		autoBlurSensitiveDomElements(document);

		const handlePointerMove = (event: PointerEvent) => {
			if (rafRef.current !== null) return;
			rafRef.current = window.requestAnimationFrame(() => {
				rafRef.current = null;
				const target = document.elementFromPoint(
					event.clientX,
					event.clientY,
				) as HTMLElement | null;

				if (!target || target.closest(`#${OVERLAY_ROOT_ID}`)) {
					setHighlightRect(null);
					return;
				}

				const rect = target.getBoundingClientRect();
				if (rect.width <= 0 || rect.height <= 0) {
					setHighlightRect(null);
					return;
				}

				setHighlightRect({
					top: rect.top,
					left: rect.left,
					width: rect.width,
					height: rect.height,
				});
			});
		};

		const handleClick = (event: MouseEvent) => {
			const target = document.elementFromPoint(
				event.clientX,
				event.clientY,
			) as HTMLElement | null;

			if (!target || target.closest(`#${OVERLAY_ROOT_ID}`)) {
				return;
			}

			event.preventDefault();
			event.stopPropagation();

			if (target.dataset.capBlurred === "true") {
				const orig = target.dataset.capOrigFilter ?? "";
				if (orig) {
					target.style.filter = orig;
				} else {
					target.style.removeProperty("filter");
				}
				target.style.removeProperty("user-select");
				delete target.dataset.capBlurred;
				delete target.dataset.capOrigFilter;
			} else {
				target.dataset.capOrigFilter = target.style.filter || "";
				target.style.setProperty("filter", "blur(12px)", "important");
				target.style.setProperty("user-select", "none", "important");
				target.dataset.capBlurred = "true";
			}
		};

		const handleKeyDown = (event: KeyboardEvent) => {
			if (event.key === "Escape") {
				event.preventDefault();
				event.stopPropagation();
				onDone();
			}
		};

		window.addEventListener("pointermove", handlePointerMove, {
			capture: true,
			passive: true,
		});
		window.addEventListener("click", handleClick, {
			capture: true,
		});
		window.addEventListener("keydown", handleKeyDown, { capture: true });

		return () => {
			if (rafRef.current !== null) {
				window.cancelAnimationFrame(rafRef.current);
				rafRef.current = null;
			}
			window.removeEventListener("pointermove", handlePointerMove, {
				capture: true,
			});
			window.removeEventListener("click", handleClick, {
				capture: true,
			});
			window.removeEventListener("keydown", handleKeyDown, { capture: true });
		};
	}, [active, onDone]);

	if (!active || !highlightRect) return null;

	return (
		<div
			className="cap-extension-blur-highlight"
			style={{
				top: `${highlightRect.top}px`,
				left: `${highlightRect.left}px`,
				width: `${highlightRect.width}px`,
				height: `${highlightRect.height}px`,
			}}
			aria-hidden
		>
			<span className="cap-extension-blur-tooltip">Click to blur / unblur</span>
		</div>
	);
}
