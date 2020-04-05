(async () => {
    let gizmo = await import("./gizmo.js");

    console.log("Starting Gizmo Wasm Tests...", gizmo)

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

    run_test(
        "get a single vertex",
        g.V("<alice>"),
        ["<alice>"]
    )

    /////////////////////////

    run_test(
        "use .out()",
        g.V("<alice>").out("<follows>"),
        ["<bob>"]
    )

    /////////////////////////

    run_test(
        "use .out() (any)",
        g.V("<bob>").out(),
        ["<fred>", "cool_person"]
    )

    /////////////////////////

    run_test(
        "use .in()",
        g.V("<bob>").in("<follows>"),
        ["<alice>", "<charlie>", "<dani>"]
    )

    /////////////////////////

    run_test(
        "use .in() (any)",
        g.V("<bob>").in(),
        ["<alice>", "<charlie>", "<dani>"]
    )

    /////////////////////////

    run_test(
        "use .in() with .filter()",
        g.V("<bob>").in("<follows>").filter({gt: "<c>", lt: "<d>"}),
        ["<charlie>"]
    )

    /////////////////////////

    run_test(
        "use .in() with .filter(regex)",
        g.V("<bob>").in("<follows>").filter({regex: "ar?li.*e"}),
        []
    )

    /////////////////////////

    run_test(
        "use .in() with .filter(prefix)",
        g.V("<bob>").in("<follows>").filter({like: "al%"}),
        ["<alice>"]
    )

    /////////////////////////

    run_test(
        "use .in() with .filter(wildcard)",
        g.V("<bob>").in("<follows>").filter({like: "a?i%e"}),
        ["<alice>"]
    )

    /////////////////////////

    run_test(
        "use .in() with .filter(regex with IRIs)",
        g.V("<bob>").in("<follows>").filter({regex: "ar?li.*e", iri: true}),
        ["<alice>", "<charlie>"]
    )

    /////////////////////////

    run_test(
        "use .in() with .filter(regex,gt)",
        g.V("<bob>").in("<follows>").filter({regex: "ar?li.*e", iri: true, gt: "<c>"}),
        ["<charlie>"]
    )

    /////////////////////////

    run_test_tag(
        "use .both() with tag",
        g.V("<fred>").both(null, "pred"),
        ["<follows>", "<follows>", "<follows>"],
        "pred"
    )

    /////////////////////////

    run_test(
        "use .tag()-.is()-.back()",
        g.V("<bob>").in("<follows>").tag("foo").out("<status>").is("cool_person").back("foo"),
        ["<dani>"]
    )

    /////////////////////////

    x = g.V("<charlie>").out("<follows>").tag("foo").out("<status>").is("cool_person").back("foo")

    run_test(
        "separate .tag()-.is()-.back()",
        x.in("<follows>").is("<dani>").back("foo"),
        ["<bob>"]
    )

    /////////////////////////

    run_test_tag(
        "do multiple .back()",
        g.V("<emily>").out("<follows>").as("f").out("<follows>").out("<status>").is("cool_person").back("f").in("<follows>").in("<follows>").as("acd").out("<status>").is("cool_person").back("f"),
        ["<dani>"],
        "acd"
    )   

    
})()
    
function run_test(name, q, e) {
    r = Array.from(q.values());

    if (arrays_equal(r,e)) {
        console.log("PASSED", name)
    } else {
        console.error("FAILED", name, r.sort(), e.sort())
    }
}

function run_test_tag(name, q, e, tag) {
    r = Array.from(q.all());
    r = r.map((o) => o[tag]);

    if (arrays_equal(r,e)) {
        console.log("PASSED", name)
    } else {
        console.error("FAILED", name, r.sort(), e.sort())
    }
}

function arrays_equal(array1, array2) {
    return array1.length === array2.length && array1.sort().every(function(value, index) { return value === array2.sort()[index]});
}