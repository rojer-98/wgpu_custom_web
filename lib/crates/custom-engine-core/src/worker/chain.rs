use derive_more::{Deref, DerefMut};

use crate::{errors::CoreError, traits::RenderWorker, worker::Worker};

#[derive(Debug, Deref, DerefMut)]
pub struct WorkerChain<'a>(Vec<Box<Worker<'a>>>);

impl<'a> WorkerChain<'a> {
    pub fn render(&mut self, rw: &'a mut impl RenderWorker) -> Result<(), CoreError> {
        self.0
            .iter_mut()
            .map(|w| rw.render(w))
            .fold(Ok(()), |acc, res| acc.and(res))
    }
}
