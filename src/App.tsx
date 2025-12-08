import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef, useState } from "react";
import "./App.css";

interface FnKeyEvent {
  pressed: boolean;
  timestamp: number;
}

interface ListenerError {
  error: string;
  is_permission_error: boolean;
}

function App() {
  const [fnPressed, setFnPressed] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [hasPermission, setHasPermission] = useState(true);
  const [checkingPermission, setCheckingPermission] = useState(true);
  const listenerStartedRef = useRef(false);

  // Check permission on mount
  useEffect(() => {
    checkPermission();
  }, []);

  // Listen for FN key events
  useEffect(() => {
    const unlisten = listen<FnKeyEvent>("fn-key-event", (event) => {
      setFnPressed(event.payload.pressed);
      setError(null); // Clear error if we're receiving events
    });

    return () => {
      unlisten.then((f) => f());
    };
  }, []);

  // Listen for listener errors
  useEffect(() => {
    const unlisten = listen<ListenerError>("fn-listener-error", (event) => {
      setError(event.payload.error);
      if (event.payload.is_permission_error) {
        setHasPermission(false);
      }
    });

    return () => {
      unlisten.then((f) => f());
    };
  }, []);

  // Start listener once permission is granted and the app is ready
  useEffect(() => {
    if (checkingPermission || !hasPermission || listenerStartedRef.current) return;

    listenerStartedRef.current = true;
    // invoke("start_fn_listener").catch((err) => {
    //   listenerStartedRef.current = false; // allow retry if it fails
    //   const message = err instanceof Error ? err.message : String(err);
    //   setError(message);
    // });
  }, [checkingPermission, hasPermission]);

  async function checkPermission() {
    setCheckingPermission(true);
    const permitted = await invoke<boolean>("check_accessibility_permission");
    setHasPermission(permitted);
    setCheckingPermission(false);

    if (!permitted) {
      setError("Accessibility permission required");
    }
  }

  async function requestPermission() {
    await invoke("request_accessibility_permission");
    // Wait a bit for user to potentially grant permission
    setTimeout(checkPermission, 1000);
  }

  async function restartApp() {
    await invoke("restart_app");
  }

  if (checkingPermission) {
    return (
      <main className="container">
        <h1>TypeFree - FN Key Monitor</h1>
        <p>Checking permissions...</p>
      </main>
    );
  }

  if (!hasPermission) {
    return (
      <main className="container">
        <h1>TypeFree - FN Key Monitor</h1>

        <div style={{ padding: "20px", backgroundColor: "#ff6b6b", borderRadius: "8px", marginTop: "20px" }}>
          <h2>⚠️ Permission Required</h2>
          <p>This app needs Accessibility permission to monitor keyboard events.</p>

          <div style={{ marginTop: "20px" }}>
            <h3>Setup Instructions:</h3>
            <ol style={{ textAlign: "left", maxWidth: "500px", margin: "0 auto" }}>
              <li>Click "Open System Settings" below</li>
              <li>In Privacy & Security → Accessibility</li>
              <li>Find "typefree" in the list</li>
              <li>Toggle the switch ON</li>
              <li>Click "Restart App" below</li>
            </ol>
          </div>

          <div style={{ marginTop: "20px", display: "flex", gap: "10px", justifyContent: "center" }}>
            <button onClick={requestPermission} style={{ fontSize: "16px", padding: "10px 20px" }}>
              Open System Settings
            </button>
            <button onClick={restartApp} style={{ fontSize: "16px", padding: "10px 20px", backgroundColor: "#4CAF50" }}>
              Restart App
            </button>
          </div>
        </div>
      </main>
    );
  }

  return (
    <main className="container">
      <h1>TypeFree - FN Key Monitor</h1>

      {error && (
        <div style={{ padding: "15px", backgroundColor: "#ff6b6b", borderRadius: "8px", marginTop: "20px" }}>
          <h3>Error</h3>
          <p>{error}</p>
          <button onClick={restartApp} style={{ marginTop: "10px" }}>
            Restart App
          </button>
        </div>
      )}

      <div style={{ marginTop: "40px", fontSize: "24px" }}>
        <div style={{
          display: "inline-block",
          padding: "40px 80px",
          backgroundColor: fnPressed ? "#4CAF50" : "#f44336",
          borderRadius: "12px",
          color: "white",
          fontWeight: "bold",
          transition: "all 0.2s ease",
          boxShadow: fnPressed ? "0 8px 16px rgba(76, 175, 80, 0.4)" : "0 4px 8px rgba(0,0,0,0.2)"
        }}>
          FN Pressed: {fnPressed ? "True" : "False"}
        </div>
      </div>

      <div style={{ marginTop: "30px", fontSize: "14px", opacity: 0.7 }}>
        <p>Press and hold the FN key on your keyboard to test</p>
      </div>
    </main>
  );
}

export default App;
