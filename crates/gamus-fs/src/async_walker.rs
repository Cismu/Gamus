use std::collections::HashSet;
use std::future::Future;
use std::io;
use std::path::{Path, PathBuf};

use futures::stream::{self, Stream};
use tokio::fs::{self, ReadDir};

// =============================================================================
// 1. Identificadores de Archivo (Platform Specific)
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct FileId(u64, u64);

#[cfg(unix)]
fn get_file_id(meta: &std::fs::Metadata) -> FileId {
  use std::os::unix::fs::MetadataExt;
  FileId(meta.dev(), meta.ino())
}

#[cfg(windows)]
fn get_file_id(meta: &std::fs::Metadata) -> FileId {
  use std::os::windows::fs::MetadataExt;
  // En Windows modernos, volume_serial + file_index es único
  FileId(meta.volume_serial_number().unwrap_or(0) as u64, meta.file_index().unwrap_or(0))
}

#[cfg(not(any(unix, windows)))]
fn get_file_id(_meta: &std::fs::Metadata) -> FileId {
  FileId(0, 0) // Fallback para otros SO (riesgo de ciclos si no soportan ino)
}

// =============================================================================
// 2. Configuración y Tipos
// =============================================================================

/// Configuración para controlar el recorrido.
#[derive(Debug, Clone)]
pub struct WalkConfig {
  pub follow_symlinks: bool,
  pub max_depth: usize,
  /// Deduplica directorios visitados para evitar ciclos infinitos.
  /// Recomendado true si follow_symlinks es true.
  pub dedup_dirs: bool,
}

