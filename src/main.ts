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

async function scanLibrary() {
  const result = await invoke<{
    total_files: number;
    devices: {
      id: string;
      bandwidth_mb_s: number | null;
      file_count: number;
    }[];
  }>("scan_library");

  const div = document.querySelector<HTMLDivElement>("#scan-result");
  if (!div) return;

  let html = `<p>Total de archivos: ${result.total_files}</p>`;
  html += "<ul>";
  for (const d of result.devices) {
    html +=
      `<li>Device ${d.id} – ${d.file_count} archivos` +
      (d.bandwidth_mb_s != null ? ` (~${d.bandwidth_mb_s} MB/s)` : "") +
      `</li>`;
  }
  html += "</ul>";

  div.innerHTML = html;
}

type ScannerConfigDto = {
  roots: string[];
  audio_exts: string[];
  ignore_hidden: boolean;
  max_depth: number | null;
};

async function loadScannerConfig() {
  const cfg = await invoke<ScannerConfigDto>("get_scanner_config");

  const rootsArea =
    document.querySelector<HTMLTextAreaElement>("#scanner-roots");
  const extsInput = document.querySelector<HTMLInputElement>(
    "#scanner-audio-exts"
  );
  const ignoreHidden = document.querySelector<HTMLInputElement>(
    "#scanner-ignore-hidden"
  );
  const maxDepth =
    document.querySelector<HTMLInputElement>("#scanner-max-depth");

  if (!rootsArea || !extsInput || !ignoreHidden || !maxDepth) return;

  rootsArea.value = cfg.roots.join("\n");
  extsInput.value = cfg.audio_exts.join(", ");
  ignoreHidden.checked = cfg.ignore_hidden;
  maxDepth.value = cfg.max_depth != null ? String(cfg.max_depth) : "";
}

async function saveScannerConfig() {
  const rootsArea =
    document.querySelector<HTMLTextAreaElement>("#scanner-roots");
  const extsInput = document.querySelector<HTMLInputElement>(
    "#scanner-audio-exts"
  );
  const ignoreHidden = document.querySelector<HTMLInputElement>(
    "#scanner-ignore-hidden"
  );
  const maxDepth =
    document.querySelector<HTMLInputElement>("#scanner-max-depth");

  if (!rootsArea || !extsInput || !ignoreHidden || !maxDepth) return;

  const roots = rootsArea.value
    .split("\n")
    .map((s) => s.trim())
    .filter((s) => s.length > 0);

  const audio_exts = extsInput.value
    .split(",")
    .map((s) => s.trim())
    .filter((s) => s.length > 0);

  const maxDepthVal = maxDepth.value.trim();
  const max_depth = maxDepthVal === "" ? null : Number(maxDepthVal);

  const payload: ScannerConfigDto = {
    roots,
    audio_exts,
    ignore_hidden: ignoreHidden.checked,
    max_depth: Number.isNaN(max_depth as number) ? null : max_depth,
  };

  await invoke("save_scanner_config", { input: payload });
  alert("Scanner config guardada!");
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

  document.querySelector("#scan-library")?.addEventListener("click", () => {
    scanLibrary().catch((err) => {
      console.error("scan_library error", err);
      alert("Error al escanear biblioteca: " + err);
    });
  });

  loadScannerConfig().catch((err) => {
    console.error("get_scanner_config error", err);
  });

  document.querySelector("#scanner-form")?.addEventListener("submit", (e) => {
    e.preventDefault();
    saveScannerConfig().catch((err) => {
      console.error("save_scanner_config error", err);
      alert("Error al guardar scanner config: " + err);
    });
  });
});
