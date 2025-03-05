use num::complex::Complex;
use num::traits::MulAdd;
use num_traits::float::Float;
use rug::Complex as RComplex;
//use rug::Float as RFloat;
use std::env;
use libc::strtod as c_strtod;
use std::error::Error;
use std::ptr::null_mut;

fn strtod(s: &str) -> Result<f64, String> {
    let mut buf = String::with_capacity(s.len() + 1);
    buf.push_str(s);
    buf.push('\0');
    let ptr = buf.as_ptr() as * const i8;
    let mut endptr : * mut i8 = null_mut();
    unsafe {
        let val = c_strtod(ptr, &raw mut endptr);
        if endptr.offset_from(ptr) as usize == s.len() {
            Ok(val)
        } else {
            Err(["Cant parse ", s, " as double!"].concat())
        }
    }
}

fn iters<T: Float + std::convert::From<f32> + MulAdd<Output = T>>(cx: T, cy: T) -> f64
where
    f64: From<T>,
{
    let mut x = Complex::new(cx, cy);
    let dx = x;
    let mut lx = x;
    let (mut a, mut b) = (Complex::new(1f64, 0f64), Complex::new(0f64, 0f64));
    let mut c = b;
    let mut it = 0i64;
    let mut mark = 10i64;
    let mut mark_inc = 10;
    while x.norm_sqr() < 4e13f32.into() {
        let x2t = x + x;
        let x2: Complex<f64> = Complex::new(x2t.re.into(), x2t.im.into());
        let a2 = a + a;
        c = x2.mul_add(c, a2 * b);
        b = x2.mul_add(b, a * a);
        a = x2.mul_add(a, Complex::new(1f64, 0f64));
        x = x.mul_add(x, dx);
        if x == lx {
            return 0f32.into();
        }
        it += 1;
        if it >= mark {
            println!(
                "{} {} {} {} {}",
                it,
                Complex::<f64>::new(x.re.into(), x.im.into()),
                a,
                b,
                c
            );
            mark_inc += 1;
            mark += mark_inc;
            lx = x;
        }
    }
    return (it as f64) - f64::from(x.norm_sqr()).log2().log2() + 1.5;
}

fn fixed_it(dx: Complex<f64>, it: i64) -> (Complex<f64>, Complex<f64>, Complex<f64>, Complex<f64>) {
    let mut x = dx;
    let (mut a, mut b) = (Complex::ONE, Complex::ZERO);
    let mut c = b;
    for _ in 0..it {
        let x2 = x + x;
        let a2 = a + a;
        c = x2.mul_add(c, a2 * b);
        b = x2.mul_add(b, a * a);
        a = x2.mul_add(a, Complex::ONE);
        x = x.mul_add(x, dx);
    }
    (x, a, b, c)
}

fn multiprec_it(dx: RComplex, it: i64) -> RComplex {
    let mut x = RComplex::with_val(
        {
            let (r, i) = dx.prec();
            (r * 2, i * 2)
        },
        &dx,
    );
    for _ in 0..it {
        x.square_mut();
        x += &dx;
    }
    x
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        return Err("Not enough arguments!".into());
    }
    let (x, y) = (strtod(&args[1])?, strtod(&args[2])?);
    if args.len() < 4 {
        println!("{}", iters(x, y));
    } else {
        let (dist, it) = (strtod(&args[3])?, args[4].parse::<i64>()?);
        let rx = RComplex::with_val(128, (x, y));
        let (r, a, b, c) = fixed_it(Complex::new(x, y), it);
        println!("{} {} {} {}", r, a, b, c);
        println!("{:.20e}", multiprec_it(rx, it));
        let mut sum: Complex<f64> = Complex::ZERO;
        let dirs: [Complex<f64>; 4] =
            [Complex::ONE, Complex::I, -Complex::ONE, -Complex::I];
        for dir in dirs {
            let off = dir * dist;
            let (r_off, a_off, b_off, c_off) = fixed_it(Complex::new(x, y) + off, it);
            println!(
                "{off:+24.5e} {r_off:+24.5e} {:.5e} {a_off:.4e} {b_off:.4e} {c_off:.4e}",
                off.mul_add(off.mul_add(off.mul_add(c, b), a), r - r_off)
            );
            sum += r_off;
        }
        println!("{:.5e}", sum * 0.25 - r);
    }
    Ok(())
}
