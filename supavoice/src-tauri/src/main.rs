// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod audio;
mod models;
mod transcription;

use audio::AudioRecorder;
use models::{ModelDownloader, ModelRecord, ModelRegistry};
use std::sync::Arc;
use tauri::{
    tray::{TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, State, WindowEvent,
};
use tauri_plugin_sql::{Migration, MigrationKind};
use transcription::WhisperTranscriber;

#[cfg(target_os = "macos")]
fn set_window_above_fullscreen(window: &tauri::WebviewWindow) {
    use cocoa::base::id;
    use objc::{msg_send, sel, sel_impl};
    
    unsafe {
        let ns_window = window.ns_window().unwrap() as id;
        // Use the highest possible window level
        // NSPopUpMenuWindowLevel = 101, NSScreenSaverWindowLevel = 1000
        // NSAssistiveTechHighWindowLevel = 1500 (highest system level)
        let level: i32 = 2147483647; // CGWindowLevelForKey(kCGAssistiveTechHighWindowLevelKey)
        let _: () = msg_send![ns_window, setLevel: level];
        
        // Set collection behavior for fullscreen compatibility
        let collection_behavior: u64 = 
            1 << 0 |  // NSWindowCollectionBehaviorDefault
            1 << 6 |  // NSWindowCollectionBehaviorCanJoinAllSpaces
            1 << 7 |  // NSWindowCollectionBehaviorFullScreenAuxiliary
            1 << 11;  // NSWindowCollectionBehaviorIgnoresCycle
        let _: () = msg_send![ns_window, setCollectionBehavior: collection_behavior];
        
        // Force the window to be visible on all spaces
        let _: () = msg_send![ns_window, setCanHide: false];
        let _: () = msg_send![ns_window, setHidesOnDeactivate: false];
    }
}

#[cfg(target_os = "macos")]
fn position_window_below_tray(window: &tauri::WebviewWindow, tray_icon: &tauri::tray::TrayIcon) -> Result<(), String> {
    use cocoa::base::id;
    use cocoa::foundation::{NSPoint, NSRect};
    use objc::{msg_send, sel, sel_impl, class};
    
    unsafe {
        // Get the status bar (menu bar) height - typically 24px on macOS
        let status_bar_class = class!(NSStatusBar);
        let system_status_bar: id = msg_send![status_bar_class, systemStatusBar];
        let status_bar_thickness: f64 = msg_send![system_status_bar, thickness];
        
        // Get screen dimensions
        let screen_class = class!(NSScreen);
        let main_screen: id = msg_send![screen_class, mainScreen];
        let screen_frame: NSRect = msg_send![main_screen, frame];
        
        // Get mouse cursor position as approximation for tray icon position
        let mouse_location: NSPoint = msg_send![class!(NSEvent), mouseLocation];
        
        // Calculate position - position window below the tray icon (mouse position)
        let window_width = 480.0;
        let window_height = 520.0;
        let padding_from_top = 8.0; // 8px padding from menu bar
        
        // Center the window horizontally around the tray icon position
        let x = mouse_location.x - (window_width / 2.0);
        
        // Ensure window doesn't go off screen horizontally
        let x = x.max(10.0).min(screen_frame.size.width - window_width - 10.0);
        
        let y = screen_frame.size.height - status_bar_thickness - window_height - padding_from_top;
        
        let new_origin = NSPoint::new(x, y);
        let new_size = cocoa::foundation::NSSize::new(window_width, window_height);
        let new_frame = NSRect::new(new_origin, new_size);
        
        let ns_window = window.ns_window().unwrap() as id;
        let _: () = msg_send![ns_window, setFrame:new_frame display:true];
    }
    
    Ok(())
}

#[cfg(target_os = "macos")]
fn hide_traffic_lights_keep_titlebar(window: &tauri::WebviewWindow) -> Result<(), String> {
    use cocoa::base::id;
    use objc::{msg_send, sel, sel_impl};
    use std::ffi::CStr;
    
    unsafe {
        let ns_window = window.ns_window().unwrap() as id;
        
        // Hide traffic light buttons but keep titlebar for rounded corners
        let close_button: id = msg_send![ns_window, standardWindowButton:0]; // NSWindowCloseButton = 0
        let miniaturize_button: id = msg_send![ns_window, standardWindowButton:1]; // NSWindowMiniaturizeButton = 1  
        let zoom_button: id = msg_send![ns_window, standardWindowButton:2]; // NSWindowZoomButton = 2
        
        if close_button != cocoa::base::nil {
            let _: () = msg_send![close_button, setHidden:true];
        }
        
        if miniaturize_button != cocoa::base::nil {
            let _: () = msg_send![miniaturize_button, setHidden:true];
        }
        
        if zoom_button != cocoa::base::nil {
            let _: () = msg_send![zoom_button, setHidden:true];
        }
        
        // Make the title bar transparent and hide title text
        let _: () = msg_send![ns_window, setTitlebarAppearsTransparent:true];
        let _: () = msg_send![ns_window, setTitleVisibility:1]; // NSWindowTitleHidden = 1
        
        // Allow mouse events but prevent window from becoming key
        let _: () = msg_send![ns_window, setAcceptsMouseMovedEvents:true];
        let _: () = msg_send![ns_window, setIgnoresMouseEvents:false];
    }
    
    Ok(())
}

#[cfg(target_os = "macos")]
fn enable_accepts_first_mouse(window: &tauri::WebviewWindow) -> Result<(), String> {
    use cocoa::base::id;
    use objc::{msg_send, sel, sel_impl};
    
    unsafe {
        let ns_window = window.ns_window().unwrap() as id;
        let content_view: id = msg_send![ns_window, contentView];
        
        // Try to set acceptsFirstMouse on the content view
        // This should allow clicks without focusing the window
        let _: () = msg_send![content_view, setAcceptsFirstMouse: true];
    }
    
    Ok(())
}

#[cfg(target_os = "macos")]
fn apply_native_vibrancy(window: &tauri::WebviewWindow) -> Result<(), String> {
    use cocoa::base::{id, nil};
    use cocoa::foundation::NSRect;
    use objc::{msg_send, sel, sel_impl, class};
    
    println!("Starting vibrancy application...");
    
    unsafe {
        println!("Getting NSWindow...");
        let ns_window = match window.ns_window() {
            Ok(win) => win as id,
            Err(e) => return Err(format!("Failed to get NSWindow: {}", e)),
        };
        
        println!("Setting window content view vibrancy...");
        let content_view: id = msg_send![ns_window, contentView];
        if content_view == nil {
            return Err("Content view is nil".to_string());
        }
        
        // Create a new NSVisualEffectView as the new content view
        println!("Creating NSVisualEffectView...");
        let visual_effect_view_class = class!(NSVisualEffectView);
        let visual_effect_view: id = msg_send![visual_effect_view_class, alloc];
        if visual_effect_view == nil {
            return Err("Failed to allocate NSVisualEffectView".to_string());
        }
        
        let visual_effect_view: id = msg_send![visual_effect_view, init];
        if visual_effect_view == nil {
            return Err("Failed to initialize NSVisualEffectView".to_string());
        }
        
        println!("Setting frame...");
        let frame: NSRect = msg_send![content_view, frame];
        let _: () = msg_send![visual_effect_view, setFrame: frame];
        
        println!("Setting material...");
        // Use different materials based on system appearance
        // 0 = Titlebar, 1 = Selection, 2 = Menu, 3 = Sidebar, 4 = HeaderView, 
        // 5 = Sheet, 6 = WindowBackground, 7 = HUD, 8 = FullScreenUI, 
        // 9 = Tooltip, 10 = ContentBackground, 11 = UnderWindowBackground, 12 = UnderPageBackground
        let material: i64 = 6; // WindowBackground material for better system theme integration
        let _: () = msg_send![visual_effect_view, setMaterial: material];
        
        println!("Setting blend mode...");
        let blend_mode: i64 = 0;
        let _: () = msg_send![visual_effect_view, setBlendingMode: blend_mode];
        
        println!("Setting state...");
        let state: i64 = 1;
        let _: () = msg_send![visual_effect_view, setState: state];
        
        println!("Setting autoresizing mask...");
        let autoresizing_mask: u64 = 1 << 1 | 1 << 4; // NSViewWidthSizable | NSViewHeightSizable  
        let _: () = msg_send![visual_effect_view, setAutoresizingMask: autoresizing_mask];
        
        // Move the existing content view to be a child of the visual effect view
        println!("Moving existing content to vibrancy view...");
        let _: () = msg_send![visual_effect_view, addSubview: content_view];
        
        // Set the visual effect view as the new content view
        println!("Setting vibrancy view as content view...");
        let _: () = msg_send![ns_window, setContentView: visual_effect_view];
        
        println!("Native vibrancy applied successfully");
        Ok(())
    }
}

#[cfg(not(target_os = "macos"))]
fn set_window_above_fullscreen(_window: &tauri::WebviewWindow) {
    // Platform not supported - use regular always on top
}


#[tauri::command]
async fn toggle_overlay_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("overlay") {
        if window.is_visible().unwrap_or(false) {
            window.hide().map_err(|e| e.to_string())?;
        } else {
            window.show().map_err(|e| e.to_string())?;
            window.set_focus().map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}


#[tauri::command]
fn apply_window_vibrancy(window: tauri::WebviewWindow) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        println!("Applying native vibrancy...");
        apply_native_vibrancy(&window)
    }

    #[cfg(target_os = "windows")]
    {
        Err("Windows blur not implemented".to_string())
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        Err("Vibrancy not supported on this platform".to_string())
    }
}

