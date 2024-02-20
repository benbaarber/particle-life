import React from "react";
import { createRoot } from "react-dom/client";
import Simulation from "./Simulation";

import "./style.css";

const App: React.FC = () => {
  return (
    <Simulation />
  )
}

const root = createRoot(document.getElementById("root") as HTMLDivElement);
root.render(<App />);

export default App;