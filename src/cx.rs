/*!
a complex number abstraction

Type `Cx` can use the `+`, `*`, and unary `-` operators; the type also
features constructors from Cartesian (rectangular) and polar coordinates,
and accessors to get _|z|_ and _𝜑(z)_.
*/

#![allow(clippy::from_over_into)]

use std::ops::{Add, Mul, Neg};

use ::serde_derive::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
#[serde(from = "[f64; 2]", into = "[f64; 2]")]
pub struct Cx {
    pub re: f64,
    pub im: f64,
}

impl Cx {
    pub fn rect(x: f64, y: f64) -> Cx {
        Cx { re: x, im: y }
    }

    pub fn polar(r: f64, t: f64) -> Cx {
        Cx {
            re: r * t.cos(),
            im: r * t.sin(),
        }
    }

    pub fn sqmod(&self) -> f64 {
        (self.re * self.re) + (self.im * self.im)
    }

    pub fn r(&self) -> f64 {
        self.sqmod().sqrt()
    }

    pub fn theta(&self) -> f64 {
        self.im.atan2(self.re)
    }
}

impl Add for Cx {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            re: self.re + other.re,
            im: self.im + other.im,
        }
    }
}

impl Mul for Cx {
    type Output = Self;

    fn mul(self, other: Self) -> Self::Output {
        Self {
            re: (self.re * other.re) - (self.im * other.im),
            im: (self.re * other.im) + (self.im * other.re),
        }
    }
}

impl Neg for Cx {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Cx {
            re: -self.re,
            im: -self.im,
        }
    }
}

impl From<[f64; 2]> for Cx {
    fn from(a: [f64; 2]) -> Cx {
        Cx { re: a[0], im: a[1] }
    }
}

impl Into<[f64; 2]> for Cx {
    fn into(self) -> [f64; 2] {
        [self.re, self.im]
    }
}
