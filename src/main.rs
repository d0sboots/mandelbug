use clap::Parser;
use libc::strtod as c_strtod;
use num_traits::float::Float;
use rug::{
    float::Round,
    ops::{AssignRound, CompleteRound, Pow},
    Float as RFloat,
};
use std::{error::Error, fmt::Debug, ptr::null_mut, str::FromStr};

trait BasicFloat: Float + From<f32> + Debug + Into<f64> {
    const ZERO: Self;
    const ONE: Self;
    const TWO: Self;
}
impl BasicFloat for f32 {
    const ZERO: Self = 0f32;
    const ONE: Self = 1f32;
    const TWO: Self = 2f32;
}
impl BasicFloat for f64 {
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;
    const TWO: Self = 2.0;
}

trait Downsize {
    fn convert(val: &RFloat, prec: u32) -> Self;
}
impl Downsize for RFloat {
    fn convert(val: &RFloat, prec: u32) -> Self {
        RFloat::with_val(prec, val)
    }
}
impl Downsize for f32 {
    fn convert(val: &RFloat, _: u32) -> Self {
        val.to_f32()
    }
}
impl Downsize for f64 {
    fn convert(val: &RFloat, _: u32) -> Self {
        val.to_f64()
    }
}

trait OurFloat: Sized + Debug + Downsize + PartialOrd + Clone {
    fn to_f64(&self) -> f64;
    fn m_iter(params: &MandelParams<Self>) -> (i64, Self, Self);
    fn display(n: i64, z2: &Self, d2: &Self) -> String {
        format!("it:{n:7} z2:{z2:?} d2:{d2:?}")
    }
    fn default_radius() -> Self {
        Downsize::convert(&RFloat::with_val(53, 1e40), 53)
    }
}
impl OurFloat for RFloat {
    fn to_f64(&self) -> f64 {
        RFloat::to_f64(self)
    }
    fn m_iter(params: &MandelParams<Self>) -> (i64, Self, Self) {
        m_iter_p(params)
    }
}
impl<T: BasicFloat + Downsize> OurFloat for T {
    fn to_f64(&self) -> f64 {
        (*self).into()
    }
    fn m_iter(params: &MandelParams<Self>) -> (i64, Self, Self) {
        m_iter_f(params)
    }
    fn default_radius() -> Self {
        T::max_value().sqrt().sqrt()
    }
}

#[derive(Debug)]
struct MandelParams<T> {
    x: T,
    y: T,
    cx: T,
    cy: T,
    it: i64,
    rad2: T,
    julia: bool,
}

fn m_iter_f<T: BasicFloat>(p: &MandelParams<T>) -> (i64, T, T) {
    let mut i = 0;
    let (mut x, mut y) = (p.x.clone(), p.y.clone());
    let (mut dx, mut dy) = (T::ONE, T::ZERO);
    let (mut x2, mut y2) = (x * x, y * y);
    let julia = if p.julia { T::ZERO } else { T::ONE };
    while x2 + y2 < p.rad2 && i < p.it {
        let dx_tmp = dx * x + (julia - dy * y);
        dy = T::TWO * (dy * x + dx * y);
        dx = T::TWO * dx_tmp;
        y = T::TWO * x * y + p.cy;
        x = x2 - y2 + p.cx;
        x2 = x * x;
        y2 = y * y;
        i += 1;
    }
    if i >= p.it {
        return (-1, T::ZERO, T::ZERO);
    }
    (i, x2 + y2, dx * dx + dy * dy)
}

fn m_iter_p(p: &MandelParams<RFloat>) -> (i64, RFloat, RFloat) {
    let rad2f = p.rad2.to_f64();
    let mut i = 0;
    let mut x = p.x.clone();
    let mut y = p.y.clone();
    let mut dx = RFloat::with_val(x.prec(), 1.0);
    let mut dy = RFloat::with_val(y.prec(), 0.0);
    let mut x2 = x.square_ref().complete(x.prec());
    let mut y2 = y.square_ref().complete(y.prec());
    let mut dxy = RFloat::new(x.prec());
    let mut dyy = RFloat::new(y.prec());
    while x2.to_f64() + y2.to_f64() < rad2f && i < p.it {
        dxy.assign_round(&dx * &y, Round::Nearest);
        dyy.assign_round(&dy * &y, Round::Nearest);
        if !p.julia {
            dyy -= 1.0;
        }
        dx.mul_sub_mut(&x, &dyy);
        dy.mul_add_mut(&x, &dxy);
        dx <<= 1;
        dy <<= 1;
        y <<= 1;
        y.mul_add_mut(&x, &p.cy);
        x2 -= &y2;
        x.assign_round(&x2 + &p.cx, Round::Nearest);
        x2.assign_round(x.square_ref(), Round::Nearest);
        y2.assign_round(y.square_ref(), Round::Nearest);
        i += 1;
    }

    if i >= p.it {
        return (
            -1,
            RFloat::with_val(x.prec(), 0.0),
            RFloat::with_val(y.prec(), 0.0),
        );
    }
    (i, x2 + y2, dx.square() + dy.square())
}

