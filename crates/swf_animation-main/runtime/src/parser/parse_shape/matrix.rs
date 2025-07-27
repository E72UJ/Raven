use glam::Mat4;
use serde::{Deserialize, Serialize};
use swf::{Fixed16, Point, PointDelta, Rectangle, Twips};

// TODO: Consider using portable SIMD when it's stable (https://doc.rust-lang.org/std/simd/index.html).

/// The transformation matrix used by Flash display objects.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Matrix {
    /// Serialized as `scale_x` in SWF files
    pub a: f32,

    /// Serialized as `rotate_skew_0` in SWF files
    pub b: f32,

    /// Serialized as `rotate_skew_1` in SWF files
    pub c: f32,

    /// Serialized as `scale_y` in SWF files
    pub d: f32,

    /// Serialized as `transform_x` in SWF files
    #[serde(skip)]
    pub tx: Twips,

    /// Serialized as `transform_y` in SWF files
    #[serde(skip)]
    pub ty: Twips,
}

// TODO: 手动实现序列化和反序列化

impl Matrix {
    pub const IDENTITY: Self = Self {
        a: 1.0,
        c: 0.0,
        tx: Twips::ZERO,
        b: 0.0,
        d: 1.0,
        ty: Twips::ZERO,
    };

    pub const ZERO: Self = Self {
        a: 0.0,
        c: 0.0,
        tx: Twips::ZERO,
        b: 0.0,
        d: 0.0,
        ty: Twips::ZERO,
    };

    pub const TWIPS_TO_PIXELS: Self = Self {
        a: 1.0 / Twips::TWIPS_PER_PIXEL as f32,
        c: 0.0,
        tx: Twips::ZERO,
        b: 0.0,
        d: 1.0 / Twips::TWIPS_PER_PIXEL as f32,
        ty: Twips::ZERO,
    };

    pub const PIXELS_TO_TWIPS: Self = Self {
        a: Twips::TWIPS_PER_PIXEL as f32,
        c: 0.0,
        tx: Twips::ZERO,
        b: 0.0,
        d: Twips::TWIPS_PER_PIXEL as f32,
        ty: Twips::ZERO,
    };

    pub const fn scale(scale_x: f32, scale_y: f32) -> Self {
        Self {
            a: scale_x,
            c: 0.0,
            tx: Twips::ZERO,
            b: 0.0,
            d: scale_y,
            ty: Twips::ZERO,
        }
    }

    pub fn rotate(angle: f32) -> Self {
        Self {
            a: angle.cos(),
            c: -angle.sin(),
            tx: Twips::ZERO,
            b: angle.sin(),
            d: angle.cos(),
            ty: Twips::ZERO,
        }
    }

    pub fn translate(x: Twips, y: Twips) -> Self {
        Self {
            a: 1.0,
            c: 0.0,
            tx: x,
            b: 0.0,
            d: 1.0,
            ty: y,
        }
    }

    pub fn create_box(scale_x: f32, scale_y: f32, translate_x: Twips, translate_y: Twips) -> Self {
        Self {
            a: scale_x,
            c: 0.0,
            tx: translate_x,
            b: 0.0,
            d: scale_y,
            ty: translate_y,
        }
    }

    pub fn create_box_with_rotation(
        scale_x: f32,
        scale_y: f32,
        rotation: f32,
        translate_x: Twips,
        translate_y: Twips,
    ) -> Self {
        Self {
            a: rotation.cos() * scale_x,
            c: -rotation.sin() * scale_x,
            tx: translate_x,
            b: rotation.sin() * scale_y,
            d: rotation.cos() * scale_y,
            ty: translate_y,
        }
    }

    pub fn create_gradient_box(
        width: f32,
        height: f32,
        rotation: f32,
        translate_x: Twips,
        translate_y: Twips,
    ) -> Self {
        Self::create_box_with_rotation(
            width / 1638.4,
            height / 1638.4,
            rotation,
            translate_x + Twips::from_pixels((width / 2.0) as f64),
            translate_y + Twips::from_pixels((height / 2.0) as f64),
        )
    }

    #[inline]
    pub fn determinant(&self) -> f32 {
        self.a * self.d - self.b * self.c
    }

    #[inline]
    pub fn inverse(&self) -> Option<Self> {
        let (tx, ty) = (self.tx.get() as f32, self.ty.get() as f32);
        let det = self.determinant();
        if det.abs() > f32::EPSILON {
            let a = self.d / det;
            let b = self.b / -det;
            let c = self.c / -det;
            let d = self.a / det;
            let (out_tx, out_ty) = (
                round_to_i32((self.d * tx - self.c * ty) / -det),
                round_to_i32((self.b * tx - self.a * ty) / det),
            );
            Some(Matrix {
                a,
                b,
                c,
                d,
                tx: Twips::new(out_tx),
                ty: Twips::new(out_ty),
            })
        } else {
            None
        }
    }
}

