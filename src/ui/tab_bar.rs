use iced::widget::{button, container, row, text};
use iced::{Element, Length};

use crate::app::Message;
use crate::ui::theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Connect,
    Profiles,
    Settings,
}

impl Tab {
    fn label(self) -> &'static str {
        match self {
            Tab::Connect => "Connect",
            Tab::Profiles => "Profiles",
            Tab::Settings => "Settings",
        }
    }
}

pub fn view<'a>(active: Tab) -> Element<'a, Message> {
    let segments = [Tab::Connect, Tab::Profiles, Tab::Settings]
        .into_iter()
        .map(|tab| segment(tab, active));

    let mut r = row![]
        .spacing(2)
        .width(Length::Fill)
        .align_y(iced::Alignment::Center);
    for seg in segments {
        r = r.push(seg);
    }

    container(r)
        .padding(4)
        .width(Length::Fill)
        .style(theme::tab_row)
        .into()
}

fn segment<'a>(tab: Tab, active: Tab) -> Element<'a, Message> {
    let is_active = tab == active;
    let label = text(tab.label()).size(14).center();
    let btn = button(container(label).center_x(Length::Fill))
        .padding([8, 10])
        .width(Length::Fill)
        .on_press(Message::TabChanged(tab));

    let btn = if is_active {
        btn.style(theme::tab_active)
    } else {
        btn.style(theme::tab_inactive)
    };

    btn.into()
}
