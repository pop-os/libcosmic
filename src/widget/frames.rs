//! Display an animated image in your user interface
//! Based on <https://github.com/tarkah/iced_gif/>

use std::ffi::OsStr;
use std::fmt;
use std::io;
use std::path::Path;
use std::time::{Duration, Instant};

use ::image as image_rs;
use iced_core::image::Renderer as ImageRenderer;
use iced_core::mouse::Cursor;
use iced_core::widget::{Tree, tree};
use iced_core::{
    Clipboard, ContentFit, Element, Event, Layout, Length, Rectangle, Shell, Size, Vector, Widget,
    event, layout, renderer, window,
};
use iced_runtime::Command;
use iced_widget::image::{self, Handle};
use image_rs::AnimationDecoder;
use image_rs::codecs::gif::GifDecoder;
use image_rs::codecs::png::PngDecoder;
use image_rs::codecs::webp::WebPDecoder;

#[cfg(not(feature = "tokio"))]
use iced_futures::futures::{AsyncRead, AsyncReadExt};
#[cfg(feature = "tokio")]
use tokio::io::{AsyncRead, AsyncReadExt};

use super::icon::load_icon;

#[must_use]
/// Creates a new [`AnimatedImage`] with the given [`animated_image::Frames`]
pub fn animated_image(frames: &Frames) -> AnimatedImage {
    AnimatedImage::new(frames)
}

/// Error loading or decoding a animated_image
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Decode error
    #[error(transparent)]
    Image(#[from] image_rs::ImageError),
    /// Load error
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// Missing image
    #[error("The image with the requested name is missing")]
    Missing,
    /// Unsupported Extension
    #[error("The extension is unsupported")]
    Extension,
}

#[derive(Clone)]
/// The frames of a decoded gif
pub struct Frames {
    first: Frame,
    frames: Vec<Frame>,
    total_bytes: u64,
}

impl fmt::Debug for Frames {
    #[cold]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Frames").finish()
    }
}

impl Frames {
    /// Load [`Frames`] from the supplied name
    pub fn load_from_name(
        name: &str,
        size: u16,
        theme: Option<&str>,
        default_fallbacks: bool,
    ) -> Command<Result<Frames, Error>> {
        let mut name_path_buffer = None;
        if let Some(path) = load_icon(name, size, theme) {
            name_path_buffer = Some(path);
        } else if default_fallbacks {
            for name in name.rmatch_indices('-').map(|(pos, _)| &name[..pos]) {
                if let Some(path) = load_icon(name, size, theme) {
                    name_path_buffer = Some(path);
                    break;
                }
            }
        };

        if let Some(name_path_buffer) = name_path_buffer {
            Self::load_from_path(name_path_buffer)
        } else {
            Command::perform(async { Err(Error::Missing) }, std::convert::identity)
        }
    }

    /// Load [`Frames`] from the supplied path
    pub fn load_from_path(path: impl AsRef<Path>) -> Command<Result<Frames, Error>> {
        #[inline(never)]
        fn inner(path: &Path) -> Command<Result<Frames, Error>> {
            #[cfg(feature = "tokio")]
            use tokio::fs::File;
            #[cfg(feature = "tokio")]
            use tokio::io::BufReader;

            #[cfg(not(feature = "tokio"))]
            use async_fs::File;
            #[cfg(not(feature = "tokio"))]
            use iced_futures::futures::io::BufReader;

            let path = path.as_ref().to_path_buf();

            let f = async move {
                let image_type = match &path.extension() {
                    Some(ext) if ext == &OsStr::new("gif") => ImageType::Gif,
                    Some(ext) if ext == &OsStr::new("apng") => ImageType::Apng,
                    Some(ext) if ext == &OsStr::new("webp") => ImageType::WebP,
                    _ => return Err(Error::Extension),
                };
                let reader = BufReader::new(File::open(path).await?);

                Self::from_reader(reader, image_type).await
            };

            Command::perform(f, std::convert::identity)
        }

        inner(path.as_ref())
    }

    /// Decode [`Frames`] from the supplied async reader
    /// # Errors
    /// If the type of image is not supported this function will error. IO errors may also occur.
    pub async fn from_reader<R: AsyncRead>(
        reader: R,
        image_type: ImageType,
    ) -> Result<Self, Error> {
        use iced_futures::futures::pin_mut;

        pin_mut!(reader);

        let mut bytes = vec![];

        reader.read_to_end(&mut bytes).await?;

        match image_type {
            ImageType::Gif => Self::from_decoder(GifDecoder::new(io::Cursor::new(bytes))?),
            ImageType::Apng => Self::from_decoder(PngDecoder::new(io::Cursor::new(bytes))?.apng()),
            ImageType::WebP => Self::from_decoder(WebPDecoder::new(io::Cursor::new(bytes))?),
        }
    }

    /// Decode [`Frames`] from the supplied bytes
    /// # Errors
    ///
    /// IO errors may occur.
    ///
    /// # Panics
    ///
    /// If there are no frames in the image, this panics.
    pub fn from_decoder<'a, T: AnimationDecoder<'a>>(decoder: T) -> Result<Self, Error> {
        let frames = decoder
            .into_frames()
            .map(|result| result.map(Frame::from))
            .collect::<Result<Vec<_>, _>>()?;

        let first = frames.first().cloned().unwrap();
        let total_bytes = frames
            .iter()
            .map(|f| match f.handle.data() {
                iced_core::image::Data::Path(_) => 0,
                iced_core::image::Data::Bytes(b) => b.len(),
                iced_core::image::Data::Rgba { pixels, .. } => pixels.len(),
            })
            .sum::<usize>()
            .try_into()
            .unwrap_or_default();
        Ok(Frames {
            first,
            frames,
            total_bytes,
        })
    }
}

