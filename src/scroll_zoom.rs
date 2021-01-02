use iced_native::{Point, Rectangle};
use crate::helpers::RectangleHelpers;

pub struct ScrollScaleAxis {
    pub view_start: f32,
    pub view_end: f32,
    pub content_size: f32,
}

impl ScrollScaleAxis {
    pub fn scroll(&self) -> f32 {
        self.view_start
    }

    pub fn scale(&self, bounds_size: f32) -> f32 {
        bounds_size / (self.view_end - self.view_start)
    }

    pub fn view_width(&self) -> f32 {
        self.view_end - self.view_start
    }

    pub fn start_proportion(&self) -> f32 {
        self.view_start / self.content_size
    }

    pub fn view_proportion(&self) -> f32 {
        self.view_width() / self.content_size
    }

    pub fn screen_to_inner(&self, pos: f32, bounds_offset: f32, bounds_size: f32) -> f32 {
        pos / self.scale(bounds_size) + self.scroll() - bounds_offset
    }

    pub fn inner_to_screen(&self, pos: f32, bounds_offset: f32, bounds_size: f32) -> f32 {
        (pos + bounds_offset - self.scroll()) * self.scale(bounds_size)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ScrollScaleAxisChange {
    ContentSize(f32),
    Left(f32),
    Right(f32),
}

#[derive(Default)]
pub struct ScrollZoomState {
    pub x: ScrollScaleAxis,
    pub y: ScrollScaleAxis,
}

impl Default for ScrollScaleAxis {
    fn default() -> Self {
        ScrollScaleAxis {
            view_start: 0.0,
            view_end: 1000.0,
            content_size: 2000.0,
        }
    }
}

impl ScrollZoomState {
    pub fn screen_to_inner(&self, pos: Point, bounds: &Rectangle) -> Point {
        Point::new(
            self.x.screen_to_inner(pos.x, bounds.x,bounds.width),
            self.y.screen_to_inner(pos.y, bounds.y,bounds.height),
        )
    }

    pub fn inner_to_screen(&self, pos: Point, bounds: &Rectangle) -> Point {
        Point::new(
            self.x.screen_to_inner(pos.x, bounds.x,bounds.width),
            self.y.screen_to_inner(pos.y, bounds.y,bounds.height)
        )
    }

    pub fn inner_rect_to_screen(&self, rect: Rectangle, bounds: &Rectangle) -> Rectangle {
        Rectangle {
            x: self.x.inner_to_screen(rect.x, bounds.x,bounds.width),
            y: self.y.inner_to_screen(rect.y, bounds.y,bounds.height),
            width: rect.width * self.x.scale(bounds.width),
            height: rect.height * self.y.scale(bounds.height),
        }
    }
}
