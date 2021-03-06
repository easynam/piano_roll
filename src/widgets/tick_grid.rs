#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineType {
    Bar(i32),
    Beat,
    InBetween,
}

pub struct GridLine {
    pub tick: i32,
    pub line_type: LineType,
}

pub trait TickGrid {
    fn get_grid_lines(&self, start: i32, end: i32) -> Vec<GridLine>;
    fn quantize_tick(&self, tick: i32) -> i32;
    fn grid_size(&self, tick: i32) -> i32;
}

pub struct SimpleGrid {
    pub(crate) ticks_per_16th: i32,
}

impl TickGrid for SimpleGrid {
    fn get_grid_lines(&self, start: i32, end: i32) -> Vec<GridLine> {
        (start..=end+1)
            .filter(|n| n % self.ticks_per_16th == 0)
            .map(|n| n / self.ticks_per_16th)
            .map(|n| GridLine {
                tick: n * self.ticks_per_16th,
                line_type: if n % 16 == 0 { LineType::Bar(n/16 + 1) }
                else if n % 4 == 0 { LineType::Beat }
                else { LineType::InBetween }
            })
            .collect()
    }

    fn quantize_tick(&self, tick: i32) -> i32 {
        return ((tick + self.ticks_per_16th/2) / self.ticks_per_16th) * self.ticks_per_16th;
    }

    fn grid_size(&self, _tick: i32) -> i32 {
        return self.ticks_per_16th;
    }
}