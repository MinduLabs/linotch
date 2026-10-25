<script lang="ts">
  // A device's picture: its product photo when there is one, otherwise the blue Bluetooth icon.
  import type { BtDevice } from "$lib/island.svelte";
  import { photoFor } from "./device-art";
  import bluetooth from "$lib/assets/icons/bluetooth.svg?raw";

  let { device }: { device: BtDevice } = $props();
  const photo = $derived(photoFor(device));
</script>

{#if photo}
  <img src={photo} alt="" draggable="false" />
{:else}
  <span class="bt" aria-hidden="true">{@html bluetooth}</span>
{/if}

<style>
  img { width: 100%; height: 100%; object-fit: contain; display: block; }
  .bt { display: block; width: 100%; height: 100%; color: #0a84ff; }
  .bt :global(svg) { width: 100%; height: 100%; display: block; }
</style>
