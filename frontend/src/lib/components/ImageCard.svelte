<script lang="ts">
	let { name, id } = $props<{ name: string; id: string }>();
</script>

<div
	class="group flex flex-col items-center gap-3 p-3 rounded-2xl
	       bg-white/5 border border-white/10
	       hover:bg-tw-purple/10 hover:border-tw-purple/30
	       transition-all duration-200"
>
	<img
		src="http://localhost:3000/files/{id}"
		alt={name}
		class="w-full aspect-square object-cover rounded-xl"
	/>
	<span class="text-sm text-white/70 group-hover:text-white truncate max-w-full transition-colors duration-200">
		{name}
	</span>
</div>