fn distance<T: BasicFloat>(n: i64, z2: T, d2: T) -> (T, T, T) {
    let half = T::ONE / T::TWO;
    if n == -1 {
        return (T::ZERO, T::ZERO, T::ZERO);
    }
    let sqr_rat = (z2 / d2).sqrt();
    let lnz2 = z2.ln();
    if n > 65 {
        // Since doubles max at 2^1024, log(z2) is at most ~710, or ~2^9.5.
        // Thus, G = log(z2)*2^-n will be < 2^-53 for n > 65, and at that size
        // sinh(G) = G and exp_m1(G) = G. Not only does this save computation, it protects
        // against under/overflow errors from scaling in the full computation.
        let res = sqr_rat * lnz2 * half;
        (res, res, res)
    } else {
        // 2^n and 2^-n-1
        let p: T = f32::from_bits(((127 + n) << 23) as u32).into();
        let pinv: T = f32::from_bits(((126 - n) << 23) as u32).into();
        let g = lnz2 * pinv;
        let scaled = sqr_rat * p;
        (
            (-T::TWO * g).exp_m1() * -scaled * half,
            sqr_rat * lnz2 * half,
            g.sinh() * scaled,
        )
    }
}

fn strtod(s: &str) -> Result<f64, String> {
    let mut buf = String::with_capacity(s.len() + 1);
    buf.push_str(s);
    buf.push('\0');
    let ptr = buf.as_ptr() as *const i8;
    let mut endptr: *mut i8 = null_mut();
    unsafe {
        let val = c_strtod(ptr, &raw mut endptr);
        if endptr.offset_from(ptr) as usize == s.len() {
            Ok(val)
        } else {
            Err(["Cant parse ", s, " as double!"].concat())
        }
    }
}

pub fn parse_float(val: &str) -> Result<RFloat, rug::float::ParseFloatError> {
    match RFloat::parse(val) {
        Ok(x) => Ok(x.complete(1024)),
        Err(err) => {
            if let Ok(res) = strtod(val) {
                // strtod can parse things like 0x1p5 syntax
                Ok(RFloat::with_val(53, res))
            } else {
                Err(err)
            }
        }
    }
}

#[derive(Clone, Debug)]
enum Precision {
    Single,
    Double,
    Multi(u32),
}
impl std::fmt::Display for Precision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}
fn parse_prec(val: &str) -> Result<Precision, std::num::ParseIntError> {
    match val.to_lowercase().as_str() {
        "s" | "single" => Ok(Precision::Single),
        "d" | "double" => Ok(Precision::Double),
        _ => Ok(Precision::Multi(u32::from_str(val)?)),
    }
}

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[arg(allow_hyphen_values = true)]
    #[arg(value_parser = parse_float)]
    /// real coordinate
    x: RFloat,

    #[arg(allow_hyphen_values = true)]
    #[arg(value_parser = parse_float)]
    /// imaginary coordinate
    y: RFloat,

    #[arg(short, long)]
    #[arg(default_value_t = Precision::Double)]
    #[arg(value_parser = parse_prec)]
    /// Precision of computation. "single"/"s", "double"/"d", or a number of bits.
    prec: Precision,

    #[arg(short, long)]
    #[arg(default_value_t = 100_000_000)]
    /// Maximum number of iterations before bailout
    iters: i64,

    #[arg(short, long)]
    #[arg(default_value_t = RFloat::with_val(53, 1e40))]
    #[arg(allow_hyphen_values = true)]
    #[arg(value_parser = parse_float)]
    /// Escape radius
    radius: RFloat,

    #[arg(long)]
    #[arg(allow_hyphen_values = true)]
    #[arg(num_args = 2)]
    #[arg(value_parser = parse_float)]
    /// Vector to test distance calculation along. Points will be an exponentially weighted
    /// multiple of the vector, plus the starting offset. The initial offset should be a point on
    /// the boundary of the Mandelbrot.
    vec: Option<Vec<RFloat>>,

    #[arg(long)]
    #[arg(value_parser = parse_float)]
    #[arg(default_value_t = RFloat::parse(".5623413251903490803949510397764812314682").unwrap().complete(128))]
    /// Multiplier for each step of vector distance calculation. If this is > 1.0, the points go
    /// away from the set, otherwise they approach the set.
    vec_mult: RFloat,

    #[arg(long)]
    #[arg(default_value_t = 40)]
    /// Number of vector multiplier steps to take, not including the initial point.
    vec_steps: i32,

    #[arg(long)]
    #[arg(allow_hyphen_values = true)]
    #[arg(num_args = 2)]
    #[arg(value_parser = parse_float)]
    /// Operate on a Julia set instead of the Mandelbrot set. The two values after the flag specify
    /// the constant value parameter of the Julia set, while the base coordinate is the starting value.
    julia: Option<Vec<RFloat>>,
}

