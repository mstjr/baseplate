<script lang="ts">
	import Editor from '$lib/components/Editor.svelte';
	import Console from '$lib/components/Console.svelte';
	import type { FunctionOutput } from '$lib';

	let selectedTheme = $state<'light' | 'dark' | 'catppuccin'>('dark');

	function handleThemeChange(newTheme: 'light' | 'dark' | 'catppuccin') {
		selectedTheme = newTheme;

		// Basculer la classe dark sur le body pour harmoniser l'UI externe
		if (newTheme === 'light') {
			document.documentElement.classList.remove('dark');
		} else {
			document.documentElement.classList.add('dark');
		}
	}

	let code = $state<string>(
		'import requests\n\ndef handler(sdk, context):\n    response = requests.get("https://jsonplaceholder.typicode.com/todos/1")\n    return response.json()'
	);

	function handleCodeChange(newCode: string) {
		code = newCode;
	}

	let functionOutput = $state<FunctionOutput | null>(null);
	let isExecuting = $state(false);

	async function executeCode() {
		let payload = {
			code: code,
			language: 'python',
			parameters: {}
		};

		isExecuting = true;
		functionOutput = null;

		try {
			const response = await fetch('http://localhost:8001/execute', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify(payload)
			});
			const result = await response.json();
			functionOutput = result as FunctionOutput;
		} catch (error) {
			console.error('Error executing code:', error);
		} finally {
			isExecuting = false;
		}
	}
</script>

<div
	class="flex h-screen w-screen flex-col overflow-hidden bg-white font-sans transition-colors duration-200 dark:bg-zinc-950"
>
	<!-- Editor Section (Full Screen Width) -->
	<div class="relative w-full flex-1 overflow-hidden">
		<Editor theme={selectedTheme} bind:code />
	</div>

	<!-- Console Section (Bottom) -->
	<Console
		{selectedTheme}
		onThemeChange={handleThemeChange}
		{functionOutput}
		onRun={executeCode}
		{isExecuting}
	/>
</div>
