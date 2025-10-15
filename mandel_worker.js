self.onmessage = handleMessage;

function iters(cx, cy, maxIters) {
  let x = cx, y = cy;
  let lx = cx, ly = cy;
  let it = 0;
  let mark = 10;
  let markInc = 10;
  let x2 = x * x;
  let y2 = y * y;
  while (x2 + y2 < 4e30) {
    y = 2 * x * y + cy;
    x = x2 - y2 + cx;
    if (x === lx) {
      if (y === ly) return Infinity;
    }
    x2 = x * x;
    y2 = y * y;
    it++;
    if (it >= mark) {
      if (it >= maxIters) return maxIters;
      markInc++;
      mark += markInc;
      if (mark > maxIters) {
        mark = maxIters;
      }
      lx = x;
      ly = y;
    }
  }
  return it - Math.log2(Math.log2(x2 + y2)) + 1.5;
}

function genParams(m, b, x0, v0) {
  const a = (v0 - 255.99) * (1 + Math.exp(m * (x0 - b))) + 255.99;
  return [a, m, b];
}
const PARAMS = [
    genParams(0.02, 41, 16, 0),
    genParams(0.022, 71, 16, 8),
    genParams(0.022, 71, 16, 55),
];

function logistic(x, a, m, b) {
  return Math.max(0, Math.floor(a + (255.99 - a) / (1 + Math.exp(m * (b - x)))));
}

function handleMessage(msgEvent) {
  const startTime = performance.now();
  const {cx, cy, drawId, width, height, pixelSize, maxIters, coords} = msgEvent.data;
  const sizeHalf = pixelSize * 0.5;
  const baseX = cx - (width - 1) * sizeHalf;
  const baseY = cy + (height - 1) * sizeHalf;
  const result = new ArrayBuffer(coords.length * 8);
  const view16 = new Uint16Array(result);
  const view8 = new Uint8Array(result);
  for (let i = 0; i < coords.length; i++) {
    const j = i * 4;
    const offset = i * 8;
    const px = coords[i] % width;
    const py = (coords[i] / width) | 0;
    view16[j] = px;
    view16[j + 1] = py;
    const x = baseX + px * pixelSize;
    const y = baseY - py * pixelSize;
    const it = iters(x, y, maxIters);
    if (it >= maxIters) {
      view16[j + 2] = 0;
      view16[j + 3] = 255 << 8;
    } else {
      view8[offset + 4] = logistic(it, ...PARAMS[0])
      view8[offset + 5] = logistic(it, ...PARAMS[1])
      view8[offset + 6] = logistic(it, ...PARAMS[2])
      view8[offset + 7] = 255;
    }
  }
  const evalPerMs = coords.length / (performance.now() - startTime);
  self.postMessage({drawId, evalPerMs, points: result}, [result]);
}
