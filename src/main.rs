use iced;
use iced::widget::{button, column, container, row, text};
use iced::Alignment::Center;
use iced::Length::Fill;
use iced::{Alignment, Element, Length};
use iced_lowres_test::lowres::LowRes;

//type Renderer = iced_wgpu::Renderer;
type Renderer = LowRes;

fn main() -> iced::Result {
    iced::run::<_, _, _, Renderer>("A cool counter", update, view)
}

fn update(counter: &mut u64, message: Message) {
    match dbg!(message) {
        Message::Increment => *counter += 1,
        Message::Decrement => {
            *counter = match *counter {
                ..=0 => *counter,
                1.. => *counter - 1,
            }
        }
    }
}

fn view(counter: &u64) -> Element<Message, iced::Theme, Renderer> {
    container(
        container(
            column![
                button("hello")
                    .width(50.)
                    .height(30.)
                    .on_press(Message::Increment),
                button(text(counter.to_string())).width(50.).height(30.),
                button("!!")
                    .width(50.)
                    .height(30.)
                    .on_press(Message::Decrement),
            ]
            .spacing(10.),
        )
        .style(container::bordered_box)
        .align_x(Center),
    )
    .center(Fill)
    .style(container::bordered_box)
    .into()
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
