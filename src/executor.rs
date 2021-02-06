
use futures::executor::{ThreadPool, block_on};
use futures::future::{Future, RemoteHandle, Shared};
use futures::task::SpawnExt;

#[derive(Debug, Clone)]
pub struct Executor {
  thread_pool: ThreadPool,
}

impl Executor {

  pub fn new(thread_pool: ThreadPool) -> Self {
    Executor {
      thread_pool,
    }
  }

  pub fn do_background<Fut>(&self, future: Fut) -> RemoteHandle<<Fut as Future>::Output>
   where
    Fut: Future + Send + 'static,
    <Fut as Future>::Output: Send, {
    self.thread_pool.spawn_with_handle(future).unwrap()
  }

  pub fn wait<T>(&self, handle: Shared<RemoteHandle<T>>) -> T
  where T: 'static + Clone,  {
    block_on(handle)
  }
}
