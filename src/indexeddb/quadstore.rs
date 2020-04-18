use gizmo_db::graph::value::Value;
use gizmo_db::graph::refs::{Size, Ref, Namer, Content};
use gizmo_db::graph::iterator::{Shape, Null};
use gizmo_db::graph::quad::{QuadStore, Quad, Direction, Stats, Delta, IgnoreOptions, InternalQuad, Procedure};
use gizmo_db::graph::iterator::quad_ids::QuadIds;

use std::rc::Rc;
use std::cell::RefCell;
use std::hash::Hash;
// use std::io::Cursor;
// use byteorder::{BigEndian, WriteBytesExt, ReadBytesExt, ByteOrder};
use std::collections::BTreeSet;
use std::sync::Arc;

use super::all_iterator::IndexedDbAllIterator;
use super::indexeddb::{Db, TableOptions, QuadDirection};

use futures::executor::block_on;
use wasm_bindgen::prelude::*;

pub struct InternalIndexedDb {
    pub db: Db
}

const VERSION:i32 = 1;


impl InternalIndexedDb {

    fn open(name: &str) -> Result<Self, String> {
        let db = block_on(
            Db::open(name, 1, |tx, _old, _new| {
                async move {
                    let object_store = tx.create_table("primitives", &TableOptions {
                        auto_increment: true,
                        key_path: "id".to_string(),
                    }).map_err(|_| "Unable to open database table".to_string())?;

                    object_store.create_index_with_str("hash", "hash")
                        .map_err(|_| "Unable to open database table".to_string())?;

                    tx.create_table("quad_direction", &TableOptions {
                        auto_increment: false,
                        key_path: "key".to_string()
                    }).map_err(|_| "Unable to open database table")?;

                    Ok(())
                }
            })
        ).map_err(|_| "Unable to open database".to_string())?;

        Ok(InternalIndexedDb {
            db
        })
    }

    // Primitives

    pub fn get_count(&self) -> Result<PrimitiveCount, String> {
        // TODO: Implement
        Ok(PrimitiveCount {
            values: 10,
            quads: 10
        })
    }


    pub fn get_primitive(&self, id: u64) -> Result<Option<Primitive>, String> {
        let primitive = block_on(
            self.db.read(&["primitives"], move |tx| {
                async move {
                    let primitive = tx.get_primitive(id).await?;
                    Ok(primitive)
                }
            })
        ).map_err(|_| "Unable to get primitive".to_string())?;

        Ok(primitive)
    }

    pub fn get_primitive_from_hash(&self, hash: u64) -> Result<Option<Primitive>, String> {
        let primitive = block_on(
            self.db.read(&["primitives"], move |tx| {
                async move {
                    let primitive = tx.get_primitive_from_hash(hash).await?;
                    Ok(primitive)
                }
            })
        ).map_err(|_| "Unable to get primitive".to_string())?;

        Ok(primitive)
    }


    // Only call this method after you have checked that the primitive does not yet exist
    fn update_primitive(&self, p: &Primitive) -> Result<(), String> {
        block_on(
            async move {
                self.db.write(&["primitives"], move |tx| {
                    async move {
                        tx.update_primitive(p);
                        Ok(())
                    }
                }).await.map_err(|_| "Unable to add primitive".to_string())
            }
        ).map_err(|_| "Unable to add primitive".to_string())?;

        Ok(())
    }


    // Only call this method after you have checked that the primitive does not yet exist
    fn add_primitive(&self, p: &mut Primitive) -> Result<(), String> {
        block_on(
            async move {
                self.db.write(&["primitives"], move |tx| {
                    async move {
                        let id = tx.insert_primitive(p)?;
                        p.id = id;
                        Ok(())
                    }
                }).await
            }
        ).map_err(|_| "Unable to add primitive".to_string())?;

        Ok(())
    }

    
    fn remove_primitive(&self, p: &Primitive) -> Result<(), String> {
        block_on(
            async move {
                self.db.write(&["primitives"], move |tx| {
                    async move {
                        tx.remove_primitive(p.id)?;
                        Ok(())
                    }
                }).await
            }
        ).map_err(|_| "Unable to add primitive".to_string())?;

        Ok(())
    }

