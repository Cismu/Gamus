import { invoke } from "@tauri-apps/api/core";

function renderList(
  selector: string,
  items: { id: string; name?: string; title?: string }[]
) {
  const ul = document.querySelector<HTMLUListElement>(selector);
  if (!ul) return;
  ul.innerHTML = "";
  items.forEach((i) => {
    const li = document.createElement("li");
    li.textContent = `${i.name ?? i.title} (${i.id})`;
    ul.appendChild(li);
  });
}

async function createArtist() {
  const name = (document.querySelector("#artist-name") as HTMLInputElement)
    .value;
  const bio = (document.querySelector("#artist-bio") as HTMLInputElement).value;

  await invoke("create_artist", {
    input: { name, bio: bio || null },
  });

  alert("Artista creado!");
}

async function createSong() {
  const title = (document.querySelector("#song-title") as HTMLInputElement)
    .value;
  const acoustid = (
    document.querySelector("#song-acoustid") as HTMLInputElement
  ).value;

  await invoke("create_song", {
    input: { title, acoustid: acoustid || null },
  });

  alert("Canción creada!");
}

async function loadArtists() {
  const artists = await invoke<any[]>("list_artists");
  renderList("#artists", artists);
}

async function loadSongs() {
  const songs = await invoke<any[]>("list_songs");
  renderList("#songs", songs);
}

window.addEventListener("DOMContentLoaded", () => {
  document.querySelector("#artist-form")?.addEventListener("submit", (e) => {
    e.preventDefault();
    createArtist();
  });

  document.querySelector("#song-form")?.addEventListener("submit", (e) => {
    e.preventDefault();
    createSong();
  });

  document
    .querySelector("#load-artists")
    ?.addEventListener("click", loadArtists);
  document.querySelector("#load-songs")?.addEventListener("click", loadSongs);
});
