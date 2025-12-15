import React from "react";
import ReactDOM from "react-dom/client";
import { BrowserRouter, Routes, Route } from "react-router-dom";
import App from "./App";
import RecordingPopup from "./RecordingPopup";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<App />} />
        <Route path="/recording-popup" element={<RecordingPopup />} />
      </Routes>
    </BrowserRouter>
  </React.StrictMode>,
);
