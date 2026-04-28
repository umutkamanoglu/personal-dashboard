import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { ThemeProvider } from "./components/theme-provider";
import { TooltipProvider } from "./components/ui/tooltip";
import { BrowserRouter } from "react-router-dom"

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <BrowserRouter>
      <ThemeProvider
        defaultTheme="dark"
        storageKey="vite-ui-theme"
      >
        <TooltipProvider>
          <App />
        </TooltipProvider>
      </ThemeProvider>
    </BrowserRouter>
  </React.StrictMode>,
);