    // Quad Direction Index

    fn get_quad_direction(&self, direction: &Direction, value_id: &u64) -> Result<BTreeSet<u64>, String> {
        let quad_directions = block_on(
            self.db.read(&["quad_direction"], move |tx| {
                async move {
                    let quad_directions = tx.get_quad_direction(direction, value_id).await?;
                    Ok(quad_directions)
                }
            })
        ).map_err(|_| "Unable to get primitive".to_string())?;

        Ok(quad_directions)
    }

    fn add_quad_direction(&self, value_id: u64, direction: &Direction, quad_id: u64) -> Result<(), String> {
        block_on(
            async move {
                self.db.write(&["quad_direction"], move |tx| {
                    async move {
                        tx.insert_quad_direction(value_id, direction, quad_id)?;
                        Ok(())
                    }
                }).await
            }
        ).map_err(|_| "Unable to add primitive".to_string())?;

        Ok(())
    }

    fn remove_quad_direction(&self, value_id: u64, direction: &Direction, quad_id: u64) -> Result<(), String> {
        block_on(
            async move {
                self.db.write(&["quad_direction"], move |tx| {
                    async move {
                        let id = tx.remove_quad_direction(value_id, direction, quad_id);
                        Ok(id)
                    }
                }).await.map_err(|_| "Unable to add quad_direction".to_string())?
            }
        ).map_err(|_| "Unable to add quad_direction".to_string())?;

        Ok(())
    }


    ///////////////////////


    fn resolve_val(&self, v: &Value, add: bool) -> Result<Option<u64>, String> {
        if let Value::None = v {
            return Ok(None)
        }

        let hash = v.calc_hash();
        
        let prim = self.get_primitive_from_hash(hash)?;
        
        if prim.is_some() || !add {
            // if the value exsists and we are adding it, increment refs
            let res = prim.as_ref().map(|p| p.id);
            
            if prim.is_some() && add {
                let mut p = prim.unwrap();
                p.refs += 1;

                self.update_primitive(&p)?; // update p.refs

            }
            
            return Ok(res)
        }

        let mut prim = Primitive::new_value(v.clone());
        self.add_primitive(&mut prim)?;

        Ok(Some(prim.id))
    }

    

    fn resolve_quad(&self, q: &Quad, add: bool) -> Result<Option<InternalQuad>, String> {
        let mut p = InternalQuad{s: 0, p: 0, o: 0, l: 0};

        // find all value ids for each direction of quad
        for dir in Direction::iterator() {
            let v = q.get(dir);
            if let Value::None = v {
                continue
            }
            let vid = self.resolve_val(v, add)?;
            if  let Some(i) = vid {
                p.set_dir(dir, i);
            } else {
                // if any value is not found or undefined return zero value internal quad
                return Ok(None)
            }
        }

        return Ok(Some(p))
    }


    fn find_quad(&self, q: &Quad) -> Result<Option<Primitive>, String> {
        let quad = self.resolve_quad(q, false)?;
        if let Some(q) = quad {
            return Ok(self.get_primitive_from_hash(q.calc_hash())?)
        }
        Ok(None)
    }


    fn delete_quad_nodes(&self, q: &InternalQuad) -> Result<(), String> {
        for dir in Direction::iterator() {
            let id = q.dir(dir);
            if id == 0 {
                continue
            }

            if let Some(mut p) = self.get_primitive(id)? { // value

                if p.refs == 0 {
                    return Err("remove of delete node".to_string())
                } 

                p.refs -= 1;
                
                if p.refs == 0 {

                    self.remove_primitive(&p)?;

                } else {

                    if let Err(_) = self.update_primitive(&p) { // value
                        return Err("read/write error".to_string())
                    }

                }
            }
        }

        Ok(())
    }


