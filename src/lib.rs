
use wasm_bindgen::prelude::*;

use web_sys::console;
use js_sys;

use gizmo_db::query::path;
use gizmo_db::query::gizmo;

use gizmo_db::graph::quad::{QuadStore, QuadWriter, IgnoreOptions, Quad};
use gizmo_db::graph::memstore;
use gizmo_db::graph::iterator;
use gizmo_db::query::shape;
use gizmo_db::graph::value::Value;
use gizmo_db::graph::number::Number;
use gizmo_db::graph::refs::Ref;

use std::rc::Rc;
use std::cell::RefCell;

use std::collections::HashMap;

// When the `wee_alloc` feature is enabled, this uses `wee_alloc` as the global
// allocator.
//
// If you don't want to use `wee_alloc`, you can safely delete this.
// #[cfg(feature = "wee_alloc")]
// #[global_allocator]
// static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;


// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    // This provides better error messages in debug mode.
    // It's disabled in release mode so it doesn't bloat up the file size.
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    // Your code goes here!
    console::log_1(&JsValue::from_str("Gizmo DB Loaded"));

    Ok(())
}



#[wasm_bindgen(js_name = NewMemoryGraph)]
pub fn new_memory_graph() -> GraphWrapper {
    let qs = Rc::new(RefCell::new(memstore::quadstore::MemStore::new()));
    //let qs = Rc::new(RefCell::new(graphmock::Store::new()));

    let s = Rc::new(RefCell::new(Session {
        qs: qs.clone(),
        qw: QuadWriter::new(qs.clone(), IgnoreOptions{ignore_dup: true, ignore_missing: true})
    }));

    let g = Graph::new(s.clone());

    GraphWrapper {
        graph: g,
        session: s
    }
}


#[wasm_bindgen]
pub struct GraphWrapper {
    graph: Graph,
    session: Rc<RefCell<Session>>
}


#[wasm_bindgen]
impl GraphWrapper {
    pub fn graph(&self) -> Graph {
        return self.graph.clone();
    }

    pub fn g(&self) -> Graph {
        return self.graph.clone();
    }

    pub fn write(&self, quads: &JsValue) {
        self.session.borrow().write(quads)
    }

    pub fn read(&self) -> JsValue {
        self.session.borrow().read()
    }

    pub fn delete(&self, quads: &JsValue) {
        self.session.borrow().delete(quads)
    }
}


#[wasm_bindgen]
pub struct Session {
    qs: Rc<RefCell<dyn QuadStore>>,
    qw: QuadWriter
}


#[wasm_bindgen]
impl Session {
    fn write(&self, quads: &JsValue) {
        let quads: Vec<Quad> = js_array_to_quad_vec(quads);
        for quad in &quads {
            self.qw.add_quad(quad.clone()).unwrap();
        }
    }

    fn read(&self) -> JsValue {
        // TODO: implement
        quad_vec_to_js(&vec![Quad::new("a", "b", "c", "d")])
    }

    fn delete(&self, quads: &JsValue) {
        // TODO: implement
        // let quads: Vec<Quad> = quads.into_serde().unwrap();
    }

    fn run_tag_each_iterator(&mut self, it: Rc<RefCell<dyn iterator::Shape>>) -> iterator::iterate::TagEachIterator {
        iterator::iterate::TagEachIterator::new(it, false, true)
    }

    fn run_each_iterator(&mut self, it: Rc<RefCell<dyn iterator::Shape>>) -> iterator::iterate::EachIterator {
        iterator::iterate::EachIterator::new(it, false, true)
    }
}


#[wasm_bindgen]
#[derive(Clone)]
pub struct Graph {
    session: Rc<RefCell<Session>>,
}

#[wasm_bindgen]
impl Graph {
    fn new(session: Rc<RefCell<Session>>) -> Graph {
        Graph {
            session
        }
    }

    #[wasm_bindgen(js_name = _v)]
    pub fn v(&self, js_values: &JsValue) -> Path {
        Path::new(
            self.session.clone(), 
            true, 
            path::Path::start_path(
                Some(
                    self.session.borrow().qs.clone()
                ), 
                js_array_to_values_vec(js_values)
            )
        )
    }

