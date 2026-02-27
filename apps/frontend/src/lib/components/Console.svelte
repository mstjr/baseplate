<script lang="ts">
	import type { FunctionOutput } from '$lib';

	// Props
	export let selectedTheme: 'light' | 'dark' | 'catppuccin' = 'dark';
	export let onThemeChange: ((theme: 'light' | 'dark' | 'catppuccin') => void) | undefined =
		undefined;

	export let functionOutput: FunctionOutput | null = null;
	export let isExecuting = false;
	export let onRun: (() => void) | undefined = undefined;

	let isConsoleOpen = true;
	let consoleHeight = 256; // Default h-64 equivalent in pixels

	let isResizing = false;

	function handleMousedown(e: MouseEvent) {
		isResizing = true;
		e.preventDefault();
		document.addEventListener('mousemove', handleMousemove);
		document.addEventListener('mouseup', handleMouseup);
	}

	function handleMousemove(e: MouseEvent) {
		if (!isResizing) return;
		// Calculate new height based on mouse position
		// We subtract the mouse Y from window innerHeight because the console grows upwards
		const newHeight = window.innerHeight - e.clientY;
		// Min height around 100px, max height 80% of window
		consoleHeight = Math.max(100, Math.min(newHeight, window.innerHeight * 0.8));
	}

	function handleMouseup() {
		isResizing = false;
		document.removeEventListener('mousemove', handleMousemove);
		document.removeEventListener('mouseup', handleMouseup);
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'ArrowUp') {
			consoleHeight = Math.min(window.innerHeight * 0.8, consoleHeight + 20);
			e.preventDefault();
		} else if (e.key === 'ArrowDown') {
			consoleHeight = Math.max(100, consoleHeight - 20);
			e.preventDefault();
		}
	}

	// Reactivity for theme colors to explicitly support Catppuccin Mocha hex codes
	$: consoleBgClass =
		selectedTheme === 'catppuccin'
			? 'bg-[#1e1e2e] border-[#313244] text-[#cdd6f4]'
			: selectedTheme === 'dark'
				? 'bg-zinc-950 border-zinc-800 text-gray-300'
				: 'bg-gray-50 border-gray-200 text-gray-800';

	$: consoleHeaderBgClass =
		selectedTheme === 'catppuccin'
			? 'bg-[#181825] border-[#313244]'
			: selectedTheme === 'dark'
				? 'bg-zinc-900 border-zinc-800'
				: 'bg-white border-gray-200';

	$: consoleSelectBgClass =
		selectedTheme === 'catppuccin'
			? 'bg-[#313244] border-[#45475a] text-[#cdd6f4]'
			: selectedTheme === 'dark'
				? 'bg-zinc-800 border-zinc-700 text-gray-300'
				: 'bg-gray-100 border-gray-200 text-gray-700';

	function handleThemeChange(event: Event) {
		const target = event.target as HTMLSelectElement;
		const newTheme = target.value as 'light' | 'dark' | 'catppuccin';
		if (onThemeChange) onThemeChange(newTheme);
	}
</script>

<div
	class="flex flex-col border-t transition-colors duration-300 {consoleBgClass}"
	style={isConsoleOpen
		? `height: ${consoleHeight}px; transition-property: background-color, border-color, color, fill, stroke;`
		: 'height: 40px; transition-property: all;'}
