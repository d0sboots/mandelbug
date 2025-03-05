#!/usr/bin/python3
"""Converts a "raw" mandelbrot image into a more pleasing form.

The "raw" image has pixel values denoting exact numbers of iterations.
This applies a color-mapping function and mixes down the over-sampled source.

This also contains many utility functions used to re-discover the original
color-mapping function and parameters.
"""

from __future__ import print_function
import math
import sys
from PIL import Image


def logistic(a, m, b, x):
  """A modified logistic function."""
  if x == 0:
    return 0.0
  if x < 16:
    x = 16

  v = a + (255.99 - a) / (1 + math.exp(m * (b - x)))
  if v < 0.0:
    return 0.0
  else:
    return v


def gen_params(m, b, x0, v0):
  """Produces [a, m, b] based on [m, b] ond one point on the curve."""
  a = (v0 - 255.99) * (1 + math.exp(m * (x0 - b))) + 255.99
  return [a, m, b]


CONSTANTS = [
    gen_params(0.02, 41, 16, 0),
    gen_params(0.022, 71, 16, 8),
    gen_params(0.022, 71, 16, 55),
]


def evaluate_slice(column):
  """Used for interactively checking closeness of fit."""
  print(CONSTANTS)
  lines = []
  relevant = []
  rdiff = {}
  for line in sys.stdin:
    inp = line.strip().split()
    points = [int(x) for x in inp[1:10]]
    values = [int(x) for x in inp[11:14]]
    diffs = []
    maxdiff = 0
    for i in range(1):
      param = CONSTANTS[column]
      diffs.append(
          sum(logistic(param[0], param[1], param[2], x) for x in points) / 9.0 -
          values[column])
      diff = diffs[i] - 0.5
      if diff < 0:
        diff = -diff
      if diff > maxdiff:
        maxdiff = diff

    if maxdiff > 0.48 and maxdiff < 1.25:
      relevant.append(len(lines))
      rdiff[len(lines)] = diffs[0]
    lines.append(inp)

  for x in relevant:
    print(x, " ".join(lines[x]) + " (%.6f)" % rdiff[x])


def param_search(column):
  print(CONSTANTS)
  param = CONSTANTS[column]
  for i in range(4000):
    x = 0.05 * i
    y = logistic(param[0], param[1], param[2], x)
    y4 = y * 20
    if abs(y4 - round(y4)) < .001:
      print(x, y)


def rawval(r, g, b):
  return r + 256 * (g + 256 * b)


def convert():
  """Converts a raw-iterations image to an output image."""
  im1 = Image.open(sys.argv[1])
  im2 = Image.new("RGB", (im1.size[0] / 3, im1.size[1] / 3))

  pixels1 = im1.load()
  output = im2.load()
  for y in range(im2.size[1]):
    if y % 10 == 0:
      print("Row %d" % y, file=sys.stderr)
    for x in range(im2.size[0]):
      vals = [
          rawval(*pixels1[x * 3 + ox, y * 3 + oy])
          for ox in range(3)
          for oy in range(3)
      ]
      output[x, y] = tuple(
          int(sum(logistic(p[0], p[1], p[2], x)
                  for x in vals) / 9.0)
          for p in CONSTANTS)

  im2.save(sys.argv[2])


def gen_octave():
  """Converts data to a form Octave can import."""
  values = []
  weights = []
  print("global POINTS = [")
  for line in sys.stdin:
    inp = line.strip().split()
    points = [int(x) for x in inp[1:10]]
    value = [int(x) for x in inp[11:14]]
    diffs = []
    maxdiff = 0
    for i in range(3):
      param = CONSTANTS[i]
      diffs.append(
          sum(logistic(param[0], param[1], param[2], x) for x in points) / 9.0 -
          value[i])
      diff = diffs[i] - 0.5
      if diff < 0:
        diff = -diff
      if diff > maxdiff:
        maxdiff = diff

    if maxdiff > 1.25:
      continue
    values.append(value)
    weights.append(int(inp[0]))
    print(" ".join(str(x) for x in points) + ";")
  print("]';")

  print("\nglobal WEIGHTS = [")
  print(";\n".join(str(x) for x in weights))
  print("]';")

  for i in range(3):
    print("\nglobal X" + chr(ord("0") + i) + " = [")
    print(";\n".join(str(x[i]) for x in values))
    print("]';")


convert()
