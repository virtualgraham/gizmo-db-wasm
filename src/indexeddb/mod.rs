mod indexeddb;
mod all_iterator;
mod quadstore;

use wasm_bindgen_futures::spawn_local;
use std::rc::Rc;
use std::cell::RefCell;
use futures_channel::oneshot;
use std::task::{Poll, Context};
use std::pin::Pin;
use std::future::Future;
use wasm_bindgen::JsValue;

use web_sys::{
    window, 
    Window
};


thread_local! {
    pub static WINDOW: Window = window().unwrap();
}

#[derive(Debug)]
pub struct MultiSender<A> {
    sender: Rc<RefCell<Option<oneshot::Sender<A>>>>,
}

impl<A> MultiSender<A> {
    pub fn new(sender: oneshot::Sender<A>) -> Self {
        Self {
            sender: Rc::new(RefCell::new(Some(sender))),
        }
    }

    pub fn send(&self, value: A) {
        let _ = self.sender.borrow_mut()
            .take()
            .unwrap()
            .send(value);
    }
}

impl<A> Clone for MultiSender<A> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

pub fn poll_receiver<A>(receiver: &mut oneshot::Receiver<A>, cx: &mut Context) -> Poll<A> {
    Pin::new(receiver).poll(cx).map(|x| {
        // TODO better error handling
        match x {
            Ok(x) => x,
            Err(_) => unreachable!(),
        }
    })
}

pub fn spawn<A>(future: A) where A: Future<Output = Result<(), JsValue>> + 'static {
    spawn_local(async move {
        // TODO replace with a wasm-bindgen-futures API
        if let Err(value) = future.await {
            wasm_bindgen::throw_val(value);
        }
    })
}

#[macro_export]
macro_rules! closure {
    (move || -> $ret:ty $body:block) => {
        wasm_bindgen::closure::Closure::wrap(std::boxed::Box::new(move || -> $ret { $body }) as std::boxed::Box<dyn FnMut() -> $ret>)
    };
    (move |$($arg:ident: $type:ty),*| -> $ret:ty $body:block) => {
        wasm_bindgen::closure::Closure::wrap(std::boxed::Box::new(move |$($arg: $type),*| -> $ret { $body }) as std::boxed::Box<dyn FnMut($($type),*) -> $ret>)
    };
    (move || $body:block) => {
        $crate::closure!(move || -> () $body)
    };
    (move |$($arg:ident: $type:ty),*| $body:block) => {
        $crate::closure!(move |$($arg: $type),*| -> () $body);
    };
}

