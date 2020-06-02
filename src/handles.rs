use iced_native::Rectangle;

const SELECT_MIN_WIDTH: f32 = 12.0;
const RESIZE_INNER: f32 = 8.0;
const RESIZE_OUTER: f32 = 8.0;

pub trait Handles {
    fn handle_right(self) -> Rectangle;
    fn handle_left(self) -> Rectangle;
    fn handle_up(self) -> Rectangle;
    fn handle_down(self) -> Rectangle;
}

fn inner_size(width: f32) -> f32 {
    RESIZE_INNER.min(((width - SELECT_MIN_WIDTH) * 0.5).max(1.0))
}

impl Handles for Rectangle {
    fn handle_right(self) -> Rectangle {
        let inner_size = inner_size(self.width);

        Rectangle {
            x: self.x + self.width - inner_size,
            width: inner_size + RESIZE_OUTER,
            ..self
        }
    }
    fn handle_left(self) -> Rectangle {
        let inner_size = inner_size(self.width);

        Rectangle {
            x: self.x - RESIZE_OUTER,
            width: inner_size + RESIZE_OUTER,
            ..self
        }
    }

    fn handle_up(self) -> Rectangle<f32> {
        let inner_size = inner_size(self.width);

        Rectangle {
            y: self.y - RESIZE_OUTER,
            height: inner_size + RESIZE_OUTER,
            ..self
        }
    }

    fn handle_down(self) -> Rectangle<f32> {
        let inner_size = inner_size(self.width);

        Rectangle {
            y: self.y + self.width - inner_size,
            height: inner_size + RESIZE_OUTER,
            ..self
        }
    }
}