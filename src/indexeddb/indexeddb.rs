use crate::{value_to_js, js_to_value_ignore};
use crate::indexeddb::quadstore;
use super::{WINDOW, MultiSender, poll_receiver, spawn};

use std::collections::BTreeSet;
use std::pin::Pin;
use std::future::Future;
use std::task::{Poll, Context};
use futures_channel::oneshot;
use web_sys::{
    IdbKeyRange, 
    IdbDatabase, 
    IdbRequest, 
    IdbVersionChangeEvent, 
    DomException, 
    IdbTransaction, 
    IdbTransactionMode, 
    IdbCursorWithValue, 
    IdbObjectStore, 
    IdbObjectStoreParameters
};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

use gizmo_db::graph::value::Value;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::cmp::Ordering;
use gizmo_db::graph::quad::Direction;
use gizmo_db::graph::quad;

#[derive(Debug)]
struct Request {
    _on_success: Closure<dyn FnMut(&JsValue)>,
    _on_error: Closure<dyn FnMut(&JsValue)>,
}

impl Request {
    fn new<A, B>(request: &IdbRequest, on_success: A, on_error: B) -> Self
        where A: FnOnce(JsValue) + 'static,
              B: FnOnce(DomException) + 'static {

        let on_success = {
            let request = request.clone();

            Closure::once(move |_event: &JsValue| {
                on_success(request.result().unwrap());
            })
        };

        let on_error = {
            let request = request.clone();

            Closure::once(move |_event: &JsValue| {
                on_error(request.error().unwrap().unwrap());
            })
        };

        // TODO use addEventListener ?
        request.set_onsuccess(Some(on_success.as_ref().unchecked_ref()));

        request.set_onerror(Some(on_error.as_ref().unchecked_ref()));

        Self {
            _on_success: on_success,
            _on_error: on_error,
        }
    }
}


#[derive(Debug)]
struct TransactionFuture {
    receiver: oneshot::Receiver<Result<(), JsValue>>,
    _on_complete: Closure<dyn FnMut(&JsValue)>,
    _on_error: Closure<dyn FnMut(&JsValue)>,
    _on_abort: Closure<dyn FnMut(&JsValue)>,
}

impl TransactionFuture {
    fn new(tx: &IdbTransaction) -> Self {
        let (sender, receiver) = oneshot::channel();

        let sender = MultiSender::new(sender);

        let on_complete = {
            let sender = sender.clone();

            Closure::once(move |_event: &JsValue| {
                sender.send(Ok(()));
            })
        };

        let on_error = {
            let tx = tx.clone();
            let sender = sender.clone();

            Closure::once(move |_event: &JsValue| {
                let error = tx.error().unwrap();

                sender.send(Err(error.into()));
            })
        };

        let on_abort = Closure::once(move |_event: &JsValue| {
            // TODO better error handling
            sender.send(Err(js_sys::Error::new("Transaction aborted").into()));
        });

        // TODO use addEventListener ?
        tx.set_oncomplete(Some(on_complete.as_ref().unchecked_ref()));

        tx.set_onerror(Some(on_error.as_ref().unchecked_ref()));

        tx.set_onabort(Some(on_abort.as_ref().unchecked_ref()));

        Self {
            receiver,
            _on_complete: on_complete,
            _on_error: on_error,
            _on_abort: on_abort,
        }
    }
}

impl Future for TransactionFuture {
    type Output = Result<(), JsValue>;

    #[inline]
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        poll_receiver(&mut self.receiver, cx)
    }
}


#[derive(Debug)]
struct RequestFuture<A> {
    receiver: oneshot::Receiver<Result<A, JsValue>>,
    _request: Request,
}

impl<A> RequestFuture<A> where A: 'static {
    fn new_raw<F>(
        request: &IdbRequest,
        sender: MultiSender<Result<A, JsValue>>,
        receiver: oneshot::Receiver<Result<A, JsValue>>,
        map: F,
    ) -> Self
        where F: FnOnce(JsValue) -> A + 'static {

        let onsuccess = {
            let sender = sender.clone();

            move |result| {
                sender.send(Ok(map(result)));
            }
        };

        let onerror = move |error: DomException| {
            sender.send(Err(error.into()));
        };

        Self {
            receiver,
            _request: Request::new(&request, onsuccess, onerror),
        }
    }

    fn new<F>(request: &IdbRequest, map: F) -> Self
        where F: FnOnce(JsValue) -> A + 'static {

        let (sender, receiver) = oneshot::channel();

        let sender = MultiSender::new(sender);

        Self::new_raw(request, sender, receiver, map)
    }
}

