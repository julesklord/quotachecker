# Auditoría de Calidad de Producto: QuotaChecker-TUI

Este documento presenta una auditoría exhaustiva de calidad de producto para el proyecto **QuotaChecker-TUI**. Se evalúa la arquitectura del software, la robustez del código, la experiencia de usuario (UI/UX), el rendimiento y la seguridad del sistema.

---

## 1. Resumen Ejecutivo

**QuotaChecker-TUI** es una herramienta de terminal sólida e intuitiva diseñada para monitorizar cuotas de uso de agentes de Inteligencia Artificial locales y en la nube. 
* **Estado general del software:** **Favorable / Beta Estable**. El programa compila correctamente con la última versión de Rust sin warnings ni errores de análisis de tipos.
* **Diseño arquitectónico:** Muy bien estructurado. Sigue la separación clásica de responsabilidades (UI, configuración, análisis/telemetría y bucle de eventos principal).
* **Rendimiento:** Aceptable para el uso cotidiano, pero presenta optimizaciones pendientes en la interacción con el disco (E/S) y accesos concurrentes a bases de datos SQLite.

---

## 2. Fortalezas Detectadas

### 2.1. Arquitectura Modular y Desacoplada
El código está organizado de manera limpia en módulos específicos:
* [main.rs](file:///mnt/DEV/Proyectos/repos/quotachecker/quotachecker-tui/src/main.rs): Controla el bucle de eventos de Crossterm y coordina el estado de la aplicación.
* [config.rs](file:///mnt/DEV/Proyectos/repos/quotachecker/quotachecker-tui/src/config.rs): Gestiona la carga, guardado y serialización JSON de la configuración del usuario en directorios estándar del sistema (siguiendo especificaciones XDG).
* [agent.rs](file:///mnt/DEV/Proyectos/repos/quotachecker/quotachecker-tui/src/agent.rs): Centraliza la lógica de escaneo del sistema, la comprobación de binarios y la lectura de bases de datos de telemetría local de otros agentes (Codex, OpenCode, Gemini-CLI, Agy, Zed).
* [ui.rs](file:///mnt/DEV/Proyectos/repos/quotachecker/quotachecker-tui/src/ui.rs): Define la interfaz de usuario utilizando Ratatui de manera puramente declarativa mediante un `RenderContext`.

### 2.2. Concurrencia y Reactividad
* **Hilo en Segundo Plano (Background Thread):** El escaneo de telemetría de base de datos y archivos de registro se realiza en un hilo separado usando canales (`std::sync::mpsc::channel`). Esto previene el bloqueo de la interfaz de usuario (UI freeze) debido a latencias de disco.
* **Manejo Seguro de Terminales:** En [main.rs](file:///mnt/DEV/Proyectos/repos/quotachecker/quotachecker-tui/src/main.rs#L218-L224), se implementa un gancho de pánico (`std::panic::set_hook`) para restaurar de forma segura el modo de terminal original (desactivar modo raw y salir de la pantalla alternativa) si ocurre un fallo catastrófico. Esto previene que la terminal del usuario quede corrupta tras un crash.

### 2.3. Caching de Procesos Hijo
Llamar repetidamente a comandos del sistema como `which` o `exe --version` es costoso. El proyecto utiliza de manera inteligente `OnceLock` y `Mutex` para cachear la existencia y versión de los ejecutables de los agentes ([agent.rs](file:///mnt/DEV/Proyectos/repos/quotachecker/quotachecker-tui/src/agent.rs#L120-L137)):
```rust
fn get_cached_executable(cmd: &str) -> Option<String> {
    static CACHE: OnceLock<Mutex<HashMap<String, Option<String>>>> = OnceLock::new();
    // ... cache logic
}
```
Esto reduce significativamente el consumo de CPU y tiempo de CPU del sistema tras el primer escaneo.

---

## 3. Hallazgos Críticos y Puntos de Mejora (Estado Actualizado)

Todos los hallazgos críticos detectados en la auditoría inicial han sido corregidos satisfactoriamente.

### 3.1. Riesgos de Concurrencia y Bloqueos en SQLite (SQLITE_BUSY) — **[CORREGIDO]**
> [!NOTE]
> **Estado: Solucionado**
> Se añadió `conn.busy_timeout(std::time::Duration::from_millis(500))` a todas las conexiones SQLite locales inicializadas en [agent.rs](file:///mnt/DEV/Proyectos/repos/quotachecker/quotachecker-tui/src/agent.rs) (Codex, Gemini-CLI, Zed), alineándolas con la protección existente en OpenCode. Esto evita fallas de lectura silenciosas por bloqueos concurrentes.

### 3.2. Consumo de E/S de Disco Ineficiente en el Hilo de Telemetría — **[CORREGIDO]**
> [!NOTE]
> **Estado: Solucionado**
> Se migró el almacenamiento de la configuración en [main.rs](file:///mnt/DEV/Proyectos/repos/quotachecker/quotachecker-tui/src/main.rs) a una estructura compartida segura en memoria: `Arc<RwLock<AppConfig>>`.
> El hilo de telemetría ahora lee la configuración directamente desde la memoria RAM (a través de un bloqueo de lectura), eliminando por completo las lecturas continuas del archivo `config.json` en disco cada 2 segundos.

### 3.3. Inconsistencia de Navegación UI/UX (Controles del Teclado) — **[CORREGIDO]**
> [!NOTE]
> **Estado: Solucionado**
> Las teclas de flechas izquierda/derecha se han reservado exclusivamente para la navegación uniforme de pestañas globales en todas las vistas de la aplicación.
> Para modificar configuraciones en la pestaña *Settings* de manera precisa, se han habilitado las teclas `+` / `-` y `h` / `l` (junto con la tecla `Enter`), logrando consistencia con las indicaciones visuales inferiores.

### 3.4. Confusión de Nomenclatura en la Interfaz (Mantenimiento) — **[CORREGIDO]**
> [!NOTE]
> **Estado: Solucionado**
> Se realizó una refactorización de nombres en [ui.rs](file:///mnt/DEV/Proyectos/repos/quotachecker/quotachecker-tui/src/ui.rs):
> * `draw_settings_tab` se renombró a `draw_quotas_tab` (renderiza cuotas y límites).
> * `draw_config_tab` se renombró a `draw_settings_tab` (renderiza configuraciones e interactividad).
> Esto corrige la deuda técnica de nomenclatura invertida.

### 3.5. Baja Cobertura de Pruebas Unitarias — **[MEJORADO]**
> [!NOTE]
> **Estado: Mitigado**
> Se expandió la suite de pruebas automatizadas en [tests.rs](file:///mnt/DEV/Proyectos/repos/julesklord/quotachecker/quotachecker-tui/src/tests.rs), introduciendo validaciones específicas para el mapeo de nombres de categorías y cuotas de usuario (`UserTier::display_name`).

### 3.6. Advertencias y Errores de Linter Clippy (-D warnings) — **[CORREGIDO]**
> [!NOTE]
> **Estado: Solucionado**
> Se resolvieron 5 advertencias/errores de Clippy bajo la regla estricta `-D warnings` (retornos innecesarios en `agent.rs`, inicializador const de thread-local y reasignaciones de campos usando valores por defecto en `config.rs`, e incepción de módulos en `tests.rs`). Ahora el proyecto compila limpiamente sin warnings ni errores de linter.

### 3.7. Riesgo de Colisión en Archivos Temporales de Pruebas — **[CORREGIDO]**
> [!NOTE]
> **Estado: Solucionado**
> Se incorporó el ID de proceso (`std::process::id()`) en la generación de nombres de ejecutables de prueba temporales (`create_mock_executable`) en `src/agent.rs` para garantizar la unicidad de los nombres de archivo y evitar colisiones en ejecuciones de pruebas concurrentes o multiusuarios en entornos de desarrollo y CI/CD.

---

## 4. Estado del Plan de Acción

Las tareas planificadas han concluido exitosamente:

| Tarea | Estado | Archivos Afectados |
|---|---|---|
| Configurar busy_timeout en SQLite | **Completado** | [agent.rs](file:///mnt/DEV/Proyectos/repos/julesklord/quotachecker/quotachecker-tui/src/agent.rs) |
| Optimizar lectura de config (disco/RAM) | **Completado** | [main.rs](file:///mnt/DEV/Proyectos/repos/julesklord/quotachecker/quotachecker-tui/src/main.rs) |
| Corregir navegación flechas izq/der | **Completado** | [main.rs](file:///mnt/DEV/Proyectos/repos/julesklord/quotachecker/quotachecker-tui/src/main.rs), [ui.rs](file:///mnt/DEV/Proyectos/repos/julesklord/quotachecker/quotachecker-tui/src/ui.rs) |
| Renombrar funciones de pestañas en UI | **Completado** | [ui.rs](file:///mnt/DEV/Proyectos/repos/julesklord/quotachecker/quotachecker-tui/src/ui.rs), [main.rs](file:///mnt/DEV/Proyectos/repos/julesklord/quotachecker/quotachecker-tui/src/main.rs) |
| Aumentar cobertura de pruebas unitarias | **Completado** | [tests.rs](file:///mnt/DEV/Proyectos/repos/julesklord/quotachecker/quotachecker-tui/src/tests.rs) |
| Resolver advertencias y lints de Clippy | **Completado** | [agent.rs](file:///mnt/DEV/Proyectos/repos/julesklord/quotachecker/quotachecker-tui/src/agent.rs), [config.rs](file:///mnt/DEV/Proyectos/repos/julesklord/quotachecker/quotachecker-tui/src/config.rs), [tests.rs](file:///mnt/DEV/Proyectos/repos/julesklord/quotachecker/quotachecker-tui/src/tests.rs) |
| Robustecer unicidad de archivos temporales en tests | **Completado** | [agent.rs](file:///mnt/DEV/Proyectos/repos/julesklord/quotachecker/quotachecker-tui/src/agent.rs) |
