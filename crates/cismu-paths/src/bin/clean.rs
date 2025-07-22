use cismu_paths::PATHS;
use std::path::Path;

fn main() {
    println!("🧹 Iniciando la limpieza de Cismu...");

    println!("\nSe intentarán eliminar las siguientes carpetas:");
    println!("- Configuración: {}", PATHS.config_dir.display());
    println!("- Datos: {}", PATHS.data_dir.display());
    println!("- Caché: {}", PATHS.cache_dir.display());
    println!();

    clean_directory(&PATHS.config_dir, "Configuración");
    clean_directory(&PATHS.data_dir, "Datos");
    clean_directory(&PATHS.cache_dir, "Caché");

    println!("\n✅ Limpieza completada.");
}

fn clean_directory(path: &Path, name: &str) {
    if path.exists() {
        print!("- Eliminando carpeta de {}: ", name);
        match std::fs::remove_dir_all(path) {
            Ok(_) => println!("¡Hecho! ✔️"),
            Err(e) => println!("¡Error! ❌ No se pudo eliminar: {}", e),
        }
    } else {
        println!("- La carpeta de {} no existe, se omite. 🤷", name);
    }
}
