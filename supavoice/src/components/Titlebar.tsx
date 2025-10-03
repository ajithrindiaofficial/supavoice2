import { useState, useEffect } from "react";
import { X, MoreHorizontal } from "lucide-react";
import { Button } from "@/components/ui/button";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { listen } from "@tauri-apps/api/event";
import { Menu } from "@tauri-apps/api/menu";

export function Titlebar() {
  const [isWindowFocused, setIsWindowFocused] = useState(false);

  const menuPromise = Menu.new({
    items: [
      { id: "ctx_option1", text: "Settings" },
      { id: "ctx_option2", text: "About" },
      { id: "ctx_option3", text: "Quit" },
    ],
  });

  useEffect(() => {

    // Listen for menu events
    const unlistenPromise = listen<string>("menu-event", (event) => {
      console.log("Menu event received:", event.payload);
      switch (event.payload) {
        case "ctx_option1":
          console.log("Settings clicked");
          alert("Settings clicked!");
          break;
        case "ctx_option2":
          console.log("About clicked");
          alert("About clicked!");
          break;
        case "ctx_option3":
          console.log("Quit clicked");
          alert("Quit clicked!");
          break;
        default:
          console.log("Unknown menu item:", event.payload);
      }
    });

    const handleMouseMove = () => {
      // Simplified - no button show/hide logic for now
    };

    const handleFocus = () => {
      setIsWindowFocused(true);
    };

    const handleBlur = () => {
      setIsWindowFocused(false);
    };

    // Add event listeners
    document.addEventListener('mousemove', handleMouseMove);
    window.addEventListener('focus', handleFocus);
    window.addEventListener('blur', handleBlur);

    return () => {
      document.removeEventListener('mousemove', handleMouseMove);
      window.removeEventListener('focus', handleFocus);
      window.removeEventListener('blur', handleBlur);
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, [isWindowFocused]);

  const handleClose = async () => {
    try {
      const window = getCurrentWebviewWindow();
      await window.hide();
    } catch (error) {
      console.error("Failed to hide window:", error);
    }
  };

  const handleMoreOptions = async (event: React.MouseEvent) => {
    console.log("More options button clicked!");
    event.preventDefault();
    event.stopPropagation();
    
    try {
      const menu = await menuPromise;
      await menu.popup();
    } catch (error) {
      console.error("Failed to show context menu:", error);
      alert("Menu error: " + error); // Show error if menu fails
    }
  };

  const handleDragStart = async (event: React.MouseEvent) => {
    // Manual drag implementation as fallback
    try {
      event.preventDefault();
      const window = getCurrentWebviewWindow();
      await window.startDragging();
    } catch (error) {
      console.error("Failed to start dragging:", error);
    }
  };

  return (
    <div 
      className="flex items-center justify-between h-12 px-4 select-none cursor-move"
      data-tauri-drag-region
      onMouseDown={handleDragStart}
    >
      {/* Left side - Close button */}
      <div className="flex items-center z-10 relative" style={{ pointerEvents: 'auto' }}>
        <Button
          variant="ghost"
          size="icon"
          className="h-8 w-8 hover:bg-destructive hover:text-destructive-foreground cursor-pointer opacity-100"
          onClick={handleClose}
          onMouseDown={(e) => e.stopPropagation()}
        >
          <X className="h-4 w-4" />
        </Button>
      </div>

      {/* Center - Title */}
      <div 
        className="flex-1 flex items-center justify-center"
        data-tauri-drag-region
      >
        <h1 
          className="text-lg font-semibold text-foreground pointer-events-none"
          data-tauri-drag-region
        >
          ajith-macos-starter
        </h1>
      </div>

      {/* Right side - More options button */}
      <div className="flex items-center z-50 relative" style={{ pointerEvents: 'auto' }}>
        <Button
          variant="ghost"
          size="icon"
          className="h-8 w-8 cursor-pointer opacity-100"
          onClick={handleMoreOptions}
          onMouseDown={(e) => e.stopPropagation()}
        >
          <MoreHorizontal className="h-4 w-4" />
        </Button>
      </div>
    </div>
  );
}