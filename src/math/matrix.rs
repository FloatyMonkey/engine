use super::num::{Float, FloatOps, Number, SignedNumber};
use std::ops::{Mul, MulAssign, Index, IndexMut, Add, AddAssign, Div, DivAssign, Sub, SubAssign, Neg};
use super::unit::Unit;

#[repr(C)]
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Matrix<T, const R: usize, const C: usize> {
	pub data: [[T; C]; R],
}

impl<T: Number, const R: usize, const C: usize> Matrix<T, R, C> {
	pub const ZERO: Self = Self::splat(T::ZERO);
	pub const ONE: Self = Self::splat(T::ONE);

	pub const fn splat(value: T) -> Self {
		Self { data: [[value; C]; R] }
	}
}

pub type Vector<T, const R: usize> = Matrix<T, R, 1>;

pub type Vector2<T> = Vector<T, 2>;
pub type Vector3<T> = Vector<T, 3>;
pub type Vector4<T> = Vector<T, 4>;

pub type Matrix2<T> = Matrix<T, 2, 2>;
pub type Matrix3<T> = Matrix<T, 3, 3>;
pub type Matrix4<T> = Matrix<T, 4, 4>;

impl<T: Number> Vector2<T> {
	pub const X: Unit<Self> = Unit::new_unchecked(Self::new(T::ONE, T::ZERO));
	pub const Y: Unit<Self> = Unit::new_unchecked(Self::new(T::ZERO, T::ONE));

	pub const fn new(x: T, y: T) -> Self {
		Self { data: [[x], [y]] }
	}

	pub fn extend(&self, z: T) -> Vector3<T> {
		Vector3::new(self.x, self.y, z)
	}

	pub fn cross(&self, rhs: Self) -> T {
		self.x * rhs.y - self.y * rhs.x
	}
}

impl<T: Number> Vector3<T> {
	pub const X: Unit<Self> = Unit::new_unchecked(Self::new(T::ONE, T::ZERO, T::ZERO));
	pub const Y: Unit<Self> = Unit::new_unchecked(Self::new(T::ZERO, T::ONE, T::ZERO));
	pub const Z: Unit<Self> = Unit::new_unchecked(Self::new(T::ZERO, T::ZERO, T::ONE));

	pub const fn new(x: T, y: T, z: T) -> Self {
		Self { data: [[x], [y], [z]] }
	}

	pub fn truncate(&self) -> Vector2<T> {
		Vector2::new(self.x, self.y)
	}

	pub fn extend(&self, w: T) -> Vector4<T> {
		Vector4::new(self.x, self.y, self.z, w)
	}

	pub fn cross(&self, rhs: Self) -> Self {
		Self::new(
			self.y * rhs.z - self.z * rhs.y,
			self.z * rhs.x - self.x * rhs.z,
			self.x * rhs.y - self.y * rhs.x,
		)
	}
}

impl<T: Number> Vector4<T> {
	pub const X: Unit<Self> = Unit::new_unchecked(Self::new(T::ONE, T::ZERO, T::ZERO, T::ZERO));
	pub const Y: Unit<Self> = Unit::new_unchecked(Self::new(T::ZERO, T::ONE, T::ZERO, T::ZERO));
	pub const Z: Unit<Self> = Unit::new_unchecked(Self::new(T::ZERO, T::ZERO, T::ONE, T::ZERO));
	pub const W: Unit<Self> = Unit::new_unchecked(Self::new(T::ZERO, T::ZERO, T::ZERO, T::ONE));

	pub const fn new(x: T, y: T, z: T, w: T) -> Self {
		Self { data: [[x], [y], [z], [w]] }
	}

	pub fn truncate(&self) -> Vector3<T> {
		Vector3::new(self.x, self.y, self.z)
	}
}

impl<T: Float + FloatOps<T>, const N: usize> Vector<T, N> {
	#[inline(always)]
	pub fn dot(&self, rhs: Self) -> T {
		let mut result = T::ZERO;
		for i in 0..N {
			result += self[i] * rhs[i];
		}
		result
	}

	pub fn normalize(&self) -> Unit<Self> {
		Unit::new_unchecked(*self / self.length())
	}

	pub fn length_sq(&self) -> T {
		self.dot(*self)
	}

