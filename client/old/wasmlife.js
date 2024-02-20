import { PetriDish } from "../../wasm/pkg"

const WORLD_SIZE = 800;
const pd = new PetriDish(["red", "blue", "green", "yellow", "pink"], WORLD_SIZE, 200);

const canvas = document.getElementById("canvas");
const cx = canvas.getContext("2d");
canvas.width = WORLD_SIZE;
canvas.height = WORLD_SIZE;

let last = 0;
const animate = (time) => {
  last ||= time;
  if (time - last > 5) {
    pd.step();
    const cultures = JSON.parse(pd.cultures());
    cx.clearRect();
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
