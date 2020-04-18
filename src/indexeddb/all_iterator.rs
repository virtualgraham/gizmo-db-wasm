use gizmo_db::graph::refs::{Size, Ref};
use gizmo_db::graph::iterator::{Base, Scanner, Index, Shape, Costs, ShapeType};

use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

use super::quadstore::InternalIndexedDb;

use std::sync::Arc;


pub struct IndexedDbAllIterator {
    db: Arc<InternalIndexedDb>,
    nodes: bool
}

impl IndexedDbAllIterator {
    pub fn new(db: Arc<InternalIndexedDb>, nodes: bool) -> Rc<RefCell<IndexedDbAllIterator>> {
        Rc::new(RefCell::new(IndexedDbAllIterator {
            db,
            nodes
        }))
    }
}


impl Shape for IndexedDbAllIterator {

    fn iterate(&self) -> Rc<RefCell<dyn Scanner>> {
        IndexedDbAllIteratorNext::new(self.db.clone(), self.nodes)
    }

    fn lookup(&self) -> Rc<RefCell<dyn Index>> {
        IndexedDbAllIteratorContains::new(self.db.clone(), self.nodes)
    }

    fn stats(&mut self) -> Result<Costs, String> {
        let count = self.db.get_count()?;

        Ok(Costs {
            contains_cost: 1,
            next_cost: 1,
            size: Size {
                value: count.total() as i64,
                exact: true
            }
        })
    }

    fn optimize(&mut self) -> Option<Rc<RefCell<dyn Shape>>> {
        None
    }

    fn sub_iterators(&self) -> Option<Vec<Rc<RefCell<dyn Shape>>>> {
        None
    }

    fn shape_type(&mut self) -> ShapeType {
        ShapeType::StoreIterator
    }

}



pub struct IndexedDbAllIteratorNext {
    db: Arc<InternalIndexedDb>,
    nodes: bool,
    done: bool,
    cur: Option<Ref>
}


impl IndexedDbAllIteratorNext {
    pub fn new(db: Arc<InternalIndexedDb>, nodes: bool) -> Rc<RefCell<IndexedDbAllIteratorNext>> {
        Rc::new(RefCell::new(IndexedDbAllIteratorNext {
            db,
            nodes,
            done: false,
            cur: None
        }))
    }
}


impl Base for IndexedDbAllIteratorNext {
    fn tag_results(&self, _tags: &mut HashMap<String, Ref>) {}

    fn result(&self) -> Option<Ref> {
        return self.cur.clone()
    }

    fn next_path(&mut self) -> bool {
        false
    }

    fn err(&self) -> Option<String> {
        None
    }

    fn close(&mut self) -> Result<(), String> {
        self.done = true;
        Ok(())
    }
}


impl Scanner for IndexedDbAllIteratorNext {
    fn next(&mut self) -> bool {
        
    //     if self.done {
    //         return false
    //     }

    //     // TODO: node and quad primitives should have a different prefix, this would require changing Ref to know if the key is for a value or quad 

    //     let lam = |(_, v):(Box<[u8]>, Box<[u8]>)| {
    //         match Primitive::decode(&v) {
    //             Ok(p) => {
    //                 let is_node = p.is_node();

    //                 if self.nodes && is_node {
    //                     return Some(p)
    //                 } else if !self.nodes && !is_node {
    //                     return Some(p)
    //                 } 
    
    //                 return None
    //             },
    //             Err(_) => {
    //                 // TODO: result() should return Result<Option<>>
    //                 return None
    //             }
    //         }
    //     };

    //     self.cur = if !self.done && self.cur.is_none() {

    //         self.db.db.iterator(
    //             IteratorMode::Start
    //         ).take_while(|(k,_)| {
    //             !k.is_empty() && k[0] == PRIMITIVE_KEY_PREFIX
    //         }).filter_map(
    //             lam
    //         ).map(|p| {
    //             p.to_ref(self.nodes).unwrap()
    //         }).next()

    //     } else {

    //         self.db.db.iterator(
    //             IteratorMode::From(&primitive_key(self.cur.as_ref().unwrap().k.unwrap() + 1), rocksdb::Direction::Forward)
    //         ).take_while(|(k,_)| {
    //             !k.is_empty() && k[0] == PRIMITIVE_KEY_PREFIX
    //         }).filter_map(
    //             lam
    //         ).map(|p| {
    //             p.to_ref(self.nodes).unwrap()
    //         }).next()

    //     };

    //     if !self.cur.is_some() {
    //         self.done = true;
    //         return false
    //     }

    //     return true
        return false
    }
}



pub struct IndexedDbAllIteratorContains {
    db: Arc<InternalIndexedDb>,
    nodes: bool,
    cur: Option<Ref>,
    done: bool
}


impl IndexedDbAllIteratorContains {
    pub fn new(db: Arc<InternalIndexedDb>, nodes: bool) -> Rc<RefCell<IndexedDbAllIteratorContains>> {
        Rc::new(RefCell::new(IndexedDbAllIteratorContains {
            db,
            nodes,
            cur: None,
            done: false
        }))
    }
}


impl Base for IndexedDbAllIteratorContains {
    fn tag_results(&self, _tags: &mut HashMap<String, Ref>) {}

    fn result(&self) -> Option<Ref> {
        return self.cur.clone()
    }

    fn next_path(&mut self) -> bool {
        false
    }

    fn err(&self) -> Option<String> {
        None
    }

    fn close(&mut self) -> Result<(), String> {
        self.done = true;
        Ok(())
    }
}


impl Index for IndexedDbAllIteratorContains {
    fn contains(&mut self, v:&Ref) -> bool {
        if self.done {
            return false
        }

        let id = v.key();
        
        match id {
            Some(i) => {
                match self.db.get_primitive(i) {
                    Ok(prim) => {
                        if let Some(p) = prim {
                           self.cur = p.to_ref(self.nodes);
                           return true
                        }
                        self.cur = None;
                        return false  
                    },
                    Err(_) => {
                        // TODO: change impl to return result
                        return false
                    }
                }
            },
            None => return false
        }
    }
}