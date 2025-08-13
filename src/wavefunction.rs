use std::f64::consts::PI;

use num::complex::ComplexFloat;
use scilib::math::polynomial::Poly;

fn factorial(n: i32) -> i32 {
    if n == 0 {
        return 1;
    }
    return factorial(n - 1) * n;
}

fn radial(n: i32, l: i32, r: f64, a0: f64) -> f64 {
    let n_f: f64 = n as f64;
    let l_f: f64 = l as f64;
    let p: f64 = 2.0 * r / (n_f * a0);

    let k = f64::sqrt(
        ((2.0 / n_f * a0).powi(3) * (factorial(n - l - 1) as f64))
            / (2.0 * n_f * (factorial(n + l) as f64)),
    );

    let lag = Poly::laguerre((n - l - 1).try_into().unwrap(), 2.0 * l_f + 1.0);

    return lag.compute(p) * (k * (-p / 2.0).exp() * (p.powf(l_f)));
}

fn angular(l: i32, m: i32, theta: f64, phi: f64) -> f64 {
    let leg_poly: f64 =
        Poly::gen_legendre(l.try_into().unwrap(), m.try_into().unwrap()).compute(theta.cos());
    let neg: f64 = if m % 2 == 0 { 1.0 } else { -1.0 };
    let k = (neg)
        * (((2 * l + 1) * factorial(l - (m.abs()))) as f64
            / (4.0 * PI * factorial(l + m.abs()) as f64))
            .sqrt();

    let complex_angle = num::complex::Complex::new(0.0, m as f64 * phi).exp();
    return k * leg_poly * complex_angle.re();
}

pub fn sample_wavefunction_prob(
    n: i32,
    l: i32,
    m: i32,
    a0: f64,
    r: f64,
    theta: f64,
    phi: f64,
) -> f64 {
    let rad = radial(n, l, r, a0);
    let ang = angular(m, l, theta, phi);

    let psi = rad * ang;
    return psi.powi(2);
}

#[cfg(test)]
mod test {
    use super::angular;
    use super::factorial;
    use super::radial;

    // unit tests for math I don't understand

    #[test]
    fn test_norm_radial() {
        // factorial
        let ans: [i32; 5] = [1, 1, 2, 24, 40320];
        let input: [i32; 5] = [0, 1, 2, 4, 8];
        for i in 1..5 {
            assert_eq!(factorial(input[i]), ans[i]);
        }

        // norm radial and laguerre stuff
        let mut diff: f64 = radial(2, 1, 1.0, 1.0) - 0.1238;
        assert_eq!(diff < 0.00001, diff > -0.00001);

        diff = radial(5, 1, 2.0, 3.0) - 0.14356;
        assert_eq!(diff < 0.00001, diff > -0.00001);

        diff = radial(5, 3, 2.0, 1.0) - 0.000984;
        assert_eq!(diff < 0.00001, diff > -0.00001);
    }

    #[test]
    fn test_angular() {
        let mut diff: f64 = angular(3, 1, 0.21, 2.0) + 0.10605;
        assert_eq!(diff < 0.001, diff > -0.001);

        diff = angular(4, 1, 2.0, 3.0) + 0.316835;
        assert_eq!(diff < 0.001, diff > -0.001);
    }
}
