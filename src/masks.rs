use bit_set::BitSet;
use image::{DynamicImage, GenericImage, Luma};

use cells::{Cartesian2DCoordinate, Coordinate};
use units::{Width, Height, ColumnIndex, RowIndex};

#[derive(Debug)]
pub struct BinaryMask2D {
    mask: BitSet,
    pub width: u32,
    pub height: u32,
}

impl BinaryMask2D {
    pub fn from_image(data_image: &DynamicImage) -> BinaryMask2D {

        let w = data_image.width();
        let h = data_image.height();
        let gray_scale_image = data_image.to_luma();
        let mut mask = BitSet::with_capacity((w * h) as usize);

        for x in 0..gray_scale_image.width() {
            for y in 0..gray_scale_image.height() {

                let pix: &Luma<u8> = gray_scale_image.get_pixel(x, y);
                let gray_scale_value = pix.data[0];
                let off = gray_scale_value < 128;

                if off {
                    mask.insert((y * w + x) as usize);
                }
            }
        }

        BinaryMask2D {
            mask: mask,
            width: w,
            height: h,
        }
    }

    /// Is the given coordinate masked out / turned off?
    ///
    /// A coordinate is not masked if it is outside the bounds of masks 2d space.
    pub fn is_masked<CoordT: Coordinate>(&self, coord: CoordT) -> bool {

        let mask_coordinate = coord.as_cartesian_2d();

        if mask_coordinate.x < self.width && mask_coordinate.y < self.height {
            let bit_index = (mask_coordinate.y * self.width + mask_coordinate.x) as usize;
            self.mask.contains(bit_index)
        } else {
            false
        }
    }

    /// Calculates the number of unmasked cells within a 2d space specified by `width` and `height`.
    ///
    /// All cells in the 2d space outside of the masks' own width and height are counted as unmasked.
    pub fn count_unmasked_within_dimensions(&self, width: Width, height: Height) -> usize {

        let mut count = 0;
        for x in 0..(width.0) {
            for y in 0..(height.0) {
                let masked = self.is_masked(Cartesian2DCoordinate::new(x as u32, y as u32)); // CellT::Coord from usize?
                if !masked {
                    count += 1;
                }
            }
        }

        count
    }

    pub fn first_unmasked_coordinate<CoordT: Coordinate>(&self) -> Option<CoordT> {

        // A bit in the set means masked off
        // The bitset iterator returns indices of masked values, so we cannot use that
        // (A BitVec would be more convenient for that purpose, and faster as bitset::contains
        //  does a length check)
        let mask_size = self.width * self.height;
        let index: Option<usize> = (0..mask_size)
            .position(|bit_index| !self.mask.contains(bit_index as usize));

        if let Some(i) = index {
            let x = i % self.width as usize;
            let y = i / self.height as usize;
            Some(CoordT::from_row_column_indices(ColumnIndex(x), RowIndex(y)))
        } else {
            None
        }
    }
}
