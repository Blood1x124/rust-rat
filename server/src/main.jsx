import React from "react";
import ReactDOM from "react-dom/client";
import { App } from "./App";
import { BrowserRouter, Routes, Route } from "react-router-dom";
import "./styles.css";
import "remixicon/fonts/remixicon.css";

ReactDOM.createRoot(document.getElementById("root")).render(
  <BrowserRouter>
    <Routes>
      <Route path="/*" element={<App />} />
    </Routes>
  </BrowserRouter>
);