impl<A> Future for RequestFuture<A> {
    type Output = Result<A, JsValue>;

    #[inline]
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        poll_receiver(&mut self.receiver, cx)
    }
}


#[derive(Debug)]
struct DbOpen {
    future: RequestFuture<Db>,
    _onupgradeneeded: Closure<dyn FnMut(&IdbVersionChangeEvent)>,
    _onblocked: Closure<dyn FnMut(&JsValue)>,
}

impl Future for DbOpen {
    type Output = Result<Db, JsValue>;

    #[inline]
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        Pin::new(&mut self.future).poll(cx)
    }
}


#[derive(Debug)]
struct Fold<A> {
    _on_success: Closure<dyn FnMut(&JsValue)>,
    _on_error: Closure<dyn FnMut(&JsValue)>,
    receiver: oneshot::Receiver<Result<A, JsValue>>,
}

impl<A> Future for Fold<A> {
    type Output = Result<A, JsValue>;

    #[inline]
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        poll_receiver(&mut self.receiver, cx)
    }
}


trait Cursor {
    fn next(&self);
}


#[derive(Debug)]
pub struct ReadCursor {
    cursor: IdbCursorWithValue,
}

impl ReadCursor {
    pub fn key(&self) -> JsValue {
        self.cursor.key().unwrap()
    }

    pub fn value(&self) -> JsValue {
        self.cursor.value().unwrap()
    }
}

impl Cursor for ReadCursor {
    #[inline]
    fn next(&self) {
        self.cursor.continue_().unwrap();
    }
}


#[derive(Debug)]
pub struct WriteCursor {
    cursor: ReadCursor,
}

impl WriteCursor {
    pub fn delete(&self) {
        self.cursor.cursor.delete().unwrap();
    }

    pub fn update(&self, value: &JsValue) {
        self.cursor.cursor.update(value).unwrap();
    }
}

impl std::ops::Deref for WriteCursor {
    type Target = ReadCursor;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.cursor
    }
}

impl Cursor for WriteCursor {
    #[inline]
    fn next(&self) {
        self.cursor.next();
    }
}







#[wasm_bindgen]
extern "C" {
    pub type InternalQuad;

    #[wasm_bindgen(method, getter)]
    pub fn s(this: &InternalQuad) -> JsValue;

    #[wasm_bindgen(method, getter)]
    pub fn p(this: &InternalQuad) -> JsValue;

    #[wasm_bindgen(method, getter)]
    pub fn o(this: &InternalQuad) -> JsValue;
    
    #[wasm_bindgen(method, getter)]
    pub fn l(this: &InternalQuad) -> JsValue;
}

impl InternalQuad {
    fn encode(quad: &quad::InternalQuad) -> InternalQuad {
        let x = js_sys::Object::new();

        js_sys::Reflect::set(&x, &JsValue::from(wasm_bindgen::intern("s")), &JsValue::from_f64(quad.s as f64)).unwrap();
        js_sys::Reflect::set(&x, &JsValue::from(wasm_bindgen::intern("p")), &JsValue::from_f64(quad.p as f64)).unwrap();
        js_sys::Reflect::set(&x, &JsValue::from(wasm_bindgen::intern("o")), &JsValue::from_f64(quad.o as f64)).unwrap();
        js_sys::Reflect::set(&x, &JsValue::from(wasm_bindgen::intern("l")), &JsValue::from_f64(quad.l as f64)).unwrap();

        x.unchecked_into()
    }

    fn decode(quad: &InternalQuad) -> quad::InternalQuad {
        quad::InternalQuad {
            s: quad.s().as_f64().unwrap_or(0f64) as u64,
            p: quad.p().as_f64().unwrap_or(0f64) as u64,
            o: quad.o().as_f64().unwrap_or(0f64) as u64,
            l: quad.l().as_f64().unwrap_or(0f64) as u64,
        }
    }
}


#[wasm_bindgen]
extern "C" {
    pub type Primitive;

    #[wasm_bindgen(method, getter)]
    pub fn id(this: &Primitive) -> JsValue;

    #[wasm_bindgen(method, getter)]
    pub fn hash(this: &Primitive) -> JsValue;

    #[wasm_bindgen(method, getter)]
    pub fn refs(this: &Primitive) -> JsValue;
    
    #[wasm_bindgen(method, getter)]
    pub fn is_quad(this: &Primitive) -> JsValue;

    #[wasm_bindgen(method, getter)]
    pub fn content(this: &Primitive) -> JsValue;
}

