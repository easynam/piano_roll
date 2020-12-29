pub struct ScrollScaleAxis {
    pub view_start: f32,
    pub view_end: f32,
    pub content_size: f32,
}

impl ScrollScaleAxis {
    pub fn scroll(&self) -> f32 {
        self.view_start
    }

    pub fn scale(&self, bounds_width: f32) -> f32 {
        bounds_width / (self.view_end - self.view_start)
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
