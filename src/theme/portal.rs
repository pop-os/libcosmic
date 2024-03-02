use ashpd::desktop::settings::{ColorScheme, Contrast};
use ashpd::desktop::Color;
use iced::futures::{self, select, FutureExt, SinkExt, StreamExt};
use iced::Subscription;
use iced_futures::subscription;
use tracing::error;

#[derive(Debug, Clone)]
pub enum Desktop {
    Accent(Color),
    ColorScheme(ColorScheme),
    Contrast(Contrast),
}

pub fn desktop_settings() -> iced_futures::Subscription<Desktop> {
    subscription::channel(std::any::TypeId::of::<Desktop>(), 10, |mut tx| {
        async move {
            let Ok(settings) = ashpd::desktop::settings::Settings::new().await else {
                // wait forever
                error!("Failed to create the settings proxy");
                futures::future::pending::<()>().await;
                unreachable!()
            };

            match settings.color_scheme().await {
                Ok(color_scheme) => {
                    let _ = tx.send(Desktop::ColorScheme(color_scheme)).await;
                }
                Err(err) => error!("Failed to get the color scheme {err:?}"),
            };
            match settings.contrast().await {
                Ok(contrast) => {
                    let _ = tx.send(Desktop::Contrast(contrast)).await;
                }
                Err(err) => error!("Failed to get the contrast {err:?}"),
            };

            let mut color_scheme_stream = settings.receive_color_scheme_changed().await.ok();
            if color_scheme_stream.is_none() {
                error!("Failed to receive color scheme changes");
            }

            let mut contrast_stream = settings.receive_contrast_changed().await.ok();
            if contrast_stream.is_none() {
                error!("Failed to receive contrast changes");
            }

            loop {
                let next_color_scheme = async {
                    if let Some(s) = color_scheme_stream.as_mut() {
                        return s.next().await;
                    }
                    futures::future::pending().await
                };

                let next_contrast = async {
                    if let Some(s) = contrast_stream.as_mut() {
                        return s.next().await;
                    }
                    futures::future::pending().await
                };

                select! {
                    s = next_color_scheme.fuse() => {
                        if let Some(s) = s {
                            _ = tx.send(Desktop::ColorScheme(s)).await;
                        } else {
                            color_scheme_stream = None;
                        }
                    },

                    c = next_contrast.fuse() => {
                        if let Some(c) = c {
                            _ = tx.send(Desktop::Contrast(c)).await;
                        } else {
                            contrast_stream = None;
                        }
                    }
                };
            }
        }
    })
}