	pub fn length(&self) -> T {
		self.length_sq().sqrt()
	}

	pub fn distance_sq(&self, rhs: Self) -> T {
		(*self - rhs).length_sq()
	}

	pub fn distance(&self, rhs: Self) -> T {
		(*self - rhs).length()
	}

	/// Projects this vector onto another vector.
	pub fn project_onto(&self, onto: Unit<Self>) -> Self {
		*onto * self.dot(*onto)
	}
}

impl<T: Number, const R: usize, const C: usize, const CR: usize> Mul<Matrix<T, CR, C>> for Matrix<T, R, CR> {
	type Output = Matrix<T, R, C>;

	fn mul(self, rhs: Matrix<T, CR, C>) -> Self::Output {
		let mut result = Matrix::ZERO;
		for row in 0..R {
			for col in 0..C {
				for i in 0..CR {
					result[(row, col)] += self[(row, i)] * rhs[(i, col)];
				}
			}
		}
		result
	}
}

impl<T: Number, const R: usize, const C: usize> MulAssign<T> for Matrix<T, R, C> {
	fn mul_assign(&mut self, rhs: T) {
		for i in 0..(R * C) {
			self[i] *= rhs;
		}
	}
}

impl<T: Number, const R: usize, const C: usize> Mul<T> for Matrix<T, R, C> {
	type Output = Self;

	fn mul(self, rhs: T) -> Self::Output {
		let mut result = self;
		result *= rhs;
		result
	}
}

impl<T: Number, const R: usize, const C: usize> DivAssign<T> for Matrix<T, R, C> {
	fn div_assign(&mut self, rhs: T) {
		for i in 0..(R * C) {
			self[i] /= rhs;
		}
	}
}

impl<T: Number, const R: usize, const C: usize> Div<T> for Matrix<T, R, C> {
	type Output = Self;

	fn div(self, rhs: T) -> Self::Output {
		let mut result = self;
		result /= rhs;
		result
	}
}

impl<T: Number, const R: usize, const C: usize> AddAssign for Matrix<T, R, C> {
	fn add_assign(&mut self, rhs: Self) {
		for i in 0..(R * C) {
			self[i] += rhs[i];
		}
	}
}

impl<T: Number, const R: usize, const C: usize> Add for Matrix<T, R, C> {
	type Output = Self;

	fn add(self, rhs: Self) -> Self::Output {
		let mut result = self;
		result += rhs;
		result
	}
}

impl<T: Number, const R: usize, const C: usize> SubAssign for Matrix<T, R, C> {
	fn sub_assign(&mut self, rhs: Self) {
		for i in 0..(R * C) {
			self[i] -= rhs[i];
		}
	}
}

impl<T: Number, const R: usize, const C: usize> Sub for Matrix<T, R, C> {
	type Output = Self;

	fn sub(self, rhs: Self) -> Self::Output {
		let mut result = self;
		result -= rhs;
		result
	}
}

impl<T: SignedNumber, const R: usize, const C: usize> Neg for Matrix<T, R, C> {
	type Output = Self;

	fn neg(self) -> Self::Output {
		let mut result = Matrix::ZERO;
		for i in 0..(R * C) {
			result[i] = -self[i];
		}
		result
	}
}

pub struct XY<T> {
	pub x: T,
	pub y: T,
}

pub struct XYZ<T> {
	pub x: T,
	pub y: T,
	pub z: T,
}

pub struct XYZW<T> {
	pub x: T,
	pub y: T,
	pub z: T,
	pub w: T,
}

impl<T> std::ops::Deref for Vector2<T> {
	type Target = XY<T>;

	#[inline]
	fn deref(&self) -> &Self::Target {
		unsafe { &*(self as *const Self as *const Self::Target) }
	}
}

impl<T> std::ops::DerefMut for Vector2<T> {
	#[inline]
	fn deref_mut(&mut self) -> &mut Self::Target {
		unsafe { &mut *(self as *mut Self as *mut Self::Target) }
	}
}

impl<T> std::ops::Deref for Vector3<T> {
	type Target = XYZ<T>;

	#[inline]
	fn deref(&self) -> &Self::Target {
		unsafe { &*(self as *const Self as *const Self::Target) }
	}
}

