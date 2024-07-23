// Copyright 2024 Mazze Labs. All rights reserved.
// Mazze is free software and distributed under GNU General Public License.
// See http://www.gnu.org/licenses/

use mazze_types::{hexstr_to_h256, H256}; // Importing types and conversion functions from mazze_types.
use primitives::Block; // Importing Block type from primitives.
use std::{
    collections::{HashSet, VecDeque}, // Importing necessary collections.
    sync::Arc, // Importing Arc for atomic reference counting.
};

// Function to open the database given a path.
fn open_db(db_path: &str) -> std::io::Result<Arc<db::SystemDB>> {
    // Configure the database settings.
    let db_config = db::db_config(
        std::path::Path::new(db_path),
        None,
        db::DatabaseCompactionProfile::default(),
        mazzecore::db::NUM_COLUMNS,
        false,
    );

    // Open and return the database.
    db::open_database(db_path, &db_config)
}

// Function to retrieve a block from the database given its hash.
fn retrieve_block(db: &Arc<db::SystemDB>, hash: &H256) -> Option<Block> {
    let block = db.key_value().get(mazzecore::db::COL_BLOCKS, hash.as_bytes()).expect(
        "Low level database error when fetching block. Some issue with disk?",
    )?;

    // Decode the block using RLP (Recursive Length Prefix) encoding.
    let rlp = rlp::Rlp::new(&block);
    let block = Block::decode_with_tx_public(&rlp).expect("Wrong block rlp format!");

    Some(block)
}

// Function to format a hash for printing.
fn fmt_hash(hash: &H256) -> String {
    format!("{:?}", hash)[0..14].to_string() + "..."
}

// Function to print an edge between two nodes in the graph.
fn print_edge(from: &H256, to: &H256) {
    println!("\"{}\" -> \"{}\";", fmt_hash(from), fmt_hash(to));
}

// Function to print a reference edge (dotted) between two nodes in the graph.
fn print_ref_edge(from: &H256, to: &H256) {
    println!(
        "\"{}\" -> \"{}\" [style=dotted];",
        fmt_hash(from),
        fmt_hash(to)
    );
}

// Function to print the graph starting from a given block hash up to a specified depth.
fn print_graph(db: &Arc<db::SystemDB>, from: &H256, max_depth: u32) {
    // Print the start of the graph in Graphviz dot format.
    println!("digraph G {{");
    println!("rankdir=\"RL\";");
    println!("node [shape=box];");

    // Initialize the queue and visited set for BFS traversal.
    let mut queue: VecDeque<(u32, H256)> = VecDeque::new();
    let mut visited: HashSet<H256> = HashSet::new();
    queue.push_back((0, from.clone()));

    // BFS traversal of the graph.
    while let Some((depth, hash)) = queue.pop_front() {
        if visited.contains(&hash) || depth == max_depth {
            continue;
        }

        assert!(depth < max_depth);
        visited.insert(hash);

        // Retrieve the block and process its parent and referee hashes.
        if let Some(block) = retrieve_block(&db, &hash) {
            let parent = block.block_header.parent_hash();
            let refs = block.block_header.referee_hashes();

            print_edge(&hash, &parent); // Print the edge to the parent block.
            queue.push_back((depth + 1, *parent)); // Add parent to the queue.

            // Print reference edges and add referee hashes to the queue.
            for r in refs {
                print_ref_edge(&hash, &r);
                queue.push_back((depth + 1, *r));
            }
        }
    }

    println!("}}"); // Print the end of the graph.
}

// Struct to hold the configuration parameters.
struct Config {
    db_path: String,
    from_block: H256,
    max_depth: u32,
}

// Function to validate the from_str conversion.
fn from_str_validator<T: std::str::FromStr>(arg: String) -> Result<(), String> {
    match arg.parse::<T>() {
        Ok(_) => Ok(()),
        Err(_) => Err(arg),
    }
}

// Function to parse command-line arguments and return a Config struct.
fn parse_config() -> Config {
    let matches = clap::App::new("mazze-gen-dot")
        .version("0.1")
        .about(
"Generate Graphviz dot files from your local blockchain db
Example usage:
    mazze-gen-dot
        --db-path ./run/blockchain_db
        --from-block 0x3159d8d9b125a738cc226a9b85f6d7fa0da1567018c6771f9bf658e83496834d
        --max-depth 10000
        > graph.dot
    dot -Tsvg graph.dot -o graph.svg")
        .arg(
            clap::Arg::with_name("db-path")
                .long("db-path")
                .value_name("PATH")
                .help("Specifies local blockchain db directory")
                .takes_value(true)
                .required(true),
        )
        .arg(
            clap::Arg::with_name("from-block")
                .long("from-block")
                .value_name("HASH")
                .help("Sets starting block of DAG traversal")
                .takes_value(true)
                .required(true),
        )
        .arg(
            clap::Arg::with_name("max-depth")
                .long("max-depth")
                .value_name("NUM")
                .help("Sets maximum depth for traversal")
                .takes_value(true)
                .required(true)
                .validator(from_str_validator::<u32>),
        )
        .get_matches();

    // Retrieve and parse command-line arguments.
    let db_path = matches.value_of("db-path").unwrap();
    let max_depth = matches
        .value_of("max-depth")
        .unwrap()
        .parse::<u32>()
        .unwrap();

    // Parse the starting block hash.
    let from_block = {
        let mut from = matches.value_of("from-block").unwrap();

        if from.starts_with("0x") {
            from = &from[2..];
        }

        hexstr_to_h256(from)
    };

    // Return the parsed configuration.
    Config {
        db_path: String::from(db_path),
        from_block,
        max_depth,
    }
}

// Main function to execute the program.
fn main() {
    let config = parse_config(); // Parse configuration.
    let db = open_db(&config.db_path).unwrap(); // Open the database.
    print_graph(&db, &config.from_block, config.max_depth); // Print the graph.
}