impl Default for WalkConfig {
  fn default() -> Self {
    Self { follow_symlinks: true, max_depth: 100, dedup_dirs: true }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Filtering {
  Ignore,    // Ignorar archivo, pero si es dir, entrar.
  IgnoreDir, // Ignorar archivo y NO entrar si es dir.
  Continue,  // Procesar normalmente.
}

#[derive(Debug)]
pub struct WalkEntry {
  pub path: PathBuf,
  pub depth: usize,
  /// Tipo de archivo obtenido vía `lstat` (symlink es symlink).
  pub file_type: std::fs::FileType,
}

impl WalkEntry {
  pub fn path(&self) -> &Path {
    &self.path
  }
}

// =============================================================================
// 3. Estado Interno (Máquina de Estados)
// =============================================================================

enum Frame {
  /// Estado: Vamos a intentar abrir un directorio
  Pending {
    path: PathBuf,
    depth: usize,
    /// Si venimos de un symlink resuelto, ya tenemos su ID
    id_hint: Option<FileId>,
  },
  /// Estado: Estamos iterando un directorio abierto
  Open { rd: ReadDir, depth: usize },
}

// =============================================================================
// 4. Implementación del Walker
// =============================================================================

/// Crea un Stream que recorre el directorio recursivamente (sin filtrar).
pub fn walk(
  root: impl Into<PathBuf>,
  cfg: WalkConfig,
) -> impl Stream<Item = io::Result<WalkEntry>> {
  walk_filtered(root, cfg, |_| async { Filtering::Continue })
}

/// Crea un Stream con filtrado asíncrono.
pub fn walk_filtered<F, Fut>(
  root: impl Into<PathBuf>,
  cfg: WalkConfig,
  filter: F,
) -> impl Stream<Item = io::Result<WalkEntry>>
where
  F: FnMut(&WalkEntry) -> Fut + Send + 'static,
  Fut: Future<Output = Filtering> + Send,
{
  let root = root.into();
  // Optimizamos memoria reservando un poco de espacio inicial
  let mut stack = Vec::with_capacity(16);

  // Frame inicial
  stack.push(Frame::Pending { path: root, depth: 0, id_hint: None });

  let visited = HashSet::new();
  // Usamos Arc para el filtro si fuera necesario compartir, pero aquí lo movemos al closure.
  // El 'state' del unfold contiene: (Pila, Set de Visitados, Config, Filtro)
  let state = (stack, visited, cfg, filter);

  stream::unfold(state, |(mut stack, mut visited, cfg, mut filter)| async move {
    loop {
      // 1. Obtener el tope de la pila
      let top = stack.last_mut()?; // Si None, termina el stream

      match top {
        // CASO A: Procesar un directorio pendiente
        Frame::Pending { path, depth, id_hint } => {
          let path = path.clone();
          let depth = *depth;
          let id_hint = *id_hint;

          // Quitamos el Frame Pending. Si tiene éxito, pondremos un Frame Open.
          stack.pop();

          if depth > cfg.max_depth {
            continue;
          }

          // --- Lógica de Deduplicación (Anti-Ciclos) ---
          if cfg.dedup_dirs {
            let file_id = match id_hint {
              Some(id) => Some(id),
              None => {
                // Solo hacemos metadata si no tenemos el hint
                match fs::metadata(&path).await {
                  Ok(m) => {
                    if m.is_dir() {
                      Some(get_file_id(&m))
                    } else {
                      None // Raro: path raíz no era dir
                    }
                  }
                  Err(e) => {
                    // Emitimos error y seguimos
                    return Some((Err(e), (stack, visited, cfg, filter)));
                  }
                }
              }
            };

            if let Some(id) = file_id {
              if !visited.insert(id) {
                // Ya visitado, cortamos ciclo.
                continue;
              }
            }
          }

          // --- Abrir Directorio ---
          match fs::read_dir(&path).await {
            Ok(rd) => {
              stack.push(Frame::Open { rd, depth });
            }
            Err(e) => {
              // Error al abrir (ej. Permiso Denegado). Lo emitimos pero no crasheamos.
              return Some((Err(e), (stack, visited, cfg, filter)));
            }
          }
        }

        // CASO B: Leer entradas de un directorio abierto
        Frame::Open { rd, depth } => {
          let depth = *depth;

          match rd.next_entry().await {
            Ok(Some(entry)) => {
              let path = entry.path();

              // Obtenemos tipo (lstat)
              let ft = match entry.file_type().await {
                Ok(ft) => ft,
                Err(e) => return Some((Err(e), (stack, visited, cfg, filter))),
              };

              let entry_depth = depth + 1;
              let walk_entry = WalkEntry { path: path.clone(), depth: entry_depth, file_type: ft };

              // --- Filtrado ---
              let filtering = filter(&walk_entry).await;

              // Decidir si recursamos
              // Solo recursamos si NO es IgnoreDir Y no excedemos profundidad
              let recurse = filtering != Filtering::IgnoreDir && entry_depth <= cfg.max_depth;

              // Determinamos si es un target válido para recursión (Dir o Symlink->Dir)
              let mut pending_frame = None;

              if recurse {
                if ft.is_dir() {
                  pending_frame = Some(Frame::Pending {
                    path,
                    depth: entry_depth,
                    id_hint: None, // Se calculará al entrar
                  });
                } else if ft.is_symlink() && cfg.follow_symlinks {
                  // Truco de optimización: Resolvemos metadata AHORA.
                  // Si es dir, obtenemos su ID y lo pasamos como hint.
                  match fs::metadata(&walk_entry.path).await {
                    Ok(m) if m.is_dir() => {
                      let id = if cfg.dedup_dirs { Some(get_file_id(&m)) } else { None };
                      pending_frame =
                        Some(Frame::Pending { path, depth: entry_depth, id_hint: id });
                    }
                    _ => {} // No es dir o error, no recursamos
                  }
                }
              }

              // Si hay que recursar, metemos el directorio en la pila
              if let Some(frame) = pending_frame {
                stack.push(frame);
              }

              // Emitir resultado (si no es Ignore)
              match filtering {
                Filtering::Continue => {
                  return Some((Ok(walk_entry), (stack, visited, cfg, filter)));
                }
                _ => continue, // Ignore/IgnoreDir: bucle para siguiente entrada
              }
            }
            Ok(None) => {
              // Fin del directorio actual, sacamos el Frame Open
              stack.pop();
            }
            Err(e) => {
              // Error leyendo entrada, sacamos el dir y reportamos
              stack.pop();
              return Some((Err(e), (stack, visited, cfg, filter)));
            }
          }
        }
      }
    }
  })
}
