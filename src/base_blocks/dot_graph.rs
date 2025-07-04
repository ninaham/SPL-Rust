use regex::{Captures, Regex};
use std::fmt;

use crate::optimizations::tarjan::Scc;

use super::{Block, BlockContent, BlockGraph};

/// Regular expression to match ANSI terminal color escape sequences.
/// This is used to convert colored text into equivalent DOT HTML formatting.
const REGEX_TEMINAL_COLORS: &str =
    r"\u{1b}\[(?:(?<color>[39][0-7])|38;2;(?<truecolor>\d+;\d+;\d+))m(?<text>.*?)\u{1b}\[0m";

/// Global DOT attributes for graph, node, and edge styling.
const DOT_ATTRIBUTES: &str = "
    graph[bgcolor=grey16,fontname=monospace,fontcolor=grey64,pencolor=grey32,ranksep=1,nodesep=0.5,labeljust=l];
    node [shape=box,color=grey64,fontname=monospace,fontcolor=grey64];
    edge [color=grey64,fontcolor=grey64];
";

/// Implement `Display` for `BlockGraph` to render it as a DOT graph description.
impl fmt::Display for BlockGraph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Force-enable ANSI colors even if not writing to a terminal
        colored::control::set_override(true);

        // Write the DOT header and graph styling attributes
        writeln!(f, "digraph {{{DOT_ATTRIBUTES}")?;

        // Compile the regex for detecting ANSI terminal escape sequences
        let re = Regex::new(REGEX_TEMINAL_COLORS).unwrap();

        // Render each block as a DOT node with HTML-style label formatting
        self.blocks.iter().enumerate().try_for_each(|(i, b)| {
            let mut dot_html = to_dot_html(&b.to_string(), &re);
            if dot_html.is_empty() {
                dot_html = " ".to_string(); // fallback if block has no content
            }
            writeln!(f, "{i} [xlabel=\"B{i}\",label=<{dot_html}>];")
        })?;

        // Render each edge in the control flow graph
        self.edges
            .iter()
            .enumerate()
            .try_for_each(|(i, e)| e.iter().try_for_each(|j| writeln!(f, "{i}:s -> {j}:n;")))?;

        // Render loops (strongly connected components) as DOT subgraphs, if any
        if let Some(sccs) = &self.sccs {
            for (scc_idx, _) in sccs
                .iter()
                .enumerate()
                .filter(|(_, scc)| scc.parent_idx.is_none())
            {
                write_subgraph(f, scc_idx, sccs)?;
            }
        }

        // Close DOT graph
        writeln!(f, "}}")?;

        // Reset color override after output
        colored::control::unset_override();
        Ok(())
    }
}

/// Helper to recursively write nested subgraphs for loops (SCCs).
fn write_subgraph(f: &mut fmt::Formatter<'_>, scc_idx: usize, sccs: &[Scc]) -> fmt::Result {
    let scc = &sccs[scc_idx];

    // Choose from a set of predefined colors for different loop clusters
    let colors = ["magenta", "cyan", "lime", "hotpink", "honeydew", "plum"];

    writeln!(f, "subgraph cluster{scc_idx} {{")?;
    writeln!(f, "margin=40;")?;
    writeln!(f, "label=\"Loop {scc_idx}\";")?;
    let color = colors[scc_idx % colors.len()];
    writeln!(f, "pencolor={color};")?;

    // List all nodes (blocks) that are part of the current SCC
    for n in &scc.nodes {
        writeln!(f, "{n};")?;
    }

    // Recursively write any child loops (nested SCCs)
    for &c in &scc.children_idx {
        write_subgraph(f, c, sccs)?;
    }

    writeln!(f, "}}")?;
    Ok(())
}

/// Implement `Display` for `Block` to convert its content into a printable form.
impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.content {
            BlockContent::Start => write!(f, "start"), // entry block
            BlockContent::Stop => write!(f, "stop"),   // exit block
            BlockContent::Code(code) => code.iter().try_for_each(|quad| writeln!(f, "{quad}")), // instructions
        }
    }
}

/// Convert ANSI-colored terminal text into DOT-compatible HTML-style labels.
fn to_dot_html(b: &str, re: &Regex) -> String {
    re.replace_all(b, |caps: &Captures| {
        // Convert terminal color to HTML hex or name
        let color = caps.name("color").map_or_else(
            || {
                // Convert RGB escape sequence into hex color code
                let [r, g, b] = caps["truecolor"]
                    .split(';')
                    .map(|s| s.parse::<u8>().unwrap())
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap();
                format!("#{r:x}{g:x}{b:x}")
            },
            |color| match color.as_str() {
                "35" => "magenta".to_string(),
                "94" => "lightblue".to_string(),
                c => panic!("unknown color {c}"),
            },
        );
        // Wrap the text in a <font> tag with the determined color
        format!("<font color=\"{color}\">{}</font>", &caps["text"])
    })
    // Replace spaces and tabs with HTML-safe equivalents for formatting
    .replace("  ", "&nbsp; ")
    .replace('\t', "&nbsp; &nbsp; ")
    .replace('\n', "<br align=\"left\" />")
}