    #[wasm_bindgen(js_name = _m)]
    pub fn m(&self) -> Path {
        Path::new(self.session.clone(), false, path::Path::start_morphism(Vec::new()))
    }
}


#[wasm_bindgen]
#[derive(Clone)]
pub struct Path {
    session: Rc<RefCell<Session>>,
    finals: bool,
    path: path::Path
}


#[wasm_bindgen]
impl Path {

    fn new(session: Rc<RefCell<Session>>, finals: bool, path: path::Path) -> Path {
        Path {
            session,
            finals,
            path
        }
    }


    fn build_iterator_tree(&self) -> Rc<RefCell<dyn iterator::Shape>> {
        let s = self.session.borrow();
        let qs = self.session.borrow().qs.clone();
        self.path.build_iterator_on(qs)
    }

    ///////////////
    // Finals
    ///////////////

    pub fn all(&self) -> TagIterator {
        let it = self.build_iterator_tree();
        let it = iterator::save::tag(&it, &"id");
        let qs = self.session.borrow().qs.clone();
        let iterator = self.session.borrow_mut().run_tag_each_iterator(it).filter_map(move |r| tags_to_value_map(&r, &*qs.borrow()));
        TagIterator {
            iterator: Box::new(iterator)
        }
    }

    pub fn values(&self) -> ValueIterator {
        let it = self.build_iterator_tree();
        let it = iterator::save::tag(&it, &"id");
        let qs = self.session.borrow().qs.clone();
        let iterator = self.session.borrow_mut().run_each_iterator(it).filter_map(move |r| ref_to_value(&r, &*qs.borrow()));

        ValueIterator {
            iterator: Box::new(iterator)
        }
    }

    pub fn count(&mut self) -> i64 {
        let it = self.build_iterator_tree();
        self.session.borrow_mut().run_each_iterator(it).count() as i64
    }


    ///////////////
    // Traversals
    ///////////////

    ///////////////////////////
    // Is(nodes: Value[])
    ///////////////////////////
    #[wasm_bindgen(js_name = _is)]
    pub fn is(&mut self, js_values: &JsValue) -> Result<Path, JsValue> {
        let nodes = js_array_to_values_vec(js_values);
        self.path.is(nodes);
        Ok(self.clone())
    }


    ///////////////////////////
    // In(values: String[], tags: String[])
    ///////////////////////////
    #[wasm_bindgen(js_name = _in_values)]
    pub fn in_values(&mut self, js_values: &JsValue, js_tags: &JsValue) -> Result<Path, JsValue> {
        let nodes = js_array_to_values_vec(js_values);
        let tags = js_array_to_tags_vec(js_tags);
        self.path.in_with_tags(tags, values_to_via(nodes));
        Ok(self.clone())
    }


    ///////////////////////////
    // In(path: Path, tags: String[])
    ///////////////////////////
    #[wasm_bindgen(js_name = _in_path)]
    pub fn in_path(&mut self, path: &Path, js_tags: &JsValue) -> Result<Path, JsValue> {
        let tags = js_array_to_tags_vec(js_tags);
        self.path.in_with_tags(tags, path::Via::Path(path.path.clone()));
        Ok(self.clone())
    }


    ///////////////////////////
    // Out(values: String[], tags: String[])
    ///////////////////////////
    #[wasm_bindgen(js_name = _out_values)]
    pub fn out_values(&mut self, js_values: &JsValue, js_tags: &JsValue) -> Result<Path, JsValue> {
        let nodes = js_array_to_values_vec(js_values);
        let tags = js_array_to_tags_vec(js_tags);
        self.path.out_with_tags(tags, values_to_via(nodes));
        Ok(self.clone())
    }


    ///////////////////////////
    // Out(path: Path, tags: String[])
    ///////////////////////////
    #[wasm_bindgen(js_name = _out_path)]
    pub fn out_path(&mut self, path: &Path, js_tags: &JsValue) -> Result<Path, JsValue> {
        let tags = js_array_to_tags_vec(js_tags);
        self.path.out_with_tags(tags, path::Via::Path(path.path.clone()));
        Ok(self.clone())
    }


