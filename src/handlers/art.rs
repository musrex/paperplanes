use image::{ImageBuffer, ImageEncoder, Rgb};
use rand::{Rng, SeedableRng, rngs::StdRng};
use serde::Deserialize;
use std::io::Cursor;

use axum::{
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Response},
};

#[derive(Debug, Deserialize)]
pub struct ArtParams {
    width: Option<u32>,
    height: Option<u32>,
    seed: Option<u64>,
}

pub async fn handler_art(Query(params): Query<ArtParams>) -> Response {
    let width = params.width.unwrap_or(400);
    let height = params.height.unwrap_or(400);
    let seed = params.seed.unwrap_or_else(|| rand::random());

    let mut rng = StdRng::seed_from_u64(seed);

    let mut img = ImageBuffer::new(width, height);

    for (_, _, pixel) in img.enumerate_pixels_mut() {
        let r = rng.random_range(0..=255);
        let g = rng.random_range(0..=25);
        let b = rng.random_range(0..=25);
        *pixel = Rgb([r, g, b]);
    }

    let mut buffer = Cursor::new(Vec::new());
    if image::codecs::png::PngEncoder::new(&mut buffer)
        .write_image(&img, width, height, image::ExtendedColorType::Rgb8)
        .is_err()
    {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Error generating image").into_response();
    }

    let bytes = buffer.into_inner();

    (
        axum::http::HeaderMap::from_iter([(
            axum::http::header::CONTENT_TYPE,
            "image/png".parse().unwrap(),
        )]),
        bytes,
    )
        .into_response()
}

pub async fn handler_fractal(Query(params): Query<ArtParams>) -> Response {
    let width = params.width.unwrap_or(800);
    let height = params.height.unwrap_or(800);

    let scalex = 3.0 / width as f32;
    let scaley = 3.0 / height as f32;

    let mut img = ImageBuffer::new(width, height);

    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let r = (0.3 * x as f32) as u8;
        let b = (0.3 * y as f32) as u8;
        *pixel = Rgb([r, 0, b]);
    }

    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let cx = y as f32 * scalex - 1.5;
        let cy = x as f32 * scaley - 1.5;

        let c = num_complex::Complex::new(-0.4, 0.6);
        let mut z = num_complex::Complex::new(cx, cy);

        let mut i = 0;
        while i < 255 && z.norm() <= 2.0 {
            z = z * z + c;
            i += 1;
        }

        let image::Rgb(data) = *pixel;
        *pixel = image::Rgb([data[0], i as u8, data[2]]);
    }

    let mut buffer = Cursor::new(Vec::new());
    if image::codecs::png::PngEncoder::new(&mut buffer)
        .write_image(&img, width, height, image::ExtendedColorType::Rgb8)
        .is_err()
    {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Error generating image").into_response();
    }

    let bytes = buffer.into_inner();

    (
        axum::http::HeaderMap::from_iter([(
            axum::http::header::CONTENT_TYPE,
            "image/png".parse().unwrap(),
        )]),
        bytes,
    )
        .into_response()
}
