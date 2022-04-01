use image::imageops::flip_vertical_in_place;
use image::io::Reader;
use image::ImageFormat;
use std::io::{BufReader, Cursor};

#[derive(Clone, Copy)]
pub(crate) struct Texture {
    id: u32,
}

impl Texture {
    pub(crate) fn new(image_data: &[u8], format: Format, channels: Channels) -> Self {
        let mut reader = Reader::new(BufReader::new(Cursor::new(image_data)));

        reader.set_format(match format {
            Format::Jpeg => ImageFormat::Jpeg,
            Format::Png => ImageFormat::Png,
        });

        let mut image = reader.decode().unwrap();
        flip_vertical_in_place(&mut image);
        let (width, height, pixels) = match channels {
            Channels::Rgb => {
                assert!(image.as_rgb8().is_some());
                let image = image.into_rgb8();
                (image.width(), image.height(), image.into_vec())
            }
            Channels::Rgba => {
                assert!(image.as_rgba8().is_some());
                let image = image.into_rgba8();
                (image.width(), image.height(), image.into_vec())
            }
        };

        let mut id = 0;
        unsafe {
            gl::GenTextures(1, &mut id);
            gl::BindTexture(gl::TEXTURE_2D, id);

            let channels = match channels {
                Channels::Rgb => gl::RGB,
                Channels::Rgba => gl::RGBA,
            };

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                channels as i32,
                width as i32,
                height as i32,
                0,
                channels,
                gl::UNSIGNED_BYTE,
                pixels.as_ptr() as *const _,
            );
            gl::GenerateMipmap(gl::TEXTURE_2D);
        }

        Self { id }
    }

    pub(crate) fn bind(self) {
        unsafe { gl::BindTexture(gl::TEXTURE_2D, self.id) };
    }
}

pub(crate) enum Format {
    Jpeg,
    Png,
}

pub(crate) enum Channels {
    Rgb,
    Rgba,
}