impl<T> std::ops::DerefMut for Vector3<T> {
	#[inline]
	fn deref_mut(&mut self) -> &mut Self::Target {
		unsafe { &mut *(self as *mut Self as *mut Self::Target) }
	}
}

impl<T> std::ops::Deref for Vector4<T> {
	type Target = XYZW<T>;

	#[inline]
	fn deref(&self) -> &Self::Target {
		unsafe { &*(self as *const Self as *const Self::Target) }
	}
}

impl<T> std::ops::DerefMut for Vector4<T> {
	#[inline]
	fn deref_mut(&mut self) -> &mut Self::Target {
		unsafe { &mut *(self as *mut Self as *mut Self::Target) }
	}
}

impl<T, const R: usize, const C: usize> Index<usize> for Matrix<T, R, C> {
	type Output = T;

	fn index(&self, index: usize) -> &Self::Output {
		&self.as_slice()[index]
	}
}

impl<T, const R: usize, const C: usize> IndexMut<usize> for Matrix<T, R, C> {
	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		&mut self.as_mut_slice()[index]
	}
}

impl<T, const R: usize, const C: usize> Index<(usize, usize)> for Matrix<T, R, C> {
	type Output = T;

	fn index(&self, index: (usize, usize)) -> &Self::Output {
		&self.data[index.0][index.1]
	}
}

impl<T, const R: usize, const C: usize> IndexMut<(usize, usize)> for Matrix<T, R, C> {
	fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
		&mut self.data[index.0][index.1]
	}
}

pub fn perspective(fov: f32, aspect_ratio: f32, near_clip: f32, far_clip: f32) -> Matrix4<f32> {
	let inv_y = 1.0 / (fov / 2.0).tan();
	let inv_x = inv_y / aspect_ratio;
	let inv_z = far_clip / (near_clip - far_clip);

	Matrix4::from_array([
		inv_x, 0.0, 0.0, 0.0,
		0.0, inv_y, 0.0, 0.0,
		0.0, 0.0, inv_z, near_clip * inv_z,
		0.0, 0.0, -1.0, 0.0,
	])
}

impl<T, const R: usize, const C: usize> Matrix<T, R, C> {
	pub fn as_slice(&self) -> &[T] {
		unsafe {
			std::slice::from_raw_parts(self.data.as_ptr() as *const T, R * C)
		}
	}

	pub fn as_mut_slice(&mut self) -> &mut [T] {
		unsafe {
			std::slice::from_raw_parts_mut(self.data.as_mut_ptr() as *mut T, R * C)
		}
	}
}

impl Matrix3<f32> {
	/// Returns the determinant of this matrix.
	pub fn det(&self) -> f32 {
		let m = |r: usize, c: usize| self.data[r][c];

		m(0, 0) * (m(1, 1) * m(2, 2) - m(1, 2) * m(2, 1)) +
		m(0, 1) * (m(1, 2) * m(2, 0) - m(1, 0) * m(2, 2)) +
		m(0, 2) * (m(1, 0) * m(2, 1) - m(1, 1) * m(2, 0))
	}

	/// Returns the inverse of this matrix.
	pub fn inv(&self) -> Self {
		let m = |r: usize, c: usize| self.data[r][c];

		let inv_det = 1.0 / self.det();

		let m11 = m(1, 1) * m(2, 2) - m(1, 2) * m(2, 1);
		let m12 = m(0, 2) * m(2, 1) - m(0, 1) * m(2, 2);
		let m13 = m(0, 1) * m(1, 2) - m(0, 2) * m(1, 1);

		let m21 = m(1, 2) * m(2, 0) - m(1, 0) * m(2, 2);
		let m22 = m(0, 0) * m(2, 2) - m(0, 2) * m(2, 0);
		let m23 = m(0, 2) * m(1, 0) - m(0, 0) * m(1, 2);

		let m31 = m(1, 0) * m(2, 1) - m(1, 1) * m(2, 0);
		let m32 = m(0, 1) * m(2, 0) - m(0, 0) * m(2, 1);
		let m33 = m(0, 0) * m(1, 1) - m(0, 1) * m(1, 0);

		Self { data: [
			[m11 * inv_det, m12 * inv_det, m13 * inv_det],
			[m21 * inv_det, m22 * inv_det, m23 * inv_det],
			[m31 * inv_det, m32 * inv_det, m33 * inv_det],
		]}
	}
}

