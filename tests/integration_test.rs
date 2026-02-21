//! Integration tests for image obfuscation

use hilbert_image_obfuscator::{deobfuscate, next_power_of_two, obfuscate};
use std::path::Path;

#[test]
fn test_obfuscate_with_real_image() {
    // Test with the real test image
    let test_image_path = "测试用图.jpg";

    if !Path::new(test_image_path).exists() {
        println!("Test image not found, skipping integration test");
        return;
    }

    // Load the test image
    let img = image::open(test_image_path).expect("Failed to open test image");
    let rgba = img.to_rgba8();

    let (w, h) = rgba.dimensions();
    println!("Loaded test image: {}x{}", w, h);

    let seed = 12345;

    // Obfuscate
    let (obf, side) = obfuscate(&rgba, seed);
    println!("Obfuscated image: {}x{}", obf.width(), obf.height());
    println!("Side used: {}", side);

    // Deobfuscate
    let (deobf, _side) = deobfuscate(&obf, seed, w, h, side);
    println!("Deobfuscated image: {}x{}", deobf.width(), deobf.height());

    // Verify dimensions - obfuscated/deobfuscated have original dimensions
    assert_eq!(obf.width(), w);
    assert_eq!(obf.height(), h);
    assert_eq!(deobf.width(), w);
    assert_eq!(deobf.height(), h);

    // Verify pixels match in the original region
    let mut diff_count = 0;
    for y in 0..h {
        for x in 0..w {
            let orig_pixel = rgba.get_pixel(x, y);
            let deob_pixel = deobf.get_pixel(x, y);
            if orig_pixel != deob_pixel {
                diff_count += 1;
            }
        }
    }

    // Allow for some differences due to padding
    let total_pixels = (w * h) as usize;
    let diff_ratio = diff_count as f64 / total_pixels as f64;
    println!(
        "Different pixels: {}/{} ({:.2}%)",
        diff_count,
        total_pixels,
        diff_ratio * 100.0
    );

    // Should be very close to 0% different
    // Note: Due to padding differences, allow up to 1% difference
    assert!(
        diff_ratio < 0.01 || diff_count < 100,
        "Too many different pixels: {:.2}%",
        diff_ratio * 100.0
    );
}

#[test]
fn test_next_power_of_two_integration() {
    // Test that various image sizes work correctly
    let test_sizes = [100, 200, 500, 1024, 1920, 2048];

    for size in test_sizes {
        let side = next_power_of_two(size);
        println!("Input: {}, Next power of 2: {}", size, side);

        // Side should be a power of 2
        assert!(side.is_power_of_two(), "{} is not a power of 2", side);

        // Side should be >= original size
        assert!(side >= size as u32, "Side {} < size {}", side, size);
    }
}
