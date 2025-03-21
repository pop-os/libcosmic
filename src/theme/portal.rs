use ashpd::desktop::Color;
use ashpd::desktop::settings::{ColorScheme, Contrast};
use iced::futures::{self, FutureExt, SinkExt, StreamExt, select};
use iced_futures::stream;
use tracing::error;

#[derive(Debug, Clone)]
pub enum Desktop {
    Accent(Color),
    ColorScheme(ColorScheme),
    Contrast(Contrast),
}

#[cold]
pub fn desktop_settings() -> iced_futures::Subscription<Desktop> {
    iced_futures::Subscription::run_with_id(
        std::any::TypeId::of::<Desktop>(),
        stream::channel(10, |mut tx| {
            async move {
                let mut attempts = 0;
                loop {
                    let Ok(settings) = ashpd::desktop::settings::Settings::new().await else {
                        error!("Failed to create the settings proxy");
                        #[cfg(feature = "tokio")]
                        ::tokio::time::sleep(::tokio::time::Duration::from_secs(
                            2_u64.pow(attempts),
                        ))
                        .await;
                        #[cfg(not(feature = "tokio"))]
                        {
                            futures::future::pending::<()>().await;
                            unreachable!();
                        }
                        attempts += 1;
                        continue;
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

                    let mut color_scheme_stream =
                        settings.receive_color_scheme_changed().await.ok();
                    if color_scheme_stream.is_none() {
                        error!("Failed to receive color scheme changes");
                    }

                    let mut contrast_stream = settings.receive_contrast_changed().await.ok();
                    if contrast_stream.is_none() {
                        error!("Failed to receive contrast changes");
                    }

                    loop {
                        if color_scheme_stream.is_none() && contrast_stream.is_none() {
                            break;
                        }
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
                        // Reset the attempts counter if we successfully received a change
                        attempts = 0;
                    }
                }
            }
        }),
    )
}