impl Primitive {

    fn encode_insert(p:& quadstore::Primitive) -> Primitive {
        let x = js_sys::Object::new();

        js_sys::Reflect::set(&x, &JsValue::from(wasm_bindgen::intern("hash")), &JsValue::from_str(&p.hash.to_string())).unwrap();
        js_sys::Reflect::set(&x, &JsValue::from(wasm_bindgen::intern("refs")), &JsValue::from_f64(p.id as f64)).unwrap();

        match &p.content {
            quadstore::PrimitiveContent::InternalQuad(q) => {
                js_sys::Reflect::set(&x, &JsValue::from(wasm_bindgen::intern("is_quad")), &JsValue::from_bool(true)).unwrap();
                js_sys::Reflect::set(&x, &JsValue::from(wasm_bindgen::intern("content")), &InternalQuad::encode(&q)).unwrap();
            },
            quadstore::PrimitiveContent::Value(v) => {
                js_sys::Reflect::set(&x, &JsValue::from(wasm_bindgen::intern("is_quad")), &JsValue::from_bool(false)).unwrap();
                js_sys::Reflect::set(&x, &JsValue::from(wasm_bindgen::intern("content")), &value_to_js(&v)).unwrap();
            }
        }
    
        x.unchecked_into()
    }

    fn encode_update(p:& quadstore::Primitive) -> Primitive {
        let prim = Primitive::encode_insert(p);
        js_sys::Reflect::set(&prim, &JsValue::from(wasm_bindgen::intern("id")), &JsValue::from_f64(p.id as f64)).unwrap();
        return prim
    }

    fn decode(p: &Primitive) -> quadstore::Primitive {
        quadstore::Primitive {
            id: p.id().as_f64().unwrap_or(0f64) as u64,
            hash: p.hash().as_string().unwrap_or("0".to_string()).parse().unwrap_or(0u64),
            refs: p.refs().as_f64().unwrap_or(0f64) as u64,
            content: if p.is_quad().is_truthy() {
                quadstore::PrimitiveContent::InternalQuad(InternalQuad::decode(&p.content().unchecked_into()))
            } else {
                quadstore::PrimitiveContent::Value(js_to_value_ignore(&p.content()))
            }
        }
    }
}


#[wasm_bindgen]
extern "C" {
    pub type QuadDirection;

    #[wasm_bindgen(method, getter)]
    pub fn key(this: &QuadDirection) -> JsValue;

    #[wasm_bindgen(method, getter)]
    pub fn value_id(this: &QuadDirection) -> JsValue;

    #[wasm_bindgen(method, getter)]
    pub fn direction(this: &QuadDirection) -> JsValue;

    #[wasm_bindgen(method, getter)]
    pub fn quad_id(this: &QuadDirection) -> JsValue;
}

impl QuadDirection {
    // TODO make this more efficient
    pub fn encode(value_id: u64, direction: &Direction, quad_id: u64) -> Self {
        let x = js_sys::Object::new();

        let key = format!("{}{:0>19}{:0>19}", direction.to_byte(), value_id, quad_id);
        js_sys::Reflect::set(&x, &JsValue::from(wasm_bindgen::intern("key")), &JsValue::from_str(&key)).unwrap();
        js_sys::Reflect::set(&x, &JsValue::from(wasm_bindgen::intern("value_id")), &JsValue::from_f64(value_id as f64)).unwrap();
        js_sys::Reflect::set(&x, &JsValue::from(wasm_bindgen::intern("direction")),&JsValue::from_f64(direction.to_byte() as f64)).unwrap();
        js_sys::Reflect::set(&x, &JsValue::from(wasm_bindgen::intern("quad_id")), &JsValue::from_f64(quad_id as f64)).unwrap();

        x.unchecked_into()
    }

    pub fn decode(quad_direction: &QuadDirection) -> (u64, Direction, u64) {
        let x = js_sys::Object::new();

        let value_id: u64 = quad_direction.value_id().as_f64().unwrap_or(0f64) as u64;
        let direction: Direction = Direction::from_byte(quad_direction.direction().as_f64().unwrap_or(0f64) as u8).unwrap();
        let quad_id: u64 = quad_direction.quad_id().as_f64().unwrap_or(0f64) as u64;

        (value_id, direction, quad_id)
    }
}


