use criterion::{
    Criterion,
    criterion_group,
    criterion_main
};
use image::DynamicImage;
use mazes::cells::Cartesian2DCoordinate;
use mazes::masks::BinaryMask2D;
use mazes::units::{Height, Width};
use std::path::Path;


const MASK_IMAGE_PATH: &str = "resources/mask-60x60.png";
const FULLMASK_IMAGE_PATH: &str = "resources/mask-all-60x60.png";


fn open_test_image(file_path_str: &str) -> DynamicImage {
    let img =
        image::open(&Path::new(file_path_str))
            .expect(format!("Unable to open and read test mask image file {}", file_path_str)
                        .as_ref());
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


fn bench_from_image_60_x_60(c: &mut Criterion) {
    c.bench_function("from_image_60_x_60", |b| {
        let img = open_test_image(MASK_IMAGE_PATH);
        b.iter(|| BinaryMask2D::from_image(&img))
    });
}

fn bench_is_masked(c: &mut Criterion) {
    c.bench_function("is_masked", |b| {
        let mask = load_binary_mask();
        b.iter(|| mask.is_masked(Cartesian2DCoordinate::new(30, 30)))
    });
}

fn bench_count_unmasked_within_dimensions(c: &mut Criterion) {
    c.bench_function("count_unmasked_within_dimensions", |b| {
        let mask = load_binary_mask();
        b.iter(|| mask.count_unmasked_within_dimensions(Width(100), Height(100)))
    });
}

fn bench_first_unmasked_coordinate(c: &mut Criterion) {
    c.bench_function("first_unmasked_coordinate", |b| {
        let mask = load_binary_mask();
        b.iter(|| mask.first_unmasked_coordinate::<Cartesian2DCoordinate>())
    });
}

fn bench_first_unmasked_coordinate_with_full_mask(c: &mut Criterion) {
    c.bench_function("first_unmasked_coordinate_with_full_mask", |b| {
        let mask = load_everything_masked_mask();
        b.iter(|| mask.first_unmasked_coordinate::<Cartesian2DCoordinate>())
    });
}

criterion_group!(benches,
    bench_from_image_60_x_60,
    bench_is_masked,
    bench_count_unmasked_within_dimensions,
    bench_first_unmasked_coordinate,
    bench_first_unmasked_coordinate_with_full_mask
);
criterion_main!(benches);
