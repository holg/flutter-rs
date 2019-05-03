use std::sync::{Arc, RwLock, Weak};

use crate::{
    channel::{Channel, EventHandler, MethodCallHandler},
    codec::{standard_codec::CODEC, MethodCall, MethodCodec, Value},
    desktop_window_state::{InitData, RuntimeData},
    error::MethodCallError,
};

use log::error;

pub struct EventChannel {
    name: String,
    init_data: Weak<InitData>,
    method_handler: Arc<RwLock<MethodCallHandler>>,
    plugin_name: Option<&'static str>,
}

struct EventChannelMethodCallHandler {
    event_handler: Weak<RwLock<EventHandler>>,
}

impl EventChannel {
    pub fn new(name: &str, handler: Weak<RwLock<EventHandler>>) -> Self {
        Self {
            name: name.to_owned(),
            init_data: Weak::new(),
            method_handler: Arc::new(RwLock::new(EventChannelMethodCallHandler::new(handler))),
            plugin_name: None,
        }
    }
}

impl Channel for EventChannel {
    fn name(&self) -> &str {
        &self.name
    }

    fn init_data(&self) -> Option<Arc<InitData>> {
        self.init_data.upgrade()
    }

    fn init(&mut self, init_data: Weak<InitData>, plugin_name: &'static str) {
        if self.init_data.upgrade().is_some() {
            error!("Channel {} was already initialized", self.name);
        }
        self.init_data = init_data;
        self.plugin_name.replace(plugin_name);
    }

    fn method_handler(&self) -> Option<Arc<RwLock<MethodCallHandler>>> {
        Some(Arc::clone(&self.method_handler))
    }

    fn plugin_name(&self) -> &'static str {
        self.plugin_name.unwrap()
    }

    fn codec(&self) -> &MethodCodec {
        &CODEC
    }
}

impl EventChannelMethodCallHandler {
    pub fn new(handler: Weak<RwLock<EventHandler>>) -> Self {
        Self {
            event_handler: handler,
        }
    }
}

impl MethodCallHandler for EventChannelMethodCallHandler {
    fn on_method_call(
        &mut self,
        channel: &str,
        call: MethodCall,
        _: Arc<RuntimeData>,
    ) -> Result<Value, MethodCallError> {
        if let Some(handler) = self.event_handler.upgrade() {
            let mut handler = handler.write().unwrap();
            match call.method.as_str() {
                "listen" => handler.on_listen(channel, call.args),
                "cancel" => handler.on_cancel(channel),
                _ => Err(MethodCallError::NotImplemented),
            }
        } else {
            Err(MethodCallError::ChannelClosed)
        }
    }
}
