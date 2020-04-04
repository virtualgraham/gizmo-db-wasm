(async () => {
    let gizmo = await import("./gizmo.js");

    console.log(gizmo)

    let session = gizmo.NewMemoryGraph();

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

    console.log("get a single vertex")

    let q = g.v("<alice>")

    var r = Array.from(q.values());
    var e = ["<alice>"];
    console.assert(arrays_equal(r,e), "TEST FAILED: get a single vertex", r.sort(), e.sort());


    /////////////////////////
    // use .out()
    /////////////////////////

    console.log("use .out()")

    q = g.v("<alice>").out("<alice>")

    r = Array.from(q.values());
    e = ["<bob>"];
    console.assert(arrays_equal(r,e), "TEST FAILED: use .out()", r.sort(), e.sort());


    /////////////////////////
    // use .out() (any)
    /////////////////////////

    console.log("use .out() (any)")

    q = g.v("<bob>").out()

    r = Array.from(q.values());
    e = ["<fred>", "cool_person"];
    console.assert(arrays_equal(r,e), "TEST FAILED: use .out() (any)", r.sort(), e.sort());

})()
    


// function arrays_equal(array1, array2) {
//     return array1.length === array2.length && array1.sort().every(function(value, index) { return value === array2.sort()[index]});
// }