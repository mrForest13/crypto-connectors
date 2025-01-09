use crate::client::request::Channel;
use crate::model::Market;
use crate::utils::handler::Event;
use anyhow::Result;
use async_nats::Subject;
use prost::Message;

pub trait State<E, M: Message>: Default + Send {
    fn publish(&mut self, event: Event<E>) -> Result<M> {
        match event {
            Event::Get(_) => Ok(self.get()),
            Event::Updated(_, dto) => self.update(dto),
        }
    }

    fn update(&mut self, dto: E) -> Result<M>;

    fn get(&self) -> M;

    fn topic(&self, market: &Market) -> Subject;

    fn channel(&self) -> Channel;
}
