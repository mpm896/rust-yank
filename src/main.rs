//! Program to mimic the Linux highlight-to-copy and middle click-to-paste feature on Mac, without overwriting the primary copy-paste buffer
//! Disclaimer: This uses some MacOS interop to start NSApplication and subscribes to the NSWorkspaceDidActivateApplicationNotification
//! Credit to Dmitry Malkov: https://github.com/dimusic/active-window-macos-example
//! 
use std::{thread, time};

use active_win_pos_rs::{ActiveWindow, get_active_window};
use device_query::{DeviceQuery, DeviceState, DeviceEvents, MouseState};
use get_selected_text::get_selected_text;
use cocoa::appkit::NSApplication;
use cocoa::base::nil;
use objc::declare::ClassDecl;
use objc::rc::StrongPtr;
use objc::runtime::{Class, Object, Sel};
use objc::*;


unsafe extern "C" {
    static NSWorkspaceDidActivateApplicationNotification: *mut Object;
}

extern "C" fn application_did_finish_launching(this: &mut Object, _sel: Sel, _notif: *mut Object) {
    println!("Application did finish launching");

    unsafe {
        let workspace: *mut Object = msg_send![class!(NSWorkspace), sharedWorkspace];
        let notification_center: *mut Object = msg_send![workspace, notificationCenter];

        let _: *mut Object = msg_send![notification_center, addObserver:this as *mut Object
                    selector:sel!(workspace_app_activated:)
                        name:NSWorkspaceDidActivateApplicationNotification
                        object:nil];
    };

    let device_state = DeviceState::new();
        
    let interval = time::Duration::from_millis(10);
    let mut left_click = device_state.query_pointer().button_pressed[1];
    let mut text: String = String::new();
    let mut old_text: String = String::new();

    loop {
        left_click = device_state.query_pointer().button_pressed[1];  // bool if left click is pressed

        while left_click {
            left_click = device_state.query_pointer().button_pressed[1];
            if let Ok(selection) = get_selected_text() {
                text = selection;
            }
        }

        if !old_text.eq(&text) && text.len() > 0 {
            println!("Selected text: {text}");
            old_text = text.clone();
            println!("Text: {text}");
        }

        thread::sleep(interval);
    }
    // thread::spawn(move || loop {
    //     left_click = device_state.query_pointer().button_pressed[1];  // bool if left click is pressed

    //     match get_active_window() {
    //         Ok(window) => {
    //             // println!("Active window: {:?}", window);
    //             continue
    //         }
    //         Err(_e) => {
    //             println!("No active window");
    //         }
    //     };

    //     println!("Hello");

    //     while left_click {
    //         left_click = device_state.query_pointer().button_pressed[1];
    //         if let Ok(selection) = get_selected_text() {
    //             text = selection;
    //         }
    //         println!("Left clicking");
    //     }

    //     if !old_text.eq(&text) && text.len() > 0 {
    //         println!("Selected text: {text}");
    //         old_text = text.clone();
    //         println!("Text: {text}");
    //     }

    //     thread::sleep(interval);
    // });
}

extern "C" fn handle_workspace_app_activated(_this: &mut Object, _sel: Sel, _notif: *mut Object) {
    // Could be empty; Only needed to subscribe to the workspace_app_activated event

    println!("App activated");
}

fn init_app_delegate_class() -> &'static Class {
    let mut decl = ClassDecl::new("AppDelegate", class!(NSObject)).unwrap();

    unsafe {
        decl.add_method(
            sel!(applicationDidFinishLaunching:),
            application_did_finish_launching as extern "C" fn(&mut Object, Sel, *mut Object),
        );
        decl.add_method(
            sel!(workspace_app_activated:),
            handle_workspace_app_activated as extern "C" fn(&mut Object, Sel, *mut Object),
        );

        decl.register()
    }
}

fn main() {
    unsafe {
        let cls = init_app_delegate_class();
        let app = NSApplication::sharedApplication(nil);

        let delegate: *mut Object = msg_send![cls, alloc];
        let delegate: *mut Object = msg_send![delegate, init];
        let delegate = StrongPtr::new(delegate);

        let _: () = msg_send![app, setDelegate: delegate];
        let _: () = msg_send![app, run];

        app.run();
    }
}

