# Cukcoo ENARM Aris — App de barra de menú (efecto popover) 🍎

App estilo widget del clima de macOS: vive en la barra superior, y al hacer clic
en su ícono se despliega una ventana **anclada justo debajo del ícono**. Al hacer
clic fuera, se oculta sola.

Hecha con **Tauri 2 + tauri-plugin-positioner** (Pake no soporta este efecto).

## Ya viene configurada con

- **URL**: `https://aristeovaladezmontero-coder.github.io/reloj/?sala=M7AF`
- **Nombre**: `Cukcoo ENARM Aris`
- **Ícono**: se descarga automáticamente al compilar desde
  `https://raw.githubusercontent.com/aristeovaladezmontero-coder/reloj/main/icono-cuckoo-enarm.png`
  y se convierte a todos los formatos (icns, png, tray).

## Cómo usarla

1. **Sube esta carpeta a un repositorio de GitHub.**
2. Ve a **Actions → Build macOS Menu Bar App → Run workflow**.
3. Descarga el artifact `menubar-app-macOS` (incluye `.app` en zip y `.dmg`).
4. Si macOS dice que la app está dañada (no está notarizada), ejecuta una vez:
   ```bash
   xattr -cr "/Applications/Cukcoo ENARM Aris.app"
   ```

## Compilar localmente (opcional)

Si tienes Rust y Node instalados en tu Mac:

```bash
npm install
npx tauri build --bundles app,dmg
```

## Comportamiento

| Acción | Resultado |
|---|---|
| Clic en el ícono de la barra | El panel aparece debajo del ícono |
| Clic de nuevo | Se oculta |
| Clic fuera del panel | Se oculta solo (pierde el foco) |
| Dock / Cmd+Tab | No aparece (ActivationPolicy::Accessory) |

## Personalizar íconos

El workflow regenera los íconos en cada build a partir de tu PNG de GitHub.
Si cambias `icono-cuckoo-enarm.png` en tu repo `reloj`, el siguiente build
usará el ícono nuevo automáticamente. Los archivos en `src-tauri/icons/` son
solo placeholders para compilar en local.
