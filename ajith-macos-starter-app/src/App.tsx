import { useEffect } from "react";
import { Titlebar } from "@/components/Titlebar";
import { Layout } from "@/components/Layout";
import { ComponentShowcase } from "@/components/ComponentShowcase";
import { configureOverlayWindow } from "@/lib/window";

function App() {
  useEffect(() => {
    // Configure window for overlay behavior when app starts
    configureOverlayWindow();
  }, []);

  return (
    <div className="h-screen bg-transparent-light window-with-stroke overflow-hidden flex flex-col">
      <Titlebar />
      <div className="flex-1 overflow-hidden flex flex-col">
        <Layout>
          <ComponentShowcase />
        </Layout>
      </div>
    </div>
  );
}

export default App;