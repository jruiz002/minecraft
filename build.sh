#!/bin/bash

echo "=== Raytracing Diorama Build Script ==="
echo "Configurando proyecto..."

# Crear estructura de directorios
mkdir -p src assets

# Crear directorio de assets si no existe
if [ ! -d "assets" ]; then
    mkdir assets
    echo "Directorio assets/ creado. Coloca aquí tus texturas (.png):"
    echo "  - grass.png (textura de césped)"
    echo "  - stone.png (textura de piedra)"
    echo "  - wood.png (textura de madera)"
fi

# Verificar que Rust está instalado
if ! command -v rustc &> /dev/null; then
    echo "Error: Rust no está instalado. Instala Rust desde https://rustup.rs/"
    exit 1
fi

echo "Versión de Rust: $(rustc --version)"
echo "Versión de Cargo: $(cargo --version)"

# Compilar en modo debug primero
echo ""
echo "=== Compilando en modo debug ==="
cargo build

if [ $? -ne 0 ]; then
    echo "Error: Falló la compilación en modo debug"
    exit 1
fi

# Compilar en modo release
echo ""
echo "=== Compilando en modo release ==="
cargo build --release

if [ $? -ne 0 ]; then
    echo "Error: Falló la compilación en modo release"
    exit 1
fi

echo ""
echo "=== Compilación exitosa! ==="
echo ""
echo "Para ejecutar:"
echo "  cargo run --release       (recomendado para mejor rendimiento)"
echo "  cargo run                 (modo debug)"
echo ""
echo "Controles:"
echo "  WASD: Mover cámara"
echo "  QE: Subir/Bajar"
echo "  R: Rotar escena"
echo "  ESC: Salir"
echo ""
echo "¡Disfruta tu diorama de raytracing!"