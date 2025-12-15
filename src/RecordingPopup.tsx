import { listen } from "@tauri-apps/api/event";
import { useEffect } from "react";
import "./RecordingPopup.css";

function RecordingPopup() {
  useEffect(() => {
    const unlistenStart = listen("recording-started", () => {
      console.log("[Popup] Recording started");
    });

    const unlistenStop = listen("recording-stopped", () => {
      console.log("[Popup] Recording stopped");
    });

    return () => {
      unlistenStart.then((f) => f());
      unlistenStop.then((f) => f());
    };
  }, []);

  return (
    <div className="popup-container">
      <div className="popup-content">
      </div>
    </div>
  );
}

export default RecordingPopup;
