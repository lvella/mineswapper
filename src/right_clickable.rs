use delegate::delegate;
use iced_native::{
    event::Status,
    layout::{Limits, Node},
    mouse::{self, Button, Interaction},
    overlay,
    renderer::Style,
    widget::{tree::State, tree::Tag, Operation, Tree},
    Clipboard, Event, Layout, Length, Point, Rectangle, Renderer, Shell, Widget,
};

/// A wrapper to handle right click on a widget, if the widget originally
/// ignored right clicks.
pub struct RightClickable<T, Message> {
    inner: T,
    on_right_click: Option<Message>,
}

impl<T, Message> RightClickable<T, Message> {
    pub fn new(inner: T) -> Self {
        RightClickable {
            inner,
            on_right_click: None,
        }
    }

    pub fn on_right_click(mut self, msg: Message) -> Self {
        self.on_right_click = Some(msg);
        self
    }
}

impl<T, Message: Clone, R> Widget<Message, R> for RightClickable<T, Message>
where
    R: Renderer,
    T: Widget<Message, R>,
{
    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &R,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<Message>,
    ) -> Status {
        if let Status::Captured = self.inner.on_event(
            tree,
            event.clone(),
            layout,
            cursor_position,
            renderer,
            clipboard,
            shell,
        ) {
            return Status::Captured;
        }

        match event {
            Event::Mouse(mouse::Event::ButtonReleased(Button::Right)) => {
                if let Some(msg) = &self.on_right_click {
                    let bounds = layout.bounds();

                    if bounds.contains(cursor_position) {
                        shell.publish((*msg).clone());
                    }

                    return Status::Captured;
                }
            }
            _ => {}
        }

        Status::Ignored
    }

    delegate! {
        to self.inner {
            fn width(&self) -> Length;

            fn height(&self) -> Length;

            fn layout(
                &self,
                renderer: &R,
                limits: &Limits,
            ) -> Node;

            fn draw(
                &self,
                tree: &Tree,
                renderer: &mut R,
                theme: &R::Theme,
                style: &Style,
                layout: Layout<'_>,
                cursor_position: Point,
                viewport: &Rectangle,
            );

            fn tag(&self) -> Tag;

            fn state(&self) -> State;

            fn children(&self) -> Vec<Tree>;

            fn diff(&self, _tree: &mut Tree);

            fn operate(
                &self,
                _state: &mut Tree,
                _layout: Layout<'_>,
                _renderer: &R,
                _operation: &mut dyn Operation<Message>
            );

            fn mouse_interaction(
                &self,
                _state: &Tree,
                _layout: Layout<'_>,
                _cursor_position: Point,
                _viewport: &Rectangle,
                _renderer: &R
            ) -> Interaction;

            fn overlay<'a>(
                &'a mut self,
                _state: &'a mut Tree,
                _layout: Layout<'_>,
                _renderer: &R
            ) -> Option<overlay::Element<'a, Message, R>>;
        }
    }
}

impl<'a, T, Message: Clone, Renderer> Into<iced_native::Element<'a, Message, Renderer>>
    for RightClickable<T, Message>
where
    Renderer: iced_native::Renderer,
    RightClickable<T, Message>: iced_native::Widget<Message, Renderer>,
    T: 'a,
    Message: 'a,
{
    fn into(self) -> iced_native::Element<'a, Message, Renderer> {
        iced_native::Element::new(self)
    }
}
