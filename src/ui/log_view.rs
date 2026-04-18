use iced::widget::{Space, button, column, container, row, text, text_editor};
use iced::{Element, Length};

use crate::app::Message;
use crate::ui::theme;

pub fn view<'a>(
    log_content: &'a text_editor::Content,
    log_count: usize,
) -> Element<'a, Message> {
    let title = text(format!("Log Output ({log_count})")).size(14);

    let mut header = row![title, Space::new().width(Length::Fill)]
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

    let editor = text_editor(log_content)
        .on_action(Message::LogEditorAction)
        .height(Length::Fill)
        .size(12);

    column![
        header,
        container(editor)
            .padding(theme::SPACE_SM)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(theme::card),
    ]
    .spacing(f32::from(theme::SPACE_XS))
    .height(Length::Fill)
    .into()
}
