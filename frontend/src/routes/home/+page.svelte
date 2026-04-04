<script lang="ts">
	import { onMount } from 'svelte';
	import { flip } from 'svelte/animate';
	import FolderCard from '$lib/components/FolderCard.svelte';
	import ImageCard from '$lib/components/ImageCard.svelte';
	import { folders, fetchFolders, breadcrumbs, navigateToBreadcrumb } from '$lib/stores/folders';
	import { files, fetchFiles } from '$lib/stores/files';

	let loading = $state(true);
	let editMode = $state(false);

	// Drag state
	let dragging = $state(false);
	let dragType: 'folder' | 'file' | null = $state(null);
	let dragId: string | null = $state(null);
	let dragEl: HTMLElement | null = $state(null);
	let dragClone: HTMLElement | null = $state(null);
	let startX = 0;
	let startY = 0;
	let offsetX = 0;
	let offsetY = 0;

	onMount(async () => {
		await Promise.all([fetchFolders(), fetchFiles()]);
		loading = false;
	});

	function onPointerDown(e: PointerEvent, id: string, type: 'folder' | 'file') {
		// Only allow dragging in edit mode, and only left click
		if (!editMode || e.button !== 0) return;
		const target = (e.currentTarget as HTMLElement);

		dragId = id;
		dragType = type;
		dragEl = target;
		startX = e.clientX;
		startY = e.clientY;

		const rect = target.getBoundingClientRect();
		offsetX = e.clientX - rect.left;
		offsetY = e.clientY - rect.top;
	}

	function onPointerMove(e: PointerEvent) {
		if (!dragId || !dragEl) return;

		// Start dragging after moving a few pixels (prevents accidental drags)
		if (!dragging) {
			const dx = e.clientX - startX;
			const dy = e.clientY - startY;
			if (Math.abs(dx) < 5 && Math.abs(dy) < 5) return;

			dragging = true;

			// Create a floating clone of the dragged element
			const rect = dragEl.getBoundingClientRect();
			dragClone = dragEl.cloneNode(true) as HTMLElement;
			dragClone.style.position = 'fixed';
			dragClone.style.width = rect.width + 'px';
			dragClone.style.height = rect.height + 'px';
			dragClone.style.zIndex = '1000';
			dragClone.style.pointerEvents = 'none';
			dragClone.style.opacity = '0.9';
			dragClone.style.transform = 'scale(1.05)';
			dragClone.style.transition = 'transform 0.15s, box-shadow 0.15s';
			dragClone.style.boxShadow = '0 12px 40px rgba(0,0,0,0.4)';
			dragClone.style.borderRadius = '12px';
			document.body.appendChild(dragClone);
		}

		if (dragClone) {
			dragClone.style.left = (e.clientX - offsetX) + 'px';
			dragClone.style.top = (e.clientY - offsetY) + 'px';
		}

		// Find which item the cursor is over and reorder
		const list = dragType === 'folder' ? $folders : $files;
		const container = dragType === 'folder'
			? document.querySelector('[data-droppable="folders"]')
			: document.querySelector('[data-droppable="files"]');

		if (!container) return;

		const fromIndex = list.findIndex((item) => item.id === dragId);
		if (fromIndex === -1) return;

		const children = Array.from(container.children) as HTMLElement[];
		for (let i = 0; i < children.length; i++) {
			if (i === fromIndex) continue;
			const rect = children[i].getBoundingClientRect();
			const centerX = rect.left + rect.width / 2;
			const centerY = rect.top + rect.height / 2;

			// Only swap when the cursor has crossed past the center of the target
			// (prevents jitter when hovering near the edges)
			const crossedFromLeft = i > fromIndex && e.clientX > centerX && e.clientY >= rect.top && e.clientY <= rect.bottom;
			const crossedFromRight = i < fromIndex && e.clientX < centerX && e.clientY >= rect.top && e.clientY <= rect.bottom;
			const crossedFromAbove = i > fromIndex && e.clientY > centerY && e.clientX >= rect.left && e.clientX <= rect.right;
			const crossedFromBelow = i < fromIndex && e.clientY < centerY && e.clientX >= rect.left && e.clientX <= rect.right;

			if (crossedFromLeft || crossedFromRight || crossedFromAbove || crossedFromBelow) {
				const newList = [...list];
				const [item] = newList.splice(fromIndex, 1);
				newList.splice(i, 0, item);

				if (dragType === 'folder') {
					folders.set(newList);
				} else {
					files.set(newList as any);
				}
				break;
			}
		}
	}

	function onPointerUp() {
		if (!dragId) return;

		if (dragging) {
			// Persist the final order
			if (dragType === 'folder') {
				persistOrder($folders.map((f) => f.id));
			} else if (dragType === 'file') {
				persistOrder($files.map((f) => f.id));
			}
		}

		// Cleanup
		if (dragClone) {
			dragClone.remove();
			dragClone = null;
		}
		dragging = false;
		dragId = null;
		dragType = null;
		dragEl = null;
	}

	async function persistOrder(ids: string[]) {
		await fetch('http://localhost:3000/reorder', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			credentials: 'include',
			body: JSON.stringify({ ids })
		});
	}
