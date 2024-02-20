class Particle {
  constructor(size, x, y, color) {
    this.size = size;
    this.x = x;
    this.y = y;
    this.vx = 0;
    this.vy = 0;
    this.color = color;
  }
}

// pg1 - particle group 1 (is acted upon)
// pg2 - particle group 2 (acts)
// g - positive is repulsive, negative is attractive
const interact = (pg1, pg2, g) => {
  for (const p1 of pg1) {
    // initialize force
    let fx = 0;
    let fy = 0;
    for (const p2 of pg2) {
      // calculate distance
      let dx = p1.x - p2.x;
      let dy = p1.y - p2.y;
      let d = Math.hypot(dx, dy);
      if (d > 0 && d < 80) {
        // add force to total
        let f = g * (1 / d);
        fx += f * dx;
        fy += f * dy;
      }
    }
    p1.vx = (p1.vx + fx) * 0.5;
    p1.vy = (p1.vy + fy) * 0.5;
    p1.x += p1.vx;
    p1.y += p1.vy;
    if (p1.x <= 0) p1.vx = Math.abs(p1.vx);
    if (p1.y <= 0) p1.vy = Math.abs(p1.vy);
    if (p1.x >= canvas.width) p1.vx = -Math.abs(p1.vx);
    if (p1.y >= canvas.height) p1.vy = -Math.abs(p1.vy);
  }
}

const canvas = document.getElementById("canvas");
const cx = canvas.getContext("2d");
canvas.width = 700;
canvas.height = 700;

const particles = [
  new Array(200).fill(undefined).map(() => {
    const size = 5;
    const x = Math.round(Math.random() * (canvas.width - 100)) + 50;
    const y = Math.round(Math.random() * (canvas.height - 100)) + 50;
    const color = "yellow";

    return new Particle(size, x, y, color);
  }),
  new Array(200).fill(undefined).map(() => {
    const size = 5;
    const x = Math.round(Math.random() * (canvas.width - 100)) + 50;
    const y = Math.round(Math.random() * (canvas.height - 100)) + 50;
    const color = "red";

    return new Particle(size, x, y, color);
  }),
  new Array(200).fill(undefined).map(() => {
    const size = 5;
    const x = Math.round(Math.random() * (canvas.width - 100)) + 50;
    const y = Math.round(Math.random() * (canvas.height - 100)) + 50;
    const color = "green";

    return new Particle(size, x, y, color);
  }),
  new Array(200).fill(undefined).map(() => {
    const size = 5;
    const x = Math.round(Math.random() * (canvas.width - 100)) + 50;
    const y = Math.round(Math.random() * (canvas.height - 100)) + 50;
    const color = "blue";

    return new Particle(size, x, y, color);
  }),
  new Array(200).fill(undefined).map(() => {
    const size = 5;
    const x = Math.round(Math.random() * (canvas.width - 100)) + 50;
    const y = Math.round(Math.random() * (canvas.height - 100)) + 50;
    const color = "pink";

    return new Particle(size, x, y, color);
  })
];

const [yellow, red, green, blue, pink] = particles;
let last = 0;
let randomValues = new Array(25).fill(undefined).map(() => (Math.random() * 2) - 1);

const animate = (time) => {
  last ||= time;
  if (time - last > 5) {
    cx.clearRect(0, 0, canvas.width, canvas.height);
    interact(green, green, randomValues[0])
    interact(green, red, randomValues[1])
    interact(green, yellow, randomValues[2])
    interact(green, blue, randomValues[3])
    interact(green, pink, randomValues[4])
    interact(yellow, yellow, randomValues[5])
    interact(yellow, green, randomValues[6])
    interact(yellow, red, randomValues[7])
    interact(yellow, blue, randomValues[8])
    interact(yellow, pink, randomValues[9])
    interact(red, red, randomValues[10])
    interact(red, green, randomValues[11])
    interact(red, yellow, randomValues[12])
    interact(red, blue, randomValues[13])
    interact(red, pink, randomValues[14])
    interact(blue, blue, randomValues[15])
    interact(blue, red, randomValues[16])
    interact(blue, yellow, randomValues[17])
    interact(blue, green, randomValues[18])
    interact(blue, pink, randomValues[19])
    interact(pink, pink, randomValues[20])
    interact(pink, blue, randomValues[21])
    interact(pink, red, randomValues[22])
    interact(pink, yellow, randomValues[23])
    interact(pink, green, randomValues[24])
    for (const particle of particles.flat()) {
      cx.fillStyle = particle.color;
      cx.fillRect(particle.x, particle.y, particle.size, particle.size);
    }
    last = time;
  }

  window.requestAnimationFrame(animate);
}

animate();