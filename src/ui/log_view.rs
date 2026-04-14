use iced::widget::{button, column, container, row, text, text_editor};
use iced::{Element, Length};

use crate::app::Message;

pub fn view<'a>(
    log_content: &'a text_editor::Content,
    show_logs: bool,
    log_count: usize,
) -> Element<'a, Message> {
    let chevron = if show_logs { "▾" } else { "▸" };
    let toggle_label = format!("{chevron} Log Output ({log_count})");

    let header = row![
        button(text(toggle_label).size(13))
            .on_press(Message::ToggleLogs)
            .padding([4, 8])
            .style(button::text),
    ];

    let mut col = column![header].spacing(6);

    if show_logs {
        let editor = text_editor(log_content)
            .on_action(Message::LogEditorAction)
            .height(Length::Fill)
            .size(11);

        col = col.push(
            container(editor)
                .padding(4)
                .width(Length::Fill)
                .height(Length::Fill),
        );
    }

    col.into()
}
