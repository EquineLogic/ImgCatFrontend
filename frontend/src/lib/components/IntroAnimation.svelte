<script lang="ts">
	import { onMount } from 'svelte';
	import twilightImg from '$lib/assets/twilight.png';

	// default to false if not bound, but allow parent to control visibility if desired
	let { visible = $bindable(false) }: { visible: boolean } = $props();

	let showIntro = $state(true);
	let fadeOut = $state(false);
	let sparkles: {
		id: number;
		x: number;
		y: number;
		size: number;
		delay: number;
		duration: number;
		color: string;
	}[] = $state([]);
	let ponies: { id: number; x: number; y: number; delay: number; flip: boolean }[] = $state([]);

	const sparkleColors = [
		'#9061C2',
		'#ED438D',
		'#00F5FF',
		'#FFF34B',
		'#ffffff',
		'#c792ea',
		'#ff79c6'
	];

	onMount(() => {
		// Create sparkles
		for (let i = 0; i < 80; i++) {
			sparkles.push({
				id: i,
				x: Math.random() * 100,
				y: -(Math.random() * 40),
				size: Math.random() * 8 + 4,
				delay: Math.random() * 1,
				duration: Math.random() * 1 + 1.5,
				color: sparkleColors[Math.floor(Math.random() * sparkleColors.length)]
			});
		}

		// Place ponies at fixed positions
		ponies = [
			{ id: 0, x: 10, y: 60, delay: 0.3, flip: false },
			{ id: 1, x: 78, y: 55, delay: 0.6, flip: true }
		];

		// Start fade out after animation
		setTimeout(() => {
			fadeOut = true;
			visible = true;
		}, 2500);

		// Remove intro overlay
		setTimeout(() => {
			showIntro = false;
		}, 3500);
	});
</script>

{#if showIntro}
	<div class="intro-overlay" class:fade-out={fadeOut}>
		<div class="intro-bg"></div>

		<!-- Sparkles -->
		{#each sparkles as sparkle (sparkle.id)}
			<div
				class="sparkle"
				style="
          left: {sparkle.x}%;
          top: {sparkle.y}%;
          width: {sparkle.size}px;
          height: {sparkle.size}px;
          background: {sparkle.color};
          animation-delay: {sparkle.delay}s;
          animation-duration: {sparkle.duration}s;
          box-shadow: 0 0 {sparkle.size * 2}px {sparkle.color}, 0 0 {sparkle.size *
					4}px {sparkle.color}40;
        "
			></div>
		{/each}

		<!-- Star bursts (larger sparkle shapes) -->
		{#each Array(15) as _, i}
			<div
				class="star-burst"
				style="
          left: {Math.random() * 90 + 5}%;
          top: {Math.random() * 80 + 10}%;
          animation-delay: {Math.random() * 1.5}s;
          color: {sparkleColors[i % sparkleColors.length]};
        "
			>
				&#10022;
			</div>
		{/each}

		<!-- Ponies -->
		{#each ponies as pony (pony.id)}
			<img
				src={twilightImg}
				alt="Twilight Sparkle"
				class="pony"
				style="
          left: {pony.x}%;
          top: {pony.y}%;
          animation-delay: {pony.delay}s;
          {pony.flip ? 'transform: scaleX(-1);' : ''}
        "
			/>
		{/each}

		<!-- Centered title during intro -->
		<div class="intro-title">
			<h1>ImgCat</h1>
			<p>&#10023; Welcome to the magic &#10023;</p>
		</div>
	</div>
{/if}

<style lang="postcss">
	@reference "tailwindcss";

	/* Intro overlay */
	.intro-overlay {
		position: fixed;
		inset: 0;
		z-index: 50;
		background: #172757;
		overflow: hidden;
		transition: opacity 0.8s ease-out;
	}

	.intro-overlay.fade-out {
		opacity: 0;
		pointer-events: none;
	}

	.intro-bg {
		position: absolute;
		inset: 0;
		background:
			radial-gradient(ellipse at 50% 30%, #9061c230 0%, #17275700 70%),
			radial-gradient(ellipse at 30% 70%, #ed438d20 0%, #17275700 60%),
			radial-gradient(ellipse at 70% 60%, #00f5ff15 0%, #17275700 60%);
	}

	/* Sparkles falling */
	.sparkle {
		position: absolute;
		border-radius: 50%;
		opacity: 0;
		animation-name: sparkle-fall;
		animation-timing-function: ease-in;
		animation-fill-mode: forwards;
		image-rendering: pixelated;
	}

	@keyframes sparkle-fall {
		0% {
			opacity: 0;
			transform: translateY(0) scale(0) rotate(0deg);
		}
		10% {
			opacity: 1;
			transform: translateY(10vh) scale(1) rotate(45deg);
		}
		50% {
			opacity: 1;
		}
		100% {
			opacity: 0;
			transform: translateY(110vh) scale(0.3) rotate(180deg);
		}
	}

	/* Star burst shapes */
	.star-burst {
		position: absolute;
		font-size: 2rem;
		opacity: 0;
		animation: star-pop 1.2s ease-out forwards;
		filter: drop-shadow(0 0 8px currentColor);
	}

	@keyframes star-pop {
		0% {
			opacity: 0;
			transform: scale(0) rotate(0deg);
		}
		30% {
			opacity: 1;
			transform: scale(1.3) rotate(90deg);
		}
		60% {
			opacity: 0.8;
			transform: scale(1) rotate(180deg);
		}
		100% {
			opacity: 0;
			transform: scale(0.5) rotate(360deg);
		}
	}

	/* Ponies */
	.pony {
		position: absolute;
		width: 140px;
		height: auto;
		opacity: 0;
		animation: pony-fade-in 0.8s ease-out forwards;
		filter: drop-shadow(0 0 12px #9061c280);
	}

	@keyframes pony-fade-in {
		0% {
			opacity: 0;
			transform: scale(0.8);
		}
		100% {
			opacity: 1;
			transform: scale(1);
		}
	}

	/* Intro title */
	.intro-title {
		position: absolute;
		inset: 0;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		z-index: 10;
		animation: title-entrance 1s ease-out forwards;
	}

	.intro-title h1 {
		font-size: 5rem;
		font-weight: 800;
		background: linear-gradient(135deg, #9061c2, #ed438d, #00f5ff);
		background-clip: text;
		-webkit-background-clip: text;
		-webkit-text-fill-color: transparent;
		filter: drop-shadow(0 0 20px #9061c260) drop-shadow(0 0 40px #00f5ff30);
		animation: title-glow 2s ease-in-out infinite alternate;
	}

	.intro-title p {
		color: #fff34b;
		font-size: 1.1rem;
		margin-top: 0.75rem;
		opacity: 0;
		animation: subtitle-in 0.8s ease-out 0.6s forwards;
		text-shadow: 0 0 10px #fff34b60;
	}

	@keyframes title-entrance {
		0% {
			transform: scale(0.3);
			opacity: 0;
		}
		60% {
			transform: scale(1.05);
			opacity: 1;
		}
		100% {
			transform: scale(1);
			opacity: 1;
		}
	}

	@keyframes title-glow {
		0% {
			filter: drop-shadow(0 0 20px #9061c260) drop-shadow(0 0 40px #00f5ff30);
		}
		100% {
			filter: drop-shadow(0 0 30px #ed438d60) drop-shadow(0 0 60px #9061c240);
		}
	}

	@keyframes subtitle-in {
		0% {
			opacity: 0;
			transform: translateY(10px);
		}
		100% {
			opacity: 1;
			transform: translateY(0);
		}
	}
</style>