    ///////////////////////////
    // Both(values: String[], tags: String[])
    ///////////////////////////
    #[wasm_bindgen(js_name = _both_values)]
    pub fn both_values(&mut self, js_values: &JsValue, js_tags: &JsValue) -> Result<Path, JsValue> {
        let nodes = js_array_to_values_vec(js_values);
        let tags = js_array_to_tags_vec(js_tags);
        self.path.both_with_tags(tags, values_to_via(nodes));
        Ok(self.clone())
    }


    ///////////////////////////
    // Both(path: Path, tags: String[])
    ///////////////////////////
    #[wasm_bindgen(js_name = _both_path)]
    pub fn both_path(&mut self, path: &Path, js_tags: &JsValue) -> Result<Path, JsValue> {
        let tags = js_array_to_tags_vec(js_tags);
        self.path.both_with_tags(tags, path::Via::Path(path.path.clone()));
        Ok(self.clone())
    }


    ///////////////////////////
    // Follow(path: Path)
    ///////////////////////////
    pub fn follow(&mut self, path: &Path) -> Result<Path, JsValue> {
        self.path.follow(path.path.clone());
        Ok(self.clone())
    }


    ///////////////////////////
    // FollowR(path: Path)
    ///////////////////////////
    #[wasm_bindgen(js_name = followR)]
    pub fn follow_r(&mut self, path: &Path) -> Result<Path, JsValue> {
        self.path.follow_reverse(path.path.clone());
        Ok(self.clone())
    }


    ///////////////////////////
    // FollowRecursive(path: Path, maxDepth: int, tags: Stringp[])
    ///////////////////////////
    #[wasm_bindgen(js_name = _follow_recursive_path)]
    pub fn follow_recursive_path(&mut self, path: &Path, js_tags: &JsValue, max_depth: Option<i32>) -> Result<Path, JsValue> {
        let tags = js_array_optional_to_tags_vec(js_tags);
        let max_depth = match max_depth { Some(d) => d, None => 50 };
        self.path.follow_recursive(path::Via::Path(path.path.clone()), max_depth, tags);
        Ok(self.clone())
    }


    ///////////////////////////
    // FollowRecursive(value: String, maxDepth: int, tags: Stringp[])
    ///////////////////////////
    #[wasm_bindgen(js_name = _follow_recursive_values)]
    pub fn follow_recursive_values(&mut self, js_values: &JsValue, js_tags: &JsValue, max_depth: Option<i32>) -> Result<Path, JsValue> {
        let values = js_array_to_values_vec(js_values);
        let tags = js_array_optional_to_tags_vec(js_tags);
        let max_depth = match max_depth { Some(d) => d, None => 50 };
        self.path.follow_recursive(values_to_via(values), max_depth, tags);
        Ok(self.clone())
    }


    ///////////////////////////
    // And(path: Path)
    // Intersect(path: Path)
    ///////////////////////////
    pub fn intersect(&mut self, path: &Path) -> Result<Path, JsValue> {
        self.path.and(path.path.clone());
        Ok(self.clone())
    }


    ///////////////////////////
    // Or(path: Path)
    // Union(path: Path)
    ///////////////////////////
    pub fn union(&mut self, path: &Path) -> Result<Path, JsValue> {
        self.path.or(path.path.clone());
        Ok(self.clone())
    }


    ///////////////////////////
    // Back(tag: String)
    ///////////////////////////
    pub fn back(&mut self, tag: String) -> Result<Path, JsValue> {
        let np = self.path.back(tag);
        if let Some(p) = np {
            self.path = p
        }
        Ok(self.clone())
    }

    ///////////////////////////
    // Tag(tags: String[])
    ///////////////////////////
    #[wasm_bindgen(js_name = _tag)]
    pub fn tag(&mut self, js_tags: &JsValue) -> Result<Path, JsValue> {
        let tags = js_array_optional_to_tags_vec(js_tags);
        self.path.tag(tags);
        Ok(self.clone())
    }

