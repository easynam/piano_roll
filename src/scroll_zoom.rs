use iced::Vector;

use crate::handles;

pub struct ScrollZoomControls {

}

pub struct ScrollScaleAxis {
    pub scroll: f32,
    pub scale: f32,
}

#[derive(Default)]
pub struct ScrollZoomState {
    pub x: ScrollScaleAxis,
    pub y: ScrollScaleAxis,
}

impl Default for ScrollScaleAxis {
    fn default() -> Self {
        ScrollScaleAxis {
            scroll: 0.0,
            scale: 1.0,
        }
    }
}