    fn resolve_quad_default(&self, q: &Quad, add: bool) -> Result<InternalQuad, String> {
        match self.resolve_quad(q, add)? {
            Some(q) => Ok(q),
            None => Ok(InternalQuad{s: 0, p: 0, o: 0, l: 0})
        }
    }


    fn delete(&self, id: u64) -> Result<bool, String> {
        let mut quad:Option<InternalQuad> = None;
 
        if let Some(p) = self.get_primitive(id)? {
            if let PrimitiveContent::InternalQuad(q) = &p.content {
                quad = Some(q.clone());
            }

            self.remove_primitive(&p)?;
        } else {
            return Ok(false)
        }
        
        if let Some(q) = quad {
            for d in Direction::iterator() {
                self.remove_quad_direction(q.dir(d), d, id)?;
            }

            self.delete_quad_nodes(&q)?;
        }

        return Ok(true)
    }


    fn add_quad(&self, q: Quad) -> Result<u64, String> {
        // get value_ids for each direction
        let p = self.resolve_quad_default(&q, false)?;

        // get quad id
        let hash = p.calc_hash();

        let prim = self.get_primitive_from_hash(hash)?;

        // if prim already exsits
        if let Some(p) = prim {
            return Ok(p.id)
        }

        // get value_ids for each direction, this time inserting the values as neccecery
        let p = self.resolve_quad_default(&q, true)?;

        // add value primitive
        let mut pr = Primitive::new_quad(p.clone());
        let id = self.add_primitive(&mut pr)?;

        // add to index
        for d in Direction::iterator() {
            self.add_quad_direction(p.dir(d), d, pr.id)?;
        }

        return Ok(pr.id);
    }


    fn lookup_val(&self, id: u64) -> Result<Option<Value>, String> {
        match self.get_primitive(id)? {
            Some(p) => {
                match p.content {
                    PrimitiveContent::Value(v) => Ok(Some(v)),
                    _ => Ok(None)
                }
            },
            None => Ok(None)
        }
    }


    fn internal_quad(&self, r: &Ref) -> Result<Option<InternalQuad>, String> {
        let key = if let Some(k) = r.key() { 
            self.get_primitive(k)?
        } else { 
            None 
        };

        match key {
            Some(p) => {
                match p.content {
                    PrimitiveContent::InternalQuad(q) => Ok(Some(q)),
                    _ => Ok(None)
                }
            },
            None => Ok(None)
        }
    }

    
    fn lookup_quad_dirs(&self, p: InternalQuad) -> Result<Quad, String> {
        let mut q = Quad::new_undefined_vals();
        for dir in Direction::iterator() {
            let vid = p.dir(dir);
            if vid == 0 {
                continue
            }
            let val = self.lookup_val(vid)?;
            if let Some(v) = val {
                q.set_val(dir, v);
            }
        }
        return Ok(q)
    }

}



pub struct IndexedDb {
    store: Arc<InternalIndexedDb>
}

impl IndexedDb {
    pub fn open(path: &str) -> Result<IndexedDb, String> {
        Ok(IndexedDb {
            store: Arc::new(InternalIndexedDb::open(path)?)
        })
    }
}

impl Namer for IndexedDb {
    fn value_of(&self, v: &Value) -> Option<Ref> {
        if let Value::None = v {
            return None
        }

        let hash = v.calc_hash();

        if let Ok(Some(prim)) = self.store.get_primitive_from_hash(hash) { // TODO: this method should return Result<Option>
            Some(Ref {
                k: Some(prim.id),
                content: Content::Value(v.clone())
            })
        } else {
            None
        }
    }
 
