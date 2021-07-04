/* standard use */
use bio::alphabets::dna::revcomp;
use clap::{crate_version, load_yaml, App, AppSettings};
use gfa::{gfa::GFA, parser::GFAParser};
use handlegraph::{
    handle::{Direction, Handle},
    handlegraph::*,
    hashgraph::HashGraph,
    //mutablehandlegraph::{AdditiveHandleGraph, MutableHandles},
};
use rustc_hash::FxHashMap;
use std::{str, usize};

fn get_node_seq(node: &Handle, graph: &HashGraph) -> String {
    let mut seq: Vec<u8> = graph.sequence(*node).collect();
    if node.is_reverse() {
        seq = revcomp(seq);
    }
    str::from_utf8(&seq).unwrap().to_string()
}

fn main() {
    let yaml = load_yaml!("cli.yaml");
    let app = App::from(yaml)
        .version(crate_version!())
        .setting(AppSettings::SubcommandRequiredElseHelp);
    let matches = app.get_matches();
    let _threads: usize = matches.value_of_t("threads").unwrap();
    if let Some(matches) = matches.subcommand_matches("bubble") {
        run_bubble(matches);
    }
}

fn extend_seq(node: &Handle, graph: &HashGraph, direction: Direction) -> String {
    let mut neighbors: Vec<Handle> = graph.neighbors(*node, direction).collect();
    let mut seq = "".to_string();
    while neighbors.len() == 1 {
        let neighbor = neighbors[0];
        seq.push_str(&get_node_seq(&neighbor, &graph));
        neighbors = graph.neighbors(neighbor, direction).collect();
    }
    seq
}

fn run_bubble(args: &clap::ArgMatches) {
    let gfa_file = args.value_of("GFA").unwrap();
    eprintln!("Reading from: {}", gfa_file);
    let extend = args.is_present("extend");

    let parser = GFAParser::new();
    let gfa: GFA<usize, ()> = parser.parse_file(gfa_file).unwrap();

    log::info!("constructing handle graph");
    let mut graph = HashGraph::from_gfa(&gfa);

    for node in graph.handles() {
        node.unpack_number();
        graph.node_len(node);
        let _seq: Vec<u8> = graph.sequence(node).collect();

        if extend {
            let left_seq = extend_seq(&node, &graph, Direction::Left);
            let right_seq = extend_seq(&node, &graph, Direction::Right);
            println!(
                ">{}\n{}{}{}",
                node.id(),
                left_seq,
                get_node_seq(&node, &graph),
                right_seq
            );
        }
    }

    log::debug!("handlegraph has {} edges", graph.edge_count());
    graph.paths.clear();

    log::info!("storing length of original nodes for bookkeeping");
    let mut node_lens: FxHashMap<usize, usize> = FxHashMap::default();
    node_lens.reserve(graph.node_count());
    for v in graph.handles() {
        node_lens.insert(v.unpack_number() as usize, graph.node_len(v));
    }
}
