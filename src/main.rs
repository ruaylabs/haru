use std::{io, io::Write, path::Path, path::PathBuf, process::ExitCode};

use clap::Parser;
use image::{DynamicImage, ImageEncoder, ImageFormat, RgbaImage, codecs::png::PngEncoder};

#[derive(Debug, Parser)]
#[command(
    version,
    about = "Save the clipboard image to a file, or '-' for PNG on stdout"
)]
struct Args {
    /// Output file (PNG, JPEG, GIF, TIFF, BMP), or '-' for PNG on stdout
    output: PathBuf,
}

fn main() -> ExitCode {
    let args = match Args::try_parse() {
        Ok(args) => args,
        Err(error) => {
            let code = if error.use_stderr() { 1 } else { 0 };
            let _ = error.print();
            return ExitCode::from(code);
        }
    };

    let data = match arboard::Clipboard::new().and_then(|mut clipboard| clipboard.get_image()) {
        Ok(data) => data,
        Err(error) => {
            eprintln!("haru: No image data in clipboard: {error}");
            return ExitCode::from(1);
        }
    };

    let image = match rgba_image(data) {
        Ok(image) => image,
        Err(error) => {
            eprintln!("haru: Cannot encode clipboard image: {error}");
            return ExitCode::from(2);
        }
    };

    if let Err(error) = write_image(image, &args.output) {
        eprintln!(
            "haru: Cannot write image to {}: {error}",
            args.output.display()
        );
        return ExitCode::from(2);
    }
    ExitCode::SUCCESS
}

