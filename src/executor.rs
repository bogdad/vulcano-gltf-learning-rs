use futures::executor::ThreadPool;
use futures::future::{Future, RemoteHandle};
use futures::task::SpawnExt;

#[derive(Debug, Clone)]
pub struct Executor {
  thread_pool: ThreadPool,
}

impl Executor {
  pub fn new(thread_pool: ThreadPool) -> Self {
    Executor { thread_pool }
  }

  pub fn do_background<Fut>(&self, future: Fut) -> RemoteHandle<()>
  where
    Fut: Future<Output = ()> + Send + 'static,
  {
    self.thread_pool.spawn_with_handle(future).unwrap()
  }
}
