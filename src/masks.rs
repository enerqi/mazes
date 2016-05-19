use image::{DynamicImage, GenericImage, Luma};

use squaregrid::GridCoordinate;
use utils;

/// Reads data from an image file and generates a set with each coordinate that is turned off (masked out).
///
/// Pixel(0,0) corresponds to GridCoordinate(0,0) etc.
/// The image is converted to grayscale and pixel values below 128 are considered off (masked out).
pub fn binary_grid_mask(data_image: &DynamicImage) -> utils::FnvHashSet<GridCoordinate> {

    let mut mask = utils::fnv_hashset((data_image.width() * data_image.height()) as usize);
    let gray_scale_image = data_image.to_luma();

    for x in 0..gray_scale_image.width() {
        for y in 0..gray_scale_image.height() {

            let pix: &Luma<u8> = gray_scale_image.get_pixel(x, y);
            let gray_scale_value = pix.data[0];
            let off = gray_scale_value < 128;

            if off {
               mask.insert(GridCoordinate::new(x, y));
            }
        }
    }
    mask
}