#[derive(Debug)]
pub struct Read {
    tx: IdbTransaction,
}

impl Read {
    fn store(&self, name: &str) -> IdbObjectStore {
        self.tx.object_store(wasm_bindgen::intern(name)).unwrap()
    }

    pub fn get_primitive(&self, id: u64) -> impl Future<Output = Result<Option<quadstore::Primitive>, JsValue>> {
        let store = self.store("primitive");
        let req = store.get(&JsValue::from_str(&id.to_string())).unwrap();
        RequestFuture::new(&req, move |value| {
            if value.is_undefined() {
                return None
            } else {
                return Some(Primitive::decode(&value.unchecked_into()));
            }
        })
    }

    pub fn get_primitive_from_hash(&self, hash: u64) -> impl Future<Output = Result<Option<quadstore::Primitive>, JsValue>> {
        let store = self.store("primitive");
        let index = store.index(wasm_bindgen::intern("hash")).unwrap();

        let req = index.get(&JsValue::from_str(&hash.to_string())).unwrap();

        RequestFuture::new(&req, move |value| {
            if value.is_undefined() {
                return None
            } else {
                return Some(Primitive::decode(&value.unchecked_into()));
            }
        })
    }

    pub fn get_quad_direction(&self, direction: &Direction, value_id: &u64) -> impl Future<Output = Result<BTreeSet<u64>, JsValue>> {

        let from = format!("{}{:0>19}", direction.to_byte(), value_id);
        let to = format!("{}{:0>19}~", direction.to_byte(), value_id);

        let start = IdbKeyRange::bound(&JsValue::from_str(&from), &JsValue::from_str(&to)).unwrap().into();
        let store = self.store("quad_direction");

        let req = store.get_all_with_key(&start).unwrap();

        RequestFuture::new(&req, move |values| {
            let values: js_sys::Array = values.unchecked_into();
            values.iter().filter_map(|value| {
                let quad_direction: QuadDirection = value.unchecked_into();
                quad_direction.quad_id().as_string().unwrap_or("0".to_string()).parse::<u64>().ok()
            }).collect()
        })
    }
}


#[derive(Debug)]
pub struct Write {
    read: Read,
}

impl Write {
    pub fn insert_primitive(&self, record: &quadstore::Primitive) -> Result<u64, String> {
        match self.store("primitive").add(&Primitive::encode_insert(record)) {
            Ok(request) => match request.result() {
                Ok(result) => Ok(result.as_f64().unwrap_or(0f64) as u64),
                Err(_) => Err("Unable to insert primitive".to_string())
            }
            Err(_) => Err("Unable to insert primitive".to_string())
        }
    }

    pub fn update_primitive(&self, record: &quadstore::Primitive) -> Result<(), String> {
        match self.store("primitive").put(&Primitive::encode_update(record)) {
            Ok(_) => Ok(()),
            Err(_) => Err("Unable to update primitive".to_string())
        }
    }

    pub fn remove_primitive(&self, key: u64) -> Result<(), String> {
        match self.store("primitive").delete(&JsValue::from_str(&key.to_string())) {
            Ok(_) => Ok(()),
            Err(_) => Err("Unable to remove primitive".to_string())
        }
    }
    
    pub fn insert_quad_direction(&self, value_id: u64, direction: &Direction, quad_id: u64) -> Result<(), String> {
        match self.store("quad_direction").add(&QuadDirection::encode(value_id, direction, quad_id)) {
            Ok(_) => Ok(()),
            Err(_) => Err("Unable to insert quad direction".to_string())
        }
    }

    pub fn remove_quad_direction(&self, value_id: u64, direction: &Direction, quad_id: u64) -> Result<(), String> {
        let key = format!("{}{:0>19}{:0>19}", direction.to_byte(), value_id, quad_id);
        match self.store("quad_direction").delete(&JsValue::from_str(&key)) {
            Ok(_) => Ok(()),
            Err(_) => Err("Unable to remove quad direction".to_string())
        }
    }
}

impl std::ops::Deref for Write {
    type Target = Read;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.read
    }
}


#[derive(Debug, Clone)]
pub struct TableOptions {
    pub auto_increment: bool,
    pub key_path: String
}


#[derive(Debug)]
pub struct Upgrade {
    db: IdbDatabase,
    write: Write,
}

