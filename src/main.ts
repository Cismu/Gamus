import { invoke } from "@tauri-apps/api/core";

// Definición exacta basada en tu Rust Struct
interface ScannerConfigDto {
  roots: string[];
  audio_exts: string[];
  ignore_hidden: boolean;
  max_depth: number | null;
}

const statusMsg = document.querySelector("#status-msg") as HTMLParagraphElement;
const statusBox = document.querySelector("#status-box") as HTMLDivElement;

// Referencias al DOM
const rootsInput = document.querySelector("#roots") as HTMLTextAreaElement;
const extsInput = document.querySelector("#audio-exts") as HTMLInputElement;
const hiddenInput = document.querySelector(
  "#ignore-hidden"
) as HTMLInputElement;
const depthInput = document.querySelector("#max-depth") as HTMLInputElement;

// --- Helpers ---

function setStatus(msg: string, type: "info" | "success" | "error" = "info") {
  statusMsg.textContent = msg;
  statusBox.className = `status-box ${type}`;
  statusBox.classList.remove("hidden");
}

// --- Lógica Principal ---

async function loadConfig() {
  try {
    const config = await invoke<ScannerConfigDto>("scanner_get_config");

    // 1. Convertir Array de rutas -> Texto (una por línea)
    rootsInput.value = config.roots.join("\n");

    // 2. Convertir Array de extensiones -> Texto (separado por comas)
    extsInput.value = config.audio_exts.join(", ");

    // 3. Boolean
    hiddenInput.checked = config.ignore_hidden;

    // 4. Option<u32> (si es null, dejar vacío)
    depthInput.value =
      config.max_depth !== null ? config.max_depth.toString() : "";

    setStatus("Configuración cargada.", "info");
  } catch (error) {
    setStatus(`Error cargando config: ${error}`, "error");
  }
}

async function saveConfig(e: Event) {
  e.preventDefault();

  // Parsear Roots: Dividir por saltos de línea y limpiar espacios vacíos
  const rootsArray = rootsInput.value
    .split("\n")
    .map((line) => line.trim())
    .filter((line) => line.length > 0);

  // Parsear Extensiones: Dividir por comas y limpiar
  const extsArray = extsInput.value
    .split(",")
    .map((ext) => ext.trim())
    .filter((ext) => ext.length > 0);

  // Parsear Depth: Si está vacío es null, sino es número
  const depthValue =
    depthInput.value === "" ? null : parseInt(depthInput.value);

  const newConfig: ScannerConfigDto = {
    roots: rootsArray,
    audio_exts: extsArray,
    ignore_hidden: hiddenInput.checked,
    max_depth: depthValue,
  };

  try {
    setStatus("Guardando...", "info");
    // 'input' es el nombre del argumento en tu función Rust
    await invoke("scanner_save_config", { input: newConfig });
    setStatus("Configuración guardada exitosamente.", "success");
  } catch (error) {
    setStatus(`Error al guardar: ${error}`, "error");
  }
}

async function startScan() {
  try {
    setStatus(
      "⏳ Escaneando... revisa la terminal para logs detallados.",
      "info"
    );
    const btn = document.querySelector("#btn-scan") as HTMLButtonElement;
    btn.disabled = true;

    await invoke("library_import_full");

    setStatus("✅ Importación finalizada.", "success");
  } catch (error) {
    setStatus(`❌ Error: ${error}`, "error");
  } finally {
    const btn = document.querySelector("#btn-scan") as HTMLButtonElement;
    btn.disabled = false;
  }
}

// --- Init ---

window.addEventListener("DOMContentLoaded", () => {
  document
    .querySelector("#config-form")
    ?.addEventListener("submit", saveConfig);
  document.querySelector("#btn-scan")?.addEventListener("click", startScan);
  document
    .querySelector("#btn-load-config")
    ?.addEventListener("click", loadConfig);

  loadConfig();
});
