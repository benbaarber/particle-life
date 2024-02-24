import React, { useEffect, useState } from "react";

declare class PetriDish {
  constructor(colors: string[], world_size: number, population: number);
  step(): void;
  cultures(): string;
  gravity_mesh(): string;
  free(): void;
}

const Simulation: React.FC = () => {
  const [wasm, setWasm] = useState<any>();

  useEffect(() => {
    import("~/../wasm/pkg").then(setWasm).catch(console.error);
  }, [])

  useEffect(() => {
    if (!wasm) return;
    const pd: PetriDish = new wasm.PetriDish(["red", "blue", "green", "yellow", "fuchsia", "aqua", "lime"], 1000, 700);
    console.log(pd.gravity_mesh());
    let last = 0;
    let frameId: number;
    const animate = (time?: number) => {
      last ||= time;
      if (time - last > 10) pd.step();
      frameId = window.requestAnimationFrame(animate);
    }
    animate();

    return () => window.cancelAnimationFrame(frameId);
  }, [wasm])

  return (
    <div>
      <canvas width={2000} height={2000} id="canvas" />
    </div>
  );
};

export default Simulation;