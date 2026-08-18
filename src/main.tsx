import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
// import DesktopPet from "./components/DesktopPet";
import "./App.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
    {/* <DesktopPet /> */}
  </React.StrictMode>,
);
