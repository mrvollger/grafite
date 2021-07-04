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
        //println!("{}", str::from_utf8(&seq).unwrap());
        let left: Vec<Handle> = graph.neighbors(node, Direction::Left).collect();
        let right: Vec<Handle> = graph.neighbors(node, Direction::Right).collect();
        if left.len() == 1 && right.len() == 1 && extend {
            let left = left[0];
            let right = right[0];
            println!(
                ">{} left:{}{} right:{}{}\n{}{}{}",
                node.id(),
                if left.is_reverse() { "-" } else { "" },
                left.id(),
                if right.is_reverse() { "-" } else { "" },
                right.id(),
                get_node_seq(&left, &graph),
                get_node_seq(&node, &graph),
                get_node_seq(&right, &graph),
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
