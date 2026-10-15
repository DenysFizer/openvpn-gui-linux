//! System tray integration (freedesktop StatusNotifierItem via `ksni`).
//!
//! The tray service lives on the async runtime and communicates with the
//! application through an unbounded channel: menu/activation callbacks push
//! [`TrayEvent`]s and [`subscription`] forwards them to the app as
//! [`Message::Tray`](crate::app::Message::Tray).

use std::pin::Pin;
use std::sync::{Mutex, OnceLock};

use iced::advanced::subscription::{EventStream, Hasher, Recipe};
use iced::futures::SinkExt;
use iced::futures::Stream;
use iced::futures::channel::mpsc::Sender as IcedSender;
use iced::{Subscription, Task};
use ksni::menu::StandardItem;
use ksni::{Handle, Icon, MenuItem, ToolTip, Tray, TrayMethods};

use crate::app::Message;

const ICON_BYTES: &[u8] = include_bytes!("../assets/logo.png");

/// Actions triggered from the tray icon and its context menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayEvent {
    /// The icon was clicked.
    Activate,
    /// The "Show Window" menu item was selected.
    Show,
    /// The "Hide to Tray" menu item was selected.
    Hide,
    /// The "Quit" menu item was selected.
    Quit,
}

/// Receiver side of the tray event channel; consumed by [`subscription`].
static EVENT_RX: Mutex<Option<tokio::sync::mpsc::UnboundedReceiver<TrayEvent>>> = Mutex::new(None);

/// Keeps the tray service alive for the lifetime of the process.
static TRAY_HANDLE: OnceLock<Handle<AppTray>> = OnceLock::new();

/// Starts the tray service, returning the task that launches it on the
/// runtime. The event channel is created synchronously so [`subscription`]
/// can be built without racing against the service startup.
pub fn launch() -> Task<Message> {
    let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel();
    *EVENT_RX.lock().unwrap() = Some(event_rx);

    Task::future(async move {
        match (AppTray { event_tx }).spawn().await {
            Ok(handle) => {
                let _ = TRAY_HANDLE.set(handle);
            }
            Err(e) => log::warn!("Failed to start system tray: {e}"),
        }
    })
    .discard()
}

/// Forwards tray events into the application as [`Message::Tray`].
pub fn subscription() -> Subscription<Message> {
    struct TrayRecipe;

    impl Recipe for TrayRecipe {
        type Output = Message;

        fn hash(&self, state: &mut Hasher) {
            use std::hash::Hash;
            "system_tray".hash(state);
        }

        fn stream(
            self: Box<Self>,
            _input: EventStream,
        ) -> Pin<Box<dyn Stream<Item = Message> + Send>> {
            Box::pin(iced::stream::channel(
                10,
                |mut output: IcedSender<Message>| async move {
                    let mut rx = EVENT_RX
                        .lock()
                        .unwrap()
                        .take()
                        .expect("tray event receiver already taken");

                    while let Some(event) = rx.recv().await {
                        if output.send(Message::Tray(event)).await.is_err() {
                            break;
                        }
                    }
                },
            ))
        }
    }

    iced::advanced::subscription::from_recipe(TrayRecipe)
}

struct AppTray {
    event_tx: tokio::sync::mpsc::UnboundedSender<TrayEvent>,
}

impl AppTray {
    fn send(&self, event: TrayEvent) {
        let _ = self.event_tx.send(event);
    }
}

impl Tray for AppTray {
    fn id(&self) -> String {
        "openvpn-gui-linux".to_string()
    }

    fn title(&self) -> String {
        "OpenVPN Client".to_string()
    }

    fn icon_pixmap(&self) -> Vec<Icon> {
        tray_icons()
    }

    fn tool_tip(&self) -> ToolTip {
        ToolTip {
            title: "OpenVPN Client".to_string(),
            ..ToolTip::default()
        }
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        self.send(TrayEvent::Activate);
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        vec![
            MenuItem::Standard(StandardItem {
                label: "Show Window".to_string(),
                activate: Box::new(|tray: &mut AppTray| tray.send(TrayEvent::Show)),
                ..StandardItem::default()
            }),
            MenuItem::Standard(StandardItem {
                label: "Hide to Tray".to_string(),
                activate: Box::new(|tray: &mut AppTray| tray.send(TrayEvent::Hide)),
                ..StandardItem::default()
            }),
            MenuItem::Separator,
            MenuItem::Standard(StandardItem {
                label: "Quit".to_string(),
                activate: Box::new(|tray: &mut AppTray| tray.send(TrayEvent::Quit)),
                ..StandardItem::default()
            }),
        ]
    }
}

/// Decodes the bundled logo into a few common tray sizes.
fn tray_icons() -> Vec<Icon> {
    static ICONS: OnceLock<Vec<Icon>> = OnceLock::new();

    ICONS
        .get_or_init(|| {
            let source = image::load_from_memory_with_format(ICON_BYTES, image::ImageFormat::Png)
                .expect("bundled logo.png is a valid PNG")
                .into_rgba8();

            let native = source.width().max(source.height());
            let mut icons = Vec::new();

            for size in [24, 48, native] {
                if size == 0 || icons.iter().any(|icon: &Icon| icon.width == size as i32) {
                    continue;
                }

                let scaled = if size == native {
                    source.clone()
                } else {
                    image::imageops::resize(
                        &source,
                        size,
                        size,
                        image::imageops::FilterType::Lanczos3,
                    )
                };

                icons.push(icon_from_rgba(scaled));
            }

            icons
        })
        .clone()
}

fn icon_from_rgba(image: image::RgbaImage) -> Icon {
    let (width, height) = image.dimensions();
    let mut data = image.into_vec();

    // The protocol expects ARGB32 in network byte order; our data is RGBA.
    for pixel in data.as_chunks_mut::<4>().0 {
        pixel.rotate_right(1);
    }

    Icon {
        width: width as i32,
        height: height as i32,
        data,
    }
}
