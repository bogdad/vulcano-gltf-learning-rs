pub struct CountingAvg {
  avg: f32,
  cnt: u32,
}

impl CountingAvg {
  pub fn new() -> Self {
    CountingAvg {
      avg: 0.0,
      cnt: 0,
    }
  }

  pub fn add(&mut self, t: u32) {
    if self.cnt == 0 {
      self.avg = t as f32;
      self.cnt = 1;
    } else {
      let sum = self.avg * (self.cnt as f32);
      let new_avg = (sum + t as f32) / (self.cnt as f32 + 1.0);
      self.avg = new_avg;
      self.cnt += 1;
    }
  }

  pub fn count(&self) -> f32 {
    self.avg
  }
}

pub struct CountingWindowAvg {
  all: CountingAvg,
  before: CountingAvg,
  limit: u32,
  vec: Vec<u32>,
}

impl CountingWindowAvg {
  pub fn new(limit: u32) -> Self {
    CountingWindowAvg {
      all: CountingAvg::new(),
      before: CountingAvg::new(),
      limit,
      vec: vec![0; (limit * 2) as usize],
    }
  }

  pub fn add(&mut self, t: u32) {
    if self.all.cnt > 1000 {
      self.all.cnt = 0;
      self.all.avg = 0.0;
      self.before.cnt = 0;
      self.before.avg = 0.0;
      self.vec.clear();
    }
    self.all.add(t);
    self.vec.push(t);
    if self.all.cnt > self.limit {
      self.before.add(self.vec[ self.vec.len() - self.limit as usize ]);
      if self.vec.len() == (self.limit * 2) as usize {
        self.vec.drain(0 .. self.limit as usize);
      }
    }
  }

  pub fn count(&self) -> f32 {
    if self.all.cnt == 0 {
      return 0.0;
    }
    let sum_all = self.all.count() * self.all.cnt as f32;
    let sum_before = self.before.count() * self.before.cnt as f32;
    return (sum_all - sum_before) / ((self.all.cnt - self.before.cnt) as f32);
  }

  pub fn all_count(&self) -> f32 {
    self.all.count()
  }
}