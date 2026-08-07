#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{thread, time::Duration};

use tauri::{
    image::Image,
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    webview::PageLoadEvent,
    Listener, Manager,
};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};
use tauri_plugin_global_shortcut::ShortcutState;
use tauri_plugin_positioner::{Position, WindowExt};

/// Estilos "glass": fondo transparente para ver el vidrio nativo de macOS,
/// tintes translucidos, contenido compacto (zoom) para que nada quede
/// encimado, y animaciones de entrada/salida estilo popover.
const GLASS_CSS: &str = r#"
html, body, #root { background: transparent !important; }
#root { zoom: 0.8; }
#root > div {
  border-radius: 27px;
  box-shadow:
    inset 0 1px 0 rgba(255,255,255,.34),
    inset 0 0 0 1px rgba(255,255,255,.15),
    inset 0 -18px 40px rgba(255,255,255,.05);
}
::-webkit-scrollbar { width: 0; height: 0; }

/* ===== Tema VIOLETA -> tintes translucidos ===== */
#root > div[style*="135deg,#5366da"] { background: linear-gradient(135deg, rgba(83,102,218,.16), rgba(112,96,237,.16)) !important; }
#root > div[style*="135deg,#02bde8"] { background: linear-gradient(135deg, rgba(2,189,232,.13), rgba(8,174,231,.13)) !important; }
#root > div[style*="135deg,#ff2b96"] { background: linear-gradient(135deg, rgba(255,43,150,.17), rgba(255,37,134,.17)) !important; }
#root [style*="background:#6460ec"] { background: rgba(100,96,236,.17) !important; }
#root [style*="background:#06b8ed"] { background: rgba(6,184,237,.15) !important; }
#root [style*="background:#ff278f"] { background: rgba(255,39,143,.17) !important; }

/* ===== Tema BOSQUE -> tintes translucidos ===== */
#root > div[style*="135deg,#1f8a5f"] { background: linear-gradient(135deg, rgba(31,138,95,.15), rgba(47,174,122,.15)) !important; }
#root > div[style*="135deg,#12b3a8"] { background: linear-gradient(135deg, rgba(18,179,168,.13), rgba(14,143,138,.13)) !important; }
#root > div[style*="135deg,#e8a23c"] { background: linear-gradient(135deg, rgba(232,162,60,.17), rgba(217,123,43,.17)) !important; }
#root [style*="background:#279368"] { background: rgba(39,147,104,.17) !important; }
#root [style*="background:#0f9c92"] { background: rgba(15,156,146,.15) !important; }
#root [style*="background:#dd8a30"] { background: rgba(221,138,48,.17) !important; }

/* ===== Tema MEDIANOCHE -> tintes translucidos ===== */
#root > div[style*="135deg,#242b52"] { background: linear-gradient(135deg, rgba(36,43,82,.19), rgba(58,61,122,.19)) !important; }
#root > div[style*="135deg,#1f6f8b"] { background: linear-gradient(135deg, rgba(31,111,139,.16), rgba(22,79,102,.16)) !important; }
#root > div[style*="135deg,#b23a6b"] { background: linear-gradient(135deg, rgba(178,58,107,.17), rgba(140,44,86,.17)) !important; }
#root [style*="background:#31386b"] { background: rgba(49,56,107,.19) !important; }
#root [style*="background:#1c6883"] { background: rgba(28,104,131,.16) !important; }
#root [style*="background:#a53865"] { background: rgba(165,56,101,.17) !important; }

/* Legibilidad del texto sobre el vidrio */
#root { text-shadow: 0 1px 12px rgba(0,0,0,.38); }

/* Animacion de entrada estilo popover, con rebote suave */
@keyframes cuckooPop {
  0%   { opacity: 0; transform: scale(.9) translateY(-14px); }
  60%  { opacity: 1; transform: scale(1.015) translateY(2px); }
  100% { opacity: 1; transform: scale(1) translateY(0); }
}
html.cuckoo-pop #root { animation: cuckooPop .34s cubic-bezier(.3,1.3,.5,1); transform-origin: 50% 0; }

/* Animacion de salida */
@keyframes cuckooOut {
  from { opacity: 1; transform: scale(1) translateY(0); }
  to   { opacity: 0; transform: scale(.94) translateY(-10px); }
}
html.cuckoo-out #root { animation: cuckooOut .15s ease-in forwards; transform-origin: 50% 0; }
"#;

/// JS que reproduce la animacion pop cada vez que la ventana aparece.
const POP_JS: &str = "document.documentElement.classList.remove('cuckoo-out'); document.documentElement.classList.remove('cuckoo-pop'); void document.documentElement.offsetWidth; document.documentElement.classList.add('cuckoo-pop');";

/// JS de la animacion de salida.
const OUT_JS: &str = "document.documentElement.classList.remove('cuckoo-pop'); document.documentElement.classList.add('cuckoo-out');";

fn inject_js() -> String {
    format!(
        r#"(function() {{
  if (!document.getElementById('cuckoo-glass')) {{
    var s = document.createElement('style');
    s.id = 'cuckoo-glass';
    s.textContent = {css:?};
    document.head.appendChild(s);
  }}
  if (!window.__cuckooTickTimer) {{
    window.__cuckooTickTimer = setInterval(function () {{
      var d = document.getElementById('digits');
      var t = d && d.textContent ? d.textContent.trim() : '';
      var n = window.__cuckooConectados || 0;
      var title = t;
      if (n > 0) {{ title = (t ? t + '  ' : '') + '🟢 ' + n; }}
      if (title !== window.__cuckooLast) {{
        window.__cuckooLast = title;
        if (window.__TAURI__ && window.__TAURI__.event) {{ window.__TAURI__.event.emit('cuckoo-tick', title); }}
      }}
    }}, 200);
  }}
}})();"#,
        css = GLASS_CSS
    )
}

/// Muestra u oculta la tarjeta, con animacion en ambos sentidos.
fn toggle_window(app: &tauri::AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        if win.is_visible().unwrap_or(false) {
            let _ = win.eval(OUT_JS);
            let w = win.clone();
            thread::spawn(move || {
                thread::sleep(Duration::from_millis(150));
                let _ = w.hide();
            });
        } else {
            // Ancla la tarjeta justo debajo del icono, como popover nativo
            let _ = win.as_ref().window().move_window(Position::TrayBottomCenter);
            let _ = win.show();
            let _ = win.set_focus();
            let _ = win.eval(POP_JS);
        }
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, None))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_shortcuts(["CmdOrCtrl+Shift+C"])
                .expect("atajo global invalido")
                .with_handler(|app, _shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        toggle_window(app);
                    }
                })
                .build(),
        )
        .on_page_load(|webview, payload| {
            // Inyecta los estilos glass y el reporte del tiempo al terminar de cargar
            if let PageLoadEvent::Finished = payload.event() {
                let _ = webview.eval(&inject_js());
            }
        })
        .setup(|app| {
            // Solo barra de menu: sin icono en el Dock ni en Cmd+Tab
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            // Abrir sola al encender la Mac
            let _ = app.autolaunch().enable();

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
                        toggle_window(app);
                    }
                })
                .build(app)?;

            // Tiempo restante junto al icono del cuckoo en la barra de menu
            let handle = app.handle().clone();
            app.listen("cuckoo-tick", move |event| {
                let txt: String = serde_json::from_str(event.payload()).unwrap_or_default();
                if let Some(tray) = handle.tray_by_id("main-tray") {
                    if txt.is_empty() {
                        let _ = tray.set_title(None::<&str>);
                    } else {
                        let _ = tray.set_title(Some(format!(" {txt}")));
                    }
                }
            });

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
