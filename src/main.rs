use iced;
use iced::widget::{button, container, row, text};
use iced::{Alignment, Element, Length};
use iced_lowres_test::lowres::LowRes;

fn main() -> iced::Result {
    iced::run::<_, _, _, LowRes>("A cool counter", update, view)
}

fn update(counter: &mut u64, message: Message) {
    match message {
        Message::Increment => *counter += 1,
        Message::Decrement => {
            *counter = match *counter {
                ..=0 => *counter,
                1.. => *counter - 1,
            }
        }
    }
}
fn view(counter: &u64) -> Element<Message, iced::Theme, LowRes> {
    container(()).into()
    /* !
    let counter_row = row![
        button("Decrement").on_press(Message::Decrement),
        text(counter),
        button("Increment").on_press(Message::Increment),
    ]
    .spacing(20)
    .align_y(Alignment::Center);
    container(counter_row)
        .padding(10)
        .center(Length::Fill)
        .into()
    */
}

#[derive(Debug, Clone)]
enum Message {
    Increment,
    Decrement,
}