// App state for model management
struct AppState {
    registry: Arc<ModelRegistry>,
    downloader: Arc<ModelDownloader>,
}

#[tauri::command]
async fn list_models(state: State<'_, AppState>) -> Result<Vec<ModelRecord>, String> {
    state
        .registry
        .list_models()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn start_download(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    model_id: String,
) -> Result<(), String> {
    let downloader = state.downloader.clone();
    let model_id_clone = model_id.clone();

    // Spawn download task
    tauri::async_runtime::spawn(async move {
        if let Err(e) = downloader.download_model(model_id_clone.clone(), app.clone()).await {
            eprintln!("Download failed for {}: {}", model_id_clone, e);
            // Emit error event
            let _ = app.emit(
                "download_failed",
                serde_json::json!({
                    "model_id": model_id_clone,
                    "error": e.to_string(),
                }),
            );
        }
    });

    Ok(())
}

#[tauri::command]
async fn delete_model(state: State<'_, AppState>, model_id: String) -> Result<(), String> {
    state
        .downloader
        .delete_model(model_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_disk_space() -> Result<u64, String> {
    // Get free disk space (basic implementation)
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let output = Command::new("df")
            .arg("-k")
            .arg("/")
            .output()
            .map_err(|e| e.to_string())?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = output_str.lines().collect();
        if lines.len() >= 2 {
            let parts: Vec<&str> = lines[1].split_whitespace().collect();
            if parts.len() >= 4 {
                let free_kb: u64 = parts[3].parse().unwrap_or(0);
                return Ok(free_kb * 1024); // Convert to bytes
            }
        }
    }

    Ok(0)
}

#[tauri::command]
async fn start_recording(duration: u64) -> Result<String, String> {
    // Save to Desktop for easy access
    let desktop_dir = dirs::home_dir()
        .ok_or("Could not find home directory")?
        .join("Desktop");

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let audio_path = desktop_dir.join(format!("supavoice_recording_{}.wav", timestamp));

    println!("Recording to: {:?}", audio_path);

    let recorder = AudioRecorder::new();
    recorder
        .record_to_file(audio_path.clone(), duration)
        .map_err(|e| e.to_string())?;

    println!("Recording saved successfully!");

    Ok(audio_path.to_string_lossy().to_string())
}

#[tauri::command]
async fn transcribe_audio(state: State<'_, AppState>, audio_path: String) -> Result<String, String> {
    // Try models in order: whisper-small-en, whisper-small, whisper-base-en
    let model_id = if let Ok(model) = state.registry.get_model("whisper-small-en").await {
        if model.path.is_some() {
            "whisper-small-en"
        } else if let Ok(model) = state.registry.get_model("whisper-small").await {
            if model.path.is_some() {
                "whisper-small"
            } else {
                "whisper-base-en"
            }
        } else {
            "whisper-base-en"
        }
    } else {
        "whisper-base-en"
    };

    let model = state
        .registry
        .get_model(model_id)
        .await
        .map_err(|e| e.to_string())?;

    println!("Model found: id={}, status={:?}, path={:?}", model.id, model.status, model.path);
    let model_path = model.path.ok_or("Model not installed")?;
    println!("Using model path: {:?}", model_path);

    let transcriber = WhisperTranscriber::new(model_path)
        .map_err(|e| e.to_string())?;
    let result = transcriber
        .transcribe(&audio_path)
        .map_err(|e| e.to_string())?;

    Ok(result)
}

fn main() {
    let migrations = vec![
        Migration {
            version: 1,
            description: "create_time_entries_table",
            sql: "CREATE TABLE time_entries (
                id TEXT PRIMARY KEY NOT NULL,
                date TEXT NOT NULL,
                client TEXT NOT NULL,
                project TEXT NOT NULL,
                task TEXT,
                hours REAL NOT NULL,
                notes TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );",
            kind: MigrationKind::Up,
        }
    ];

    // Initialize model registry and downloader
    let registry = Arc::new(ModelRegistry::new().expect("Failed to initialize model registry"));
    let downloader = Arc::new(ModelDownloader::new(registry.clone()));

    let app_state = AppState {
        registry,
        downloader,
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:supavoice.db", migrations)
                .build()
        )
        .manage(app_state)
        .setup(|app| {
            // Set activation policy to Accessory on macOS to allow overlay above fullscreen apps
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            
            // Create system tray with proper icon
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("Supavoice")
                .on_tray_icon_event(|_tray, event| {
                    match event {
                        TrayIconEvent::Click { 
                            button: tauri::tray::MouseButton::Left,
                            button_state: tauri::tray::MouseButtonState::Up,
                            ..
                        } => {
                            // Toggle overlay window on LEFT mouse UP (not down)
                            if let Some(window) = _tray.app_handle().get_webview_window("overlay") {
                                if window.is_visible().unwrap_or(false) {
                                    let _ = window.hide();
                                } else {
                                    // Position window below tray before showing
                                    #[cfg(target_os = "macos")]
                                    {
                                        let _ = position_window_below_tray(&window, _tray);
                                    }
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                    let _ = window.set_always_on_top(true);
                                    set_window_above_fullscreen(&window);
                                }
                            }
                        }
                        _ => {}
                    }
                })
                .build(app)?;

            // Hide window initially - only show via tray
            if let Some(window) = app.get_webview_window("overlay") {
                // Apply vibrancy effect automatically
                #[cfg(target_os = "macos")]
                {
                    if let Err(e) = apply_native_vibrancy(&window) {
                        eprintln!("Failed to apply vibrancy: {}", e);
                    }
                    
                    // Hide traffic lights but keep titlebar for rounded corners
                    if let Err(e) = hide_traffic_lights_keep_titlebar(&window) {
                        eprintln!("Failed to hide traffic lights: {}", e);
                    }
                    
                    // TODO: Enable clicks without focusing - currently causing panics
                    // if let Err(e) = enable_accepts_first_mouse(&window) {
                    //     eprintln!("Failed to enable accepts first mouse: {}", e);
                    // }
                }
                
                // Configure window for fullscreen overlay behavior
                set_window_above_fullscreen(&window);
                window.hide().unwrap();
                
                // Handle window events
                let window_clone = window.clone();
                window.on_window_event(move |event| {
                    match event {
                        WindowEvent::CloseRequested { api, .. } => {
                            // Prevent window from closing, hide instead
                            api.prevent_close();
                            window_clone.hide().unwrap();
                        }
                        _ => {}
                    }
                });
            }

            // TODO: Add global hotkey ⌥⌘L (Option+Command+L) - API needs research
            // For now using tray click to toggle

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            toggle_overlay_window,
            apply_window_vibrancy,
            list_models,
            start_download,
            delete_model,
            get_disk_space,
            start_recording,
            transcribe_audio
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}