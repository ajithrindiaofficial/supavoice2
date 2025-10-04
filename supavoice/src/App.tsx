import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Titlebar } from "@/components/Titlebar";
import { Layout } from "@/components/Layout";
import Settings from "@/pages/Settings";
import { configureOverlayWindow } from "@/lib/window";
import { Mic, Settings as SettingsIcon, Copy, Check, Loader2 } from "lucide-react";
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
  const [recordingTime, setRecordingTime] = useState(0);
  const [copied, setCopied] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [isFormatting, setIsFormatting] = useState(false);
  const [formattedText, setFormattedText] = useState<string>("");

  useEffect(() => {
    // Configure window for overlay behavior when app starts
    configureOverlayWindow();
  }, []);

  useEffect(() => {
    let interval: ReturnType<typeof setInterval> | undefined;
    if (isRecording) {
      setRecordingTime(0);
      interval = setInterval(() => {
        setRecordingTime((t) => t + 1);
      }, 1000);
    }
    return () => {
      if (interval) clearInterval(interval);
    };
  }, [isRecording]);

  const handleRecord = async () => {
    try {
      if (isRecording) {
        // Stop recording
        const audioPath = await invoke<string>("stop_recording");
        setIsRecording(false);
        setIsTranscribing(true);

        // Transcribe the audio
        const result = await invoke<string>("transcribe_audio", {
          audioPath,
        });

        setTranscript(result);
        setIsTranscribing(false);
      } else {
        // Start recording
        setIsRecording(true);
        setTranscript("");
        setError(null);
        await invoke("start_recording_toggle");
      }
    } catch (error) {
      console.error("Recording/transcription failed:", error);
      setIsRecording(false);
      setIsTranscribing(false);
      setError(error as string);
    }
  };

  const handleCopy = async () => {
    const textToCopy = formattedText || transcript;
    if (textToCopy) {
      await navigator.clipboard.writeText(textToCopy);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  const handleFormat = async (formatType: 'email' | 'notes') => {
    if (!transcript) return;

    try {
      setIsFormatting(true);
      setError(null);
      const result = await invoke<string>("format_transcript", {
        transcript,
        formatType,
      });
      setFormattedText(result);
    } catch (error) {
      console.error("Formatting failed:", error);
      setError(error as string);
    } finally {
      setIsFormatting(false);
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
                {/* <h1 className="text-2xl font-bold mb-6">Supavoice</h1> */}

                <button
                  onClick={handleRecord}
                  disabled={isTranscribing}
                  className={`w-24 h-24 rounded-full flex items-center justify-center transition-all ${
                    isRecording
                      ? "bg-red-500 animate-pulse"
                      : isTranscribing
                      ? "bg-blue-500"
                      : "bg-primary hover:bg-primary/90 disabled:opacity-50"
                  }`}
                >
                  {isTranscribing ? (
                    <Loader2 className="h-10 w-10 text-white animate-spin" />
                  ) : (
                    <Mic className="h-10 w-10 text-white" />
                  )}
                </button>

                <p className="mt-4 text-sm text-muted-foreground">
                  {isRecording
                    ? `Recording... ${recordingTime}s (click to stop)`
                    : isTranscribing
                    ? "Transcribing with Whisper..."
                    : "Click to start recording"}
                </p>

                {error && (
                  <div className="mt-4 w-full max-w-md p-3 bg-red-100 border border-red-300 rounded-lg">
                    <p className="text-sm text-red-800">{error}</p>
                  </div>
                )}

                {transcript && (
                  <div className="mt-8 w-full max-w-md">
                    <div className="flex justify-between items-center mb-2">
                      <h3 className="text-sm font-semibold">
                        {formattedText ? "Formatted:" : "Transcript:"}
                      </h3>
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={handleCopy}
                        className="gap-2"
                      >
                        {copied ? (
                          <>
                            <Check className="h-3 w-3" />
                            Copied!
                          </>
                        ) : (
                          <>
                            <Copy className="h-3 w-3" />
                            Copy
                          </>
                        )}
                      </Button>
                    </div>
                    <div className="p-4 bg-muted rounded-lg">
                      <p className="text-sm whitespace-pre-wrap">
                        {formattedText || transcript}
                      </p>
                    </div>

                    {formattedText && (
                      <div className="mt-2 text-xs text-muted-foreground flex justify-between items-center">
                        <span>Original transcript preserved</span>
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => setFormattedText("")}
                          className="text-xs h-6"
                        >
                          Show Original
                        </Button>
                      </div>
                    )}

                    <div className="mt-4 flex gap-2">
                      <Button
                        variant="secondary"
                        size="sm"
                        onClick={() => handleFormat('email')}
                        disabled={isFormatting}
                        className="flex-1"
                      >
                        {isFormatting ? (
                          <>
                            <Loader2 className="h-3 w-3 mr-2 animate-spin" />
                            Formatting...
                          </>
                        ) : (
                          "Format as Email"
                        )}
                      </Button>
                      <Button
                        variant="secondary"
                        size="sm"
                        onClick={() => handleFormat('notes')}
                        disabled={isFormatting}
                        className="flex-1"
                      >
                        {isFormatting ? (
                          <>
                            <Loader2 className="h-3 w-3 mr-2 animate-spin" />
                            Formatting...
                          </>
                        ) : (
                          "Format as Notes"
                        )}
                      </Button>
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