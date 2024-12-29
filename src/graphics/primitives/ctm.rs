/*!
 * Current Transformation Matrix (CTM) operations
 *
 * This is the api for interacting with the PDF current transformation matrix.
 *
 * The CTM is a 4x4 matrix that is derived from 6 numbers. The matrix is used to transform the
 *
 * - x and y coordinates on the page
 * - the width and height of shapes
 * - the angle of rotation
*/
use std::{iter::Product, ops::Mul};

use lopdf::Object;
use tracing::debug;

use crate::{
    document::PdfResources,
    graphics::{OperationKeys, OperationWriter, PdfObjectType, PdfPosition},
    units::Pt,
    TuxPdfError,
};

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub enum CurTransMat {
    Position(PdfPosition<Pt>),
    Rotate(f32),
    Scale(Pt, Pt),
    Raw([f32; 6]),
    #[default]
    Identity,
}

impl Mul for CurTransMat {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        let matrix_a: CtmMatrix = self.into();
        let matrix_b: CtmMatrix = rhs.into();
        let result: CtmMatrix = matrix_a * matrix_b;
        result.into()
    }
}

impl Product for CurTransMat {
    fn product<I>(iter: I) -> Self
    where
        I: Iterator<Item = Self>,
    {
        let mut matrix: CtmMatrix = CurTransMat::Identity.into();
        for transform in iter {
            let matrix_b: CtmMatrix = transform.into();
            matrix = matrix * matrix_b;
        }
        matrix.into()
    }
}
impl From<CurTransMat> for CtmMatrix {
    fn from(value: CurTransMat) -> Self {
        let [a, b, c, d, e, f]: [f32; 6] = value.into();
        let matrix = [
            [a, b, 0.0, 0.0],
            [c, d, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [e, f, 0.0, 1.0],
        ];
        CtmMatrix(matrix)
    }
}
impl From<CtmMatrix> for CurTransMat {
    fn from(value: CtmMatrix) -> Self {
        let matrix = value.0;
        CurTransMat::Raw([
            matrix[0][0],
            matrix[0][1],
            matrix[1][0],
            matrix[1][1],
            matrix[3][0],
            matrix[3][1],
        ])
    }
}
impl From<CurTransMat> for [f32; 6] {
    fn from(value: CurTransMat) -> Self {
        match value {
            CurTransMat::Position(position) => {
                [1.0, 0.0, 0.0, 1.0, position.x.into(), position.y.into()]
            }
            CurTransMat::Rotate(angle) => {
                let rad = (360.0 - angle).to_radians();
                [rad.cos(), -rad.sin(), rad.sin(), rad.cos(), 0.0, 0.0]
            }
            CurTransMat::Scale(x, y) => [x.into(), 0.0, 0.0, y.into(), 0.0, 0.0],
            CurTransMat::Raw(raw) => raw,
            CurTransMat::Identity => [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
        }
    }
}
impl From<CurTransMat> for Vec<Object> {
    fn from(value: CurTransMat) -> Self {
        let matrix_nums: [f32; 6] = value.into();
        matrix_nums.iter().copied().map(Object::Real).collect()
    }
}

impl PdfObjectType for CurTransMat {
    fn write(self, _: &PdfResources, writer: &mut OperationWriter) -> Result<(), TuxPdfError> {
        let values: Vec<Object> = self.into();

        writer.add_operation(OperationKeys::CurrentTransformationMatrix, values);

        Ok(())
    }
}
impl PdfObjectType for Vec<CurTransMat> {
    #[inline(always)]
    fn write(
        self,
        resources: &PdfResources,
        writer: &mut OperationWriter,
    ) -> Result<(), TuxPdfError> {
        let matrix: CurTransMat = self.into_iter().product();
        debug!(?matrix, "CTM");
        matrix.write(resources, writer)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct CtmMatrix(pub [[f32; 4]; 4]);
impl Mul for CtmMatrix {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        let a = self.0;
        let b = rhs.0;
        let mut result = [[0.0f32; 4]; 4];
        for i in 0..4 {
            for j in 0..4 {
                result[i][j] =
                    a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j] + a[i][3] * b[3][j];
            }
        }
        CtmMatrix(result)
    }
}

#[cfg(test)]
mod tests {
    use crate::graphics::PdfPosition;

    use super::*;

    #[test]
    fn basic_ctm_combine() {
        let a = CurTransMat::Position(PdfPosition::new(Pt(10.0), Pt(10.0)));
        let b = CurTransMat::Rotate(90.0);
        let c = CurTransMat::Scale(Pt(2.0), Pt(2.0));

        let vec: Vec<CurTransMat> = vec![a, b, c];
        let result: CurTransMat = vec.into_iter().product();

        assert_eq!(
            result,
            CurTransMat::Raw([2.3849761e-8, 2.0, -2.0, 2.3849761e-8, -20.0, 20.0])
        );
    }

    #[test]
    fn high_position() {
        let a = CurTransMat::Position(PdfPosition::new(Pt(10.0), Pt(700.0)));
        let b = CurTransMat::Scale(Pt(2.0), Pt(2.0));

        let vec: Vec<CurTransMat> = vec![a, b];
        let result: CurTransMat = vec.into_iter().product();

        println!("{:?}", result);
    }
}
