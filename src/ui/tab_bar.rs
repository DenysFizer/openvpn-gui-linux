use iced::widget::{button, container, row, text};
use iced::{Element, Length};

use crate::app::Message;
use crate::ui::theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Connect,
    Profiles,
    Logs,
    Settings,
}

impl Tab {
    fn label(self) -> &'static str {
        match self {
            Tab::Connect => "Connect",
            Tab::Profiles => "Profiles",
            Tab::Logs => "Logs",
            Tab::Settings => "Settings",
        }
    }
}

pub fn view<'a>(active: Tab, enable_logs: bool) -> Element<'a, Message> {
    let mut tabs: Vec<Tab> = vec![Tab::Connect, Tab::Profiles, Tab::Settings];
    if enable_logs {
        tabs.push(Tab::Logs);
    }

    let mut r = row![]
        .spacing(2)
        .width(Length::Fill)
        .align_y(iced::Alignment::Center);
    for tab in tabs {
        r = r.push(segment(tab, active));
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
