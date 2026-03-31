<script lang="ts">
	import IntroAnimation from '$lib/components/IntroAnimation.svelte';

	let introComplete = $state(false);
	let message = $state('');
	let isError = $state(false);

	async function handleRegister(e: SubmitEvent) {
		e.preventDefault();
		const form = e.target as HTMLFormElement;
		const formData = new FormData(form);

		const res = await fetch('http://localhost:3000/register', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({
				username: formData.get('username'),
				name: formData.get('name'),
				password: formData.get('password')
			})
		});

		const text = await res.text();
		message = text;
		isError = !res.ok;
	}
</script>

<IntroAnimation bind:visible={introComplete} />

<div class="main-content" class:visible={introComplete}>
	<div class="min-h-screen bg-tw-darkblue flex flex-col items-center justify-center px-4 pb-20">
		<h1
			class="text-6xl sm:text-7xl font-extrabold mb-14 pb-2
               leading-normal
               bg-gradient-to-r from-tw-purple to-tw-pink
               bg-clip-text text-transparent
               drop-shadow-[0_0_12px_rgba(0,245,255,0.5)]"
		>
			ImgCat
		</h1>

		<form
			onsubmit={handleRegister}
			class="w-full max-w-sm bg-white/5 backdrop-blur
             border border-tw-purple/30 rounded-2xl p-8
             flex flex-col gap-5"
		>
			<h2 class="text-tw-neon text-xl font-semibold text-center">Register</h2>

			<label class="flex flex-col gap-1">
				<span class="text-tw-yellow text-sm">Username</span>
				<input
					type="text"
					name="username"
					required
					placeholder="twilight_sparkle"
					class="rounded-lg px-4 py-2.5 bg-tw-darkblue/80
                 border border-tw-purple/40 text-white
                 placeholder:text-white/30
                 focus:outline-none focus:ring-2 focus:ring-tw-neon"
				/>
			</label>

			<label class="flex flex-col gap-1">
				<span class="text-tw-yellow text-sm">Display Name</span>
				<input
					type="text"
					name="name"
					required
					placeholder="Twilight Sparkle"
					class="rounded-lg px-4 py-2.5 bg-tw-darkblue/80
                 border border-tw-purple/40 text-white
                 placeholder:text-white/30
                 focus:outline-none focus:ring-2 focus:ring-tw-neon"
				/>
			</label>

			<label class="flex flex-col gap-1">
				<span class="text-tw-yellow text-sm">Password</span>
				<input
					type="password"
					name="password"
					required
					placeholder="********"
					class="rounded-lg px-4 py-2.5 bg-tw-darkblue/80
                 border border-tw-purple/40 text-white
                 placeholder:text-white/30
                 focus:outline-none focus:ring-2 focus:ring-tw-neon"
				/>
			</label>

			<button
				type="submit"
				class="mt-2 rounded-lg py-2.5 font-semibold text-white
               bg-tw-purple hover:bg-tw-pink cursor-pointer
               transition-colors duration-200
               focus:outline-none focus:ring-2 focus:ring-tw-neon"
			>
				Register
			</button>

			{#if message}
				<p class="text-sm text-center {isError ? 'text-red-400' : 'text-green-400'}">{message}</p>
			{/if}
		</form>
	</div>
</div>

<style lang="postcss">
	@reference "tailwindcss";

	:global(html) {
		background-color: #172757;
	}

	/* Main content fade in */
	.main-content {
		opacity: 0;
		transition: opacity 1s ease-in;
	}

	.main-content.visible {
		opacity: 1;
	}
</style>
