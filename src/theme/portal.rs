use ashpd::desktop::settings::{ColorScheme, Contrast};
use ashpd::desktop::Color;
use iced::futures::{self, select, FutureExt, SinkExt, StreamExt};
use iced_futures::subscription;

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
                futures::future::pending::<()>().await;
                unreachable!()
            };
            let mut color_scheme_stream = settings.receive_color_scheme_changed().await.ok();
            let mut accent_stream = settings.receive_accent_color_changed().await.ok();
            let mut contrast_stream = settings.receive_contrast_changed().await.ok();

            loop {
                let next_color_scheme = async {
                    if let Some(s) = color_scheme_stream.as_mut() {
                        return s.next().await;
                    }
                    futures::future::pending().await
                };
                let next_accent = async {
                    if let Some(s) = accent_stream.as_mut() {
                        // Item type is wrong in this version
                        // updating requires updating to zbus 4
                        return if s.next().await.is_some() {
                            settings.accent_color().await.ok()
                        } else {
                            None
                        };
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
                    a = next_accent.fuse() => {
                        if let Some(a) = a {
                            _ = tx.send(Desktop::Accent(a)).await;
                        } else {
                            accent_stream = None;
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
