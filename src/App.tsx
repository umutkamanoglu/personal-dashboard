import "./App.css";
import { SidebarProvider, SidebarTrigger } from "./components/ui/sidebar";
import { InputGroup, InputGroupInput, InputGroupAddon } from "./components/ui/input-group";
import { AppSidebar } from "./components/AppSidebar";
import { Search } from "lucide-react";
import { Route, Routes } from "react-router-dom";
import Home from "./pages/Home";
import { Switch } from "./components/ui/switch";
import ThemeToggle from "./components/ThemeToggle";

function App() {
  return (
    <SidebarProvider defaultOpen={true}>
      <AppSidebar />
      {/* w-full yerine flex-1 ve min-w-0 ekledik */}
      <main className="flex-1 flex flex-col min-w-0 max-h-screen">
        <header className="bg-sidebar p-2 flex items-center justify-between border-b shrink-0">
          <SidebarTrigger />
          
          <InputGroup className="max-w-92 mx-4">
            <InputGroupAddon>
              <Search className="size-4" />
            </InputGroupAddon>
            <InputGroupInput placeholder="Ara.." />
          </InputGroup>

          <div className="px-2">
            <ThemeToggle />
          </div>
        </header>

        {/* İçeriğin kendi içinde scroll olması için */}
        <div className="flex-1 overflow-auto p-4">
          <Routes>
            <Route path="/" element={<Home />} />
          </Routes>
        </div>
      </main>
    </SidebarProvider>
  );
}

export default App;
