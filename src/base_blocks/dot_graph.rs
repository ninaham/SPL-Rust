use regex::{Captures, Regex};
use std::fmt;

use super::{Block, BlockContent, BlockGraph};

const REGEX_TEMINAL_COLORS: &str =
    r"\u{1b}\[(?:(?<color>[39][0-7])|38;2;(?<truecolor>\d+;\d+;\d+))m(?<text>.*?)\u{1b}\[0m";

const DOT_ATTRIBUTES: &str = "
    graph[bgcolor=grey16,fontcolor=grey64,ranksep=1,nodesep=0.5];
    node [shape=box,color=grey64,fontname=monospace,fontcolor=grey64];
    edge [color=grey64,fontcolor=grey64];
";

impl fmt::Display for BlockGraph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Enable color even if stdout is not a terminal
        colored::control::set_override(true);

        writeln!(f, "digraph {{{}", DOT_ATTRIBUTES)?;

        let re = Regex::new(REGEX_TEMINAL_COLORS).unwrap();
        self.blocks.iter().enumerate().try_for_each(|(i, b)| {
            writeln!(f, "{i} [label=<{}>];", to_dot_html(&b.to_string(), &re))
        })?;

        self.edges.iter().enumerate().try_for_each(|(i, e)| {
            e.iter()
                .try_for_each(|j| writeln!(f, "{}:s -> {}:n;", i, j))
        })?;

        writeln!(f, "}}")?;

        colored::control::unset_override();
        Ok(())
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.content {
            BlockContent::Start => write!(f, "start"),
            BlockContent::Stop => write!(f, "stop"),
            BlockContent::Code(code) => code.iter().try_for_each(|quad| writeln!(f, "{}", quad)),
        }
    }
}

fn to_dot_html(b: &str, re: &Regex) -> String {
    re.replace_all(&b, |caps: &Captures| {
        let color = if let Some(color) = caps.name("color") {
            match color.as_str() {
                "35" => "magenta",
                "94" => "lightblue",
                c => panic!("unknown color {c}"),
            }
        } else {
            let [r, g, b] = caps["truecolor"]
                .split(';')
                .map(|s| s.parse::<u8>().unwrap())
                .collect::<Vec<_>>()
                .try_into()
                .unwrap();
            &format!("#{r:x}{g:x}{b:x}")
        };
        format!("<font color=\"{color}\">{}</font>", &caps["text"])
    })
    .replace("  ", "&nbsp; ")
    .replace('\t', "&nbsp; &nbsp; ")
    .replace('\n', "<br align=\"left\" />")
}
