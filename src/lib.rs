#![feature(proc_macro, wasm_custom_section, wasm_import_module, test)]
#[macro_use]
extern crate lazy_static;
extern crate wasm_bindgen;
extern crate rand;
extern crate test;
use wasm_bindgen::prelude::*;
use rand::{ Rng, XorShiftRng, weak_rng };

static F2: f64 = 0.366025403784_f64;
static G2: f64 = 0.211324865405_f64;

fn if_else(cond: bool, if_true: f64, if_false: f64) -> f64 {
    if cond {
        if_true
    } else {
        if_false
    }
}

/// Compute 2D gradient-dot-residual vector.
fn grad2(hash: u8, x: f64, y: f64) -> f64 {
    // Convert low 3 bits of hash code into 8 simple gradient directions,
    // and compute the dot product with (x,y).
    let h: u8 = hash & 7;
    let u: f64 = if_else(h < 4, x, y);
    let v: f64 = if_else(h < 4, y, x);

    if_else(h & 1 != 0, -u, u) + if_else(h & 2 != 0, -2.0 * v, 2.0 * v)
}

//#[derive(Clone, PartialEq, Eq)]
#[wasm_bindgen]
pub struct Simplex {
    perm: Vec<u8>
}

impl Simplex {
    /// Initializes a new simplex instance with a random seed using XorShiftRng.
    pub fn new() -> Simplex {
        let mut rng: XorShiftRng = weak_rng();

        let p: Vec<u8> = (0..256).map(|_| rng.gen::<u8>()).collect();
        let perm: Vec<u8> = (0..512).map(|idx:i32| {p[(idx & 255) as usize]}).collect();

        Simplex { perm: perm }
    }

    /// Initializes a new simplex instance with a random number generator.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use std::rand::StdRng;
    /// use noisy::gen::Simplex;
    ///
    /// let mut rng: StdRng = StdRng::new().unwrap();
    /// let simplex = Simplex::from_rng(&mut rng);
    /// ```
    ///
    /// This also allows you to initialize the instance with a seed:
    ///
    /// # Example
    ///
    /// ```rust
    /// # use std::rand::{StdRng, SeedableRng};
    /// use noisy::gen::Simplex;
    ///
    /// let seed: &[_] = &[1337];
    /// let mut rng: StdRng = SeedableRng::from_seed(seed);
    /// let simplex = Simplex::from_rng(&mut rng);
    /// ```
    pub fn from_rng<R: Rng>(rng: &mut R) -> Simplex {
        let p: Vec<u8> = (0..256).map(|_| rng.gen::<u8>()).collect();
        let perm: Vec<u8> = (0..512).map(|idx:i32| {p[(idx & 255) as usize]}).collect();

        Simplex { perm: perm }
    }
    /// Given a (x, y) coordinate, return a value in the interval [-1, 1].
    ///
    /// # Example
    ///
    /// ```rust
    /// let simplex = Simplex::new();
    /// let val = simplex.noise2d(2.46, 2.64);
    /// ```
    pub fn noise2d(&self, xin: f64, yin: f64) -> f64 {
        // Noise contributions from the three corners
        let n0: f64;
        let n1: f64;
        let n2: f64;

        // Skew the input space to determine which simplex cell we're in
        let s = (xin + yin) * F2; // Hairy factor for 2D
        let i = (xin + s).floor();
        let j = (yin + s).floor();
        let t = (i + j) * G2;

        // Unskew the cell origin back to (x, y) space
        // The x and y distances from the cell origin
        let x0 = xin - (i - t);
        let y0 = yin - (j - t);

        // For the 2D case, the simplex shape is an equilateral triangle.
        // Determine which shape we are in.
        let i1; // Offsets for second (middle) corner of simplex in (i, j) coords
        let j1;
        if x0 > y0 { // Lower triangle, XY order: (0, 0) -> (1, 0) -> (1, 1)
            i1 = 1;
            j1 = 0;
        } else { // Upper triangle, YX order: (0, 0) -> (0, 1) -> (1, 1)
            i1 = 0;
            j1 = 1;
        }

        // A step of (1, 0) in (i, j) means a step of (1 - c, -c) in (x, y), and
        // a step of (0, 1) in (i, j) means a step of (-c, 1 - c) in (x, y), where
        // c = (3 - sqrt(3.0))/6.

        // Offsets for middle corner in (x,y) unskewed coords
        let x1 = x0 - (i1 as f64) + G2;
        let y1 = y0 - (j1 as f64) + G2;
        // Offsets for last corner in (x,y) unskewed coords
        let x2 = x0 - 1.0 + 2.0 * G2;
        let y2 = y0 - 1.0 + 2.0 * G2;

        // Wrap the integer indices at 256, to avoid indexing perm[] out of bounds
        let ii = (i as usize) & 255;
        let jj = (j as usize) & 255;
        // Work out the hashed gradient indices of the three simplex corners
        let gi0 = self.perm[ii + self.perm[jj] as usize] as u8;
        let gi1 = self.perm[ii + i1 + (self.perm[jj + j1] as usize)] as u8;
        let gi2 = self.perm[ii + 1 + (self.perm[jj + 1] as usize)] as u8;

        // Calculate the contribution from the three corners
        let mut t0: f64 = 0.5 - x0 * x0 - y0 * y0;
        if t0 < 0.0 {
            n0 = 0.0;
        } else {
            t0 *= t0;
            n0 = t0 * t0 * grad2(gi0, x0, y0);
        }

        let mut t1: f64 = 0.5 - x1 * x1 - y1 * y1;
        if t1 < 0.0 {
            n1 = 0.0;
        } else {
            t1 *= t1;
            n1 = t1 * t1 * grad2(gi1, x1, y1);
        }

        let mut t2: f64 = 0.5 - x2 * x2 - y2 * y2;
        if t2 < 0.0 {
            n2 = 0.0;
        } else {
            t2 *= t2;
            n2 = t2 * t2 * grad2(gi2, x2, y2);
        }

        // Add contributions from each corner to get the final noise value.
        // The result is scaled to return values in the interval [-1, 1].
        40.0 * (n0 + n1 + n2)
    }
}

lazy_static! {
    pub static ref SIMPLEX: Simplex = Simplex::new();
}

#[wasm_bindgen]
pub fn noise(x: f64, y: f64) -> f64 {
    SIMPLEX.noise2d(x, y)
}


#[cfg(test)]
mod tests {
    use test::Bencher;
    use super::Simplex;

    #[test]
    fn can_create_new() {
        let simplex = Simplex::new();
        assert_eq!(simplex.noise2d(2.46, 2.64), 2.0);
    }

    #[bench]
    fn bench_initialization(b: &mut Bencher) {
        b.iter(|| {
            Simplex::new()
        })
    }
}