    ///////////////////////////
    // Has(predicate: String, object: String)
    // *Has(predicate: Path, object: String)
    // *Has(predicate: String, filters: Filter[])
    // *Has(predicate: Path, filters: Filter[])
    // HasR(predicate: String, object: String)
    // *HasR(predicate: Path, object: String)
    // *HasR(predicate: String, filters: Filter[])
    // *HasR(predicate: Path, filters: Filter[])
    ///////////////////////////
    #[wasm_bindgen(js_name = _has_value_value)]
    pub fn has_value_value(&mut self, predicate: &JsValue, object: &JsValue, rev: bool) -> Result<Path, JsValue> {
        let nodes = js_array_to_values_vec(predicate);
        let object = js_array_to_values_vec(object);
        self.path.has(values_to_via(nodes), rev, object);
        Ok(self.clone())
    }

    #[wasm_bindgen(js_name = _has_path_value)]
    pub fn has_path_value(&mut self, predicate: &Path, object: &JsValue, rev: bool) -> Result<Path, JsValue> {
        let object = js_array_to_values_vec(object);
        self.path.has(path::Via::Path(predicate.path.clone()), rev, object);
        Ok(self.clone())
    }

    #[wasm_bindgen(js_name = _has_value_filter)]
    pub fn has_value_filter(&mut self, predicate: &JsValue, object: &JsValue, rev: bool) -> Result<Path, JsValue> {
        let nodes = js_array_to_values_vec(predicate);
        self.path.has_filter(values_to_via(nodes), rev, js_array_to_value_filters(object));
        Ok(self.clone())
    }

    #[wasm_bindgen(js_name = _has_path_filter)]
    pub fn has_path_filter(&mut self, predicate: &Path, object: &JsValue, rev: bool) -> Result<Path, JsValue> {
        self.path.has_filter(path::Via::Path(predicate.path.clone()), rev, js_array_to_value_filters(object));
        Ok(self.clone())
    }

    ///////////////////////////
    // Save(values: String[], tag: String)
    ///////////////////////////
    #[wasm_bindgen(js_name = _save_values)]
    pub fn save_values(&mut self, js_values: &JsValue, tag: String, rev: bool, opt: bool) -> Result<Path, JsValue> {
        let nodes = js_array_to_values_vec(js_values);
        self.path.save(values_to_via(nodes), tag, rev, opt);
        Ok(self.clone())
    }

    ///////////////////////////
    // Save(path: Path, tags: String)
    ///////////////////////////
    #[wasm_bindgen(js_name = _save_path)]
    pub fn save_path(&mut self, path: &Path, tag: String, rev: bool, opt: bool) -> Result<Path, JsValue> {
        self.path.save(path::Via::Path(path.path.clone()), tag, rev, opt);
        Ok(self.clone())
    }



    ///////////////////////////
    // Except(path: Path)
    // Difference(path: Path)
    ///////////////////////////
    pub fn except(&mut self, path: &Path) -> Result<Path, JsValue> {
        self.path.except(path.path.clone());
        Ok(self.clone())
    }

    ///////////////////////////
    // Unique()
    ///////////////////////////
    pub fn unique(&mut self) -> Result<Path, JsValue> {
        self.path.unique();
        Ok(self.clone())
    }

    ///////////////////////////
    // Labels()
    ///////////////////////////
    pub fn labels(&mut self) -> Result<Path, JsValue> {
        self.path.labels();
        Ok(self.clone())
    }

    ///////////////////////////
    // InPredicates(tag:String)
    ///////////////////////////
    #[wasm_bindgen(js_name = inPredicates)]
    pub fn in_predicates(&mut self) -> Result<Path, JsValue> {
        self.path.predicates(true);
        Ok(self.clone())
    }

    ///////////////////////////
    // OutPredicates()
    ///////////////////////////
    #[wasm_bindgen(js_name = outPredicates)]
    pub fn out_predicates(&mut self) -> Result<Path, JsValue> {
        self.path.predicates(false);
        Ok(self.clone())
    }

    ///////////////////////////
    // SaveInPredicates(tag:String)
    ///////////////////////////
    #[wasm_bindgen(js_name = saveInPredicates)]
    pub fn save_in_predicates(&mut self, tag: String) -> Result<Path, JsValue> {
        self.path.save_predicates(tag, true);
        Ok(self.clone())
    }

