import React, { useEffect, useRef, useState } from "react";

const Simulation: React.FC = () => {
  const [wasm, setWasm] = useState<any>();

  useEffect(() => {
    import("~/../wasm/pkg").then(setWasm);
  }, [])

  useEffect(() => {
    if (!wasm) return;
    const pd = new wasm.PetriDish(["red", "blue", "green", "yellow", "fuchsia", "aqua", "lime"], 1000, 700);
    console.log(pd.gravity_mesh());
    let last = 0;
    const animate = (time?: number) => {
      last ||= time;
      if (time - last > 10) pd.step();
      window.requestAnimationFrame(animate);
    }
    animate();
  }, [wasm])

  return (
    <canvas width={2000} height={2000} id="canvas" />
  );
};

export default Simulation;