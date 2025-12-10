<script setup lang="ts">
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import { ref, computed, onMounted, onUnmounted } from 'vue'

// --- ESTADO ---
const isRunning = ref(false)
const progress = ref(0)
const total = ref(0)
const currentFile = ref('Esperando iniciar...')
const logs = ref<string[]>([])

// Calculamos el porcentaje para la barra CSS
const percentage = computed(() => {
  if (total.value === 0) return 0
  return Math.min(100, Math.round((progress.value / total.value) * 100))
})

// --- LISTENERS ---
let unlistenStart: () => void
let unlistenSuccess: () => void
let unlistenError: () => void
let unlistenFinish: () => void

onMounted(async () => {
  // Escuchar inicio
  unlistenStart = await listen<number>('library:import:start', (event) => {
    isRunning.value = true
    total.value = event.payload
    progress.value = 0
    logs.value = [] // Limpiar logs anteriores
    logs.value.push(`🚀 Iniciando importación de ${event.payload} archivos...`)
  })

  // Escuchar progreso (éxito)
  unlistenSuccess = await listen<string>('library:import:success', (event) => {
    progress.value++
    currentFile.value = event.payload
  })

  // Escuchar errores
  unlistenError = await listen<{ path: string; error: string }>('library:import:error', (event) => {
    progress.value++
    logs.value.push(`❌ Error en ${event.payload.path}: ${event.payload.error}`)
  })

  // Escuchar finalización
  unlistenFinish = await listen('library:import:finish', () => {
    isRunning.value = false
    logs.value.push('✅ Importación finalizada con éxito.')
    currentFile.value = 'Proceso completado.'
  })
})

// Limpiar listeners al salir de la vista para evitar fugas de memoria
onUnmounted(() => {
  if (unlistenStart) unlistenStart()
  if (unlistenSuccess) unlistenSuccess()
  if (unlistenError) unlistenError()
  if (unlistenFinish) unlistenFinish()
})

// --- ACCIONES ---
async function runImport() {
  if (isRunning.value) return

  try {
    logs.value.push('⏳ Solicitando escaneo al Core...')
    await invoke('library_import_full')
  } catch (e) {
    logs.value.push(`💀 Error crítico al lanzar import: ${e}`)
    console.error(e)
  }
}
</script>

<template>
  <div class="importer-card">
    <h2>Gestión de Biblioteca</h2>

    <div class="controls">
      <button
        @click="runImport"
        class="import-btn"
        :disabled="isRunning"
        :class="{ processing: isRunning }"
      >
        {{ isRunning ? 'Importando...' : 'Escanear Biblioteca' }}
      </button>
    </div>

    <div class="progress-container" v-if="total > 0">
      <div class="progress-labels">
        <span>{{ progress }} / {{ total }}</span>
        <span>{{ percentage }}%</span>
      </div>
      <div class="progress-track">
        <div class="progress-fill" :style="{ width: percentage + '%' }"></div>
      </div>
      <div class="current-file" title="Archivo actual">
        {{ currentFile }}
      </div>
    </div>

    <div class="logs-console">
      <ul>
        <li v-for="(log, index) in logs" :key="index">{{ log }}</li>
        <li v-if="logs.length === 0" class="placeholder">Esperando logs...</li>
      </ul>
    </div>
  </div>
</template>

<style scoped>
.importer-card {
  border: 1px solid var(--color-border);
  background: var(--color-background-soft);
  border-radius: 12px;
  padding: 1.5rem;
  width: 100%;
  max-width: 600px;
  margin: 0 auto;
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
}

h2 {
  margin-top: 0;
  margin-bottom: 1rem;
  font-size: 1.2rem;
  color: var(--color-heading);
}

.import-btn {
  background-color: #42b883; /* Vue Green */
  color: white;
  border: none;
  padding: 0.8rem 1.5rem;
  border-radius: 8px;
  font-weight: bold;
  cursor: pointer;
  transition: background 0.2s;
  width: 100%;
}

.import-btn:hover:not(:disabled) {
  background-color: #3aa876;
}

.import-btn:disabled {
  background-color: #798b84;
  cursor: not-allowed;
  opacity: 0.7;
}

.progress-container {
  margin-top: 1.5rem;
}

.progress-labels {
  display: flex;
  justify-content: space-between;
  font-size: 0.9rem;
  margin-bottom: 0.4rem;
  font-weight: bold;
}

.progress-track {
  height: 10px;
  background-color: var(--color-border);
  border-radius: 5px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background-color: #42b883;
  transition: width 0.2s ease;
}

.current-file {
  margin-top: 0.5rem;
  font-size: 0.8rem;
  color: var(--color-text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  opacity: 0.8;
  font-family: monospace;
}

.logs-console {
  margin-top: 1.5rem;
  background-color: #1e1e1e;
  border-radius: 6px;
  height: 150px;
  overflow-y: auto;
  padding: 0.5rem;
  border: 1px solid #333;
}

.logs-console ul {
  list-style: none;
  padding: 0;
  margin: 0;
  font-family: 'Courier New', Courier, monospace;
  font-size: 0.8rem;
}

.logs-console li {
  color: #d4d4d4;
  margin-bottom: 2px;
  word-break: break-all;
}

.logs-console li.placeholder {
  color: #555;
  font-style: italic;
}
</style>
