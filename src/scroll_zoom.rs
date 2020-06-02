use iced::Vector;

use crate::handles;

pub struct ScrollZoomControls {

}

pub struct ScrollZoomState {
    pub scroll_x: f32,
    pub scroll_y: f32,
    pub scale_x: f32,
    pub scale_y: f32,
}

impl Default for ScrollZoomState {
    fn default() -> Self {
        ScrollZoomState {
            scroll_x: 0.0,
            scroll_y: 0.0,
            scale_x: 1.0,
            scale_y: 1.0
        }
    }
}