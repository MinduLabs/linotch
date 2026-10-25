import type { BtDevice } from "$lib/island.svelte";

// Every picture in assets/devices, by file name without extension (resolved at build time).
const files = import.meta.glob<string>("$lib/assets/devices/*.{png,webp,jpg,svg}", { eager: true, query: "?url", import: "default" });
const photos = new Map(Object.entries(files).map(([path, url]) => [path.split("/").pop()!.replace(/\.\w+$/, ""), url]));

/**
 * BlueZ does not report the brand, so the picture is chosen by the device name (AirPods also by Apple's
 * Bluetooth product id, see bt.rs). Returns null when there is no picture: the caller shows the Bluetooth icon.
 */
export function photoFor(d: BtDevice): string | null {
  const n = d.name.toLowerCase();
  let key: string | null = null;
  if (d.model) key = d.model;
  else if (n.includes("airpods max")) key = "airpods-max";
  else if (n.includes("airpods pro")) key = "airpods-pro";
  else if (n.includes("airpods")) key = "airpods";
  else if (n.includes("redmi buds 8 lite")) key = "redmi-buds-8-lite";
  return (key && photos.get(key)) || null;
}
