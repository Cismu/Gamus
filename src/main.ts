import { invoke } from "@tauri-apps/api/core";

type ArtistDto = { id: string; name: string };

async function loadArtists() {
  try {
    const artists = await invoke<ArtistDto[]>("list_artists");

    const list = document.querySelector<HTMLUListElement>("#artists");
    if (!list) return;

    list.innerHTML = "";
    artists.forEach((artist) => {
      const li = document.createElement("li");
      li.textContent = `${artist.name} (${artist.id})`;
      list.appendChild(li);
    });
  } catch (err) {
    console.error("Error loading artists", err);
  }
}

window.addEventListener("DOMContentLoaded", () => {
  const btn = document.querySelector<HTMLButtonElement>("#load-artists");
  if (btn) {
    btn.addEventListener("click", () => {
      loadArtists();
    });
  }
});