#[derive(Clone)]
struct Frame {
    delay: Duration,
    handle: image::Handle,
}

impl From<image_rs::Frame> for Frame {
    fn from(frame: image_rs::Frame) -> Self {
        let (width, height) = frame.buffer().dimensions();

        let delay = frame.delay().into();

        let handle = image::Handle::from_pixels(width, height, frame.into_buffer().into_vec());

        Self { delay, handle }
    }
}

struct State {
    index: usize,
    current: Current,
    total_bytes: u64,
}

struct Current {
    frame: Frame,
    started: Instant,
}

impl From<Frame> for Current {
    fn from(frame: Frame) -> Self {
        Self {
            started: Instant::now(),
            frame,
        }
    }
}

/// A frame that displays an animated image while keeping aspect ratio
#[derive(Debug)]
pub struct AnimatedImage<'a> {
    frames: &'a Frames,
    width: Length,
    height: Length,
    content_fit: ContentFit,
}

pub enum ImageType {
    Gif,
    Apng,
    WebP,
}

impl<'a> AnimatedImage<'a> {
    #[must_use]
    /// Creates a new [`AnimatedImage`] with the given [`Frames`]
    pub fn new(frames: &'a Frames) -> Self {
        AnimatedImage {
            frames,
            width: Length::Shrink,
            height: Length::Shrink,
            content_fit: ContentFit::Contain,
        }
    }

    #[must_use]
    /// Sets the width of the [`AnimatedImage`] boundaries.
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    #[must_use]
    /// Sets the height of the [`AnimatedImage`] boundaries.
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    #[must_use]
    /// Sets the [`ContentFit`] of the [`AnimatedImage`].
    ///
    /// Defaults to [`ContentFit::Contain`]
    pub fn content_fit(self, content_fit: ContentFit) -> Self {
        Self {
            content_fit,
            ..self
        }
    }
}

impl<'a, Message, Renderer> Widget<Message, crate::Theme, Renderer> for AnimatedImage<'a>
where
    Renderer: ImageRenderer<Handle = Handle>,
{
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State {
            index: 0,
            current: self.frames.first.clone().into(),
            total_bytes: self.frames.total_bytes,
        })
    }

    fn diff(&mut self, tree: &mut Tree) {
        let state = tree.state.downcast_mut::<State>();

        // Reset state if new gif Frames is used w/
        // same state tree.
        //
        // Total bytes of the gif should be a good enough
        // proxy for it changing.
        if state.total_bytes != self.frames.total_bytes {
            *state = State {
                index: 0,
                current: self.frames.first.clone().into(),
                total_bytes: self.frames.total_bytes,
            };
        }
    }

    fn layout(&self, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        iced_widget::image::layout(
            renderer,
            limits,
            &self.frames.first.handle,
            self.width,
            self.height,
            self.content_fit,
        )
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        _layout: Layout<'_>,
        _cursor_position: Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        let state = tree.state.downcast_mut::<State>();

        if let Event::Window(_, window::Event::RedrawRequested(now)) = event {
            let elapsed = now.duration_since(state.current.started);

            if elapsed > state.current.frame.delay {
                state.index = (state.index + 1) % self.frames.frames.len();

                state.current = self.frames.frames[state.index].clone().into();

                shell.request_redraw(window::RedrawRequest::At(now + state.current.frame.delay));
            } else {
                let remaining = state.current.frame.delay - elapsed;

                shell.request_redraw(window::RedrawRequest::At(now + remaining));
            }
        }

        event::Status::Ignored
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        _theme: &crate::Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor_position: Cursor,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();

        // Pulled from iced_native::widget::<Image as Widget>::draw
        //
        // TODO: export iced_native::widget::image::draw as standalone function
        {
            let Size { width, height } = renderer.dimensions(&state.current.frame.handle);
            let image_size = Size::new(width as f32, height as f32);

            let bounds = layout.bounds();
            let adjusted_fit = self.content_fit.fit(image_size, bounds.size());

            let render = |renderer: &mut Renderer| {
                let offset = Vector::new(
                    (bounds.width - adjusted_fit.width).max(0.0) / 2.0,
                    (bounds.height - adjusted_fit.height).max(0.0) / 2.0,
                );

                let drawing_bounds = Rectangle {
                    width: adjusted_fit.width,
                    height: adjusted_fit.height,
                    ..bounds
                };

                renderer.draw(state.current.frame.handle.clone(), drawing_bounds + offset);
            };

            if adjusted_fit.width > bounds.width || adjusted_fit.height > bounds.height {
                renderer.with_layer(bounds, render);
            } else {
                render(renderer);
            }
        }
    }
}

impl<'a, Message, Renderer> From<AnimatedImage<'a>> for Element<'a, Message, crate::Theme, Renderer>
where
    Renderer: ImageRenderer<Handle = Handle> + 'a,
{
    fn from(gif: AnimatedImage<'a>) -> Element<'a, Message, crate::Theme, Renderer> {
        Element::new(gif)
    }
}
