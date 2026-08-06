#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{
    image::Image,
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};
use tauri_plugin_positioner::{Position, WindowExt};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_positioner::init())
        .setup(|app| {
            // Solo barra de menú: sin ícono en el Dock ni en Cmd+Tab
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            // Ícono de la barra de menú (a color, usa tu ícono cuckoo)
            let tray_icon = Image::from_bytes(include_bytes!("../icons/tray.png"))?;

            TrayIconBuilder::with_id("main-tray")
                .icon(tray_icon)
                .icon_as_template(false)
                .on_tray_icon_event(|tray, event| {
                    let app = tray.app_handle();
                    // Le avisa al plugin dónde está el ícono del tray
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
                                // Ancla la ventana justo debajo del ícono, como popover nativo
                                let _ = win
                                    .as_ref()
                                    .window()
                                    .move_window(Position::TrayBottomCenter);
                                let _ = win.show();
                                let _ = win.set_focus();
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
