use num::complex::ComplexFloat;
use scilib::math::polynomial::Poly;
use std::{f64::consts::PI, vec};

const GRID_LIMIT: f64 = 1000.;
const HALF_GRID_LIMIT: f64 = GRID_LIMIT / 2.;

struct Wavefunction {
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
        let lag = Poly::laguerre((n - l - 1).try_into().unwrap(), 2.0 * l as f32 + 1.0) * lag_coef;
        let neg = if m % 2 == 0 { 1.0 } else { -1.0 };
        let leg_coef = (neg)
            * (((2 * l + 1) * factorial(l - (m.abs()))) as f64
                / (4.0 * PI * factorial(l + m.abs()) as f64))
                .sqrt();
        let leg = Poly::gen_legendre(l.try_into().unwrap(), m.try_into().unwrap()) * leg_coef;

        Wavefunction {
            n: n,
            l: l,
            m: m,
            a0: a0,
            laguerre_poly: lag,
            legendre_poly: leg,
        }
    }

    fn radial(&self, r: f64) -> f64 {
        let n_f: f64 = self.n as f64;

        let p: f64 = 2.0 * r / (n_f * self.a0);
        return self.laguerre_poly.compute(p) * (-p / 2.0).exp() * (p.powi(self.l));
    }

    fn angular(&self, theta: f64, phi: f64) -> f64 {
        let complex_angle = num::complex::Complex::new(0.0, self.m as f64 * phi).exp();
        return self.legendre_poly.compute(theta.cos()) * complex_angle.re();
    }

    //fn wf(&self, r: f64, theta: f64, phi: f64) -> f64 {
    //    return self.radial(r) * self.angular(theta, phi);
    //}

    //fn pdf(&self, r: f64, theta: f64, phi: f64) -> f64 {
    //    return self.wf(r, theta, phi).powi(2);
    //}
}

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

struct CDF {
    total: f64,
    length: i32,
    points: Vec<CDFEntry>,
}

impl CDF {
    // secretely just a binary search
    fn inverse_transform(&self, f: f64) {
        println!("{}", self.length);
    }
}

pub fn sample_wf_pdf(n: i32, l: i32, m: i32, scale: f64, reso: i32) -> f64 {
    let a0 = scale * 5.29; // bohr radius = 5.29 * 10E-11

    // solve the general form of the wavefunction for this state first
    let wavefunction = Wavefunction::new(n, l, m, a0);
    let odd = reso % 2;

    // use half the resolution steps and exploit symmetry
    let linspace = 0..((reso + odd) / 2);

    let scale = GRID_LIMIT / (reso - 1) as f64; // 10/3
    let linspace = linspace
        .into_iter()
        .map(|x| HALF_GRID_LIMIT - (x as f64 * scale));

    let mut rcdf = CDF {
        total: 0.,
        length: 0,
        points: Vec::new(),
    };

    // check positive
    for i in linspace {
        rcdf.total += wavefunction.radial(i).powi(2);
        rcdf.length += 1;
        rcdf.points.append(&mut vec![CDFEntry {
            frac: rcdf.total,
            x: i,
        }]);
    }

    rcdf.inverse_transform(f);

    return 1.;
}

#[cfg(test)]
mod test {
    use super::Wavefunction;
    use super::factorial;
    use crate::wavefunction::sample_wf_pdf;

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
    #[test]
    fn test_sampler() {
        sample_wf_pdf(1, 0, 0, 5., 100);
    }
}
