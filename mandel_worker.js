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
  return Math.floor(a + (255.99 - a) / (1 + Math.exp(m * (b - x))));
}

function handleMessage(msgEvent) {
  const startTime = performance.now();
  const {cx, cy, drawId, width, height, pixelSize, maxIters, coords} = msgEvent.data;
  const sizeHalf = pixelSize * 0.5;
  const baseX = cx - (width - 1) * sizeHalf;
  const baseY = cy + (height - 1) * sizeHalf;
  const result = new Uint8ClampedArray(coords.length * 4);
  for (let i = 0; i < coords.length; i++) {
    const x = baseX + (coords[i] % width) * pixelSize;
    const y = baseY - ((coords[i] / width) | 0) * pixelSize;
    const it = iters(x, y, maxIters);
    const j = i * 4;
    if (it >= maxIters) {
      result[j] = 0;
      result[j+1] = 0;
      result[j+2] = 0;
    } else {
      result[j] = logistic(it, ...PARAMS[0])
      result[j+1] = logistic(it, ...PARAMS[1])
      result[j+2] = logistic(it, ...PARAMS[2])
    }
    result[j+3] = 255;
  }
  const evalPerMs = coords.length / (performance.now() - startTime);
  self.postMessage({drawId, evalPerMs, coords, points: result}, [coords.buffer, result.buffer]);
}
