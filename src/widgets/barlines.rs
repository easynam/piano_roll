pub enum LineType {
    Bar,
    Beat,
    InBetween,
}

pub struct GridLine {
    pub tick: u32,
    pub line_type: LineType,
}

pub trait QuantizeGrid {
    fn get_grid_lines(&self, start: u32, end: u32) -> Vec<GridLine>;
    fn quantize_tick(&self, tick: u32) -> u32;
}

pub struct SimpleGrid {
    pub(crate) ticks_per_16th: u32,
}

impl QuantizeGrid for SimpleGrid {
    fn get_grid_lines(&self, start: u32, end: u32) -> Vec<GridLine> {
        (start..=end+1)
            .filter(|n| n % self.ticks_per_16th == 0)
            .map(|n| n / self.ticks_per_16th)
            .map(|n| GridLine {
                tick: n * self.ticks_per_16th,
                line_type: if n % 16 == 0 { LineType::Bar }
                else if n % 4 == 0 { LineType::Beat }
                else { LineType::InBetween }
            })
            .collect()
    }

    fn quantize_tick(&self, tick: u32) -> u32 {
        return ((tick + self.ticks_per_16th/2) / self.ticks_per_16th) * self.ticks_per_16th;
    }
}