impl Matrix4<f32> {
	pub const IDENTITY: Self = Self {
		data: [
			[1.0, 0.0, 0.0, 0.0],
			[0.0, 1.0, 0.0, 0.0],
			[0.0, 0.0, 1.0, 0.0],
			[0.0, 0.0, 0.0, 1.0],
		]
	};

	/// Returns the determinant of this matrix.
	pub fn det(&self) -> f32 {
		let m = |r: usize, c: usize| self.data[r][c];

		m(0, 0) * m(1, 1) * m(2, 2) * m(3, 3) + m(0, 0) * m(1, 2) * m(2, 3) * m(3, 1) +
		m(0, 0) * m(1, 3) * m(2, 1) * m(3, 2) + m(0, 1) * m(1, 0) * m(2, 3) * m(3, 2) +
		m(0, 1) * m(1, 2) * m(2, 0) * m(3, 3) + m(0, 1) * m(1, 3) * m(2, 2) * m(3, 0) +
		m(0, 2) * m(1, 0) * m(2, 1) * m(3, 3) + m(0, 2) * m(1, 1) * m(2, 3) * m(3, 0) +
		m(0, 2) * m(1, 3) * m(2, 0) * m(3, 1) + m(0, 3) * m(1, 0) * m(2, 2) * m(3, 1) +
		m(0, 3) * m(1, 1) * m(2, 0) * m(3, 2) + m(0, 3) * m(1, 2) * m(2, 1) * m(3, 0) -
		m(0, 0) * m(1, 1) * m(2, 3) * m(3, 2) - m(0, 0) * m(1, 2) * m(2, 1) * m(3, 3) -
		m(0, 0) * m(1, 3) * m(2, 2) * m(3, 1) - m(0, 1) * m(1, 0) * m(2, 2) * m(3, 3) -
		m(0, 1) * m(1, 2) * m(2, 3) * m(3, 0) - m(0, 1) * m(1, 3) * m(2, 0) * m(3, 2) -
		m(0, 2) * m(1, 0) * m(2, 3) * m(3, 1) - m(0, 2) * m(1, 1) * m(2, 0) * m(3, 3) -
		m(0, 2) * m(1, 3) * m(2, 1) * m(3, 0) - m(0, 3) * m(1, 0) * m(2, 1) * m(3, 2) -
		m(0, 3) * m(1, 1) * m(2, 2) * m(3, 0) - m(0, 3) * m(1, 2) * m(2, 0) * m(3, 1)
	}

