import * as lib from "../pkg/index.js"; // output from wasm_bindgen build

function is_path(args) {
    args.length >= 1 && 
    (((args[0] || {}).prototype || {}).constructor || {}).name == "Path"
}

lib.TagIterator.prototype[Symbol.iterator] = function() { return this; }
lib.ValueIterator.prototype[Symbol.iterator] = function() { return this; }

// lib.GraphWrapper.prototype.write
// lib.GraphWrapper.prototype.delete

lib.Graph.prototype.v = function() {
    return this._v(Array.prototype.slice.call(arguments));
}

lib.Graph.prototype.m = function() {
    return this._m(Array.prototype.slice.call(arguments));
}

lib.Path.prototype.is = function() {
    return this._is(Array.prototype.slice.call(arguments));
}

lib.Path.prototype.in = function() {
    if (is_path(arguments)) {
        return this._in_path(arguments[0], arguments[1])
    } else {
        return this._in_values(arguments[0], arguments[1])
    }
}

lib.Path.prototype.out = function() {
    if (is_path(arguments)) {
        return this._out_path(arguments[0], arguments[1])
    } else {
        return this._out_values(arguments[0], arguments[1])
    }
}

lib.Path.prototype.both = function() {
    if (is_path(arguments)) {
        return this._both_path(arguments[0], arguments[1])
    } else {
        return this._both_values(arguments[0], arguments[1])
    }
}

lib.Path.prototype.follow_recursive = function() {
    if (is_path(arguments)) {
        return this._follow_recursive_path(arguments[0], arguments[1], arguments[2])
    } else {
        return this._follow_recursive_values(arguments[0], arguments[1], arguments[2])
    }
}

const lt = lib.lt;
const lte =lib.lte;
const gt =lib.gt;
const gte =lib.gte;
const regex =lib.regex;
const like =lib.like;
const NewMemoryGraph =lib.NewMemoryGraph;
const Graph =lib.Graph;
const GraphWrapper =lib.GraphWrapper;
const Path =lib.Path;
const Session =lib.Session;
const TagIterator =lib.TagIterator;
const ValueIterator =lib.ValueIterator;
const ValueFilter = lib.ValueFilter;

export {
    lt,
    lte,
    gt,
    gte,
    regex,
    like,
    NewMemoryGraph,
    Graph,
    GraphWrapper,
    Path,
    Session,
    TagIterator,
    ValueIterator,
    ValueFilter
}