>
	<!-- Resize Handle -->
	{#if isConsoleOpen}
		<div
			class="h-1 w-full cursor-ns-resize hover:bg-indigo-500/50 active:bg-indigo-500 {selectedTheme ===
			'catppuccin'
				? 'bg-[#313244]'
				: 'bg-gray-200 dark:bg-zinc-800'}"
			on:mousedown={handleMousedown}
			on:keydown={handleKeydown}
			role="slider"
			aria-valuenow={consoleHeight}
			aria-valuemin={100}
			aria-valuemax={window.innerHeight * 0.8}
			tabindex="0"
			aria-label="Resize console"
		></div>
	{/if}
	<!-- Console Header with Theme Switcher -->
	<div
		class="flex items-center justify-between border-b px-4 py-2 shadow-sm transition-colors duration-200 {consoleHeaderBgClass}"
	>
		<div class="flex items-center gap-2">
			<button
				aria-label={isConsoleOpen ? 'Collapse console' : 'Expand console'}
				class="rounded p-1 transition-colors hover:bg-black/10 focus:ring-2 focus:ring-indigo-500 focus:outline-none dark:hover:bg-white/10"
				on:click={() => (isConsoleOpen = !isConsoleOpen)}
			>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					class="h-4 w-4 transform text-emerald-600 transition-transform duration-300 dark:text-emerald-400 {isConsoleOpen
						? 'rotate-0'
						: '-rotate-90'}"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"
				>
					<polyline points="6 9 12 15 18 9"></polyline>
				</svg>
			</button>
			<span
				class="font-mono text-sm font-semibold {selectedTheme === 'catppuccin'
					? 'text-[#cdd6f4]'
					: 'text-gray-700 dark:text-gray-300'}">Console Output</span
			>
		</div>

		<div class="flex items-center gap-3">
			{#if onRun}
				<button
					class="rounded px-3 py-1 text-xs font-semibold text-white transition-colors focus:ring-2 focus:ring-emerald-500 focus:outline-none dark:hover:bg-emerald-600 {isExecuting
						? 'cursor-not-allowed bg-emerald-600/50 dark:bg-emerald-500/50'
						: 'bg-emerald-600 hover:bg-emerald-700 dark:bg-emerald-500'}"
					on:click={onRun}
					disabled={isExecuting}
				>
					{isExecuting ? 'Running...' : 'Run Code'}
				</button>
			{/if}
			<label
				for="theme-selector"
				class="text-xs font-medium {selectedTheme === 'catppuccin'
					? 'text-[#bac2de]'
					: 'text-gray-500 dark:text-gray-400'}">Theme:</label
			>
			<div class="relative">
				<select
					id="theme-selector"
					class="cursor-pointer appearance-none rounded border py-1 pr-8 pl-3 text-xs font-medium transition-colors duration-200 focus:ring-1 focus:ring-indigo-500 focus:outline-none {consoleSelectBgClass}"
					bind:value={selectedTheme}
					on:change={handleThemeChange}
				>
					<option value="light">Light</option>
					<option value="dark">Dark</option>
					<option value="catppuccin">Catppuccin</option>
				</select>
				<div
					class="pointer-events-none absolute inset-y-0 right-0 flex items-center px-2 {selectedTheme ===
					'catppuccin'
						? 'text-[#a6adc8]'
						: 'text-gray-500 dark:text-gray-400'}"
				>
					<svg class="h-3 w-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"
						><path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M19 9l-7 7-7-7"
						></path></svg
					>
				</div>
			</div>
		</div>
	</div>

	<!-- Console Body -->
	<div
		class="w-full flex-1 overflow-y-auto p-4 font-mono text-sm {isConsoleOpen
			? 'opacity-100'
			: 'pointer-events-none opacity-0'} transition-opacity duration-300"
	>
		{#if isExecuting}
			<div class="mb-2 flex items-center gap-2">
				<svg
					class="h-4 w-4 animate-spin text-emerald-600 dark:text-emerald-500"
					xmlns="http://www.w3.org/2000/svg"
					fill="none"
					viewBox="0 0 24 24"
				>
					<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"
					></circle>
					<path
						class="opacity-75"
						fill="currentColor"
						d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
					></path>
				</svg>
				<span
					class="italic {selectedTheme === 'catppuccin'
						? 'text-[#a6adc8]'
						: 'text-gray-500 dark:text-gray-400'}">Function running...</span
				>
			</div>
		{:else if functionOutput}
			{#each functionOutput.logs as [time, log]}
				<div class="mb-1 flex items-start gap-2">
					<span
						class="shrink-0 text-emerald-600 {selectedTheme === 'catppuccin'
							? 'text-[#a6e3a1]'
							: 'dark:text-emerald-500'}">❯</span
					>
					<span class="text-xs text-gray-500 dark:text-gray-400">[{time}]</span>
					<span
						class="flex-1 whitespace-pre-wrap {log.type === 'error'
							? 'text-red-500 dark:text-red-400'
							: 'text-gray-800 dark:text-gray-300'}"
					>
						{log.message}
					</span>
				</div>
			{/each}

			{#if functionOutput.output !== undefined && functionOutput.output !== null}
				<div
					class="mt-4 mb-2 flex items-start gap-2 border-t border-gray-200 pt-3 dark:border-zinc-800"
				>
					<span
						class="shrink-0 text-emerald-600 {selectedTheme === 'catppuccin'
							? 'text-[#a6e3a1]'
							: 'dark:text-emerald-500'}">❯</span
					>
					<span class="font-bold text-gray-800 dark:text-gray-300">Output:</span>
				</div>
				<pre
					class="ml-6 whitespace-pre-wrap text-emerald-600 dark:text-emerald-400">{JSON.stringify(
						functionOutput.output,
						null,
						2
					)}</pre>
			{/if}
		{:else}
			<div class="mb-2 flex items-start gap-2">
				<span
					class="shrink-0 text-emerald-600 {selectedTheme === 'catppuccin'
						? 'text-[#a6e3a1]'
						: 'dark:text-emerald-500'}">❯</span
				>
				<span
					class="italic {selectedTheme === 'catppuccin'
						? 'text-[#a6adc8]'
						: 'text-gray-500 dark:text-gray-400'}">Ready to execute.</span
				>
			</div>
		{/if}
	</div>
</div>