    ///////////////////////////
    // SaveOutPredicates(tag:String)
    ///////////////////////////
    #[wasm_bindgen(js_name = saveOutPredicates)]
    pub fn save_out_predicates(&mut self, tag: String) -> Result<Path, JsValue> {
        self.path.save_predicates(tag, false);
        Ok(self.clone())
    }


    ///////////////////////////
    // LabelContext(values: String[], tags: String[])
    ///////////////////////////
    #[wasm_bindgen(js_name = _label_context_values)]
    pub fn label_context_values(&mut self, js_values: &JsValue, js_tags: &JsValue) -> Result<Path, JsValue> {
        let labels = js_array_to_values_vec(js_values);
        let tags = js_array_to_tags_vec(js_tags);
        self.path.label_context_with_tags(values_to_via(labels), tags);
        Ok(self.clone())
    }


    ///////////////////////////
    // LabelContext(values: Path, tags: String[])
    ///////////////////////////
    #[wasm_bindgen(js_name = _label_context_path)]
    pub fn label_context_path(&mut self, path: &Path, js_tags: &JsValue) -> Result<Path, JsValue> {
        let tags = js_array_to_tags_vec(js_tags);
        self.path.label_context_with_tags(path::Via::Path(path.path.clone()), tags);
        Ok(self.clone())
    }


    ///////////////////////////
    // Filter(filter: Filter)
    ///////////////////////////
    pub fn filter(&mut self, filters: &JsValue) -> Result<Path, JsValue> {
        self.path.filters(js_array_to_value_filters(filters));
        Ok(self.clone())
    }

    ///////////////////////////
    // Limit(limit: Number)
    ///////////////////////////
    pub fn limit(&mut self, limit: i64) -> Result<Path, JsValue> {
        self.path.limit(limit);
        Ok(self.clone())
    }

    ///////////////////////////
    // Skip(offset: Number)
    ///////////////////////////
    pub fn skip(&mut self, offset: i64) -> Result<Path, JsValue> {
        self.path.skip(offset);
        Ok(self.clone())
    }

    ///////////////////////////
    // Order()
    ///////////////////////////
    pub fn order(&mut self) -> Result<Path, JsValue> {
        self.path.order();
        Ok(self.clone())
    }
}



#[wasm_bindgen]
pub struct TagIterator {
    iterator: Box<dyn Iterator<Item = HashMap<String, Value>>>
}

#[wasm_bindgen]
impl TagIterator {
    pub fn next(&mut self) -> Result<JsValue, JsValue> {
        let obj:JsValue = js_sys::Object::new().into();

        if let Some(next) = self.iterator.next() {
            js_sys::Reflect::set(&obj, &"value".into(), &hash_map_to_js_obj(&next))?;
            js_sys::Reflect::set(&obj, &"done".into(), &JsValue::from_bool(false))?;
        } else {
            js_sys::Reflect::set(&obj, &"done".into(), &JsValue::from_bool(true))?;
        }

        Ok(obj)
    }
}


#[wasm_bindgen]
pub struct ValueIterator {
    iterator: Box<dyn Iterator<Item = Value>>
}

#[wasm_bindgen]
impl ValueIterator {
    pub fn next(&mut self) -> Result<JsValue, JsValue> {
        let obj:JsValue = js_sys::Object::new().into();

        if let Some(next) = self.iterator.next() {
            js_sys::Reflect::set(&obj, &"value".into(), &value_to_js(&next))?;
            js_sys::Reflect::set(&obj, &"done".into(), &JsValue::from_bool(false))?;
        } else {
            js_sys::Reflect::set(&obj, &"done".into(), &JsValue::from_bool(true))?;
        }

        Ok(obj)
    }
}


fn js_array_to_value_filters(v: &JsValue) -> Vec<Rc<dyn shape::ValueFilter>> {
    if js_sys::Array::is_array(v) {
        let mut res = Vec::new();
        let array = js_sys::Array::from(v);
        for value in array.values().into_iter() {
            if let Ok(val) = value {
                res.append(&mut js_object_to_value_filters(&val));
            }
        }
        res
    } else {
        return js_object_to_value_filters(&v)
    }
}


