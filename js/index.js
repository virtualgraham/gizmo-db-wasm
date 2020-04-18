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



    // let quad_iter = session.read();

    // session.delete([
    //     ["<alice>", "<follows>", "<bob>"],
    //     ["<bob>", "<follows>", "<fred>"]
    // ]);

    // let read_result = Array.from(session.read({sub: []}));
    // console.log("read_result", quad_iter, read_result);



    let g = session.g();

    /////////////////////////

    run_test(
        "get a single vertex",
        g.V("<alice>").all(),
        ["<alice>"]
    )

    /////////////////////////

    run_test(
        "use .out()",
        g.V("<alice>").out("<follows>").all(),
        ["<bob>"]
    )

    /////////////////////////

    run_test(
        "use .out() (any)",
        g.V("<bob>").out().all(),
        ["<fred>", "cool_person"]
    )

    /////////////////////////

    run_test(
        "use .in()",
        g.V("<bob>").in("<follows>").all(),
        ["<alice>", "<charlie>", "<dani>"]
    )

    /////////////////////////

    run_test(
        "use .in() (any)",
        g.V("<bob>").in().all(),
        ["<alice>", "<charlie>", "<dani>"]
    )

    /////////////////////////

    run_test(
        "use .in() with .filter()",
        g.V("<bob>").in("<follows>").filter({gt: "<c>", lt: "<d>"}).all(),
        ["<charlie>"]
    )

    /////////////////////////

    run_test(
        "use .in() with .filter(regex)",
        g.V("<bob>").in("<follows>").filter({regex: "ar?li.*e"}).all(),
        []
    )

    /////////////////////////

    run_test(
        "use .in() with .filter(prefix)",
        g.V("<bob>").in("<follows>").filter({like: "al*"}).all(),
        ["<alice>"]
    )

    /////////////////////////

    run_test(
        "use .in() with .filter(wildcard)",
        g.V("<bob>").in("<follows>").filter({like: "a?i*e"}).all(),
        ["<alice>"]
    )

    /////////////////////////

    run_test(
        "use .in() with .filter(regex with IRIs)",
        g.V("<bob>").in("<follows>").filter({regex: "ar?li.*e", iri: true}).all(),
        ["<alice>", "<charlie>"]
    )

    /////////////////////////

    run_test(
        "use .in() with .filter(regex,gt)",
        g.V("<bob>").in("<follows>").filter({regex: "ar?li.*e", iri: true, gt: "<c>"}).all(),
        ["<charlie>"]
    )

    /////////////////////////

    run_test_tag(
        "use .both() with tag",
        g.V("<fred>").both(null, "pred").all(),
        ["<follows>", "<follows>", "<follows>"],
        "pred"
    )

    /////////////////////////

    run_test(
        "use .tag()-.is()-.back()",
        g.V("<bob>").in("<follows>").tag("foo").out("<status>").is("cool_person").back("foo").all(),
        ["<dani>"]
    )

    /////////////////////////

    {
        x = g.V("<charlie>").out("<follows>").tag("foo").out("<status>").is("cool_person").back("foo")

        run_test(
            "separate .tag()-.is()-.back()",
            x.in("<follows>").is("<dani>").back("foo").all(),
            ["<bob>"]
        )
    }

    /////////////////////////

    run_test_tag(
        "do multiple .back()",
        g.V("<emily>").out("<follows>").as("f").out("<follows>").out("<status>").is("cool_person").back("f").in("<follows>").in("<follows>").as("acd").out("<status>").is("cool_person").back("f").all(),
        ["<dani>"],
        "acd"
    )   

     /////////////////////////

     run_test(
         "use Except to filter out a single vertex",
         g.V("<alice>", "<bob>").except(g.V("<alice>")).all(),
         ["<bob>"]
     )   

    /////////////////////////

    run_test(
        "use chained Except",
        g.V("<alice>", "<bob>", "<charlie>").except(g.V("<bob>")).except(g.V("<charlie>")).all(),
        ["<alice>"]
    )   

    /////////////////////////

    run_test(
        "use Unique",
        g.V("<alice>", "<bob>", "<charlie>").out("<follows>").unique().all(),
        ["<bob>", "<dani>", "<fred>"]
    )  

    /////////////////////////

    {
        grandfollows = g.M().out("<follows>").out("<follows>")

        run_test(
            "show simple morphism",
            g.V("<charlie>").follow(grandfollows).all(),
            ["<greg>", "<fred>", "<bob>"]
        )  
    }

    /////////////////////////

    {
        grandfollows = g.M().out("<follows>").out("<follows>")

        run_test(
            "show reverse morphism",
            g.V("<fred>").followR(grandfollows).all(),
            ["<alice>", "<charlie>", "<dani>"]
        )  
    }

    /////////////////////////
    
    {
        function follows(x) { return g.V(x).out("<follows>") }

        run_test(
            "show simple intersection",
            follows("<dani>").and(follows("<charlie>")).all(),
            ["<bob>"]
        )  
    }
    
    /////////////////////////

    {
        grandfollows = g.M().out("<follows>").out("<follows>")
        function gfollows(x) { return g.V(x).follow(grandfollows) }

        run_test(
            "show simple morphism intersection",
            gfollows("<alice>").and(gfollows("<charlie>")).all(),
            ["<fred>"]
        )  
    }

    /////////////////////////

    {
        grandfollows = g.M().out("<follows>").out("<follows>")
        function gfollows(x) { return g.V(x).follow(grandfollows) }

        run_test(
            "show double morphism intersection",
            gfollows("<emily>").and(gfollows("<charlie>")).and(gfollows("<bob>")).all(),
            ["<greg>"]
        )  
    }

    /////////////////////////

     {
        grandfollows = g.M().out("<follows>").out("<follows>")

        run_test(
            "show reverse intersection",
            g.V("<greg>").followR(grandfollows).intersect(g.V("<fred>").followR(grandfollows)).all(),
            ["<charlie>"]
        )  
    }
       
    /////////////////////////

    {
        gfollowers = g.M().in("<follows>").in("<follows>")
		function cool(x) { return g.V(x).as("a").out("<status>").is("cool_person").back("a") }

        run_test(
            "show standard sort of morphism intersection, continue follow",
            cool("<greg>").follow(gfollowers).intersect(cool("<bob>").follow(gfollowers)).all(),
            ["<charlie>"]
        )  
    }
    
    /////////////////////////

    run_test(
        "test Or()",
        g.V("<bob>").out("<follows>").or(g.V().has("<status>", "cool_person")).all(),
        ["<fred>", "<bob>", "<greg>", "<dani>"]
    )    


    /////////////////////////

    run_test(
        "show a simple Has",
        g.V().has("<status>", "cool_person").all(),
        ["<greg>", "<dani>", "<bob>"]
    )    

    /////////////////////////

    run_test(
        "show a simple HasR",
        g.V().hasR("<status>", "<bob>").all(),
        ["cool_person"]
    )    

    /////////////////////////

    run_test(
        "show a double Has",
        g.V().has("<status>", "cool_person").has("<follows>", "<fred>").all(),
        ["<bob>"]
    )    

    /////////////////////////

    run_test(
        "show a Has with filter",
        g.V().has("<follows>", { gt: "<f>" }).all(),
        ["<bob>", "<dani>", "<emily>", "<fred>"]
    )    


    /////////////////////////

    run_test(
        "use Limit",
        g.V().has("<status>", "cool_person").limit(2).all(),
        ["<bob>", "<dani>"]
    )    

    /////////////////////////

    run_test(
        "use Skip",
        g.V().has("<status>", "cool_person").skip(2).all(),
        ["<greg>"]
    )   

    /////////////////////////

    run_test(
        "use Skip and Limit",
        g.V().has("<status>", "cool_person").skip(1).limit(1).all(),
        ["<dani>"]
    )    

    /////////////////////////

    run_test_direct(
        "show Count",
        g.V().has("<status>").count(),
        5
    )    

    /////////////////////////

    run_test_direct(
        "use Count value",
        g.V().has("<status>").count()+1,
        6
    )    

    /////////////////////////

    run_test_tag(
        "show a simple save",
        g.V().save("<status>", "somecool").all(),
        ["cool_person", "cool_person", "cool_person", "smart_person", "smart_person"],
        "somecool"
    )    

    /////////////////////////

    run_test_tag(
        "show a simple save optional",
        g.V("<bob>","<charlie>").out("<follows>").saveOpt("<status>", "somecool").all(),
        ["cool_person", "cool_person"],
        "somecool"
    )   
 
    /////////////////////////

    run_test_tag(
        "save iri no tag",
        g.V().save("<status>").all(),
        ["cool_person", "cool_person", "cool_person", "smart_person", "smart_person"],
        "<status>"
    )   

    /////////////////////////

    run_test_tag(
        "show a simple saveR",
        g.V("cool_person").saveR("<status>", "who").all(),
        ["<greg>", "<dani>", "<bob>"],
        "who"
    )   

    /////////////////////////

    run_test_tag(
        "show an out save",
        g.V("<dani>").out(null, "pred").all(),
        ["<follows>", "<follows>", "<status>"],
        "pred"
    )   

    /////////////////////////

    run_test_tag(
        "show a tag list",
        g.V("<dani>").out(null, ["pred", "foo", "bar"]).all(),
        ["<follows>", "<follows>", "<status>"],
        "foo"
    )   

    /////////////////////////

    run_test(
        "show a pred list",
        g.V("<dani>").out(["<follows>", "<status>"]).all(),
        ["<bob>", "<greg>", "cool_person"]
    )   

    /////////////////////////

    run_test(
        "show a predicate path",
        g.V("<dani>").out(g.V("<follows>"), "pred").all(),
        ["<bob>", "<greg>"]
    )   

    /////////////////////////

    run_test(
        "list all bob's incoming predicates",
        g.V("<bob>").inPredicates().all(),
        ["<follows>"]
    )  
    
    /////////////////////////

    run_test_tag(
        "save all bob's incoming predicates",
        g.V("<bob>").saveInPredicates("pred").all(),
        ["<follows>", "<follows>", "<follows>"],
        "pred"
    )  
    

    /////////////////////////

    run_test (
        "list all labels",
        g.V().labels().all(),
        ["<smart_graph>"],
    ) 

    /////////////////////////

    run_test (
        "list all in predicates",
        g.V().inPredicates().all(),
        ["<are>", "<follows>", "<status>"],
    ) 

    /////////////////////////

    run_test (
        "list all out predicates",
        g.V().outPredicates().all(),
        ["<are>", "<follows>", "<status>"],
    ) 

    /////////////////////////

    run_test (
        "traverse using LabelContext",
        g.V("<greg>").labelContext("<smart_graph>").out("<status>").all(),
        ["smart_person"],
    ) 



})()
    
function run_test_direct(name, d, e) {
    if (d === e) {
        console.log("PASSED", name)
    } else {
        console.error("FAILED", name, r.sort(), e.sort())
    }
}

function run_test(name, tag_iter, e) {
    let r = Array.from(tag_iter).map((o) => o.id);

    if (arrays_equal(r,e)) {
        console.log("PASSED", name)
    } else {
        console.error("FAILED", name, r.sort(), e.sort())
    }
}

function run_test_tag(name, tag_iter, e, tag) {
    let r = Array.from(tag_iter).filter((o) => o.hasOwnProperty(tag)).map((o) => o[tag]);

    if (arrays_equal(r,e)) {
        console.log("PASSED", name)
    } else {
        console.error("FAILED", name, r.sort(), e.sort())
    }
}

function arrays_equal(array1, array2) {
    return array1.length === array2.length && array1.sort().every(function(value, index) { return value === array2.sort()[index]});
}