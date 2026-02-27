<script lang="ts">
	import { onMount } from 'svelte';
	import { EditorView, basicSetup } from 'codemirror';
	import { Compartment } from '@codemirror/state';
	import { python } from '@codemirror/lang-python';
	import { languageServer } from 'codemirror-languageserver';
	import { oneDark } from '@codemirror/theme-one-dark';
	import { catppuccinMocha } from '@catppuccin/codemirror';

	// Props

	let { theme = 'dark', code = $bindable('') } = $props();

	let editorContainer: HTMLDivElement;
	let view: EditorView;

	const themeCompartment = new Compartment();

	const themes = {
		light: [],
		dark: oneDark,
		catppuccin: catppuccinMocha
	} as Record<string, any>;

	// Effect to update theme when prop changes
	$effect(() => {
		if (view && theme) {
			view.dispatch({
				effects: themeCompartment.reconfigure(themes[theme])
			});
		}
	});

	onMount(() => {
		// 1. Configuration de l'extension LSP
		const ls = languageServer({
			serverUri: 'ws://localhost:3000/python',
			rootUri: 'file:///app/workspace',
			documentUri: 'file:///app/workspace/script.py',
			languageId: 'python',
			workspaceFolders: [{ uri: 'file:///app/workspace', name: 'workspace' }]
		});

		// 2. Initialisation de l'éditeur
		view = new EditorView({
			doc: code,
			extensions: [
				basicSetup,
				python(),
				themeCompartment.of(themes[theme]),
				ls, // On injecte l'extension LSP ici
				EditorView.updateListener.of((update) => {
					if (update.docChanged) {
						const newCode = update.state.doc.toString();
						code = newCode; // Met à jour le code lié au composant
					}
				})
			],
			parent: editorContainer
		});

		return () => {
			view.destroy();
		};
	});
</script>

<div
	class="editor-wrapper relative h-full w-full overflow-hidden"
	bind:this={editorContainer}
></div>

<style>
	/* Make CodeMirror take full height and enforce a beautiful font */
	:global(.editor-wrapper .cm-editor) {
		height: 100%;
		width: 100%;
		font-family:
			ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, 'Liberation Mono', 'Courier New',
			monospace;
		font-size: 14.5px;
		line-height: 1.6;
	}
	:global(.editor-wrapper .cm-scroller) {
		overflow: auto !important;
	}
	/* Focus outline removal to keep the UI looking premium */
	:global(.editor-wrapper .cm-editor.cm-focused) {
		outline: none !important;
	}

	/* Styling the gutter for a more polished look */
	:global(.editor-wrapper .cm-gutters) {
		border-right: 1px solid rgba(128, 128, 128, 0.15) !important;
		background-color: transparent !important;
	}

	:global(.editor-wrapper .cm-lineNumbers .cm-gutterElement) {
		padding: 0 16px 0 8px !important;
		opacity: 0.6;
	}
</style>