    fn name_of(&self, key: &Ref) -> Option<Value> {
        if let Content::Value(v) = &key.content {
            return Some(v.clone())
        }

        if let Some(i) = key.key() {
            if let Ok(v) = self.store.lookup_val(i) {
                return v
            } else {    
                // TODO: return Err
                return None
            }
        } else {
            return None
        }
    }
}


impl QuadStore for IndexedDb {
    fn quad(&self, r: &Ref) -> Option<Quad> {

        let internal_quad:Option<InternalQuad> = match &r.content {
            Content::Quad(q) => {
                return Some(q.clone())
            },
            Content::InternalQuad(iq) => {
                Some(iq.clone())
            }
            _ => {
                match self.store.internal_quad(r) {
                    Ok(iq) => {
                        iq
                    },
                    Err(_) => {
                        // TODO: return Err
                        return None
                    } 
                }
            }
        };

        match internal_quad {
            Some(q) => {
                if let Ok(dirs) = self.store.lookup_quad_dirs(q) {
                    return Some(dirs)
                } else {
                    // TODO: return Err
                    return None
                }
            }
            None => None
        }
    }

    fn quad_iterator(&self, d: &Direction, r: &Ref) -> Rc<RefCell<dyn Shape>> {
        if let Some(i) = r.key() {
            if let Ok(quad_ids) = self.store.get_quad_direction(d, &i) {
                if !quad_ids.is_empty() {
                    return QuadIds::new(Rc::new(quad_ids), d.clone())
                }
            }
        } 
            
        Null::new()
    }

    fn quad_iterator_size(&self, d: &Direction, r: &Ref) -> Result<Size, String> {
        if let Some(i) = r.key() {

            let quad_ids = self.store.get_quad_direction(d, &i)?;

            if !quad_ids.is_empty() {
                return Ok(Size{value: quad_ids.len() as i64, exact: true})
            }
        } 
            
        return Ok(Size{value: 0, exact: true})
    }

    fn quad_direction(&self, r: &Ref, d: &Direction) -> Option<Ref> {
        let quad = match self.store.internal_quad(r) {
            Ok(q) => q,
            Err(_) => {
                return None
                // TODO: return Result<Option>>
            }
        };

        match quad {
            Some(q) => {
                let id = q.dir(d);
                if id == 0 {
                    // The quad exsists, but the value is none
                    return Some(Ref::none())
                }
                return Some(Ref {
                    k: Some(id),
                    content: Content::None
                })
            }
            // the quad does not exsist
            None => None
        }
    }

    fn stats(&self, _exact: bool) -> Result<Stats, String> {
        let count = self.store.get_count()?;

        Ok(Stats {
            nodes: Size {
                value: count.values as i64,
                exact: true
            },
            quads: Size {
                value: count.quads as i64,
                exact: true
            }
        })
    }
    
    fn apply_deltas(&mut self, deltas: Vec<Delta>, ignore_opts: &IgnoreOptions) -> Result<(), String> {
        if !ignore_opts.ignore_dup || !ignore_opts.ignore_missing {
            for d in &deltas {
                match d.action {
                    Procedure::Add => {
                        if !ignore_opts.ignore_dup {
                            if let Some(_) = self.store.find_quad(&d.quad)? {
                                return Err("ErrQuadExists".into())
                            }
                        }
                    },
                    Procedure::Delete => {
                        if !ignore_opts.ignore_missing {
                            if let Some(_) = self.store.find_quad(&d.quad)? {
                            } else {
                                return Err("ErrQuadNotExist".into())
                            }
                        }
                    },
                }
            }
        }

        for d in &deltas {
            match &d.action {
                Procedure::Add => {
                    self.store.add_quad(d.quad.clone())?;
                },
                Procedure::Delete => {
                   if let Some(prim) = self.store.find_quad(&d.quad)? {
                    self.store.delete(prim.id)?;
                   }
                }
            }
        }

        Ok(())
    }

    fn nodes_all_iterator(&self) -> Rc<RefCell<dyn Shape>> {
        IndexedDbAllIterator::new(self.store.clone(), true)
  
    }