impl Upgrade {
    pub fn create_table(&self, name: &str, options: &TableOptions) -> Result<IdbObjectStore, JsValue> {
        self.db.create_object_store_with_optional_parameters(
            wasm_bindgen::intern(name),
            IdbObjectStoreParameters::new()
                .auto_increment(options.auto_increment)
                .key_path(Some(&JsValue::from(wasm_bindgen::intern(&options.key_path)))),
        )
    }

    pub fn delete_table(&self, name: &str) {
        // TODO intern this ?
        self.db.delete_object_store(name).unwrap();
    }
}

impl std::ops::Deref for Upgrade {
    type Target = Write;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.write
    }
}


#[derive(Debug)]
pub struct Db {
    db: IdbDatabase,
}

impl Db {
    // TODO this should actually be u64
    // TODO handle versionchange event
    pub fn open<A, F>(name: &str, version: u32, on_upgrade: F) -> impl Future<Output = Result<Self, JsValue>>
        // TODO remove the 'static from A ?
        where A: Future<Output = Result<(), JsValue>> + 'static,
              F: FnOnce(Upgrade, Option<u32>, u32) -> A + 'static {

        let (sender, receiver) = oneshot::channel();

        let sender = MultiSender::new(sender);

        let request = WINDOW.with(|x| x.indexed_db()
            .unwrap()
            .unwrap()
            // TODO should this intern the name ?
            .open_with_u32(wasm_bindgen::intern(name), version)
            .unwrap());

        let onupgradeneeded = {
            let request = request.clone();

            Closure::once(move |event: &IdbVersionChangeEvent| {
                // TODO are these u32 conversions correct ?
                let old_version = event.old_version() as u32;
                let new_version = event.new_version().unwrap() as u32;

                let db = request.result().unwrap().dyn_into().unwrap();

                let tx = request.transaction().unwrap();

                // TODO test that this always works correctly
                let complete = TransactionFuture::new(&tx);

                // TODO test this with oldVersion and newVersion
                let fut = on_upgrade(
                    Upgrade { db, write: Write { read: Read { tx } } },

                    if old_version == 0 {
                        None
                    } else {
                        Some(old_version)
                    },

                    new_version,
                );

                spawn(async move {
                    fut.await?;
                    complete.await?;
                    Ok(())
                });
            })
        };

        let onblocked = {
            let sender = sender.clone();

            Closure::once(move |_event: &JsValue| {
                // TODO better error handling
                sender.send(Err(js_sys::Error::new("Database is blocked").into()));
            })
        };

        request.set_onupgradeneeded(Some(onupgradeneeded.as_ref().unchecked_ref()));

        request.set_onblocked(Some(onblocked.as_ref().unchecked_ref()));

        DbOpen {
            future: RequestFuture::new_raw(&request, sender, receiver, move |result| {
                Self {
                    db: result.dyn_into().unwrap(),
                }
            }),
            _onupgradeneeded: onupgradeneeded,
            _onblocked: onblocked,
        }
    }

    fn transaction(&self, names: &[&str], mode: IdbTransactionMode) -> IdbTransaction {
        // TODO can the names be converted more efficiently ?
        // TODO verify that the names are interned properly when calling JsValue::from
        let names = names.into_iter().map(|x| JsValue::from(wasm_bindgen::intern(*x))).collect::<js_sys::Array>();

        self.db.transaction_with_str_sequence_and_mode(&names, mode).unwrap()
    }

    pub fn read<A, B, F>(&self, names: &[&str], f: F) -> impl Future<Output = Result<A, JsValue>>
        where B: Future<Output = Result<A, JsValue>>,
              F: FnOnce(Read) -> B {

        let tx = self.transaction(names, IdbTransactionMode::Readonly);

        // TODO test that this always works correctly
        let complete = TransactionFuture::new(&tx);

        // TODO should this be inside the async ?
        let fut = f(Read { tx });

        async move {
            let value = fut.await?;
            complete.await?;
            Ok(value)
        }
    }

    pub fn write<A, B, F>(&self, names: &[&str], f: F) -> impl Future<Output = Result<A, JsValue>>
        where B: Future<Output = Result<A, JsValue>>,
              F: FnOnce(Write) -> B {

        let tx = self.transaction(names, IdbTransactionMode::Readwrite);

        // TODO test that this always works correctly
        let complete = TransactionFuture::new(&tx);

        // TODO should this be inside the async ?
        let fut = f(Write { read: Read { tx } });

        async move {
            let value = fut.await?;
            complete.await?;
            Ok(value)
        }
    }
}

impl Drop for Db {
    #[inline]
    fn drop(&mut self) {
        self.db.close();
    }
}
