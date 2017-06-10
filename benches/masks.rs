#![feature(test)]

extern crate image;
extern crate mazes;
extern crate test;


use image::{DynamicImage, GenericImage};

use mazes::cells::Cartesian2DCoordinate;
use mazes::masks::BinaryMask2D;
use mazes::units::{Height, Width};
use std::path::Path;
use test::Bencher;


const MASK_IMAGE_PATH: &str = "resources/mask-60x60.png";
const FULLMASK_IMAGE_PATH: &str = "resources/mask-all-60x60.png";


fn open_test_image(file_path_str: &str) -> DynamicImage {
    let img =
        image::open(&Path::new(file_path_str))
            .expect(format!("Unable to open and read test mask image file {}", file_path_str)
                        .as_ref());
    assert_eq!(img.width(), 60);
    assert_eq!(img.height(), 60);
    img
}

fn load_binary_mask() -> BinaryMask2D {
    let img = open_test_image(MASK_IMAGE_PATH);
    BinaryMask2D::from_image(&img)
}

fn load_everything_masked_mask() -> BinaryMask2D {
    let img = open_test_image(FULLMASK_IMAGE_PATH);
    BinaryMask2D::from_image(&img)
}


#[bench]
fn bench_from_image_60_x_60(b: &mut Bencher) {

    let img = open_test_image(MASK_IMAGE_PATH);

    b.iter(|| BinaryMask2D::from_image(&img));
}

#[bench]
fn bench_is_masked(b: &mut Bencher) {

    let mask = load_binary_mask();

    b.iter(|| mask.is_masked(Cartesian2DCoordinate::new(30, 30)));
}

#[bench]
fn bench_count_unmasked_within_dimensions(b: &mut Bencher) {

    let mask = load_binary_mask();

    b.iter(|| mask.count_unmasked_within_dimensions(Width(100), Height(100)));
}

#[bench]
fn bench_first_unmasked_coordinate(b: &mut Bencher) {

    let mask = load_binary_mask();

    b.iter(|| mask.first_unmasked_coordinate::<Cartesian2DCoordinate>());
}

#[bench]
fn bench_first_unmasked_coordinate_with_full_mask(b: &mut Bencher) {

    let mask = load_everything_masked_mask();

    b.iter(|| mask.first_unmasked_coordinate::<Cartesian2DCoordinate>());
}