fn js_object_to_value_filters(obj: &JsValue) -> Vec<Rc<dyn shape::ValueFilter>> {

    let mut res = Vec::new();

    if !obj.is_object() {
        return res
    }

    if let Ok(keys) = js_sys::Reflect::own_keys(obj) {
        for key in keys.values().into_iter() {
            if let Ok(k) = key {
                if let Some(name) = k.as_string() {

                    if name == "lt" {
                        if let Ok(value) = js_sys::Reflect::get(obj, &k) {
                            res.push(gizmo::lt(js_to_value_ignore(&value)))
                        }
                    } 
                    
                    if name == "lte" {
                        if let Ok(value) = js_sys::Reflect::get(obj, &k) {
                            res.push(gizmo::lte(js_to_value_ignore(&value)))
                        }
                    } 
                    
                    if name == "gt" {
                        if let Ok(value) = js_sys::Reflect::get(obj, &k) {
                            res.push(gizmo::gt(js_to_value_ignore(&value)))
                        }
                    } 
                    
                    if name == "gte" {
                        if let Ok(value) = js_sys::Reflect::get(obj, &k) {
                            res.push(gizmo::gte(js_to_value_ignore(&value)))
                        }
                    } 
                    
                    if name == "like" {
                        if let Ok(pattern) = js_sys::Reflect::get(obj, &k) {
                            if let Some(p) = pattern.as_string() {
                                //console::log_2(&JsValue::from_str("like"), &JsValue::from_str(&p));
                                res.push(gizmo::like(p))
                            }
                        }
                    }

                    if name == "regex" {
                        if let Ok(pattern) = js_sys::Reflect::get(obj, &k) {
                            if let Some(p) = pattern.as_string() {
                                let iri = if let Ok(iri) = js_sys::Reflect::get(obj, &"iri".into()) {
                                    //console::log_3(&JsValue::from_str("regex iri"), &iri, &JsValue::from_bool(iri.is_truthy()));
                                    iri.is_truthy()
                                } else {
                                    //console::log_1(&JsValue::from_str("regex iri js_sys::Reflect::get Err"));
                                    false
                                };
                                res.push(gizmo::regex(p, iri))
                            }
                        }
                    } 
                }
            }
        }
    }

    res
}


fn ref_to_value(r: &Ref, qs: &dyn QuadStore) -> Option<Value> {
    qs.name_of(r) 
}

fn tags_to_value_map(m: &HashMap<String, Ref>, qs: &dyn QuadStore) -> Option<HashMap<String, Value>> {
    let mut output_map = HashMap::new();

    for (key, value) in m {
        match qs.name_of(value) {
            Some(v) => { output_map.insert(key.clone(), v); },
            None => {}
        };
    }
    
    if output_map.is_empty() {
        return None
    }

    return Some(output_map)
}


fn hash_map_to_js_obj(hash_map: &HashMap<String, Value>) -> JsValue{
    let obj:JsValue = js_sys::Object::new().into();

    for (k, v) in hash_map {
        js_sys::Reflect::set(&obj, &k.into(), &value_to_js(&v));
    }

    obj
}



fn js_array_to_values_vec(js: &JsValue) -> Vec<Value> {
    if !js_sys::Array::is_array(js) {
        if js.is_undefined() || js.is_null() {
            return Vec::new()
        } else {
            if let Some(v) = js_to_value(&js) {
                return vec![v]
            } else {
                return Vec::new()
            }
        }
    }
    
    let array = js_sys::Array::from(js);

    let values: Vec<Value> = array.values().into_iter().filter_map(|v| v.ok()).filter_map(|v| js_to_value(&v)).collect();

    values
}


fn js_array_optional_to_tags_vec(js: &JsValue) -> Vec<String> {
    if let Some(s) = js.as_string() {
        return vec![s]
    }

    if !js_sys::Array::is_array(js) {
        return Vec::new()
    }
    
    let array = js_sys::Array::from(js);

    let values: Vec<String> = array.values().into_iter().filter_map(|v| v.ok()).filter_map(|v| v.as_string()).collect();

    values
}


