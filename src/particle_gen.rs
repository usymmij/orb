const GRID_LIMIT: f64 = 1000.;
const HALF_GRID_LIMIT: f64 = GRID_LIMIT / 2.;

mod wavefunction {
    use num::complex::ComplexFloat;
    use scilib::math::polynomial::Poly;
    use std::f64::consts::PI;

    pub struct Wavefunction {
        n: i32,
        l: i32,
        m: i32,
        a0: f64,
        laguerre_poly: Poly,
        legendre_poly: Poly,
    }

    fn factorial(n: i32) -> i32 {
        if n == 0 {
            return 1;
        }
        return factorial(n - 1) * n;
    }

    impl Wavefunction {
        pub fn new(n: i32, l: i32, m: i32, a0: f64) -> Wavefunction {
            let n_f = n as f64;
            let lag_coef = f64::sqrt(
                ((2.0 / n_f * a0).powi(3) * (factorial(n - l - 1) as f64))
                    / (2.0 * n_f * (factorial(n + l) as f64)),
            );
            let lag =
                Poly::laguerre((n - l - 1).try_into().unwrap(), 2.0 * l as f32 + 1.0) * lag_coef;
            let neg = if m % 2 == 0 { 1.0 } else { -1.0 };
            let leg_coef = (neg)
                * (((2 * l + 1) * factorial(l - (m.abs()))) as f64
                    / (4.0 * PI * factorial(l + m.abs()) as f64))
                    .sqrt();
            let leg = Poly::gen_legendre(l.try_into().unwrap(), m.try_into().unwrap()) * leg_coef;

            Wavefunction {
                n,
                l,
                m,
                a0,
                laguerre_poly: lag,
                legendre_poly: leg,
            }
        }

        pub fn radial(&self, r: f64) -> f64 {
            let n_f: f64 = self.n as f64;

            let p: f64 = 2.0 * r / (n_f * self.a0);
            return self.laguerre_poly.compute(p) * (-p / 2.0).exp() * (p.powi(self.l));
        }

        pub fn angular(&self, theta: f64, phi: f64) -> f64 {
            let complex_angle = num::complex::Complex::new(0.0, self.m as f64 * phi).exp();
            return self.legendre_poly.compute(theta.cos()) * complex_angle.re();
        }

        // NOTE: unused for now

        fn wf(&self, r: f64, theta: f64, phi: f64) -> f64 {
            return self.radial(r) * self.angular(theta, phi);
        }

        fn pdf(&self, r: f64, theta: f64, phi: f64) -> f64 {
            return self.wf(r, theta, phi).powi(2);
        }
    }

    #[cfg(test)]
    mod test {
        use super::Wavefunction;
        use super::factorial;
        #[test]
        fn test_facorials() {
            let ans: [i32; 5] = [1, 1, 2, 24, 40320];
            let input: [i32; 5] = [0, 1, 2, 4, 8];
            for i in 1..5 {
                assert_eq!(factorial(input[i]), ans[i]);
            }
        }

        #[test]
        fn test_norm_radial() {
            // norm radial and laguerre stuff

            let atom = Wavefunction::new(2, 1, 1, 1.0);
            let diff: f64 = atom.radial(1.0) - 0.1238;
            assert_eq!(diff < 0.00001, diff > -0.00001);

            let atom = Wavefunction::new(5, 1, 1, 3.0);
            let diff: f64 = atom.radial(2.0) - 0.14356;
            assert_eq!(diff < 0.00001, diff > -0.00001);

            let atom = Wavefunction::new(5, 3, 1, 1.0);
            let diff: f64 = atom.radial(2.0) - 0.000984;
            assert_eq!(diff < 0.00001, diff > -0.00001);
        }

