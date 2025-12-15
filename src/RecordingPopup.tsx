import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Square, X } from "lucide-react";
import { useEffect, useState } from "react";
import "./RecordingPopup.css";

function RecordingPopup() {
  const [recording, setRecording] = useState(true);
  const [transcribing, setTranscribing] = useState(false);

  useEffect(() => {
    // Set up event listeners
    const setupListeners = async () => {
      const unlistenStart = await listen("recording-started", () => {
        console.log("[Popup] Recording started");
        setRecording(true);
        setTranscribing(false);
      });

      const unlistenTranscribing = await listen("recording-transcribing", () => {
        console.log("[Popup] Recording transcribing");
        setRecording(false);
        setTranscribing(true);

      });

      const unlistenStop = await listen("recording-stopped", () => {
        console.log("[Popup] Recording stopped");
        setRecording(true);
        setTranscribing(false);
      });

      const unlistenCancelled = await listen("recording-cancelled", () => {
        console.log("[Popup] Recording cancelled");
        setRecording(true);
        setTranscribing(false);
      });

      return () => {
        unlistenStart();
        unlistenTranscribing();
        unlistenStop();
        unlistenCancelled();
      };
    };

    let cleanup: (() => void) | undefined;
    setupListeners().then((cleanupFn) => {
      cleanup = cleanupFn;
    });

    return () => {
      if (cleanup) cleanup();
    };
  }, []);

  const handleCancel = async () => {
    console.log("Cancel clicked");
    try {
      await invoke("cancel_recording");
    } catch (error) {
      console.error("Failed to cancel recording:", error);
    }
  };

  const handleStop = async () => {
    console.log("Stop recording clicked");
    try {
      await invoke("stop_recording");
    } catch (error) {
      console.error("Failed to stop recording:", error);
    }
  };

  return (
    <div className="w-screen h-screen rounded-2xl p-[1px] border-[2px] border-gray-600 bg-gray-800 overflow-hidden font-sans">

      { recording &&
        <div className="flex flex-col w-full h-full justify-between items-center py-3 px-4">
          {/* Top - Audio Level Indicator */}
          <div className="flex text-gray-400 justify-center items-center text-xl font-bold">
            ▄ ▆ ▄
          </div>

          {/* Bottom - Button Row */}
          <div className="flex gap-[10px]">
            {/* Cancel Button */}
            <button
              onClick={handleCancel}
              className="w-[35px] h-[35px] rounded-lg shrink-0 bg-gray-700 hover:bg-gray-600 flex items-center justify-center transition-colors cursor-pointer"
            >
              <X className="w-5 h-5 text-white" strokeWidth={2.5} />
            </button>

            {/* Stop Recording Button */}
            <button
              onClick={handleStop}
              className="w-[35px] h-[35px] rounded-lg shrink-0 bg-red-500 hover:bg-red-600 flex items-center justify-center transition-colors cursor-pointer"
            >
              <Square className="w-4 h-4 text-white" fill="white" strokeWidth={0} />
            </button>
          </div>
        </div>
      }

      { transcribing &&
          <div className="flex w-full h-full text-gray-500 justify-center items-center text-base">
          ━━━━━━
          </div>
      }
    </div>
  );
}

export default RecordingPopup;