fn js_array_to_tags_vec(js: &JsValue) -> Vec<String> {

    if let Some(s) = js.as_string() {
        return vec![s]
    }

    if !js_sys::Array::is_array(js) {
        return Vec::new()
    }
    
    let array = js_sys::Array::from(js);

    let values: Vec<String> = array.values().into_iter().filter_map(|v| v.ok()).filter_map(|v| v.as_string()).collect();

    values
}


fn js_to_value(js: &JsValue) -> Option<Value> {

    if js.is_undefined() {
        return Some(Value::None)
    } 

    if js.is_null() {
        return Some(Value::Null)
    } 
    
    let opt_b = js.as_bool();
    if let Some(b) = opt_b {
        return Some(Value::Bool(b))
    } 

    let opt_n = js.as_f64();
    if let Some(n) = opt_n {
        if let Some(f) = Number::from_f64(n) {
            return Some(Value::Number(f))
        }
    } 

    let opt_s = js.as_string();
    if let Some(s) = opt_s {
        return Some(Value::from(s))
    } 

    None
}


fn js_to_value_ignore(js: &JsValue) -> Value {
    if let Some(s) = js_to_value(js) {
        s
    } else {
        Value::None
    }
}


// fn js_prop_to_value(res: Result<JsValue, JsValue>) -> Value {
//     if let Ok(js) = res {
//         js_to_value_ignore(&js)
//     }  else {
//         Value::None
//     }
// }


fn js_to_quad(js: &JsValue) -> Option<Quad> {

    if !js_sys::Array::is_array(js) {
        return None
    } 

    let arr = js_sys::Array::from(js);

    if js_sys::Array::length(&arr) < 3 ||  js_sys::Array::length(&arr) > 4 {
        return None
    } 

    let res_s = arr.get(0);
    let subject = js_to_value_ignore(&res_s);

    let res_p = arr.get(1);
    let predicate = js_to_value_ignore(&res_p);

    let res_o = arr.get(2);
    let object = js_to_value_ignore(&res_o);

    let label = if js_sys::Array::length(&arr) == 4 {
        let res_l = arr.get(3);
        js_to_value_ignore(&res_l)
    } else {
        Value::None
    };

    Some(Quad {
        subject,
        predicate,
        object,
        label
    })
}


fn value_to_js(value: &Value) -> JsValue {
    match value {
        Value::None => JsValue::undefined(),
        Value::Null => JsValue::null(),
        Value::Bool(b) => JsValue::from_bool(*b),
        Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                JsValue::from_f64(f)
            } else {
                JsValue::undefined()
            }
        },
        Value::IRI(_) => JsValue::from_str(&value.to_string()),
        Value::String(s) => JsValue::from_str(&s),
    }
}


fn quad_to_js(quad: &Quad) -> JsValue {
    let arr = js_sys::Array::new();
    
    js_sys::Array::push(&arr, &value_to_js(&quad.subject));
    js_sys::Array::push(&arr, &value_to_js(&quad.predicate));
    js_sys::Array::push(&arr, &value_to_js(&quad.object));
    js_sys::Array::push(&arr, &value_to_js(&quad.label));
    
    arr.into()
}


fn quad_vec_to_js(quads: &Vec<Quad>) -> JsValue {
    let a = js_sys::Array::new();
    for q in quads {
        a.push(&quad_to_js(q));
    }
    a.into()
}


// instead of ignoring invalid data return result with error
fn js_array_to_quad_vec(js: &JsValue) -> Vec<Quad> {
    if !js_sys::Array::is_array(js) {
        return Vec::new()
    }
    
    let array = js_sys::Array::from(js);

    let values: Vec<Quad> = array.values().into_iter().filter_map(|v| v.ok()).filter_map(|v| js_to_quad(&v)).collect();

    values
}

fn values_to_via(values: Vec<Value>) -> path::Via {
    if values.is_empty() {
        return path::Via::None
    } else {
        return path::Via::Values(values)
    }
}