import { Channel } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Mirage } from "ldrs/react";
import "ldrs/react/Mirage.css";
import { Square, X } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import "./RecordingPopup.css";
import {
  useCancelRecording,
  useStopRecording,
  useRetryTranscription,
  useDismissError,
  useResizePopupForError,
} from "@/hooks/useRecording";
import { commands } from "@/bindings";

// Maximum recording duration (10 minutes). Change to 10000 for 10-second testing
const MAX_RECORDING_DURATION_MS = 10 * 60 * 1000;

interface RecordingErrorPayload {
  error_type: string;
  error_message: string;
  user_message: string;
  can_retry: boolean;
  audio_file_path: string | null;
}

function RecordingPopup() {
  const [recording, setRecording] = useState(true);
  const [transcribing, setTranscribing] = useState(false);
  const [error, setError] = useState<RecordingErrorPayload | null>(null);
  const [audioLevel, setAudioLevel] = useState(0);
  const [smoothedLevel, setSmoothedLevel] = useState(0);
  const [elapsedMs, setElapsedMs] = useState(0);
  const animationFrameRef = useRef<number | undefined>(undefined);
  const startTimeRef = useRef<number | undefined>(undefined);
  const timerIntervalRef = useRef<NodeJS.Timeout | undefined>(undefined);

  // TanStack Query mutation hooks
  const cancelRecording = useCancelRecording();
  const stopRecording = useStopRecording();
  const retryTranscription = useRetryTranscription();
  const dismissError = useDismissError();
  const resizePopupForError = useResizePopupForError();

  // Smooth the audio level using requestAnimationFrame
  useEffect(() => {
    const animate = () => {
      setSmoothedLevel((current) => {
        const diff = audioLevel - current;
        // Fast response to increases, slower decay
        const speed = diff > 0 ? 0.3 : 0.15;
        return current + diff * speed;
      });
      animationFrameRef.current = requestAnimationFrame(animate);
    };

    animationFrameRef.current = requestAnimationFrame(animate);

    return () => {
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
      }
    };
  }, [audioLevel]);

  const handleCancel = async () => {
    console.log("Cancel clicked");
    try {
      await cancelRecording.mutateAsync();
    } catch (error) {
      console.error("Failed to cancel recording:", error);
    }
  };

  const handleStop = async () => {
    console.log("Stop recording clicked");
    try {
      await stopRecording.mutateAsync();
    } catch (error) {
      console.error("Failed to stop recording:", error);
    }
  };

  const handleRetry = async () => {
    console.log("Retry clicked");
    setError(null);
    setTranscribing(true);

    try {
      await retryTranscription.mutateAsync();
    } catch (err) {
      console.error("Retry failed:", err);
      // Error will be re-emitted via event
    }
  };

  const handleDismiss = async () => {
    console.log("Dismiss clicked");

    try {
      await dismissError.mutateAsync();
    } catch (err) {
      console.error("Failed to dismiss:", err);
    }
  };

  // Timer cleanup helper
  const cleanupTimer = () => {
    if (timerIntervalRef.current) {
      clearInterval(timerIntervalRef.current);
      timerIntervalRef.current = undefined;
    }
    startTimeRef.current = undefined;
    setElapsedMs(0);
  };

  // Timer update helper
  const startTimer = () => {
    console.log('[Popup] Starting countdown timer');
    // Clean up any existing timer first
    cleanupTimer();

    // Set start time
    startTimeRef.current = Date.now();

    // Initialize display to max duration
    setElapsedMs(MAX_RECORDING_DURATION_MS);

    // Start countdown interval
    timerIntervalRef.current = setInterval(() => {
      if (startTimeRef.current === undefined) return;

      const elapsed = Date.now() - startTimeRef.current;
      const remaining = Math.max(0, MAX_RECORDING_DURATION_MS - elapsed);
      setElapsedMs(remaining);

      if (remaining <= 0) {
        console.log('[Popup] Countdown reached 0, auto-stopping');
        cleanupTimer();
        handleStop();
      }
    }, 1000); // Update every 100ms for smooth display
  };

  useEffect(() => {
    // Set up audio level channel
    const setupAudioLevelChannel = async () => {
      const audioLevelChannel = new Channel<number>();

      audioLevelChannel.onmessage = (level: number) => {
        console.log("[Popup] Audio level received:", level);
        setAudioLevel(level);
      };

      try {
        const result = await commands.registerAudioLevelChannel(audioLevelChannel);
        if (result.status === 'error') {
          throw new Error(result.error);
        }
        console.log("[Popup] Audio level channel registered");
      } catch (error) {
        console.error("[Popup] Failed to register audio level channel:", error);
      }
    };

    setupAudioLevelChannel();
  }, []);

  useEffect(() => {
    // Set up event listeners
    const setupListeners = async () => {
      const unlistenStart = await listen("recording-started", () => {
        console.log("[Popup] Recording started");
        setRecording(true);
        setTranscribing(false);
        setError(null); // Clear any previous error
        startTimer(); // Start countdown timer
      });

      const unlistenTranscribing = await listen("recording-transcribing", () => {
        console.log("[Popup] Recording transcribing");
        setRecording(false);
        setTranscribing(true);
        cleanupTimer(); // Stop and reset timer
      });

      const unlistenStop = await listen("recording-stopped", () => {
        console.log("[Popup] Recording stopped");
        setRecording(true);
        setTranscribing(false);
        cleanupTimer(); // Stop and reset timer
      });

      const unlistenCancelled = await listen("recording-cancelled", () => {
        console.log("[Popup] Recording cancelled");
        setRecording(true);
        setTranscribing(false);
        cleanupTimer(); // Stop and reset timer
      });

      const unlistenError = await listen<RecordingErrorPayload>("recording-error", async (event) => {
        console.log("[Popup] Recording error:", event.payload);
        setRecording(false);
        setTranscribing(false);
        setError(event.payload);

        // Resize window for error display
        try {
          await resizePopupForError.mutateAsync();
        } catch (err) {
          console.error("Failed to resize popup:", err);
        }
      });

      return () => {
        unlistenStart();
        unlistenTranscribing();
        unlistenStop();
        unlistenCancelled();
        unlistenError();
        cleanupTimer(); // Clean up timer on unmount
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

  // Format milliseconds to MM:SS
  const formatTime = (ms: number): string => {
    const totalSeconds = Math.floor(ms / 1000);
    const minutes = Math.floor(totalSeconds / 60);
    const seconds = totalSeconds % 60;
    return `${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`;
  };

  // Calculate inset shadow based on audio level
  // Creates a white glow from the edges inward when speaking
  const getInsetShadow = (level: number) => {
    // Adjust the spread and opacity based on audio level
    const spreadSize = Math.round(level * 60); // 0-60px spread
    const opacity = level * 0.3; // 0-0.3 opacity for subtle effect
    return `inset 0 0 ${spreadSize}px rgba(255, 255, 255, ${opacity})`;
  };

  return (
    <div className="w-screen h-screen rounded-2xl border-[2px] border-gray-600 bg-gray-800 overflow-hidden font-sans">

      {/* Error State */}
      {error && (
        <div className="flex items-center justify-between w-full h-full px-4 py-3">
          {/* Error Icon and Message */}
          <div className="flex items-center gap-3 flex-1 min-w-0">
            <div className="flex-1 min-w-0">
              <div className="text-red-400 text-xs font-semibold mb-0.5">
                {error.error_type === "recording" ? "Recording Failed" : "Transcription Failed"}
              </div>
              <div className="text-gray-300 text-[10px] leading-tight truncate">
                {error.user_message}
              </div>
            </div>
          </div>

          {/* Action Buttons */}
          <div className="flex gap-2 flex-shrink-0 ml-3">
            {error.can_retry && (
              <button
                onClick={handleRetry}
                disabled={retryTranscription.isPending}
                className="h-6 px-3 text-xs rounded bg-gray-600 hover:bg-gray-500 text-white font-medium transition-colors flex items-center disabled:opacity-50"
              >
                {retryTranscription.isPending ? 'Retrying...' : 'Retry'}
              </button>
            )}
            <button
              onClick={handleDismiss}
              disabled={dismissError.isPending}
              className="w-6 h-6 rounded bg-gray-600 hover:bg-gray-500 flex items-center justify-center transition-colors disabled:opacity-50"
            >
              <X className="w-4 h-4 text-white" strokeWidth={2.5} />
            </button>
          </div>
        </div>
      )}

      {/* Transcribing State */}
      {transcribing && !error && (
        <div className="flex w-full h-full justify-center items-center">
          <Mirage
            size="60"
            speed="2.5"
            color="#9ca3af"
          />
        </div>
      )}

      {/* Recording State */}
      {recording && !error && !transcribing && (
        <div
          className="flex flex-col items-center justify-center w-full h-full bg-gray-800"
          style={{ boxShadow: getInsetShadow(smoothedLevel) }}
        >
          {/* Timer Display */}
          <div className="text-gray-300 font-mono text-xs mb-2">
            {formatTime(elapsedMs)}
          </div>

          {/* Button Row */}
          <div className="flex gap-2">
            {/* Cancel Button */}
            <button
              onClick={handleCancel}
              disabled={cancelRecording.isPending}
              className="w-6 h-6 aspect-square rounded-lg shrink-0 bg-gray-700 hover:bg-gray-600 flex items-center justify-center transition-colors cursor-pointer disabled:opacity-50"
            >
              <X className="w-4 h-4 text-white" strokeWidth={2.5} />
            </button>

            {/* Stop Recording Button */}
            <button
              onClick={handleStop}
              disabled={stopRecording.isPending}
              className="w-6 h-6 aspect-square rounded-lg shrink-0 bg-red-500 hover:bg-red-600 flex items-center justify-center transition-colors cursor-pointer disabled:opacity-50"
            >
              <Square className="w-3 h-3 text-white" fill="white" strokeWidth={0} />
            </button>
          </div>
        </div>
      )}
    </div>
  );
}

export default RecordingPopup;
