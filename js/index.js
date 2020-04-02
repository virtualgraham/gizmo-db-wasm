(async() => {
    let gizmo_db = await import("../pkg/index.js");
    let session = gizmo_db.NewMemoryGraph();

    session.write([
        ["<alice>", "<follows>", "<bob>"],
        ["<bob>", "<follows>", "<fred>"],
        ["<bob>", "<status>", "cool_person"],

        ["<dani>", "<follows>", "<bob>"],
        ["<charlie>", "<follows>", "<bob>"],
        ["<charlie>", "<follows>", "<dani>"],

        ["<dani>", "<follows>", "<greg>"],
        ["<dani>", "<status>", "cool_person"],
        ["<emily>", "<follows>", "<fred>"],

        ["<fred>", "<follows>", "<greg>"],
        ["<greg>", "<status>", "cool_person"],
        ["<predicates>", "<are>", "<follows>"],

        ["<predicates>", "<are>", "<status>"],
        ["<emily>", "<status>", "smart_person", "<smart_graph>"],
        ["<greg>", "<status>", "smart_person", "<smart_graph>"]
    ]);

    let g = session.g();

    /////////////////////////
    // get a single vertex
    /////////////////////////

    let iter = g.v(["<alice>"]).values()
    iter[Symbol.iterator] = function() { return this; }

    var r = Array.from(iter);
    var e = ["<alice>"];

    console.assert(arrays_equal(r,e), "TEST FAILED: get a single vertex", r.sort(), e.sort());


    /////////////////////////
    // use .out()
    /////////////////////////

    iter = g.v(["<alice>"]).out_values(["<follows>"]).values()
    iter[Symbol.iterator] = function() { return this; }

    r = Array.from(iter);
    e = ["<bob>"];

    console.assert(arrays_equal(r,e), "TEST FAILED: use .out()", r.sort(), e.sort());


    /////////////////////////
    // use .out() (any)
    /////////////////////////

    iter = g.v(["<bob>"]).out_values().values()
    iter[Symbol.iterator] = function() { return this; }

    r = Array.from(iter);
    e = ["<fred>", "cool_person"];

    console.assert(arrays_equal(r,e), "TEST FAILED: use .out() (any)", r.sort(), e.sort());


    
})()

function arrays_equal(array1, array2) {
    return array1.length === array2.length && array1.sort().every(function(value, index) { return value === array2.sort()[index]});
}