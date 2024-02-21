import React, { useEffect, useRef, useState } from "react";

const animation = (cx: CanvasRenderingContext2D, pd: any) => {
  let last = 0;
  const animate = (time?: number) => {
    last ||= time;
    if (true) {
      pd.step();
      const cultures = JSON.parse(pd.cultures());
      cx.clearRect(0, 0, 2000, 2000);
      for (const { color, particles } of cultures) {
        cx.fillStyle = color;
        for (const p of particles) {
          cx.fillRect(p[0], p[1], 5, 5);
        }
      }
      last = time;
    }
    window.requestAnimationFrame(animate);
  }
  animate();
}

const Simulation: React.FC = () => {
  const [wasm, setWasm] = useState<any>();
  const canvas = useRef<HTMLCanvasElement>();

  useEffect(() => {
    import("~/../wasm/pkg").then(setWasm);
  }, [])

  useEffect(() => {
    if (!wasm) return;
    const cx = canvas.current.getContext("2d");
    const pd = new wasm.PetriDish(["red", "blue", "green", "yellow", "pink"], 500, 50);
    animation(cx, pd);
  }, [wasm])

  return (
    <canvas width={2000} height={2000} ref={canvas} />
  );
};

export default Simulation;