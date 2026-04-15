use iced::widget::{Space, button, column, container, row, text, text_editor};
use iced::{Element, Length};

use crate::app::Message;
use crate::ui::theme;

pub fn view<'a>(
    log_content: &'a text_editor::Content,
    show_logs: bool,
    log_count: usize,
) -> Element<'a, Message> {
    let chevron = if show_logs { "▾" } else { "▸" };
    let toggle_label = format!("{chevron} Log Output ({log_count})");

    let toggle_btn = button(text(toggle_label).size(13))
        .on_press(Message::ToggleLogs)
        .padding([4, 8])
        .style(button::text);

    let mut header = row![toggle_btn, Space::new().width(Length::Fill)]
        .spacing(f32::from(theme::SPACE_SM))
        .align_y(iced::Alignment::Center);

    if log_count > 0 {
        header = header
            .push(
                button(text("Copy").size(12))
                    .on_press(Message::CopyLogs)
                    .padding([4, 10])
                    .style(button::text),
            )
            .push(
                button(text("Clear").size(12))
                    .on_press(Message::ClearLogs)
                    .padding([4, 10])
                    .style(button::text),
            );
    }

    let mut col = column![header].spacing(f32::from(theme::SPACE_XS));

    if show_logs {
        let editor = text_editor(log_content)
            .on_action(Message::LogEditorAction)
            .height(Length::Fill)
            .size(11);

        col = col.push(
            container(editor)
                .padding(theme::SPACE_SM)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(theme::card),
        );
    }

    col.into()
}
