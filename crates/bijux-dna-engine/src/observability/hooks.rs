use super::EngineEvent;

pub trait EngineHooks: Send + Sync {
    fn on_event(&self, event: EngineEvent);
}
