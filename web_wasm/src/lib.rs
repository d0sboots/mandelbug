#![no_std]
use core::marker;
use core::panic::PanicInfo;
use libm::log2f;

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    // We check the compiled result to ensure that panic can't happen
    loop {}
}

#[repr(C)]
pub union Data {
    coord: u32,
    coord_out: (u16, u16),
    it: f32,
}

#[repr(C)]
pub struct Params {
    cx: f64,
    cy: f64,
    pixel_size: f64,
    max_iters: i32,
    coords_size: u32,
    width: u16,
    height: u16,
    data: marker::PhantomData<Data>,
}

#[target_feature(enable = "simd128")]
fn iters(cx: f64, cy: f64, max_iters: i64) -> f32 {
    let (mut x, mut y) = (cx, cy);
    let (mut lx, mut ly) = (cx, cy);
    let mut it = 0;
    let mut mark = 10;
    let mut mark_inc = 10;
    let mut x2 = x * x;
    let mut y2 = y * y;
    while x2 + y2 < 4e15 {
        y = 2.0 * x * y + cy;
        x = x2 - y2 + cx;
        if x == lx && y == ly {
            return f32::INFINITY;
        }
        x2 = x * x;
        y2 = y * y;
        it += 1;
        if it >= mark {
            if it >= max_iters {
                return max_iters as f32;
            }
            mark_inc += 1;
            mark += mark_inc;
            if mark > max_iters {
                mark = max_iters;
            }
            lx = x;
            ly = y;
        }
    }
    return (it as f32) - log2f(log2f((x2 + y2) as f32)) + 1.5f32;
}

impl Params {
    #[target_feature(enable = "simd128")]
    pub fn calculate(&mut self) {
        let width = u32::from(self.width);  // Avoid aliasing issues
        if width == 0 {
            return;
        }
        let size_half = self.pixel_size * 0.5;
        let base_x = self.cx - (f64::from(width) - 1.0) * size_half;
        let base_y = self.cy - (f64::from(self.height) - 1.0) * size_half;
        for i in (0..(self.coords_size as usize)).rev() {
            let coord: u32;
            unsafe {
                coord = (*(&raw const self.data as *const Data).offset(i as isize)).coord;
            }
            let px = coord % width;
            let py = coord / width;
            unsafe {
                (*(&raw mut self.data as *mut Data).offset((i * 2) as isize)).coord_out = (px as u16, py as u16);
            }
            let x = base_x + f64::from(px) * self.pixel_size;
            let y = base_y + f64::from(py) * self.pixel_size;
            let it = iters(x, y, self.max_iters.into());
            unsafe {
                (*(&raw mut self.data as *mut Data).offset((i * 2 + 1) as isize)).it = it;
            }
        }
    }
}

#[unsafe(no_mangle)]
#[target_feature(enable = "simd128")]
pub fn calculate() {
    unsafe {
        (*(8 as *mut Params)).calculate();
    }
}
