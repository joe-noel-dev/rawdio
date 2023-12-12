///Connect a graph
///
///
/// # Example
///
/// ```
/// connect_nodes!("input" => oscillator => gain);
/// ```
#[macro_export]
macro_rules! connect_nodes {
    ("input" => $destination:expr $(=> $($rest:tt)+)?) => {
        $destination.node.connect_to_input();
        $(connect_nodes!($destination => $($rest)+);)?
    };
    ($source:expr => "output") => {
        $source.node.connect_to_output();
    };
    ($source:expr => $destination:expr $(=> $($nodes:tt)+)?) => {
        $source.node.connect_to(&$destination.node);
        $(connect_nodes!($destination => $($nodes)+);)?
    };
}
