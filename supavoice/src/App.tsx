import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Titlebar } from "@/components/Titlebar";
import { Layout } from "@/components/Layout";
import Settings from "@/pages/Settings";
import { configureOverlayWindow } from "@/lib/window";
import { Mic, Settings as SettingsIcon } from "lucide-react";
import { Button } from "@/components/ui/button";

interface TranscriptionResult {
  text: string;
  segments: Array<{
    start: number;
    end: number;
    text: string;
  }>;
}

function App() {
  const [view, setView] = useState<'main' | 'settings'>('main');
  const [isRecording, setIsRecording] = useState(false);
  const [isTranscribing, setIsTranscribing] = useState(false);
  const [transcript, setTranscript] = useState<string>("");

  useEffect(() => {
    // Configure window for overlay behavior when app starts
    configureOverlayWindow();
  }, []);

  const handleRecord = async () => {
    try {
      setIsRecording(true);
      setTranscript("");

      // Record for 10 seconds
      const audioPath = await invoke<string>("start_recording", { duration: 10 });

      setIsRecording(false);
      setIsTranscribing(true);

      // Transcribe the audio
      const result = await invoke<TranscriptionResult>("transcribe_audio", {
        audioPath,
      });

      setTranscript(result.text);
      setIsTranscribing(false);
    } catch (error) {
      console.error("Recording/transcription failed:", error);
      setIsRecording(false);
      setIsTranscribing(false);
      alert(`Error: ${error}`);
    }
  };

  return (
    <div className="h-screen bg-transparent-light window-with-stroke overflow-hidden flex flex-col">
      <Titlebar />
      <div className="flex-1 overflow-hidden flex flex-col">
        <Layout>
          {view === 'main' ? (
            <div className="flex flex-col h-full p-6">
              <div className="flex justify-end mb-4">
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => setView('settings')}
                >
                  <SettingsIcon className="h-4 w-4" />
                </Button>
              </div>

              <div className="flex-1 flex flex-col items-center justify-center">
                <h1 className="text-2xl font-bold mb-6">Supavoice</h1>

                <button
                  onClick={handleRecord}
                  disabled={isRecording || isTranscribing}
                  className={`w-24 h-24 rounded-full flex items-center justify-center transition-all ${
                    isRecording
                      ? "bg-red-500 animate-pulse"
                      : isTranscribing
                      ? "bg-blue-500"
                      : "bg-primary hover:bg-primary/90"
                  }`}
                >
                  <Mic className="h-10 w-10 text-white" />
                </button>

                <p className="mt-4 text-sm text-muted-foreground">
                  {isRecording
                    ? "Recording... (10s)"
                    : isTranscribing
                    ? "Transcribing..."
                    : "Click to record"}
                </p>

                {transcript && (
                  <div className="mt-8 w-full max-w-md">
                    <h3 className="text-sm font-semibold mb-2">Transcript:</h3>
                    <div className="p-4 bg-muted rounded-lg">
                      <p className="text-sm">{transcript}</p>
                    </div>
                  </div>
                )}
              </div>
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