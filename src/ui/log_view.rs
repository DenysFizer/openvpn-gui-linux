use iced::widget::{button, column, row, text, text_editor};
use iced::{Element, Length};

use crate::app::Message;

pub fn view<'a>(
    log_content: &'a text_editor::Content,
    show_logs: bool,
    log_count: usize,
) -> Element<'a, Message> {
    let mut col = column![].spacing(4);

    let toggle_label = if show_logs {
        format!("Log Output ({log_count}) [-]")
    } else {
        format!("Log Output ({log_count}) [+]")
    };

    let header = row![
        button(text(toggle_label).size(13))
            .on_press(Message::ToggleLogs)
            .padding([4, 8])
            .style(button::text),
    ];

    col = col.push(header);

    if show_logs {
        let editor = text_editor(log_content)
            .on_action(Message::LogEditorAction)
            .height(Length::Fill)
            .size(11);

        col = col.push(editor);
    }

    col.into()
}
