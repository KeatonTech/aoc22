use advent_of_code::helpers::parsing::{line_ending_or_eof, text_u32, ParsingError};
use nom::{
    bytes::complete::tag,
    character::complete::not_line_ending,
    combinator::iterator,
    sequence::{preceded, separated_pair, terminated},
    Parser,
};
use petgraph::{graph::NodeIndex, visit::EdgeRef, Direction};

// PARSING

#[derive(Debug)]
struct File {
    is_directory: bool,
    size: Option<u32>,
}

impl File {
    fn parse<'a>(input: &'a [u8]) -> Result<(&'a [u8], Self), ParsingError<'a>> {
        nom::branch::alt((
            preceded(tag("dir "), not_line_ending).map(|_name| File {
                is_directory: true,
                size: None,
            }),
            separated_pair(text_u32(), tag(" "), not_line_ending).map(|(size, _name)| File {
                is_directory: false,
                size: Some(size),
            }),
        ))(input)
    }
}

#[derive(Debug)]
enum ChangeDirCommand<'a> {
    Root(),
    Down { directory: &'a [u8] },
    Up(),
}

#[derive(Debug)]
enum Command<'a> {
    ChangeDir(ChangeDirCommand<'a>),
    List,
}

impl<'a> Command<'a> {
    fn parse(input: &'a [u8]) -> Result<(&'a [u8], Self), ParsingError<'a>> {
        preceded(
            tag("$ "),
            nom::branch::alt((
                preceded(tag("cd "), not_line_ending).map(|arg: &'a [u8]| {
                    Command::ChangeDir(match arg {
                        b"/" => ChangeDirCommand::Root(),
                        b".." => ChangeDirCommand::Up(),
                        _ => ChangeDirCommand::Down { directory: arg },
                    })
                }),
                tag("ls").map(|_| Command::List),
            )),
        )(input)
    }
}

#[derive(Debug)]
enum InputLine<'a> {
    Command(Command<'a>),
    File(File),
}

impl<'a> InputLine<'a> {
    fn parse(input: &'a [u8]) -> Result<(&'a [u8], Self), ParsingError<'a>> {
        nom::branch::alt((
            Command::parse.map(InputLine::Command),
            File::parse.map(InputLine::File),
        ))(input)
    }
}

// TREE BUILDING

#[derive(Debug)]
struct DirectoryNode<'a> {
    name: &'a [u8],
    files: Vec<File>,
    size: Option<usize>,
}

impl<'a> DirectoryNode<'a> {
    fn new_root() -> Self {
        DirectoryNode {
            name: "/".as_bytes(),
            files: vec![],
            size: None,
        }
    }
}

fn change_dir<'a>(
    graph: &mut petgraph::Graph<DirectoryNode<'a>, ()>,
    current_dir: NodeIndex,
    command: ChangeDirCommand<'a>,
) -> NodeIndex {
    match command {
        ChangeDirCommand::Up() => graph
            .edges_directed(current_dir, Direction::Incoming)
            .next()
            .expect("Directory node does not have a parent")
            .source(),
        ChangeDirCommand::Root() => {
            assert!(graph
                .edges_directed(current_dir, Direction::Incoming)
                .next()
                .is_none());
            current_dir
        }
        ChangeDirCommand::Down { directory } => {
            // Check if the directory already exists
            graph
                .edges_directed(current_dir, Direction::Outgoing)
                .map(|edge_ref| edge_ref.target())
                .find(|node_index| {
                    graph
                        .node_weight(*node_index)
                        .expect("Target must exist")
                        .name
                        == directory
                })
                .unwrap_or_else(|| {
                    let new_dir_index = graph.add_node(DirectoryNode {
                        name: directory,
                        files: vec![],
                        size: None,
                    });
                    graph.add_edge(current_dir, new_dir_index, ());
                    new_dir_index
                })
        }
    }
}

fn parse_from_command_input<'a>(
    input: &'a [u8],
) -> (petgraph::Graph<DirectoryNode<'a>, ()>, NodeIndex) {
    let mut graph = petgraph::Graph::<DirectoryNode<'a>, ()>::new();
    let root_node_index = graph.add_node(DirectoryNode::new_root());
    let mut current_node_index = root_node_index.clone();

    let mut input_iterator = iterator(input, terminated(InputLine::parse, line_ending_or_eof()));
    for input_line in &mut input_iterator {
        match input_line {
            InputLine::File(file) => {
                if !file.is_directory {
                    graph
                        .node_weight_mut(current_node_index)
                        .expect("Current directory node does not exist")
                        .files
                        .push(file)
                }
            }
            InputLine::Command(Command::ChangeDir(change_dir_command)) => {
                current_node_index = change_dir(&mut graph, current_node_index, change_dir_command);
            }
            InputLine::Command(Command::List) => (),
        }
    }

    return (graph, root_node_index);
}

fn calculate_directory_sizes<'a>(
    mut dir_graph: petgraph::Graph<DirectoryNode<'a>, ()>,
    root_node: NodeIndex,
) -> petgraph::Graph<DirectoryNode<'a>, ()> {
    let mut bottom_up_traversal = petgraph::visit::DfsPostOrder::new(&dir_graph, root_node);
    while let Some(visited) = bottom_up_traversal.next(&dir_graph) {
        let dir_node = dir_graph.node_weight(visited).unwrap();
        let total_file_size: usize = dir_node
            .files
            .iter()
            .map(|file| file.size.unwrap() as usize)
            .sum();
        let total_child_dir_size: usize = dir_graph
            .edges_directed(visited, Direction::Outgoing)
            .map(|edge_ref| edge_ref.target())
            .map(|node_index| dir_graph.node_weight(node_index).unwrap())
            .map(|dir_node| {
                dir_node
                    .size
                    .expect("Traversal error! Directory was visited before its child")
            })
            .sum();
        dir_graph.node_weight_mut(visited).unwrap().size =
            Some(total_file_size + total_child_dir_size);
    }
    dir_graph
}

// RUNTIME

pub fn part_one(input: &str) -> Option<u32> {
    let (dir_graph, root_node) = parse_from_command_input(input.as_bytes());
    let sized_dir_graph = calculate_directory_sizes(dir_graph, root_node);
    Some(
        sized_dir_graph
            .node_weights()
            .map(|dir| dir.size.unwrap())
            .filter(|size| *size < 100000usize)
            .sum::<usize>() as u32,
    )
}

pub fn part_two(input: &str) -> Option<u32> {
    let (dir_graph, root_node) = parse_from_command_input(input.as_bytes());
    let sized_dir_graph = calculate_directory_sizes(dir_graph, root_node);
    let size_of_root = sized_dir_graph
        .node_weight(root_node)
        .unwrap()
        .size
        .unwrap();
    let available_disk_space = 70000000usize - size_of_root;
    let space_to_free = 30000000isize - available_disk_space as isize;
    if space_to_free < 0 {
        panic!("Disk already has enough free space!");
    }
    Some(
        sized_dir_graph
            .node_weights()
            .map(|dir| dir.size.unwrap())
            .filter(|size| *size as isize > space_to_free)
            .min()
            .unwrap() as u32,
    )
}

fn main() {
    let input = &advent_of_code::read_file("inputs", 7);
    advent_of_code::solve!(1, part_one, input);
    advent_of_code::solve!(2, part_two, input);
}
