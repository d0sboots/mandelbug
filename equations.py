#!/usr/bin/python
"""Converts a "raw" mandelbrot image into a set of equations.

The raw image has pixel values denote exact numbers of iterations, and the
result can be used drive a non-linear best-fit or NLP approach to discovering
the color function.

Output format is "weight coef1 coef2 ... coef9 -> red green blue", where the
weight is the number of times this tuple was seen, coefficients are the number
of iterations before escape, and the RGB triple is the output value of the
resulting pixel. This assumes 3x AA, and that the contents of a pixel are
strictly determined by the 9 subsamples.

Coefficients between 1 and 16 are collapsed to 16, because of how the color
function works. This reduces the amount of data for future steps.

The coefficients will be sorted inside the tuple, and the tuples will be
sorted in the output. Each 9-tuple only appears once.
"""

from __future__ import print_function
import math
import sys
from PIL import Image

im1 = Image.open(sys.argv[1])
im2 = Image.open(sys.argv[2])

pixels1 = im1.load()
pixels2 = im2.load()

tuplemap = {}

def rawval(r, g, b):
  return r + 256*(g + 256*b)

for y in range(im2.size[1]):
  if y % 10 == 0:
    print("Row %d" % y, file=sys.stderr)
  for x in range(im2.size[0]):
    vals = [rawval(*pixels1[x*3+ox,y*3+oy])
            for ox in range(3) for oy in range(3)]
    vals = [16 if k > 0 and k < 16 else k for k in vals]
    vals.sort()
    vals = tuple(vals)
    pix = list(pixels2[x, y])
    pix.append(0)
    res = tuplemap.setdefault(vals, pix)
    if res[:3] != pix[:3]:
      print(
          "Inconsistent result! Previous pixel value and weight for",
          "%s was %s, but value at (%d,%d) is %s!" % (vals, res, x, y, pix),
          file=sys.stderr)
    else:
      res[3] += 1

formatstr = " ".join(["%d"] * 10) + " -> %d %d %d"

result = [(v[-1],) + k + tuple(v[:-1]) for k, v in tuplemap.items()]
# Sort first item (weight) descending, so we get the most common items first,
# but everything else sorted ascending.
result.sort(key=lambda x: (-x[0],) + x[1:])
for line in result:
  print(formatstr % line)
