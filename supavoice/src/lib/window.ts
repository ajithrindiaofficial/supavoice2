import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

export async function configureOverlayWindow() {
  try {
    const window = getCurrentWebviewWindow();
    
    // Configure window to appear above fullscreen apps on macOS
    await window.setVisibleOnAllWorkspaces(true);
    await window.setAlwaysOnTop(true);
    
    console.log("Window configured for fullscreen overlay behavior");
  } catch (error) {
    console.error("Failed to configure overlay window:", error);
  }
}