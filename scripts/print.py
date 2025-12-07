#!/usr/bin/env python3
import sys
from pathlib import Path

# Mapeo extensión -> lenguaje para el bloque ```lang
LANG_MAP = {
    ".rs": "rust",
    ".ts": "ts",
    ".tsx": "tsx",
    ".js": "js",
    ".jsx": "jsx",
    ".json": "json",
    ".toml": "toml",
    ".yaml": "yaml",
    ".yml": "yaml",
    ".md": "md",
    ".html": "html",
    ".css": "css",
    ".lock": "",           # normalmente no hace falta lang
    ".dbml": "sql",
    ".txt": "",
}

# Carpetas típicas a ignorar (ajusta si quieres)
IGNORE_DIRS = {".git", "node_modules", "target", "dist", "build"}


def guess_lang(path: Path) -> str:
    ext = path.suffix.lower()
    return LANG_MAP.get(ext, "")


def is_text_file(path: Path, sample_size: int = 2048) -> bool:
    """Intenta detectar binarios (png, ico, etc.) mirando null bytes."""
    try:
        with path.open("rb") as f:
            chunk = f.read(sample_size)
        if b"\x00" in chunk:
            return False
        return True
    except Exception:
        return False


def iter_files(base: Path):
    for p in sorted(base.rglob("*")):
        if p.is_dir():
            # Saltar carpetas ignoradas
            if p.name in IGNORE_DIRS:
                # Evitar seguir recursión dentro de estas carpetas
                # rglob ya las habrá expandido, pero podemos simplemente seguir
                continue
            continue
        yield p


def main():
    if len(sys.argv) < 2:
        print("Uso:")
        print("  python dump_folder.py <carpeta_a_recorrer> [root_del_proyecto]")
        print()
        print("Si no pasas root_del_proyecto, se usa el directorio actual (pwd).")
        sys.exit(1)

    folder = Path(sys.argv[1]).resolve()
    if len(sys.argv) >= 3:
        project_root = Path(sys.argv[2]).resolve()
    else:
        project_root = Path.cwd().resolve()

    if not folder.exists() or not folder.is_dir():
        print(f"La carpeta '{folder}' no existe o no es un directorio.", file=sys.stderr)
        sys.exit(1)

    for file_path in iter_files(folder):
        if not is_text_file(file_path):
            continue  # saltar binarios

        try:
            rel_path = file_path.relative_to(project_root)
        except ValueError:
            # Si por alguna razón no se puede relativizar, usar nombre absoluto
            rel_path = file_path

        lang = guess_lang(file_path)

        # Cabecera con la ruta relativa
        print(f"/// {rel_path}")
        print()

        # Apertura de bloque de código con lang si existe
        if lang:
            print(f"```{lang}")
        else:
            print("```")

        # Contenido del archivo
        try:
            with file_path.open("r", encoding="utf-8", errors="replace") as f:
                sys.stdout.write(f.read())
        except Exception as e:
            sys.stdout.write(f"<<Error leyendo archivo: {e}>>")

        # Cierre de bloque de código
        print()
        print("```")
        print()  # línea en blanco entre archivos


if __name__ == "__main__":
    main()