</script>

<svelte:window onpointermove={onPointerMove} onpointerup={onPointerUp} />

<div class="p-8 select-none">
	<div class="flex items-center justify-between mb-6">
		<!-- Breadcrumbs -->
		{#if $breadcrumbs.length > 1}
			<nav class="flex items-center gap-1.5 text-sm">
				{#each $breadcrumbs as crumb, i}
					{#if i > 0}
						<span class="text-white/20">/</span>
					{/if}
					{#if i < $breadcrumbs.length - 1}
						<button
							onclick={() => navigateToBreadcrumb(i)}
							class="text-white/40 hover:text-tw-neon cursor-pointer transition-colors duration-150"
						>
							{crumb.name}
						</button>
					{:else}
						<span class="text-white">{crumb.name}</span>
					{/if}
				{/each}
			</nav>
		{:else}
			<div></div>
		{/if}

		<!-- Edit mode toggle -->
		<button
			onclick={() => (editMode = !editMode)}
			class="px-3 py-1.5 rounded-lg text-sm font-medium transition-colors duration-150 cursor-pointer
			       {editMode
					? 'bg-tw-neon text-tw-darkblue hover:bg-tw-neon/90'
					: 'bg-white/5 text-white/60 hover:text-white hover:bg-white/10'}"
		>
			{editMode ? 'Done' : 'Edit'}
		</button>
	</div>

	{#if loading}
		<div class="flex justify-center mt-20">
			<div class="w-6 h-6 border-2 border-tw-neon/30 border-t-tw-neon rounded-full animate-spin"></div>
		</div>
	{:else if $folders.length === 0 && $files.length === 0}
		<div class="w-full max-w-lg mx-auto mt-20 flex flex-col items-center gap-4 text-center">
			<div class="w-16 h-16 rounded-2xl bg-white/5 border border-white/10 flex items-center justify-center">
				<svg
					xmlns="http://www.w3.org/2000/svg"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="1.5"
					stroke-linecap="round"
					stroke-linejoin="round"
					class="w-8 h-8 text-white/20"
				>
					<path d="M2 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2z" />
				</svg>
			</div>
			<p class="text-white/30 text-sm">This folder is empty.</p>
		</div>
	{:else}
		{#if $folders.length > 0}
			<div class="flex flex-wrap gap-2 mb-6" data-droppable="folders">
				{#each $folders as folder (folder.id)}
					<!-- svelte-ignore a11y_no_static_element_interactions -->
					<div
						animate:flip={{ duration: 200 }}
						onpointerdown={(e) => onPointerDown(e, folder.id, 'folder')}
						class="{editMode ? 'cursor-grab active:cursor-grabbing' : ''}
						       {dragging && dragId === folder.id ? 'opacity-30' : ''}"
					>
						<div class={editMode ? 'pointer-events-none' : ''}>
							<FolderCard name={folder.name} id={folder.id} />
						</div>
					</div>
				{/each}
			</div>
		{/if}

		{#if $files.length > 0}
			<div class="grid grid-cols-[repeat(auto-fill,minmax(180px,1fr))] gap-4" data-droppable="files">
				{#each $files as file (file.id)}
					<!-- svelte-ignore a11y_no_static_element_interactions -->
					<div
						animate:flip={{ duration: 200 }}
						onpointerdown={(e) => onPointerDown(e, file.id, 'file')}
						class="{editMode ? 'cursor-grab active:cursor-grabbing' : ''}
						       {dragging && dragId === file.id ? 'opacity-30' : ''}"
					>
						<div class={editMode ? 'pointer-events-none' : ''}>
							<ImageCard name={file.name} id={file.id} />
						</div>
					</div>
				{/each}
			</div>
		{/if}
	{/if}
</div>
