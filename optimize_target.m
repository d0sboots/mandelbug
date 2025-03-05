function retval = optimize_target (points, weights, actual, params)
  retval = sum(weights .* (max(0.0, -0.5 + abs(
          sum(merge(points == 0,
                    zeros(size(points)), params(4) + (params(1) - params(4)) ./
                    (1 + exp(params(2) .* (points - params(3))))))
          ./ 9 - actual - 0.5)) .^ 2));
endfunction

% Used to reduce the data to just that with weight > 1, i.e. the data we can
% trust to not be outliers. It turns out this is sufficient to solve the
% mystery.
function chop_weights
  global WEIGHTS;
  global X0;
  global X1;
  global X2;
  global POINTS;
  newsize = lookup(WEIGHTS, 2) + 1
  WEIGHTS = resize(WEIGHTS, 1, newsize);
  X0 = resize(X0, 1, newsize);
  X1 = resize(X1, 1, newsize);
  X2 = resize(X2, 1, newsize);
  POINTS = resize(POINTS, 9, newsize);
endfunction

function retval = calc_b (x, target)
  retval = target(1) - log((target(2) - x(1))/(x(3) - target(2))) / x(2);
endfunction

function retval = calc_a (x, target)
  retval = (target(2) - x(3)) * (1 + exp(x(1) * (target(1) - x(2)))) + x(3);
endfunction