fn convert_args<T: OurFloat>(args: &Cli) -> (MandelParams<T>, u32) {
    let prec = match args.prec {
        Precision::Multi(x) => x,
        _ => 64,
    };
    // Patch radius possibly being too large, especially for small types
    let default = T::default_radius();
    let cvt = T::convert(&args.radius, prec);
    let x = T::convert(&args.x, prec);
    let y = T::convert(&args.y, prec);
    (
        MandelParams::<T> {
            cx: match args.julia.as_ref() {
                Some(v) => T::convert(&v[0], prec),
                None => x.clone(),
            },
            cy: match args.julia.as_ref() {
                Some(v) => T::convert(&v[1], prec),
                None => y.clone(),
            },
            x: x,
            y: y,
            it: args.iters,
            rad2: if cvt < default { cvt } else { default },
            julia: args.julia.is_some(),
        },
        prec,
    )
}

fn do_point<T: OurFloat>(args: &Cli) {
    let (params, _) = convert_args::<T>(args);
    let (n, z2, d2) = T::m_iter(&params);
    let (dist1, dist2, dist3) = distance(n, z2.to_f64(), d2.to_f64());
    println!(
        "{} dist1:{dist1:?} dist2:{dist2:?} dist3:{dist3:?}",
        T::display(n, &z2, &d2)
    );
}

fn do_distance<T: OurFloat>(args: &Cli) {
    let (mut params, prec) = convert_args::<T>(args);
    // Do coordinate math at high precision
    let vvec = args.vec.as_ref().unwrap();
    let (vx, vy) = (&vvec[0], &vvec[1]);
    let mag = vx.hypot_ref(vy).complete(64).to_f64();
    println!(
        "{:>7} {:>21} {:>21} {:>21} {:>21} {:>21} {:>21}",
        "iters", "x", "y", "dist1", "dist2", "dist3", "ratio"
    );
    for i in 0..args.vec_steps + 1 {
        let vmul = args.vec_mult.clone().pow(i);
        let x_tmp = (vx * &vmul + &args.x).complete(vmul.prec());
        let vmulf = vmul.to_f64();
        let y_tmp = vy * vmul + &args.y;
        params.x = T::convert(&x_tmp, prec);
        params.y = T::convert(&y_tmp, prec);
        if !params.julia {
            params.cx = params.x.clone();
            params.cy = params.y.clone();
        }
        let (n, z2, d2) = T::m_iter(&params);
        let (dist1, dist2, dist3) = distance(n, z2.to_f64(), d2.to_f64());
        let ratio = dist1 / (vmulf * mag);
        println!(
            "{n:7} {x_tmp:21.15e} {y_tmp:21.15e} {dist1:21.15e} {dist2:21.15e} {dist3:21.15e} {ratio:21.15e}",
        );
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();

    if args.vec.is_some() {
        match args.prec {
            Precision::Single => do_distance::<f32>(&args),
            Precision::Double => do_distance::<f64>(&args),
            Precision::Multi(_) => do_distance::<RFloat>(&args),
        }
    } else {
        match args.prec {
            Precision::Single => do_point::<f32>(&args),
            Precision::Double => do_point::<f64>(&args),
            Precision::Multi(_) => do_point::<RFloat>(&args),
        }
    }
    Ok(())
}
