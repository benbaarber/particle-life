import React, { useEffect, useState } from "react";
import { Slider } from "./primitives/slider";
import { Button } from "./primitives/button";
import { Separator } from "./primitives/separator";
import { Switch } from "./primitives/switch";

declare class PetriDish {
  constructor(
    colors: string[],
    width: number,
    height: number,
    population: number,
    particle_aoe: number,
    theta: number,
    show_quadtrees: boolean,
  );
  step(): void;
  cultures(): string;
  gravity_mesh(): string;
  free(): void;
}

const colors = [
  "aqua",
  "magenta",
  "lime",
  "crimson",
  "yellow",
  "blue",
  "green",
  "white",
];

const Simulation: React.FC = () => {
  const [wasm, setWasm] = useState<any>();
  const [petriDish, setPetriDish] = useState<PetriDish>();
  const [population, setPopulation] = useState<number>(500);
  const [numCultures, setNumCultures] = useState<number>(5);
  const [particleAoe, setParticleAoe] = useState<number>(80);
  const [showQts, setShowQts] = useState<boolean>(false);

  const handleSimulate = () => {
    if (!wasm) return;
    const pd: PetriDish = new wasm.PetriDish(
      colors.slice(0, numCultures),
      1000,
      window.innerHeight,
      population,
      particleAoe,
      2,
      showQts,
    );
    setPetriDish(pd);
  };

  useEffect(() => {
    import("~/../wasm/pkg").then(setWasm).catch(console.error);
  }, []);

  useEffect(() => {
    handleSimulate();
  }, [wasm]);

  useEffect(() => {
    if (!petriDish) return;
    console.log(petriDish.gravity_mesh());
    let last = 0;
    let frameId: number;
    const animate = (time?: number) => {
      last ||= time;
      if (time - last > 5) {
        petriDish.step();
        last = time;
      }
      frameId = window.requestAnimationFrame(animate);
    };
    animate();

    return () => {
      console.log("DESTRUCTION");
      petriDish.free();
      window.cancelAnimationFrame(frameId);
    };
  }, [petriDish]);

  return (
    <div className="flex">
      <canvas
        width={1000}
        height={window.innerHeight}
        className="bg-[#080817]"
        id="canvas"
      />
      <div className="flex w-full flex-col items-center gap-3 p-8 text-white">
        <h1 className="text-xl font-bold">Particle Life</h1>
        <Separator />
        <div className="w-full pt-8">
          <p className="pb-4 text-sm font-semibold">
            Num Cultures: {numCultures}
          </p>
          <Slider
            value={[numCultures]}
            onValueChange={([v]) => setNumCultures(v)}
            min={1}
            max={8}
          />
        </div>
        <div className="w-full pt-8">
          <p className="pb-4 text-sm font-semibold">Population: {population}</p>
          <Slider
            value={[population]}
            onValueChange={([v]) => setPopulation(v)}
            min={100}
            max={2000}
            step={100}
          />
        </div>
        <div className="w-full pt-8">
          <p className="pb-4 text-sm font-semibold">
            Particle AOE: {particleAoe}
          </p>
          <Slider
            value={[particleAoe]}
            onValueChange={([v]) => setParticleAoe(v)}
            min={10}
            max={500}
            step={10}
          />
        </div>
        <div className="flex w-full items-center gap-3 pt-8">
          <p className="text-sm font-semibold">Show Quadtrees</p>
          <Switch checked={showQts} onCheckedChange={setShowQts} />
        </div>
        <div className="flex gap-2 pt-8">
          <Button onClick={handleSimulate}>Simulate</Button>
          <Button
            onClick={() => {
              const mesh = petriDish?.gravity_mesh();
              navigator.clipboard.writeText(mesh);
            }}
          >
            Copy Gravity Mesh
          </Button>
        </div>
        <Separator />
      </div>
    </div>
  );
};

export default Simulation;