fn rgba_image(data: arboard::ImageData<'_>) -> Result<RgbaImage, &'static str> {
    let width = u32::try_from(data.width).map_err(|_| "image width is too large")?;
    let height = u32::try_from(data.height).map_err(|_| "image height is too large")?;
    if width == 0 || height == 0 {
        return Err("image dimensions must be nonzero");
    }
    let expected = data
        .width
        .checked_mul(data.height)
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or("image dimensions are too large")?;
    if data.bytes.len() != expected {
        return Err("invalid RGBA clipboard data");
    }
    RgbaImage::from_raw(width, height, data.bytes.into_owned()).ok_or("invalid RGBA clipboard data")
}

// The bool indicates an unknown extension, which should warn and fall back to PNG.
fn output_format(path: &Path) -> (ImageFormat, bool) {
    let extension = path.extension().and_then(|value| value.to_str());
    match extension.map(str::to_ascii_lowercase).as_deref() {
        None | Some("png") => (ImageFormat::Png, false),
        Some("jpg" | "jpeg") => (ImageFormat::Jpeg, false),
        Some("gif") => (ImageFormat::Gif, false),
        Some("tif" | "tiff") => (ImageFormat::Tiff, false),
        Some("bmp") => (ImageFormat::Bmp, false),
        _ => (ImageFormat::Png, true),
    }
}

fn encodable_image(image: RgbaImage, format: ImageFormat) -> DynamicImage {
    let image = DynamicImage::ImageRgba8(image);
    // JPEG does not support an alpha channel; discard alpha without compositing.
    if format == ImageFormat::Jpeg {
        DynamicImage::ImageRgb8(image.to_rgb8())
    } else {
        image
    }
}

fn write_image(image: RgbaImage, output: &Path) -> Result<(), image::ImageError> {
    if output == Path::new("-") {
        let mut stdout = io::stdout().lock();
        PngEncoder::new(&mut stdout).write_image(
            image.as_raw(),
            image.width(),
            image.height(),
            image::ExtendedColorType::Rgba8,
        )?;
        stdout.flush()?;
    } else {
        let (format, unknown) = output_format(output);
        if unknown {
            eprintln!(
                "haru: Unknown output extension for {}; using PNG",
                output.display()
            );
        }
        encodable_image(image, format).save_with_format(output, format)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{borrow::Cow, io::Cursor};

    #[test]
    fn detects_formats_and_fallbacks() {
        for (extension, format) in [
            ("png", ImageFormat::Png),
            ("jpg", ImageFormat::Jpeg),
            ("jpeg", ImageFormat::Jpeg),
            ("gif", ImageFormat::Gif),
            ("tif", ImageFormat::Tiff),
            ("tiff", ImageFormat::Tiff),
            ("bmp", ImageFormat::Bmp),
            ("PNG", ImageFormat::Png),
            ("JpEg", ImageFormat::Jpeg),
        ] {
            assert_eq!(
                output_format(Path::new(&format!("image.{extension}"))),
                (format, false)
            );
        }
        assert_eq!(output_format(Path::new("image")), (ImageFormat::Png, false));
        assert_eq!(
            output_format(Path::new("image.xyz")),
            (ImageFormat::Png, true)
        );
    }

    #[test]
    fn rejects_invalid_clipboard_data() {
        for (width, height, bytes) in [
            (0, 1, vec![]),
            (1, 0, vec![]),
            (1, 1, vec![0; 3]),
            (1, 1, vec![0; 5]),
            (usize::MAX, 2, vec![]),
        ] {
            assert!(
                rgba_image(arboard::ImageData {
                    width,
                    height,
                    bytes: Cow::Owned(bytes)
                })
                .is_err()
            );
        }
        assert!(
            rgba_image(arboard::ImageData {
                width: 1,
                height: 1,
                bytes: Cow::Borrowed(&[255, 0, 0, 255])
            })
            .is_ok()
        );
    }

    #[test]
    fn all_formats_encode_and_decode() {
        for format in [
            ImageFormat::Png,
            ImageFormat::Jpeg,
            ImageFormat::Gif,
            ImageFormat::Tiff,
            ImageFormat::Bmp,
        ] {
            let image = RgbaImage::from_pixel(2, 2, image::Rgba([255, 0, 0, 255]));
            let mut buffer = Cursor::new(Vec::new());
            encodable_image(image, format)
                .write_to(&mut buffer, format)
                .unwrap();
            let decoded = image::load_from_memory_with_format(buffer.get_ref(), format).unwrap();
            assert_eq!((decoded.width(), decoded.height()), (2, 2));
        }
    }

    #[test]
    fn overwrites_files_in_all_formats() {
        let directory = tempfile::tempdir().unwrap();
        for extension in ["png", "jpg", "gif", "tiff", "bmp"] {
            let output = directory.path().join(format!("image.{extension}"));
            std::fs::write(&output, b"existing output").unwrap();
            let image = RgbaImage::from_pixel(2, 2, image::Rgba([255, 0, 0, 255]));
            write_image(image, &output).unwrap();
            let decoded = image::open(&output).unwrap();
            assert_eq!((decoded.width(), decoded.height()), (2, 2));
        }
        assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 5);
    }

    #[test]
    fn rejects_directory_destination() {
        let directory = tempfile::tempdir().unwrap();
        let output = directory.path().join("image.png");
        std::fs::create_dir(&output).unwrap();
        let existing = output.join("existing");
        std::fs::write(&existing, b"keep me").unwrap();
        let image = RgbaImage::from_pixel(2, 2, image::Rgba([255, 0, 0, 255]));
        assert!(write_image(image, &output).is_err());
        assert_eq!(std::fs::read(existing).unwrap(), b"keep me");
        assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 1);
    }

    #[test]
    fn file_output_preserves_opaque_pixels() {
        let directory = tempfile::tempdir().unwrap();
        let pixels = vec![255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255];
        for extension in ["png", "gif", "tiff", "bmp"] {
            let output = directory.path().join(format!("image.{extension}"));
            let image = RgbaImage::from_raw(3, 1, pixels.clone()).unwrap();
            write_image(image, &output).unwrap();
            assert_eq!(image::open(output).unwrap().to_rgba8().as_raw(), &pixels);
        }
    }

    #[test]
    fn png_and_tiff_preserve_partial_alpha() {
        let directory = tempfile::tempdir().unwrap();
        let pixels = vec![255, 0, 0, 0, 0, 255, 0, 128, 0, 0, 255, 255];
        for extension in ["png", "tiff"] {
            let output = directory.path().join(format!("image.{extension}"));
            write_image(RgbaImage::from_raw(3, 1, pixels.clone()).unwrap(), &output).unwrap();
            assert_eq!(image::open(output).unwrap().to_rgba8().as_raw(), &pixels);
        }
    }

    #[test]
    fn jpeg_discards_alpha_without_compositing() {
        let directory = tempfile::tempdir().unwrap();
        let output = directory.path().join("image.jpg");
        let image = RgbaImage::from_pixel(8, 8, image::Rgba([200, 100, 50, 0]));
        let rgb = encodable_image(image.clone(), ImageFormat::Jpeg).to_rgb8();
        assert_eq!(rgb.get_pixel(0, 0).0, [200, 100, 50]);
        write_image(image, &output).unwrap();
        let decoded = image::open(output).unwrap().to_rgb8();
        for pixel in decoded.pixels() {
            for (actual, expected) in pixel.0.into_iter().zip([200_u8, 100, 50]) {
                assert!(actual.abs_diff(expected) <= 5, "JPEG pixel: {pixel:?}");
            }
        }
    }

    #[test]
    fn unknown_and_missing_extensions_write_png() {
        let directory = tempfile::tempdir().unwrap();
        for name in ["image.xyz", "image", "image.PNG"] {
            let output = directory.path().join(name);
            write_image(
                RgbaImage::from_pixel(1, 1, image::Rgba([1, 2, 3, 128])),
                &output,
            )
            .unwrap();
            let bytes = std::fs::read(output).unwrap();
            assert_eq!(image::guess_format(&bytes).unwrap(), ImageFormat::Png);
            assert_eq!(
                image::load_from_memory(&bytes).unwrap().to_rgba8().as_raw(),
                &[1, 2, 3, 128]
            );
        }
    }

    #[test]
    fn rejects_missing_parent_directory() {
        let directory = tempfile::tempdir().unwrap();
        let output = directory.path().join("missing/image.png");
        let error = write_image(RgbaImage::new(1, 1), &output).unwrap_err();
        assert!(matches!(error, image::ImageError::IoError(_)));
        assert!(!output.exists());
    }

    #[test]
    fn parses_stdout_and_rejects_missing_output() {
        assert_eq!(
            Args::try_parse_from(["haru", "-"]).unwrap().output,
            Path::new("-")
        );
        assert!(Args::try_parse_from(["haru"]).is_err());
        assert!(Args::try_parse_from(["haru", "a", "b"]).is_err());
    }
}
