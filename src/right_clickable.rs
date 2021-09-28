use delegate::delegate;

pub struct RightClickable<T, Message>
{
    inner: T,
    on_right_click: Option<Message>
}

impl<T, Message> RightClickable<T, Message>
{
    pub fn new(inner: T) -> Self
    {
        RightClickable{inner, on_right_click: None}
    }

    pub fn on_right_click(mut self, msg: Message) -> Self
    {
        self.on_right_click = Some(msg);
        self
    }
}

impl<T, Message: Clone, Renderer> iced_native::Widget<Message, Renderer>
    for RightClickable<T, Message>
where
    Renderer: iced_native::Renderer,
    T: iced_native::Widget<Message, Renderer>
{
    delegate! {
        to self.inner {
            fn width(&self) -> iced::Length;
            fn height(&self) -> iced::Length;

            fn layout(
                &self, renderer: &Renderer, limits: &iced_native::layout::Limits
            ) -> iced_native::layout::Node;

            fn draw(
                &self, renderer: &mut Renderer, defaults: &Renderer::Defaults,
                layout: iced_native::Layout<'_>, cursor_position: iced_native::Point,
                viewport: &iced_native::Rectangle
            ) -> Renderer::Output;

            fn hash_layout(&self, state: &mut iced_native::Hasher);
        }
    }

    fn on_event(
        &mut self,
        event: iced_native::Event,
        layout: iced_native::Layout<'_>,
        cursor_position: iced_native::Point,
        renderer: &Renderer,
        clipboard: &mut dyn iced_native::Clipboard,
        messages: &mut Vec<Message>,
    ) -> iced_native::event::Status {
        if let iced_native::event::Status::Captured = self.inner.on_event(
            event.clone(),
            layout,
            cursor_position,
            renderer,
            clipboard,
            messages,
        ) {
            return iced_native::event::Status::Captured;
        }

        match event {
            iced_native::Event::Mouse(iced_native::mouse::Event::ButtonReleased(
                    iced_native::mouse::Button::Right)) => {
                if let Some(msg) = &self.on_right_click {
                    let bounds = layout.bounds();

                    if bounds.contains(cursor_position) {
                        messages.push((*msg).clone());
                    }

                    return iced_native::event::Status::Captured;
                }
            },
            _ => {}
        }

        iced_native::event::Status::Ignored
    }
}

impl<'a, T, Message: Clone, Renderer> Into<iced_native::Element<'a, Message, Renderer>>
    for RightClickable<T, Message>
where
    Renderer: iced_native::Renderer,
    RightClickable<T, Message>: iced_native::Widget<Message, Renderer>,
    T: 'a,
    Message: 'a
{
    fn into(self) -> iced_native::Element<'a, Message, Renderer> {
        iced_native::Element::new(self)
    }
}
