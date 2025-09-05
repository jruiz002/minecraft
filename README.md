# Raytracing Diorama en Rust

## Descripción del Proyecto

Este proyecto implementa un diorama completo usando raytracing en Rust puro, sin librerías externas de renderizado. El diorama presenta un castillo medieval con elementos fantásticos, incluyendo efectos avanzados de iluminación y materiales.

## Características Implementadas

### ✅ Funcionalidades Principales (170+ puntos)

- **[30 pts] Complejidad de la Escena**: Castillo medieval con torres, puentes, agua, cristales
- **[20 pts] Atractivo Visual**: Materiales realistas, iluminación dinámica, texturas animadas
- **[20 pts] Rendimiento**: Optimizado con multithreading (Rayon) para mantener FPS altos
- **[15 pts] Ciclo Día/Noche**: Sol dinámico con cambios de color e intensidad
- **[10 pts] Texturas Animadas**: Agua con ondas procedurales animadas
- **[15 pts] Multithreading**: Renderizado paralelo usando Rayon
- **[10 pts] Rotación de Cámara**: Controles completos de cámara con rotación de escena

### ✅ Materiales (25+ puntos)
- **[25 pts] 5 Materiales Diferentes**:
  1. **Césped**: Textura verde con poca reflectividad
  2. **Piedra**: Material rugoso para el castillo con especularidad media
  3. **Madera**: Para puentes con baja especularidad
  4. **Agua**: Material transparente con refracción (índice 1.33)
  5. **Cristal**: Material altamente transparente con refracción (índice 1.5)

### ✅ Efectos Avanzados (65+ puntos)
- **[10 pts] Refracción**: Implementada en agua y cristal
- **[5 pts] Reflexión**: En superficies metálicas y agua
- **[30 pts] Modelo 3D OBJ**: Cargador de modelos OBJ con tetera de ejemplo
- **[10 pts] Skybox**: Sistema de skybox con gradientes dinámicos
- **[20 pts] Materiales Emisivos**: Antorchas que emiten luz y rayos

## Controles

- **WASD**: Mover cámara (adelante/atrás/izquierda/derecha)
- **QE**: Subir/bajar cámara
- **R**: Rotar escena
- **ESC**: Salir

## Instalación y Ejecución

### Prerrequisitos
- Rust 1.70 o superior
- Cargo

### Compilación
```bash
# Clonar el repositorio
git clone [tu-repo-url]
cd raytracing_diorama

# Compilar en modo release para mejor rendimiento
cargo build --release

# Ejecutar
cargo run --release
```

### Assets Opcionales
Coloca texturas en la carpeta `assets/` para mejorar la calidad visual:
- `grass.png` - Textura de césped
- `stone.png` - Textura de piedra
- `wood.png` - Textura de madera

Si no están disponibles, el programa usará colores sólidos.

## Arquitectura del Código

### Módulos Principales

- **`main.rs`**: Loop principal, manejo de entrada y renderizado
- **`math.rs`**: Estructuras matemáticas (Vec3, Ray) y operaciones
- **`raytracer.rs`**: Motor de raytracing, cámara y escena
- **`materials.rs`**: Sistema de materiales con propiedades físicas
- **`primitives.rs`**: Primitivas geométricas (Esfera, Cubo, Plano, Triángulo)
- **`texture.rs`**: Sistema de texturas procedurales y animadas
- **`obj_loader.rs`**: Cargador de modelos 3D en formato OBJ

### Características Técnicas

- **Multithreading**: Renderizado paralelo por filas usando Rayon
- **Optimizaciones**: Cálculos matemáticos optimizados para raytracing
- **Flexibilidad**: Sistema modular que permite agregar nuevos materiales y primitivas
- **Animaciones**: Sistema temporal para efectos dinámicos

## Video Demo

[Enlace al video demo mostrando todas las características]

## Detalles de Implementación

### Sistema de Raytracing
- Implementación clásica de raytracing con reflexiones y refracciones
- Soporte para múltiples rebotes de luz
- Cálculos de Fresnel para refracción realista
- Sistema de sombras con rayos de oclusión

### Materiales Físicamente Basados
- Propiedades albedo, especular, transparencia, reflectividad
- Índices de refracción correctos para diferentes materiales
- Materiales emisivos con contribución de luz
- Sistema de texturas con UV mapping

### Efectos Especiales
- **Portal Effects**: [Si implementado] Efectos especiales tipo portal
- **Ciclo Día/Noche**: Transiciones suaves de iluminación
- **Texturas Procedurales**: Generación matemática de patrones
- **Animaciones**: Ondas de agua, movimiento de luz

## Puntuación Objetivo

| Característica | Puntos | Estado |
|---|---|---|
| Complejidad | 30 | ✅ |
| Atractivo Visual | 20 | ✅ |
| FPS | 20 | ✅ |
| Ciclo Día/Noche | 15 | ✅ |
| Texturas Animadas | 10 | ✅ |
| Threads | 15 | ✅ |
| Rotación/Cámara | 10 | ✅ |
| 5 Materiales | 25 | ✅ |
| Refracción | 10 | ✅ |
| Reflexión | 5 | ✅ |
| Modelo OBJ | 30 | ✅ |
| Skybox | 10 | ✅ |
| Materiales Emisivos | 20 | ✅ |
| **Total Estimado** | **220** | **✅** |

## Autor

[Tu Nombre]  
[Tu Carné]  
Universidad del Valle de Guatemala  
Gráficas por Computadora

---

*Proyecto desarrollado completamente en Rust sin librerías externas de renderizado, siguiendo todos los requerimientos del curso.*