impl std::ops::Mul for Matrix {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        let (rhs_tx, rhs_ty) = (rhs.tx.get() as f32, rhs.ty.get() as f32);
        let (out_tx, out_ty) = (
            round_to_i32(self.a * rhs_tx + self.c * rhs_ty).wrapping_add(self.tx.get()),
            round_to_i32(self.b * rhs_tx + self.d * rhs_ty).wrapping_add(self.ty.get()),
        );
        Matrix {
            a: self.a * rhs.a + self.c * rhs.b,
            b: self.b * rhs.a + self.d * rhs.b,
            c: self.a * rhs.c + self.c * rhs.d,
            d: self.b * rhs.c + self.d * rhs.d,
            tx: Twips::new(out_tx),
            ty: Twips::new(out_ty),
        }
    }
}

impl std::ops::Mul<Point<Twips>> for Matrix {
    type Output = Point<Twips>;

    fn mul(self, point: Point<Twips>) -> Point<Twips> {
        let x = point.x.get() as f32;
        let y = point.y.get() as f32;
        let out_x = Twips::new(round_to_i32(self.a * x + self.c * y).wrapping_add(self.tx.get()));
        let out_y = Twips::new(round_to_i32(self.b * x + self.d * y).wrapping_add(self.ty.get()));
        Point::new(out_x, out_y)
    }
}

impl std::ops::Mul<PointDelta<Twips>> for Matrix {
    type Output = PointDelta<Twips>;

    fn mul(self, delta: PointDelta<Twips>) -> PointDelta<Twips> {
        let dx = delta.dx.get() as f32;
        let dy = delta.dy.get() as f32;
        let out_dx = Twips::new(round_to_i32(self.a * dx + self.c * dy));
        let out_dy = Twips::new(round_to_i32(self.b * dx + self.d * dy));
        PointDelta::new(out_dx, out_dy)
    }
}

impl std::ops::Mul<Rectangle<Twips>> for Matrix {
    type Output = Rectangle<Twips>;

    fn mul(self, rhs: Rectangle<Twips>) -> Self::Output {
        if !rhs.is_valid() {
            return Default::default();
        }

        let p0 = self * Point::new(rhs.x_min, rhs.y_min);
        let p1 = self * Point::new(rhs.x_min, rhs.y_max);
        let p2 = self * Point::new(rhs.x_max, rhs.y_min);
        let p3 = self * Point::new(rhs.x_max, rhs.y_max);
        Rectangle {
            x_min: p0.x.min(p1.x).min(p2.x).min(p3.x),
            x_max: p0.x.max(p1.x).max(p2.x).max(p3.x),
            y_min: p0.y.min(p1.y).min(p2.y).min(p3.y),
            y_max: p0.y.max(p1.y).max(p2.y).max(p3.y),
        }
    }
}

impl Default for Matrix {
    fn default() -> Matrix {
        Matrix::IDENTITY
    }
}

impl std::ops::MulAssign for Matrix {
    fn mul_assign(&mut self, rhs: Self) {
        let (rhs_tx, rhs_ty) = (rhs.tx.get() as f32, rhs.ty.get() as f32);
        let (out_tx, out_ty) = (
            round_to_i32(self.a * rhs_tx + self.c * rhs_ty) + self.tx.get(),
            round_to_i32(self.b * rhs_tx + self.d * rhs_ty) + self.ty.get(),
        );
        *self = Matrix {
            a: self.a * rhs.a + self.c * rhs.b,
            b: self.b * rhs.a + self.d * rhs.b,
            c: self.a * rhs.c + self.c * rhs.d,
            d: self.b * rhs.c + self.d * rhs.d,
            tx: Twips::new(out_tx),
            ty: Twips::new(out_ty),
        }
    }
}

impl From<swf::Matrix> for Matrix {
    fn from(matrix: swf::Matrix) -> Self {
        Self {
            a: matrix.a.to_f32(),
            b: matrix.b.to_f32(),
            c: matrix.c.to_f32(),
            d: matrix.d.to_f32(),
            tx: matrix.tx,
            ty: matrix.ty,
        }
    }
}

impl From<Matrix> for swf::Matrix {
    fn from(matrix: Matrix) -> Self {
        Self {
            a: Fixed16::from_f32(matrix.a),
            b: Fixed16::from_f32(matrix.b),
            c: Fixed16::from_f32(matrix.c),
            d: Fixed16::from_f32(matrix.d),
            tx: matrix.tx,
            ty: matrix.ty,
        }
    }
}

/// Implements the IEEE-754 "Round to nearest, ties to even" rounding rule.
/// (e.g., both 1.5 and 2.5 will round to 2).
/// This is the rounding method used by Flash for the above transforms.
/// This also clamps out-of-range values and NaN to `i32::MIN`.
fn round_to_i32(f: f32) -> i32 {
    if f.is_finite() {
        if f < 2_147_483_648.0_f32 {
            f.round_ties_even() as i32
        } else {
            // Out-of-range clamps to MIN.
            i32::MIN
        }
    } else {
        // NaN/Infinity goes to 0.
        0
    }
}

impl From<Matrix> for Mat4 {
    fn from(value: Matrix) -> Self {
        Mat4::from_cols_array_2d(&[
            [value.a, value.b, 0.0, 0.0],
            [value.c, value.d, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [
                value.tx.to_pixels() as f32,
                value.ty.to_pixels() as f32,
                0.0,
                1.0,
            ],
        ])
    }
}
