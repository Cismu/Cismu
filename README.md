# Cismu

**Un reproductor de música moderno y multiplataforma**

## 🎵 Filosofía del Proyecto

Cismu es un reproductor de música diseñado con una filosofía de **modularidad, portabilidad y precisión**. El proyecto se basa en principios fundamentales que priorizan la integridad de los datos musicales y la flexibilidad de implementación.

### Principios Fundamentales

- **Arquitectura Modular**: El proyecto está estructurado en crates independientes que manejan responsabilidades específicas
- **Portabilidad**: Soporta tanto despliegue estándar como modo portable mediante la variable de entorno `CISMU_BASE_DIR`
- **Análisis Avanzado**: Integra análisis de características musicales avanzado usando `bliss-audio` para recomendaciones y similitud

## 🏗️ Arquitectura

Cismu está construido como una aplicación híbrida usando **Tauri** con un backend en Rust y frontend web:

- **Backend**: Rust nativo que maneja el procesamiento de música y datos
- **Frontend**: Interfaz web usando Astro
- **IPC**: Comunicación asíncrona entre frontend y backend via comandos Tauri

### Crates Principales

| Crate                 | Propósito                                                          |
| --------------------- | ------------------------------------------------------------------ |
| `cismu-core`          | Modelos de datos fundamentales y contratos de tipos                |
| `cismu-local-library` | Implementación de biblioteca local y procesamiento                 |
| `cismu-paths`         | Gestión centralizada de rutas y utilidades del sistema de archivos |

## 🎼 Formatos Soportados

Cismu soporta una amplia gama de formatos de audio con configuraciones específicas de tamaño mínimo y duración:

- **MP3**: Archivo mínimo 500KB, duración mínima 30s
- **AAC**: Archivo mínimo 500KB, duración mínima 30s
- **FLAC**: Archivo mínimo 2MB, duración mínima 30s
- **WAV**: Archivo mínimo 5MB, duración mínima 30s
- **OGG/OPUS**: Archivo mínimo 500KB, duración mínima 30s
- **MP4/M4A**: Archivo mínimo 1MB, duración mínima 30s

## 🚀 Instalación y Compilación

### Requisitos del Sistema

- **Rust**: Versión estable más reciente
- **Node.js**: Para el frontend Astro
- **Dependencias del Sistema**:
  - SQLite3 para la base de datos
  - FFmpeg para análisis de audio

### Compilación en Modo Desarrollo

```bash
# Clonar el repositorio
git clone https://github.com/Cismu/Cismu.git
cd Cismu

# Compilar el backend
cd desktop-app/backend
cargo build

# Ejecutar la aplicación de escritorio
cargo run
```

### Compilación para Producción

```bash
# Compilación optimizada para tamaño
cargo build --release
```

El perfil de release está optimizado para binarios compactos con LTO habilitado

## 🛠️ Comandos de Desarrollo

### Comandos del Backend (Rust)

```bash
# Ejecutar tests
cargo test

# Ejecutar con logs detallados
RUST_LOG=debug cargo run

# Verificar código
cargo clippy
cargo fmt
```

### Modo Portable

Para ejecutar en modo portable:

```bash
# Establecer directorio base personalizado
export CISMU_BASE_DIR=/ruta/a/directorio/portable
cargo run
```

## 📁 Estructura del Proyecto

```
Cismu/
├── crates/                    # Bibliotecas Rust modulares
│   ├── cismu-core/           # Modelos de datos fundamentales
│   ├── cismu-local-library/  # Implementación de biblioteca local
│   └── cismu-paths/          # Gestión de rutas del sistema
├── desktop-app/              # Aplicación de escritorio Tauri
│   ├── backend/              # Backend en Rust
│   └── isolation/            # Configuración de aislamiento
├── docs/                     # Documentación del proyecto
└── packages/                 # Paquetes adicionales
```

## 🤝 Contribuir al Proyecto

### Preparación del Entorno

1. Fork el repositorio en GitHub
2. Clona tu fork localmente
3. Crea una rama para tu característica: `git checkout -b mi-nueva-caracteristica`

### Estándares de Código

- **Rust**: Seguir las convenciones estándar de Rust (`cargo fmt`, `cargo clippy`)
- **Commits**: Usa mensajes descriptivos en español o inglés
- **Tests**: Incluye tests para nueva funcionalidad
- **Documentación**: Documenta APIs públicas

### Proceso de Pull Request

1. Asegúrate de que todos los tests pasan: `cargo test`
2. Verifica que el código compila sin warnings: `cargo clippy`
3. Formatea el código: `cargo fmt`
4. Crea un pull request con descripción detallada

## 📊 Gestión de Datos

### Sistema de Base de Datos

Cismu utiliza SQLite con migraciones manejadas por Refinery

### Gestión de Caché

El sistema de caché utiliza una jerarquía de directorios optimizada para el rendimiento del sistema de archivos [20](#0-19)

## 🔧 Configuración Avanzada

### Variables de Entorno

| Variable         | Propósito                                            |
| ---------------- | ---------------------------------------------------- |
| `CISMU_BASE_DIR` | Habilita modo portable especificando directorio base |
| `RUST_LOG`       | Controla nivel de logging (debug, info, warn, error) |

### Estructura de Directorios

El sistema crea automáticamente la estructura de directorios necesaria al primer arranque

## 📖 Documentación Adicional

Para información más detallada sobre componentes específicos, consulta:

- [Arquitectura del Core](docs/) - Modelos de datos y arquitectura interna
- [API Reference](docs/) - Documentación de APIs
- [Guía de Desarrollo](docs/) - Guías detalladas para desarrolladores

## 📝 Licencia

Este proyecto está licenciado bajo los términos especificados en el archivo LICENSE del repositorio.

---

**¡Contribuciones bienvenidas!** Si encuentras bugs o tienes ideas para mejoras, no dudes en abrir un issue o enviar un pull request.
