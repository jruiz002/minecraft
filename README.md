# Minecraft Raytracing Diorama (Rust)

## Descripción

Diorama interactivo estilo Minecraft renderizado por CPU en Rust (ray tracing puro) usando `minifb` para la ventana y `rayon` para paralelismo. La escena incluye terreno, casa, torre, estanque, portal, fogata, árbol procedural y mobiliario (banco) cargado desde OBJ; además de ciclo de día/noche con sol, materiales emisivos y texturas animadas.

[Video demo (YouTube)](https://youtu.be/KkdcXJnNe3k?si=X5lfg3E2tIoirAOQ)

## Características (alineadas a la rúbrica)

- Complejidad de escena (30): terreno en capas, casa con ventanas, torre circular, estanque, portal Nether, fogata, árbol procedural, banco OBJ y antorchas.
- Atractivo visual (20): skybox día/noche con sol y estrellas, materiales variados, portal animado, chispas en fogata, diamante giratorio, agua que responde a la hora del día.
- FPS (20): BVH de aceleración, escalado dinámico 1–4, Ultra Mode (checkerboard + temporal reuse), control de sombras y profundidad, LTO en release.
- Day/Night + sol (15): ciclo con sol direccional sincronizado con skybox; velocidad ajustable.
- Texturas animadas (10): agua, fuego y portal procedurales.
- Threads (15): render paralelo por filas con `rayon`.
- Rotación/zoom (10): rotación del diorama y zoom con trackpad/mouse; cámara WASD/QE.
- Materiales (5×5): césped, piedra, madera, vidrio, agua, diamante, obsidiana, glowstone, portal, fogata (cada uno con textura/procedural y albedo/specular/transparencia/reflectividad propios).
- Refracción (10): agua (1.33), vidrio (1.5), diamante (2.4).
- Efecto portal (20): material emisivo animado tipo Nether.
- Reflexión (5): obsidiana, vidrio y diamante con reflectividad.
- Modelo OBJ (30): `assets/tree.obj` y `assets/bench.obj` (autogenerados si faltan).
- Skybox con texturas (10): gradiente día/noche con sol y estrellas.
- Emisivos con luz (20): glowstone, fogata y antorchas generan luces puntuales.

## Controles

- Movimiento: `W/A/S/D` (frente/izq/atrás/der), `Q/E` (bajar/subir)
- Mirar: arrastrar con mouse (click izquierdo)
- Rotar diorama: mantener `R`
- Zoom: scroll de mouse/trackpad (siempre activo)
- Ciclo día/noche: `T` alterna AUTO; con AUTO OFF usar `J/K` para scrub y `H` para saltar medio ciclo
- Velocidad día/noche: `N/M` disminuye/aumenta
- Rendimiento: `1–4` escala de resolución; `Y/U/I` sombras None/SunOnly/Full; `F/G` profundidad +/-
- Ultra Mode: `Z` (checkerboard + temporal reuse)
- Salir: `ESC`

## Instalación y ejecución

Requisitos: Rust estable (1.70+), Cargo.

```bash
cargo run --release
```

Sugerencias de rendimiento en laptops:
- Usa escala 3–4, sombras `Y`/`U`, profundidad 2–3 y Ultra Mode `Z` activado.

## Arquitectura

- `main.rs`: loop principal, entrada, control de calidad/escala y composición de frame (incluye Ultra Mode y checkerboard).
- `raytracer.rs`: cámara, luces, materiales, skybox, fog, BVH, intersecciones y sombreado (reflexión/refracción).
- `primitives.rs`: primitivas (Esfera, Plano, Cubo, Triángulo, Cilindro, Toroide) y `SpinningCube` animado para el diamante.
- `materials.rs`: materiales PBR-lite con builder (albedo, specular, transparencia, reflectividad, IOR, roughness, emissive).
- `texture.rs`: texturas procedurales y animadas (agua, fuego, portal, bloques estilo Minecraft) con calidades (High/Medium/Low).
- `obj_loader.rs`: cargador simple OBJ (triangulación por fan); autogenera `tree.obj` y `bench.obj` si faltan.

## Cómo funciona (resumen técnico)

- Rayos primarios por píxel (o checkerboard en Ultra Mode), intersección acelerada por BVH; sombreado directo (Lambert + Blinn-Phong), sombras por rayos de oclusión, reflexión y refracción con Fresnel simplificado.
- Ciclo día/noche: sol direccional y skybox sincronizados; emisivos (glowstone/antorchas/fogata) aportan luz puntual.
- Texturas animadas: funciones trig/noise/FBM; calidad adaptativa por distancia y profundidad de rebote.

## Assets

- `assets/tree.obj` y `assets/bench.obj` se crean automáticamente si no existen (no necesitas descargar nada).
- El proyecto usa texturas procedurales; no requiere imágenes externas.

## Tabla de cumplimiento (estimado)

| Criterio | Puntos | Estado |
|---|---:|:--:|
| Complejidad de escena | 30 | ✅ |
| Atractivo visual | 20 | ✅ |
| FPS (optimización) | 20 | ✅ |
| Día/Noche con sol | 15 | ✅ |
| Texturas animadas | 10 | ✅ |
| Threads (Rayon) | 15 | ✅ |
| Rotación y zoom | 10 | ✅ |
| 5 materiales distintos | 25 | ✅ |
| Refracción | 10 | ✅ |
| Reflexión | 5 | ✅ |
| Modelo 3D OBJ | 30 | ✅ |
| Skybox con texturas | 10 | ✅ |
| Emisivos con luz | 20 | ✅ |

## Autor

Nombre: José Gerardo Ruiz García

Carné: 23719

Curso: Gráficas por Computadora – UVG

---

Proyecto desarrollado completamente en Rust, con énfasis en claridad, modularidad y rendimiento en CPU.