        #[test]
        fn test_angular() {
            let atom = Wavefunction::new(4, 3, 1, 1.0);
            let diff: f64 = atom.angular(0.21, 2.0) + 0.10605;
            assert_eq!(diff < 0.001, diff > -0.001);

            let atom = Wavefunction::new(5, 4, 1, 1.0);
            let diff: f64 = atom.angular(2.0, 3.0) + 0.316835;
            assert_eq!(diff < 0.001, diff > -0.001);
        }
    }
}

mod cdf {
    #[derive(Copy, Clone)]
    struct CDFEntry {
        frac: f64,
        x: f64,
    }

    impl CDFEntry {
        pub fn new() -> CDFEntry {
            return CDFEntry { frac: 0., x: 0. };
        }
    }

    #[derive(Clone)]
    pub struct CDF {
        points: Vec<CDFEntry>,
    }

    impl CDF {
        pub fn new() -> CDF {
            CDF { points: Vec::new() }
        }

        pub fn add_point(&mut self, frac: f64, x: f64) {
            self.points.append(&mut vec![CDFEntry { frac: frac, x: x }]);
        }

        // PERF: recursive might be slower, switch to iterative?
        fn recurse_inverse_transform(&self, f: f64, i: usize) -> Option<f64> {
            if i >= self.points.len() {
                return None;
            }

            match i {
                1 => Some(2.),
                _ => None,
            };
            Some(1.0)
        }

        // just a binary search
        // TODO: return lambda for actual inverse transform
        pub fn inverse_transform(&self, f: f64) -> f64 {
            match self.recurse_inverse_transform(f, self.points.len() / 2) {
                None => 0.,
                Some(result) => result,
            }
        }
    }

    pub struct CDFTriple {
        pub radial: CDF,
        pub polar: CDF,
        pub azimuthal: CDF,
    }

    impl CDFTriple {
        pub fn new(r: CDF, p: CDF, a: CDF) -> CDFTriple {
            CDFTriple {
                radial: r,
                polar: p,
                azimuthal: a,
            }
        }
    }
}

use cdf::*;
use wavefunction::*;

/* NOTE:
    The generation method here takes a reimann sum over the PDF (square of the
    wavefunction) to get a CDF, then uniformly samples its inverse.
    It should be doable to do closed form integration of the PDF instead with
    IBP, treating n,l,m as constants.
*/
pub fn gen_cdf(n: i32, l: i32, m: i32, scale: f64, reso: i32) -> CDFTriple {
    let a0 = scale * 5.29; // INFO: bohr radius = 5.29 * 10E-11

    // solve the general form of the wavefunction for this state first
    let wavefunction = wavefunction::Wavefunction::new(n, l, m, a0);
    let odd = reso % 2;

    // use half the resolution steps and exploit symmetry
    let linspace = 0..((reso + odd) / 2);

    let scale = GRID_LIMIT / (reso - 1) as f64; // 10/3
    let linspace = linspace
        .into_iter()
        .map(|x| HALF_GRID_LIMIT - (x as f64 * scale));

    let mut rcdf = cdf::CDF::new();

    let mut rtotal: f64 = 0.;

    // NOTE: Radial sampling

    // check positive
    for i in linspace {
        rtotal += wavefunction.radial(i).powi(2);
        rcdf.add_point(rtotal, i);
    }

    return CDFTriple::new(rcdf.clone(), rcdf.clone(), rcdf);
}

fn sample_cdf(cdfs: CDFTriple) -> f64 {
    let radial_cdf = cdfs.radial;
    let polar_cdf = cdfs.polar;
    let azimuthal_cdf = cdfs.azimuthal;

    // TODO: generate spaced random numbers

    // TODO: sample inverse transform
    radial_cdf.inverse_transform(0.);

    // TODO: angular sampling
    // TODO: randomize octant
    // PERF: check if randomizing octant is better before or after

    // TODO: add in angular CDFs
    return 1.;
}

#[cfg(test)]
mod test {
    use super::gen_cdf;

    #[test]
    fn test_sampler() {
        gen_cdf(1, 0, 0, 5., 100);
    }
}
