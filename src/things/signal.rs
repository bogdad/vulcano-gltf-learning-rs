use std::sync::{Mutex, Condvar};

pub struct Signal {
  lock: Mutex<bool>,
  cvar: Condvar,
}

impl Signal {
  pub fn new() -> Self {
    Signal {
      lock: Mutex::new(false),
      cvar: Condvar::new(),
    }
  }

  pub fn signal(&self) {
    let mut signalled = self.lock.lock().unwrap();
    *signalled = true;
    self.cvar.notify_one();
  }

  pub fn wait_and_reset(&self) {
    {
      let mut signaled = self.lock.lock().unwrap();
      // As long as the value inside the `Mutex<bool>` is `false`, we wait.
      while !*signaled {
        signaled = self.cvar.wait(signaled).unwrap();
      }
    }
    {
      // reset
      let mut signaled = self.lock.lock().unwrap();
      *signaled = false;
    }
  }
}