	/// Returns the inverse of this matrix.
	pub fn inv(&self) -> Self {
		let m = |r: usize, c: usize| self.data[r][c];

		let inv_det = 1.0 / self.det();

		let m11 = m(1, 1) * m(2, 2) * m(3, 3) + m(1, 2) * m(2, 3) * m(3, 1) + m(1, 3) * m(2, 1) * m(3, 2) - m(1, 1) * m(2, 3) * m(3, 2) - m(1, 2) * m(2, 1) * m(3, 3) - m(1, 3) * m(2, 2) * m(3, 1);
		let m12 = m(0, 1) * m(2, 3) * m(3, 2) + m(0, 2) * m(2, 1) * m(3, 3) + m(0, 3) * m(2, 2) * m(3, 1) - m(0, 1) * m(2, 2) * m(3, 3) - m(0, 2) * m(2, 3) * m(3, 1) - m(0, 3) * m(2, 1) * m(3, 2);
		let m13 = m(0, 1) * m(1, 2) * m(3, 3) + m(0, 2) * m(1, 3) * m(3, 1) + m(0, 3) * m(1, 1) * m(3, 2) - m(0, 1) * m(1, 3) * m(3, 2) - m(0, 2) * m(1, 1) * m(3, 3) - m(0, 3) * m(1, 2) * m(3, 1);
		let m14 = m(0, 1) * m(1, 3) * m(2, 2) + m(0, 2) * m(1, 1) * m(2, 3) + m(0, 3) * m(1, 2) * m(2, 1) - m(0, 1) * m(1, 2) * m(2, 3) - m(0, 2) * m(1, 3) * m(2, 1) - m(0, 3) * m(1, 1) * m(2, 2);
		
		let m21 = m(1, 0) * m(2, 3) * m(3, 2) + m(1, 2) * m(2, 0) * m(3, 3) + m(1, 3) * m(2, 2) * m(3, 0) - m(1, 0) * m(2, 2) * m(3, 3) - m(1, 2) * m(2, 3) * m(3, 0) - m(1, 3) * m(2, 0) * m(3, 2);
		let m22 = m(0, 0) * m(2, 2) * m(3, 3) + m(0, 2) * m(2, 3) * m(3, 0) + m(0, 3) * m(2, 0) * m(3, 2) - m(0, 0) * m(2, 3) * m(3, 2) - m(0, 2) * m(2, 0) * m(3, 3) - m(0, 3) * m(2, 2) * m(3, 0);
		let m23 = m(0, 0) * m(1, 3) * m(3, 2) + m(0, 2) * m(1, 0) * m(3, 3) + m(0, 3) * m(1, 2) * m(3, 0) - m(0, 0) * m(1, 2) * m(3, 3) - m(0, 2) * m(1, 3) * m(3, 0) - m(0, 3) * m(1, 0) * m(3, 2);
		let m24 = m(0, 0) * m(1, 2) * m(2, 3) + m(0, 2) * m(1, 3) * m(2, 0) + m(0, 3) * m(1, 0) * m(2, 2) - m(0, 0) * m(1, 3) * m(2, 2) - m(0, 2) * m(1, 0) * m(2, 3) - m(0, 3) * m(1, 2) * m(2, 0);
		
		let m31 = m(1, 0) * m(2, 1) * m(3, 3) + m(1, 1) * m(2, 3) * m(3, 0) + m(1, 3) * m(2, 0) * m(3, 1) - m(1, 0) * m(2, 3) * m(3, 1) - m(1, 1) * m(2, 0) * m(3, 3) - m(1, 3) * m(2, 1) * m(3, 0);
		let m32 = m(0, 0) * m(2, 3) * m(3, 1) + m(0, 1) * m(2, 0) * m(3, 3) + m(0, 3) * m(2, 1) * m(3, 0) - m(0, 0) * m(2, 1) * m(3, 3) - m(0, 1) * m(2, 3) * m(3, 0) - m(0, 3) * m(2, 0) * m(3, 1);
		let m33 = m(0, 0) * m(1, 1) * m(3, 3) + m(0, 1) * m(1, 3) * m(3, 0) + m(0, 3) * m(1, 0) * m(3, 1) - m(0, 0) * m(1, 3) * m(3, 1) - m(0, 1) * m(1, 0) * m(3, 3) - m(0, 3) * m(1, 1) * m(3, 0);
		let m34 = m(0, 0) * m(1, 3) * m(2, 1) + m(0, 1) * m(1, 0) * m(2, 3) + m(0, 3) * m(1, 1) * m(2, 0) - m(0, 0) * m(1, 1) * m(2, 3) - m(0, 1) * m(1, 3) * m(2, 0) - m(0, 3) * m(1, 0) * m(2, 1);
		
		let m41 = m(1, 0) * m(2, 2) * m(3, 1) + m(1, 1) * m(2, 0) * m(3, 2) + m(1, 2) * m(2, 1) * m(3, 0) - m(1, 0) * m(2, 1) * m(3, 2) - m(1, 1) * m(2, 2) * m(3, 0) - m(1, 2) * m(2, 0) * m(3, 1);
		let m42 = m(0, 0) * m(2, 1) * m(3, 2) + m(0, 1) * m(2, 2) * m(3, 0) + m(0, 2) * m(2, 0) * m(3, 1) - m(0, 0) * m(2, 2) * m(3, 1) - m(0, 1) * m(2, 0) * m(3, 2) - m(0, 2) * m(2, 1) * m(3, 0);
		let m43 = m(0, 0) * m(1, 2) * m(3, 1) + m(0, 1) * m(1, 0) * m(3, 2) + m(0, 2) * m(1, 1) * m(3, 0) - m(0, 0) * m(1, 1) * m(3, 2) - m(0, 1) * m(1, 2) * m(3, 0) - m(0, 2) * m(1, 0) * m(3, 1);
		let m44 = m(0, 0) * m(1, 1) * m(2, 2) + m(0, 1) * m(1, 2) * m(2, 0) + m(0, 2) * m(1, 0) * m(2, 1) - m(0, 0) * m(1, 2) * m(2, 1) - m(0, 1) * m(1, 0) * m(2, 2) - m(0, 2) * m(1, 1) * m(2, 0);

		Self { data: [
			[m11 * inv_det, m12 * inv_det, m13 * inv_det, m14 * inv_det],
			[m21 * inv_det, m22 * inv_det, m23 * inv_det, m24 * inv_det],
			[m31 * inv_det, m32 * inv_det, m33 * inv_det, m34 * inv_det],
			[m41 * inv_det, m42 * inv_det, m43 * inv_det, m44 * inv_det],
		]}
	}
}

