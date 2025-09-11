use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    Emitter, Manager,
};

/// 设置系统托盘
pub fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    // 创建菜单项
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "设置", true, None::<&str>)?;
    let show = MenuItem::with_id(app, "show", "显示", true, None::<&str>)?;
    let hide = MenuItem::with_id(app, "hide", "隐藏", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[
            &show,
            &hide,
            &PredefinedMenuItem::separator(app)?,
            &settings,
            &PredefinedMenuItem::separator(app)?,
            &quit,
        ],
    )?;

    // 直接使用原始字节数据创建图标
    let icon_bytes = include_bytes!("../../icons/icon.ico");
    let icon = tauri::image::Image::from_bytes(icon_bytes)?;

    let _tray = TrayIconBuilder::with_id("main-tray")
        .icon(icon)
        .menu(&menu)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "quit" => {
                std::process::exit(0);
            }
            "settings" => {
                let window = app.get_webview_window("main").unwrap();
                window.show().unwrap();
                window.set_focus().unwrap();
                window.emit("open-settings", {}).unwrap();
            }
            "show" => {
                let window = app.get_webview_window("main").unwrap();
                window.show().unwrap();
                window.set_focus().unwrap();
            }
            "hide" => {
                let window = app.get_webview_window("main").unwrap();
                window.hide().unwrap();
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: tauri::tray::MouseButton::Left,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let is_visible = window.is_visible().unwrap_or(false);
                    if is_visible {
                        let _ = window.hide();
                    } else {
                        let _ = window.show();
                        // 不要设置焦点，避免窗口获得焦点而显示在前台
                        // let _ = window.set_focus();

                        // 设置窗口到桌面层级
                        #[cfg(target_os = "windows")]
                        {
                            use std::ffi::c_void;
                            if let Ok(hwnd) = window.hwnd() {
                                unsafe {
                                    // 使用 Windows API 设置窗口层级
                                    let hwnd_ptr = hwnd.0 as *mut c_void;
                                    let user32 =
                                        libloading::Library::new("user32.dll").unwrap();
                                    let set_window_pos: libloading::Symbol<
                                        unsafe extern "system" fn(
                                            *mut c_void,
                                            *mut c_void,
                                            i32,
                                            i32,
                                            i32,
                                            i32,
                                            u32,
                                        )
                                            -> i32,
                                    > = user32.get(b"SetWindowPos").unwrap();
                                    let _set_window_long: libloading::Symbol<
                                        unsafe extern "system" fn(
                                            *mut c_void,
                                            i32,
                                            i32,
                                        )
                                            -> i32,
                                    > = user32.get(b"SetWindowLongA").unwrap();
                                    let _get_window_long: libloading::Symbol<
                                        unsafe extern "system" fn(*mut c_void, i32) -> i32,
                                    > = user32.get(b"GetWindowLongA").unwrap();

                                    // 设置窗口样式，使其不能获得焦点 - 注释掉以允许输入框获得焦点
                                    // GWL_EXSTYLE = -20, WS_EX_NOACTIVATE = 0x08000000
                                    // let ex_style = get_window_long(hwnd_ptr, -20);
                                    // set_window_long(hwnd_ptr, -20, ex_style | 0x08000000);

                                    // 设置窗口位置到最底层
                                    // HWND_BOTTOM = 1, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE = 0x0013
                                    set_window_pos(
                                        hwnd_ptr,
                                        1 as *mut c_void,
                                        0,
                                        0,
                                        0,
                                        0,
                                        0x0013,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        })
        .build(app)?;

    Ok(())
}

/// 设置窗口层级（Windows特定）
#[cfg(target_os = "windows")]
pub fn setup_window_layer(window: &tauri::WebviewWindow) {
    use std::ffi::c_void;
    if let Ok(hwnd) = window.hwnd() {
        unsafe {
            let hwnd_ptr = hwnd.0 as *mut c_void;
            let user32 = libloading::Library::new("user32.dll").unwrap();
            let set_window_pos: libloading::Symbol<
                unsafe extern "system" fn(
                    *mut c_void,
                    *mut c_void,
                    i32,
                    i32,
                    i32,
                    i32,
                    u32,
                ) -> i32,
            > = user32.get(b"SetWindowPos").unwrap();
            let _set_window_long: libloading::Symbol<
                unsafe extern "system" fn(*mut c_void, i32, i32) -> i32,
            > = user32.get(b"SetWindowLongA").unwrap();
            let _get_window_long: libloading::Symbol<
                unsafe extern "system" fn(*mut c_void, i32) -> i32,
            > = user32.get(b"GetWindowLongA").unwrap();

            // 设置窗口样式，使其不能获得焦点 - 注释掉以允许输入框获得焦点
            // GWL_EXSTYLE = -20, WS_EX_NOACTIVATE = 0x08000000
            // let ex_style = get_window_long(hwnd_ptr, -20);
            // set_window_long(hwnd_ptr, -20, ex_style | 0x08000000);

            // 设置窗口位置到最底层
            // HWND_BOTTOM = 1, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE = 0x0013
            set_window_pos(hwnd_ptr, 1 as *mut c_void, 0, 0, 0, 0, 0x0013);
        }
    }
}
