use crate::chess::ZobristKey;

#[derive(Debug, Clone, Copy)]
struct CorrHistEntry(f32);

impl CorrHistEntry {
    fn update(&mut self, new_value: f32, weight: f32) {
        self.0 = self.0 * (1.0 - weight) + new_value * weight;
    }
}

#[derive(Debug)]
pub struct CorrHist {
    entries: Vec<CorrHistEntry>,
}

impl CorrHist {
    const NUM_ENTRIES: usize = 16384;

    pub fn new() -> Self {
        Self {
            entries: vec![CorrHistEntry(0.0); Self::NUM_ENTRIES],
        }
    }

    pub fn get_corr(&self, key: ZobristKey) -> f32 {
        return self.entries[key.value() as usize % Self::NUM_ENTRIES].0;
    }

    pub fn update_corr(&mut self, key: ZobristKey, q: f32, static_eval: f32) {
        let entry = &mut self.entries[key.value() as usize % Self::NUM_ENTRIES];
        entry.update(q - static_eval, 1.0 / 256.0);
    }
}
