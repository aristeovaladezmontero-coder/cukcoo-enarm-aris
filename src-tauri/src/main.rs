#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{
    image::Image,
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    webview::PageLoadEvent,
    Manager,
};
use tauri_plugin_positioner::{Position, WindowExt};

/// Estilos "glass": fondo transparente para que se vea el vidrio nativo de
/// macOS, tintes translucidos en lugar de los gradientes solidos, y una
/// animacion pop de entrada estilo popover.
const GLASS_CSS: &str = r#"
html, body, #root { background: transparent !important; }
#root > div { border-radius: 22px; }

/* ===== Tema VIOLETA -> tintes translucidos ===== */
#root > div[style*="135deg,#5366da"] { background: linear-gradient(135deg, rgba(83,102,218,.28), rgba(112,96,237,.28)) !important; }
#root > div[style*="135deg,#02bde8"] { background: linear-gradient(135deg, rgba(2,189,232,.24), rgba(8,174,231,.24)) !important; }
#root > div[style*="135deg,#ff2b96"] { background: linear-gradient(135deg, rgba(255,43,150,.30), rgba(255,37,134,.30)) !important; }
#root [style*="background:#6460ec"] { background: rgba(100,96,236,.30) !important; }
#root [style*="background:#06b8ed"] { background: rgba(6,184,237,.26) !important; }
#root [style*="background:#ff278f"] { background: rgba(255,39,143,.30) !important; }

/* ===== Tema BOSQUE -> tintes translucidos ===== */
#root > div[style*="135deg,#1f8a5f"] { background: linear-gradient(135deg, rgba(31,138,95,.26), rgba(47,174,122,.26)) !important; }
#root > div[style*="135deg,#12b3a8"] { background: linear-gradient(135deg, rgba(18,179,168,.24), rgba(14,143,138,.24)) !important; }
#root > div[style*="135deg,#e8a23c"] { background: linear-gradient(135deg, rgba(232,162,60,.30), rgba(217,123,43,.30)) !important; }
#root [style*="background:#279368"] { background: rgba(39,147,104,.30) !important; }
#root [style*="background:#0f9c92"] { background: rgba(15,156,146,.26) !important; }
#root [style*="background:#dd8a30"] { background: rgba(221,138,48,.30) !important; }

/* ===== Tema MEDIANOCHE -> tintes translucidos ===== */
#root > div[style*="135deg,#242b52"] { background: linear-gradient(135deg, rgba(36,43,82,.32), rgba(58,61,122,.32)) !important; }
#root > div[style*="135deg,#1f6f8b"] { background: linear-gradient(135deg, rgba(31,111,139,.28), rgba(22,79,102,.28)) !important; }
#root > div[style*="135deg,#b23a6b"] { background: linear-gradient(135deg, rgba(178,58,107,.30), rgba(140,44,86,.30)) !important; }
#root [style*="background:#31386b"] { background: rgba(49,56,107,.32) !important; }
#root [style*="background:#1c6883"] { background: rgba(28,104,131,.28) !important; }
#root [style*="background:#a53865"] { background: rgba(165,56,101,.30) !important; }

/* Legibilidad del texto sobre el vidrio */
#root { text-shadow: 0 1px 10px rgba(0,0,0,.22); }

/* Animacion de entrada estilo popover */
@keyframes cuckooPop {
  from { opacity: 0; transform: scale(.92) translateY(-10px); }
  to   { opacity: 1; transform: scale(1) translateY(0); }
}
html.cuckoo-pop #root { animation: cuckooPop .28s cubic-bezier(.3, 1.4, .6, 1); transform-origin: 50% 0; }
"#;

/// JS que replays la animacion pop cada vez que la ventana aparece.
const POP_JS: &str = "document.documentElement.classList.remove('cuckoo-pop'); void document.documentElement.offsetWidth; document.documentElement.classList.add('cuckoo-pop');";

fn inject_js() -> String {
    format!(
        r#"(function() {{
  if (!document.getElementById('cuckoo-glass')) {{
    var s = document.createElement('style');
    s.id = 'cuckoo-glass';
    s.textContent = {css:?};
    document.head.appendChild(s);
  }}
}})();"#,
        css = GLASS_CSS
    )
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_positioner::init())
        .on_page_load(|webview, payload| {
            // Inyecta los estilos glass cuando el sitio termina de cargar
            if let PageLoadEvent::Finished = payload.event() {
                let _ = webview.eval(&inject_js());
            }
        })
        .setup(|app| {
            // Solo barra de menu: sin icono en el Dock ni en Cmd+Tab
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            // Icono de la barra de menu (a color, usa tu icono cuckoo)
            let tray_icon = Image::from_bytes(include_bytes!("../icons/tray.png"))?;

            TrayIconBuilder::with_id("main-tray")
                .icon(tray_icon)
                .icon_as_template(false)
                .on_tray_icon_event(|tray, event| {
                    let app = tray.app_handle();
                    // Le avisa al plugin donde esta el icono del tray
                    tauri_plugin_positioner::on_tray_event(app, &event);

                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        if let Some(win) = app.get_webview_window("main") {
                            if win.is_visible().unwrap_or(false) {
                                let _ = win.hide();
                            } else {
                                // Ancla la ventana justo debajo del icono, como popover nativo
                                let _ = win
                                    .as_ref()
                                    .window()
                                    .move_window(Position::TrayBottomCenter);
                                let _ = win.show();
                                let _ = win.set_focus();
                                // Reproduce la animacion de entrada
                                let _ = win.eval(POP_JS);
                            }
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            // Se oculta al hacer clic fuera, igual que el widget del clima
            if let tauri::WindowEvent::Focused(false) = event {
                let _ = window.hide();
            }
        })
        .run(tauri::generate_context!())
        .expect("error al iniciar la app");
}
