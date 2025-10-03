import { useEffect, useState } from "react";
import { Titlebar } from "@/components/Titlebar";
import { Layout } from "@/components/Layout";
import Settings from "@/pages/Settings";
import { configureOverlayWindow } from "@/lib/window";

function App() {
  const [view, setView] = useState<'main' | 'settings'>('main');

  useEffect(() => {
    // Configure window for overlay behavior when app starts
    configureOverlayWindow();
  }, []);

  return (
    <div className="h-screen bg-transparent-light window-with-stroke overflow-hidden flex flex-col">
      <Titlebar />
      <div className="flex-1 overflow-hidden flex flex-col">
        <Layout>
          {view === 'main' ? (
            <div className="flex flex-col items-center justify-center h-full p-6">
              <h1 className="text-3xl font-bold mb-4">Supavoice</h1>
              <p className="text-muted-foreground mb-6">Voice transcription with AI formatting</p>
              <button
                onClick={() => setView('settings')}
                className="px-4 py-2 rounded-lg bg-primary text-primary-foreground hover:bg-primary/90"
              >
                Open Settings
              </button>
            </div>
          ) : (
            <div className="flex flex-col h-full">
              <div className="flex justify-start p-2 border-b">
                <button
                  onClick={() => setView('main')}
                  className="px-3 py-1 text-sm rounded hover:bg-gray-100"
                >
                  ← Back
                </button>
              </div>
              <Settings />
            </div>
          )}
        </Layout>
      </div>
    </div>
  );
}

export default App;