//#![allow(unused)]

use tao::{
    event::Event,
    event_loop::{ControlFlow, EventLoopBuilder},
};
use tray_icon::{
    TrayIconBuilder, TrayIconEvent,
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
};

enum UserEvent {
    TrayIconEvent(tray_icon::TrayIconEvent),
    MenuEvent(tray_icon::menu::MenuEvent),
}

use async_std::task;
use std::process::Command;

#[tokio::main]
async fn main() {
    //define Tray Icon .
    let path = "./assets/toolbox.png";

    let mut tray_icon = None;

    let _menu_channel = MenuEvent::receiver();
    let _tray_channel = TrayIconEvent::receiver();

    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();

    // set a tray event handler that forwards the event and wakes up the event loop
    let proxy = event_loop.create_proxy();
    TrayIconEvent::set_event_handler(Some(move |event| {
        let _ = proxy.send_event(UserEvent::TrayIconEvent(event));
    }));

    // set a menu event handler that forwards the event and wakes up the event loop
    let proxy = event_loop.create_proxy();
    MenuEvent::set_event_handler(Some(move |event| {
        let _ = proxy.send_event(UserEvent::MenuEvent(event));
    }));

    // Create Menu
    let tray_menu = Menu::new();

    // Define Menu text
    let txt_lock = " Lock Screen";
    let txt_suspend = "󰒲 Suspend";
    let txt_tf = "󰠝 Thinkfan";
    let txt_tf_act = "󰈐 Thinkfan";
    let txt_quit = " Quit";

    // define MenuItems
    let lock_i = MenuItem::new(txt_lock, true, None);
    let suspend_i = MenuItem::new(txt_suspend, true, None);
    let quit_i = MenuItem::new(txt_quit, true, None);
    let tgl_tf_i = MenuItem::new(txt_tf, true, None);

    // Assemble TrayMenu
    let _ = tray_menu.append_items(&[
        &lock_i,
        &suspend_i,
        &PredefinedMenuItem::separator(),
        &tgl_tf_i,
        &PredefinedMenuItem::separator(),
        &quit_i,
    ]);

    // Check if thinkfan is already running -> Set Icon
    let service_name = "thinkfan";
    if check_service(service_name) {
        println!("Service {} is running!.", service_name);
        MenuItem::set_text(&tgl_tf_i, txt_tf_act);
    } else {
        println!("Service {} not running!", service_name);
        MenuItem::set_text(&tgl_tf_i, txt_tf);
    }

    // define EventLoop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::NewEvents(tao::event::StartCause::Init) => {
                let icon = load_icon(std::path::Path::new(path));

                // We create the icon once the event loop is actually running
                // to prevent issues like https://github.com/tauri-apps/tray-icon/issues/90
                tray_icon = Some(
                    TrayIconBuilder::new()
                        .with_menu(Box::new(tray_menu.clone()))
                        .with_tooltip("Sleep Timer")
                        .with_icon(icon)
                        .build()
                        .unwrap(),
                );
            }

            Event::UserEvent(UserEvent::TrayIconEvent(event)) => {
                println!("{event:?}");
            }

            Event::UserEvent(UserEvent::MenuEvent(event)) => {
                println!("{event:?}");

                if let Ok(event) = MenuEvent::receiver().try_recv() {
                    println!("menu event: {:?}", event);
                }

                // "Lock" Button
                if event.id == lock_i.id() {
                    tray_icon.take();
                    task::block_on(fn_lock());
                }

                // "Suspend" Button
                if event.id == suspend_i.id() {
                    tray_icon.take();

                    task::spawn(async {
                        fn_pause_media().await;
                    });
                    task::spawn(async {
                        fn_lock().await;
                    });
                    task::spawn(async {
                        fn_suspend().await;
                    });
                }

                // "Thinkfan" Button
                if event.id == tgl_tf_i.id() {
                    tray_icon.take();

                    // Start the Thinkfan service
                    let result = fn_start_thinkfan();

                    if result {
                        // If the command was successful, update the menu items
                        let service_name = "thinkfan";
                        if check_service(service_name) {
                            println!("Service {} is running!.", service_name);
                            MenuItem::set_text(&tgl_tf_i, txt_tf_act);
                        } else {
                            println!("Service {} not running!", service_name);
                            MenuItem::set_text(&tgl_tf_i, txt_tf);
                        }
                    } else {
                        // Handle failure case, if necessary
                    }
                }

                // "Quit" Button
                if event.id == quit_i.id() {
                    tray_icon.take();
                    *control_flow = ControlFlow::Exit;
                }
            }
            _ => {}
        }
    })
}

fn load_icon(path: &std::path::Path) -> tray_icon::Icon {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open(path)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    tray_icon::Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to open icon")
}

async fn fn_pause_media() {
    Command::new("playerctl")
        .arg("pause")
        .output()
        .expect("failed to execute process");
}

async fn fn_lock() {
    Command::new("hyprlock")
        .arg("-q")
        .output()
        .expect("failed to execute process");
}

async fn fn_suspend() {
    Command::new("systemctl")
        .arg("suspend")
        .output()
        .expect("failed to execute process");
}

fn fn_start_thinkfan() -> bool {
    let service_name = "thinkfan";
    let status;

    if check_service(service_name) {
        println!("Service {} running!", service_name);
        status = Command::new("pkexec")
            .arg("systemctl")
            .arg("stop")
            .arg(service_name)
            .output();
    } else {
        println!("Service {} not running!", service_name);
        status = Command::new("pkexec")
            .arg("systemctl")
            .arg("start")
            .arg(service_name)
            .output();
    }

    match status {
        Ok(output) if output.status.success() => {
            println!("{} toggled.", service_name);
            true
        }
        Ok(output) => {
            // Fehlerausgabe von stderr einlesen
            eprintln!(
                "Failed to toggle {}: {}",
                service_name,
                String::from_utf8_lossy(&output.stderr)
            );
            false
        }
        Err(err) => {
            eprintln!("Error executing command: {}", err);
            false
        }
    }
}

fn check_service(service_name: &str) -> bool {
    let output = Command::new("systemctl")
        .arg("is-active")
        .arg(service_name)
        .output();

    match output {
        Ok(output) => output.stdout == b"active\n",
        Err(_) => false,
    }
}
