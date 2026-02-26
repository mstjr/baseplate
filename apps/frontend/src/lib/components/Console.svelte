<script lang="ts">
	// Props
	export let selectedTheme: 'light' | 'dark' | 'catppuccin' = 'dark';
	export let onThemeChange: ((theme: 'light' | 'dark' | 'catppuccin') => void) | undefined =
		undefined;

	let isConsoleOpen = true;

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
	class="flex flex-col border-t transition-all duration-300 {consoleBgClass} {isConsoleOpen
		? 'h-64'
		: 'h-10'}"
>
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
		<div class="mb-2 flex items-start gap-2">
			<span
				class="shrink-0 text-emerald-600 {selectedTheme === 'catppuccin'
					? 'text-[#a6e3a1]'
					: 'dark:text-emerald-500'}">❯</span
			>
			<span
				class="italic {selectedTheme === 'catppuccin'
					? 'text-[#a6adc8]'
					: 'text-gray-500 dark:text-gray-400'}">Initializing language server environment...</span
			>
		</div>
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
	</div>
</div>