    fn quads_all_iterator(&self) -> Rc<RefCell<dyn Shape>> {
        IndexedDbAllIterator::new(self.store.clone(), false)
    }

    fn close(&self) -> Option<String> {
        // TODO: how to close the IndexedDB, destroy()?
        return None
    }
}

pub struct PrimitiveCount {
    values: u64,
    quads: u64
}

impl PrimitiveCount {
    fn zero() -> PrimitiveCount {
        PrimitiveCount {
            values: 0,
            quads: 0
        }
    }

    pub fn total(&self) -> u64 {
        return self.values + self.quads
    }

    fn increment_quads(&mut self, n: i64) {
        if n < 0 {
            let m = n.abs() as u64;
            if m > self.quads {
                // return Err("Attempted to set quad count to less than 0".to_string());
            } else {
                self.quads -= m;
            }
        } else {
            if n as u64 > u64::max_value() - self.quads  {
                // return Err("quad count is invalid u64::max_value()".to_string());
            } else {
                self.quads += n as u64;
            }
        }
    }

    fn increment_values(&mut self, n: i64) {
        if n < 0 {
            let m = n.abs() as u64;
            if m > self.values {
                // return Err("Attempted to set quad count to less than 0".to_string());
            } else {
                self.values -= m;
            }
        } else {
            if n as u64 > u64::max_value() - self.values  {
                // return Err("quad count is invalid u64::max_value()".to_string());
            } else {
                self.values += n as u64;
            }
        }
    }
}



#[derive(Clone, PartialEq, Debug)]
pub struct Primitive {
    pub id: u64,
    pub hash: u64,
    pub refs: u64,
    pub content: PrimitiveContent
}


impl Primitive {

    fn calc_hash(&self) -> u64 {
        match &self.content {
            PrimitiveContent::Value(v) => {
                return v.calc_hash()
            },
            PrimitiveContent::InternalQuad(q) => {
                return q.calc_hash()
            },
        }
    }

    pub fn to_ref(&self, nodes: bool) -> Option<Ref> {

        match &self.content {
            PrimitiveContent::Value(v) => {
                if nodes {
                    return Some(Ref {
                        k: Some(self.id),
                        content: Content::Value(v.clone())
                    });
                }
            },
            PrimitiveContent::InternalQuad(q) => {
                if !nodes {
                    return Some(Ref {
                        k: Some(self.id),
                        content: Content::InternalQuad(q.clone())
                    });
                }
            }
        }

        return None
    }

    pub fn is_quad(&self) -> bool {
        if let PrimitiveContent::InternalQuad(_) = self.content {
            return true
        }

        return false
    }

    pub fn is_node(&self) -> bool {
        if let PrimitiveContent::Value(_) = self.content {
            return true
        }

        return false
    }

    pub fn new_value(v: Value) -> Primitive {
        let hash = v.calc_hash();
        let pc = PrimitiveContent::Value(v);
        Primitive {
            id: 0,
            hash: hash,
            content: pc,
            refs: 1
        }
    }

    pub fn new_quad(q: InternalQuad) -> Primitive {
        let hash = q.calc_hash();
        let pc = PrimitiveContent::InternalQuad(q);

        Primitive {
            id: 0,
            hash: hash,
            content: pc,
            refs: 1
        }
    }

    fn new(content: PrimitiveContent) -> Primitive {
        let hash = match &content {
            PrimitiveContent::Value(v) => v.calc_hash(),
            PrimitiveContent::InternalQuad(q) => q.calc_hash(),
        };

        Primitive {
            id: 0,
            hash: hash,
            refs: 1,
            content
        }
    }

}

#[derive(Clone, PartialEq, Debug, Hash)]
pub enum PrimitiveContent {
    Value(Value),
    InternalQuad(InternalQuad)
}


#[test]
fn testing() {
   
}