impl<T: Number, const R: usize, const C: usize> Matrix<T, R, C> {
	/// Component-wise multiplication.
	pub fn cmul(self, rhs: Self) -> Self {
		let mut result = Matrix::ZERO;
		for i in 0..(R * C) {
			result[i] = self[i] * rhs[i];
		}
		result
	}
}

impl<T: Number, const R: usize, const C: usize> Matrix<T, R, C> {
	pub fn transpose(&self) -> Matrix<T, C, R> {
		let mut result = Matrix::ZERO;
		for row in 0..R {
			for col in 0..C {
				result[(col, row)] = self[(row, col)];
			}
		}
		result
	}
}

impl<T: Float> Matrix3<T> {
	pub fn from_diagonal(diagonal: Vector3<T>) -> Self {
		Self { data: [
			[diagonal.x, T::ZERO, T::ZERO],
			[T::ZERO, diagonal.y, T::ZERO],
			[T::ZERO, T::ZERO, diagonal.z],
		]}
	}

	pub fn from_axes(x: Vector3<T>, y: Vector3<T>, z: Vector3<T>) -> Self {
		Self { data: [
			[x.x, y.x, z.x],
			[x.y, y.y, z.y],
			[x.z, y.z, z.z],
		]}
	}

	/// Creates a matrix from an array of 9 elements stored in row-major order.
	/// This allows the code to be formatted as if it were a 3x3 matrix.
	pub const fn from_array(array: [T; 9]) -> Self {
		Self { data: [
			[array[0], array[1], array[2]],
			[array[3], array[4], array[5]],
			[array[6], array[7], array[8]],
		]}
	}
}

impl<T: Float> Matrix4<T> {
	pub fn from_axes(x: Vector4<T>, y: Vector4<T>, z: Vector4<T>, w: Vector4<T>) -> Self {
		Self { data: [
			[x.x, y.x, z.x, w.x],
			[x.y, y.y, z.y, w.y],
			[x.z, y.z, z.z, w.z],
			[x.w, y.w, z.w, w.w],
		]}
	}

	/// Creates a matrix from an array of 16 elements stored in row-major order.
	/// This allows the code to be formatted as if it were a 4x4 matrix.
	pub const fn from_array(array: [T; 16]) -> Self {
		Self { data: [
			[array[ 0], array[ 1], array[ 2], array[ 3]],
			[array[ 4], array[ 5], array[ 6], array[ 7]],
			[array[ 8], array[ 9], array[10], array[11]],
			[array[12], array[13], array[14], array[15]],
		]}
	}
}

impl Matrix<f32, 3, 4> {
	pub const IDENTITY: Self = Self {
		data: [
			[1.0, 0.0, 0.0, 0.0],
			[0.0, 1.0, 0.0, 0.0],
			[0.0, 0.0, 1.0, 0.0],
		]
	};
}

impl<T: Copy> From<Matrix4<T>> for Matrix3<T> {
	fn from(m: Matrix4<T>) -> Self {
		Self { data: [
			[m[0], m[1], m[ 2]],
			[m[4], m[5], m[ 6]],
			[m[8], m[9], m[10]],
		]}
	}
}

impl<T: Copy> From<Matrix4<T>> for Matrix<T, 3, 4> {
	fn from(m: Matrix4<T>) -> Self {
		Self { data: [
			[m[0], m[1], m[ 2], m[ 3]],
			[m[4], m[5], m[ 6], m[ 7]],
			[m[8], m[9], m[10], m[11]],
		]}
	}
}
