<script lang="ts">
  // Album art with a default cover underneath: shown when there is no art, while the image loads, after a
  // skip until the new track's art arrives, and if the image fails to load.
  // The default is one red-pink gradient: the same colour family as the wave, like Apple Music's placeholder.
  let { src = null, small = false }: { src?: string | null; small?: boolean } = $props();
  let loaded = $state(false);
  $effect(() => {
    src;
    loaded = false;
  });
</script>

<div class="cover" class:small>
  <svg class="note" viewBox="0 0 24 24" aria-hidden="true">
    <path d="M9 17.5V6.2l10-2.2v11" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" stroke-linejoin="round" />
    <circle cx="6.5" cy="17.5" r="2.6" fill="currentColor" />
    <circle cx="16.5" cy="15" r="2.6" fill="currentColor" />
  </svg>
  {#if src}
    <img {src} alt="" class:loaded onload={() => (loaded = true)} onerror={() => (loaded = false)} />
  {/if}
</div>

<style>
  .cover {
    position: relative;
    width: 52px;
    height: 52px;
    border-radius: 12px;
    overflow: hidden;
    flex: none;
    display: grid;
    place-items: center;
    color: rgba(255, 255, 255, 0.92);
    background:
      radial-gradient(120% 90% at 20% 10%, rgba(255, 255, 255, 0.22), transparent 55%),
      linear-gradient(145deg, #ff5f6d, #c2185b);
  }
  .cover.small { width: 19px; height: 19px; border-radius: 5px; }
  .note { width: 50%; height: 50%; filter: drop-shadow(0 1px 2px rgba(0, 0, 0, 0.25)); }
  .small .note { filter: none; }
  img { position: absolute; inset: 0; width: 100%; height: 100%; object-fit: cover; opacity: 0; transition: opacity 0.2s ease-out; }
  img.loaded { opacity: 1; }
</style>
