import * as lib from "../pkg/index.js";
export * from "../pkg/index.js";

// args[0].prototype.contructor.name == "Path"
function has_path(args) {
    if (args.length >= 1 && args[0] != null && typeof args[0] === "object") {
        return ((Object.getPrototypeOf(args[0]) || {}).constructor || {}).name == "Path"
    }
    return false
}

// the argument at idx is an object or an array of objects
function has_filter(args, idx) {
    return args.length > idx &&
    (
        (
            Array.isArray(args[idx]) && 
            args[idx].length > 0 && 
            args[idx][0] != null && 
            typeof args[idx][0] === 'object'
        ) || 
        (
            args[idx] != null && 
            typeof args[idx] === 'object'
        )
    )
}

lib.TagIterator.prototype[Symbol.iterator] = function() { return this; }
lib.ValueIterator.prototype[Symbol.iterator] = function() { return this; }
lib.QuadIterator.prototype[Symbol.iterator] = function() { return this; }

// lib.GraphWrapper.prototype.write
// lib.GraphWrapper.prototype.delete

lib.Graph.prototype.V = function() {
    return this._v(Array.prototype.slice.call(arguments));
}

lib.Graph.prototype.M = function() {
    return this._m(Array.prototype.slice.call(arguments));
}



lib.Path.prototype.toArray = function() {
    return Array.from(this.iterValues())
}

lib.Path.prototype.toValue = function() {
    return this.iter_values().next().value
}

lib.Path.prototype.tagArray = function() {
    return Array.from(this.iterTags())
}

lib.Path.prototype.tagValue = function() {
    return this.iter_tags().next().value
}

lib.Path.prototype.forEach = function(a, b) {

    let iter, callback;

    if (typeof a === "function") {
        iter = this.iter_tags();
        callback = a;
    } else if (typeof b === "function") {
        iter = this.iter_tags(a);
        callback = b;
    }

    for (let result of iter) {
        callback(result);
    }
    
}



lib.Path.prototype.is = function() {
    return this._is(Array.prototype.slice.call(arguments));
}

lib.Path.prototype.in = function() {
    if (has_path(arguments)) {
        return this._in_path(arguments[0], arguments[1])
    } else {
        return this._in_values(arguments[0], arguments[1])
    }
}

lib.Path.prototype.out = function() {
    if (has_path(arguments)) {
        return this._out_path(arguments[0], arguments[1])
    } else {
        return this._out_values(arguments[0], arguments[1])
    }
}

lib.Path.prototype.both = function() {
    if (has_path(arguments)) {
        return this._both_path(arguments[0], arguments[1])
    } else {
        return this._both_values(arguments[0], arguments[1])
    }
}

lib.Path.prototype.followRecursive = function() {
    if (has_path(arguments)) {
        return this._follow_recursive_path(arguments[0], arguments[1], arguments[2])
    } else {
        return this._follow_recursive_values(arguments[0], arguments[1], arguments[2])
    }
}

lib.Path.prototype.and = function() {
    return this.intersect(...arguments)
}

lib.Path.prototype.or = function() {
    return this.union(...arguments)
}

lib.Path.prototype.tag = function() {
    return this._tag(Array.prototype.slice.call(arguments));
}

lib.Path.prototype.as = function() {
    return this.tag(...arguments)
}

lib.Path.prototype.has = function() {
    if (has_path(arguments)) {
        if (has_filter(arguments, 1)) {
            return this._has_path_filter(arguments[0], arguments[1], false)
        } else {
            return this._has_path_value(arguments[0], arguments[1], false)
        }
    } else {
        if (has_filter(arguments, 1)) {
            //console.log("lib.Path.prototype.has has_filter", arguments[0], arguments[1])
            return this._has_value_filter(arguments[0], arguments[1], false)
        } else {
            //console.log("lib.Path.prototype.has !has_filter", arguments[0], arguments[1])
            return this._has_value_value(arguments[0], arguments[1], false)
        }
    }
}

lib.Path.prototype.hasR = function() {
    if (has_path(arguments)) {
        if (has_filter(arguments, 1)) {
            return this._has_path_filter(arguments[0], arguments[1], true)
        } else {
            return this._has_path_value(arguments[0], arguments[1], true)
        }
    } else {
        if (has_filter(arguments, 1)) {
            return this._has_value_filter(arguments[0], arguments[1], true)
        } else {
            return this._has_value_value(arguments[0], arguments[1], true)
        }
    }
}

lib.Path.prototype.save = function() {
    if (has_path(arguments)) {
        return this._save_path(arguments[0], arguments[1], false, false)
    } else {
        return this._save_values(arguments[0], arguments[1], false, false)
    }
}

lib.Path.prototype.saveR = function() {
    if (has_path(arguments)) {
        return this._save_path(arguments[0], arguments[1], true, false)
    } else {
        return this._save_values(arguments[0], arguments[1], true, false)
    }
}

lib.Path.prototype.saveOpt = function() {
    if (has_path(arguments)) {
        return this._save_path(arguments[0], arguments[1], false, true)
    } else {
        return this._save_values(arguments[0], arguments[1], false, true)
    }
}

lib.Path.prototype.saveOptR = function() {
    if (has_path(arguments)) {
        return this._save_path(arguments[0], arguments[1], false, false)
    } else {
        return this._save_values(arguments[0], arguments[1], false, false)
    }
}

lib.Path.prototype.difference = function() {
    return this.except(...arguments)
}

lib.Path.prototype.labelContext = function() {
    if (has_path(arguments)) {
        return this._label_context_path(arguments[0], arguments[1])
    } else {
        return this._label_context_values(arguments[0], arguments[1])
    }
}


export function lt(value) {
    return {
        lt: value
    }
}

export function lte(value) {
    return {
        lte: value
    }
}

export function gt(value) {
    return {
        gt: value
    }
}

export function gte(value) {
    return {
        gte: value
    }
}

export function regex(pattern, iri) {
    return {
        regex: pattern,
        iri: iri
    }
}

export function like(pattern) {
    return {
        like: pattern,
    }
}