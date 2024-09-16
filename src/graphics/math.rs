#![allow(unused)]

pub fn div_round_up(num: u32, div: u32) -> u32 {
    (num + div - 1) / div
}

#[repr(transparent)]
#[derive(Clone, Copy, Default, Debug)]
pub struct Vec4(pub [f32; 4]);

impl Vec4 {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self([x, y, z, w])
    }

    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }

    pub fn x(&self) -> f32 {
        self[0]
    }

    pub fn y(&self) -> f32 {
        self[1]
    }

    pub fn z(&self) -> f32 {
        self[2]
    }

    pub fn w(&self) -> f32 {
        self[3]
    }

    pub fn dot(&self, other: &Self) -> f32 {
        self.x() * other.x() + self.y() * other.y() + self.z() * other.z() + self.w() * other.w()
    }
}

impl std::ops::Index<usize> for Vec4 {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl std::ops::IndexMut<usize> for Vec4 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl std::fmt::Display for Vec4 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {} {}", self.x(), self.y(), self.z(), self.w())
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Default, Debug)]
pub struct Matrix(pub [Vec4; 4]);

impl Matrix {
    pub fn new(col0: Vec4, col1: Vec4, col2: Vec4, col3: Vec4) -> Self {
        Self([col0, col1, col2, col3])
    }

    pub fn zero() -> Self {
        Self::new(Vec4::zero(), Vec4::zero(), Vec4::zero(), Vec4::zero())
    }

    pub fn identity() -> Self {
        Self::new(
            Vec4::new(1.0, 0.0, 0.0, 0.0),
            Vec4::new(0.0, 1.0, 0.0, 0.0),
            Vec4::new(0.0, 0.0, 1.0, 0.0),
            Vec4::new(0.0, 0.0, 0.0, 1.0),
        )
    }

    pub fn rot_x(angle: f32) -> Self {
        let (s, c) = angle.sin_cos();

        Self::new(
            Vec4::new(1.0, 0.0, 0.0, 0.0),
            Vec4::new(0.0, c, -s, 0.0),
            Vec4::new(0.0, s, c, 0.0),
            Vec4::new(0.0, 0.0, 0.0, 1.0),
        )
    }

    pub fn rot_y(angle: f32) -> Self {
        let (s, c) = angle.sin_cos();

        Self::new(
            Vec4::new(c, 0.0, s, 0.0),
            Vec4::new(0.0, 1.0, 0.0, 0.0),
            Vec4::new(-s, 0.0, c, 0.0),
            Vec4::new(0.0, 0.0, 0.0, 1.0),
        )
    }

    pub fn rot_z(angle: f32) -> Self {
        let (s, c) = angle.sin_cos();

        Self::new(
            Vec4::new(c, -s, 0.0, 0.0),
            Vec4::new(s, c, 0.0, 0.0),
            Vec4::new(0.0, 0.0, 1.0, 0.0),
            Vec4::new(0.0, 0.0, 0.0, 1.0),
        )
    }

    pub fn scale(x: f32, y: f32, z: f32) -> Self {
        Self::new(
            Vec4::new(x, 0.0, 0.0, 0.0),
            Vec4::new(0.0, y, 0.0, 0.0),
            Vec4::new(0.0, 0.0, z, 0.0),
            Vec4::new(0.0, 0.0, 0.0, 1.0),
        )
    }

    pub fn translate(x: f32, y: f32, z: f32) -> Self {
        Self::new(
            Vec4::new(1.0, 0.0, 0.0, x),
            Vec4::new(0.0, 1.0, 0.0, y),
            Vec4::new(0.0, 0.0, 1.0, z),
            Vec4::new(0.0, 0.0, 0.0, 1.0),
        )
    }

    pub fn row(&self, row: usize) -> Vec4 {
        Vec4::new(
            self[(row, 0)],
            self[(row, 1)],
            self[(row, 2)],
            self[(row, 3)],
        )
    }

    pub fn col(&self, col: usize) -> &Vec4 {
        &self.0[col]
    }

    pub fn mul(&self, other: &Self) -> Self {
        let mut m = Self::zero();

        for i in 0..4 {
            let row = self.row(i);

            for j in 0..4 {
                m[(i, j)] = row.dot(other.col(j));
            }
        }

        m
    }

    pub fn as_4x3(&self) -> [f32; 12] {
        [
            self.0[0][0],
            self.0[0][1],
            self.0[0][2],
            self.0[0][3],
            self.0[1][0],
            self.0[1][1],
            self.0[1][2],
            self.0[1][3],
            self.0[2][0],
            self.0[2][1],
            self.0[2][2],
            self.0[2][3],
        ]
    }
}

impl std::ops::Index<(usize, usize)> for Matrix {
    type Output = f32;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.0[index.1][index.0]
    }
}

impl std::ops::IndexMut<(usize, usize)> for Matrix {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        &mut self.0[index.1][index.0]
    }
}

impl std::fmt::Display for Matrix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}\n {}\n {}\n {}]",
            self.row(0),
            self.row(1),
            self.row(2),
            self.row(3),
        )
    }
}
