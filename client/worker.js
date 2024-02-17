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
        let f = g * (1/d);
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
    if (p1.x >= 500) p1.vx = -Math.abs(p1.vx);
    if (p1.y >= 500) p1.vy = -Math.abs(p1.vy); 
  }

  return [pg1, pg2]
}

onmessage = (e) => {
  const { pg1, pg2, g } = e.data;
  for (let i = 0; i < pg1.length; i++) {
    // initialize force
    let p1 = pg1[i];
    let fx = 0;
    let fy = 0;
    for (let p2 of pg2) {
      // calculate distance
      let dx = p1.x - p2.x;
      let dy = p1.y - p2.y;
      let d = Math.hypot(dx, dy);
      if (d > 0 && d < 80) {
        // add force to total
        let f = g * (1/d);
        fx += f * dx;
        fy += f * dy;
      }
    }
    pg1[i].vx = (p1.vx + fx) * 0.5;
    pg1[i].vy = (p1.vy + fy) * 0.5;
    pg1[i].x += pg1[i].vx;
    pg1[i].y += pg1[i].vy;
    if (p1.x <= 0) pg1[i].vx = Math.abs(p1.vx);
    if (p1.y <= 0) pg1[i].vy = Math.abs(p1.vy);
    if (p1.x >= 500) pg1[i].vx = -Math.abs(p1.vx);
    if (p1.y >= 500) pg1[i].vy = -Math.abs(p1.vy); 
  }
  postMessage(pg1);
}