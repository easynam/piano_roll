use iced_native::{Rectangle, Point};

const SELECT_MIN_WIDTH: f32 = 12.0;
const RESIZE_INNER: f32 = 8.0;
const RESIZE_OUTER: f32 = 8.0;

pub trait RectangleHelpers {
    fn handle_right(&self) -> Rectangle;
    fn handle_left(&self) -> Rectangle;
    fn handle_up(&self) -> Rectangle;
    fn handle_down(&self) -> Rectangle;
    fn expand_to_bounds(&self, bounds: &Rectangle) -> Rectangle;
    fn normalize_within_bounds(&self, bounds: &Rectangle) -> Rectangle;
}

fn inner_size(width: f32) -> f32 {
    RESIZE_INNER.min(((width - SELECT_MIN_WIDTH) * 0.5).max(1.0))
}

impl RectangleHelpers for Rectangle {
    fn handle_right(&self) -> Rectangle {
        let inner_size = inner_size(self.width);

        Rectangle {
            x: self.x + self.width - inner_size,
            width: inner_size + RESIZE_OUTER,
            ..*self
        }
    }
    fn handle_left(&self) -> Rectangle {
        let inner_size = inner_size(self.width);

        Rectangle {
            x: self.x - RESIZE_OUTER,
            width: inner_size + RESIZE_OUTER,
            ..*self
        }
    }

    fn handle_up(&self) -> Rectangle {
        let inner_size = inner_size(self.width);

        Rectangle {
            y: self.y - RESIZE_OUTER,
            height: inner_size + RESIZE_OUTER,
            ..*self
        }
    }

    fn handle_down(&self) -> Rectangle {
        let inner_size = inner_size(self.width);

        Rectangle {
            y: self.y + self.height - inner_size,
            height: inner_size + RESIZE_OUTER,
            ..*self
        }
    }

    fn expand_to_bounds(&self, bounds: &Rectangle) -> Rectangle {
        Rectangle {
            x: bounds.x + self.x * bounds.width,
            y: bounds.y + self.y * bounds.height,
            width: self.width * bounds.width,
            height: self.height * bounds.height,
        }
    }

    fn normalize_within_bounds(&self, _bounds: &Rectangle) -> Rectangle<f32> {
        unimplemented!()
    }
}

pub trait PointHelpers {
    fn expand_to_bounds(&self, bounds: &Rectangle) -> Point;
    fn normalize_within_bounds(&self, bounds: &Rectangle) -> Point;
}

impl PointHelpers for Point {
    fn expand_to_bounds(&self, bounds: &Rectangle) -> Point {
        Point::new(bounds.x + self.x * bounds.width, bounds.y + self.y * bounds.height)
    }

    fn normalize_within_bounds(&self, bounds: &Rectangle) -> Point {
        Point::new((self.x - bounds.x) / bounds.width, (self.y - bounds.y) / bounds.height)
    }

}