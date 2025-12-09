use std::path::Path;

pub fn apply_linux_patches() {
  #[cfg(target_os = "linux")]
  unsafe {
    if is_dangerous_combo() {
      println!("🔧 Infraestructura: Ajustando renderizado para Nvidia+Wayland");
      std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }
  }
}

#[cfg(target_os = "linux")]
fn is_dangerous_combo() -> bool {
  let is_wayland = std::env::var("XDG_SESSION_TYPE")
    .map(|v| v.to_lowercase().contains("wayland"))
    .unwrap_or(false);

  let has_nvidia = Path::new("/sys/module/nvidia").exists();

  is_wayland && has_nvidia
}
