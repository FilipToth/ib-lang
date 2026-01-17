use ibc::analysis;

/// The graph of `source` as Graphviz DOT, or a panic naming why it has none.
fn graph(source: &str) -> String {
    let (errors, dot) = analysis::control_flow_graph(source.to_string());

    match dot {
        Some(dot) => dot,
        None => {
            let messages: Vec<String> =
                errors.errors.iter().map(|e| e.kind.format()).collect();

            panic!("{:?} has no graph: {:?}", source, messages)
        }
    }
}

/// The clusters of a graph, as (label, body) pairs in the order they appear.
/// Each function is drawn in one, and so is the program itself.
fn clusters(dot: &str) -> Vec<(String, String)> {
    let mut clusters: Vec<(String, String)> = Vec::new();

    let mut label: Option<String> = None;
    let mut body = String::new();

    for line in dot.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("subgraph cluster_") {
            label = None;
            body = String::new();
            continue;
        }

        if trimmed.starts_with("label=\"") && label.is_none() {
            let text = trimmed.trim_start_matches("label=\"").trim_end_matches('"');
            label = Some(text.to_string());
            continue;
        }

        if trimmed == "}" {
            if let Some(name) = label.take() {
                clusters.push((name, body.clone()));
            }

            continue;
        }

        body += line;
        body += "\n";
    }

    clusters
}

/// The ids of the nodes declared in a cluster body.
fn node_ids(body: &str) -> Vec<String> {
    let mut ids: Vec<String> = Vec::new();

    for line in body.lines() {
        let trimmed = line.trim();

        let Some((id, rest)) = trimmed.split_once(' ') else {
            continue;
        };

        if !rest.starts_with("[label=") {
            continue;
        }

        if !ids.contains(&id.to_string()) {
            ids.push(id.to_string());
        }
    }

    ids
}

const TWO_FUNCTIONS: &str = "function double(N: Int) -> Int\n\
                                 return N * 2\n\
                             end\n\
                             function classify(N: Int) -> String\n\
                                 if N < 0 then\n\
                                     return \"negative\"\n\
                                 end\n\
                                 return \"positive\"\n\
                             end\n\
                             X = 3\n\
                             output double(X)";

#[test]
fn the_program_gets_a_graph_of_its_own() {
    let dot = graph("X = 3\noutput X");
    let clusters = clusters(&dot);

    assert_eq!(clusters.len(), 1, "{}", dot);

    let (label, body) = &clusters[0];

    assert_eq!(label, "<program>");
    assert!(body.contains("X = 3"), "{}", body);
    assert!(body.contains("output X"), "{}", body);
}

#[test]
fn each_function_gets_a_graph_beside_the_program() {
    let dot = graph(TWO_FUNCTIONS);
    let labels: Vec<String> = clusters(&dot).into_iter().map(|(l, _)| l).collect();

    assert_eq!(
        labels,
        vec![
            "<program>",
            "function double(...) -> Int",
            "function classify(...) -> String",
        ],
        "{}",
        dot
    );
}

/// Every graph used to number its nodes from 1, and they are all drawn into one
/// digraph, so graphviz read the first node of each function as the same node
/// and merged the functions into a single tangle.
#[test]
fn graphs_do_not_share_node_ids() {
    let dot = graph(TWO_FUNCTIONS);

    let mut seen: Vec<String> = Vec::new();
    for (label, body) in clusters(&dot) {
        for id in node_ids(&body) {
            assert!(
                !seen.contains(&id),
                "{:?} reuses node id {:?}\n{}",
                label,
                id,
                dot
            );

            seen.push(id);
        }
    }
}

#[test]
fn a_function_body_stays_out_of_the_program_graph() {
    let dot = graph(TWO_FUNCTIONS);
    let clusters = clusters(&dot);
    let (_, program) = &clusters[0];

    // the declaration is a step in the program
    assert!(program.contains("function double(...) -> Int"), "{}", program);

    // its body is not
    assert!(!program.contains("return N * 2"), "{}", program);
}

#[test]
fn a_branch_splits_and_rejoins() {
    let dot = graph(TWO_FUNCTIONS);
    let clusters = clusters(&dot);
    let (_, classify) = &clusters[2];

    assert!(classify.contains("if N < 0"), "{}", classify);
    assert!(classify.contains("[label=\"<condition>\"]"), "{}", classify);
    assert!(classify.contains("end if"), "{}", classify);

    // both arms reach a return
    assert!(classify.contains("return \\\"negative\\\""), "{}", classify);
    assert!(classify.contains("return \\\"positive\\\""), "{}", classify);
}

#[test]
fn every_graph_has_a_start_and_functions_reach_an_end() {
    let dot = graph(TWO_FUNCTIONS);

    for (label, body) in clusters(&dot) {
        assert!(body.contains("<Start>"), "{} has no start\n{}", label, body);

        if label == "<program>" {
            continue;
        }

        assert!(body.contains("<End>"), "{} has no end\n{}", label, body);
    }
}

#[test]
fn a_program_that_does_not_compile_has_no_graph() {
    for source in ["output MISSING", "output 1 ,"] {
        let (errors, dot) = analysis::control_flow_graph(source.to_string());

        assert!(dot.is_none(), "{:?}", source);
        assert!(!errors.errors.is_empty(), "{:?}", source);
    }
}

/// A chain is parsed into the tree the nested form gives, so everything after
/// the parser -- the graph included -- cannot tell them apart.
#[test]
fn an_else_if_chain_draws_like_the_nested_form() {
    let chained = "X = 2\n\
                   if X == 1 then\n\
                       output 1\n\
                   else if X == 2 then\n\
                       output 2\n\
                   else\n\
                       output 3\n\
                   end";

    let nested = "X = 2\n\
                  if X == 1 then\n\
                      output 1\n\
                  else\n\
                      if X == 2 then\n\
                          output 2\n\
                      else\n\
                          output 3\n\
                      end\n\
                  end";

    assert_eq!(graph(chained), graph(nested));
}
