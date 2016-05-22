use bit_set::BitSet;
use image::{DynamicImage, GenericImage, Luma};

use squaregrid::GridCoordinate;

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
    pub fn is_masked(&self, coord: GridCoordinate) -> bool {

        if coord.x < self.width && coord.y < self.height {
            let bit_index = (coord.y * self.width + coord.x) as usize;
            self.mask.contains(bit_index)
        } else {
            false
        }
    }

    /// Calculates the number of unmasked cells within a 2d space specified by `width` and `height`.
    ///
    /// All cells in the 2d space outside of the masks' own width and height are counted as unmasked.
    pub fn count_unmasked_within_dimensions(&self, width: u32, height: u32) -> usize {

        let mut count = 0;
        for x in 0..width {
            for y in 0..height {
                let masked = self.is_masked(GridCoordinate::new(x, y));
                if !masked {
                    count += 1;
                }
            }
        }

        count
    }

    pub fn first_unmasked_coordinate(&self) -> Option<GridCoordinate> {

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
            Some(GridCoordinate::new(x as u32, y as u32))
        } else {
            None
        }
    }
}
