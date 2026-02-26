<script lang="ts">
	import Editor from '$lib/components/Editor.svelte';
	import Console from '$lib/components/Console.svelte';

	let selectedTheme: 'light' | 'dark' | 'catppuccin' = 'dark';

	function handleThemeChange(newTheme: 'light' | 'dark' | 'catppuccin') {
		selectedTheme = newTheme;

		// Basculer la classe dark sur le body pour harmoniser l'UI externe
		if (newTheme === 'light') {
			document.documentElement.classList.remove('dark');
		} else {
			document.documentElement.classList.add('dark');
		}
	}

	function handleCodeChange(code: string) {
		// Logique pour gérer les changements de code ici
		// console.log("Code changed:", code);
	}
</script>

<div
	class="flex h-screen w-screen flex-col overflow-hidden bg-white font-sans transition-colors duration-200 dark:bg-zinc-950"
>
	<!-- Editor Section (Full Screen Width) -->
	<div class="relative w-full flex-1 overflow-hidden">
		<Editor theme={selectedTheme} onChange={handleCodeChange} />
	</div>

	<!-- Console Section (Bottom) -->
	<Console {selectedTheme} onThemeChange={handleThemeChange} />
</div>
