//! Fast and easy queue abstraction.

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::cargo)]

const LINK: &str = "https://datatracker.ietf.org/doc/html/rfc";

#[derive(Debug, Clone, Copy)]
enum RfcStatus {
    Unknown,            // #FFF
    Draft,              // #F44
    Informational,      // #FA0
    Experimental,       // #EE0
    BestCommonPractice, // #F4F
    ProposedStandard,   // #66F
    DraftStandard,      // #4DD
    InternetStandard,   // #4F4
    Historic,           // #666
    Obsolete,           // #840
}

impl Default for RfcStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

impl RfcStatus {
    const fn as_color(self) -> &'static str {
        match self {
            Self::Unknown => "#F0F0F0",
            Self::Draft => "#F04040",
            Self::Informational => "#F0A000",
            Self::Experimental => "#E0E000",
            Self::BestCommonPractice => "#F040F0",
            Self::ProposedStandard => "#6060F0",
            Self::DraftStandard => "#40D0D0",
            Self::InternetStandard => "#40F040",
            Self::Historic => "#606060",
            Self::Obsolete => "#804000",
        }
    }

    fn from_classes(classes: Vec<&str>) -> Option<Self> {
        for i in classes {
            let found = match i {
                "bgwhite" => Some(Self::Unknown),
                "bgred" => Some(Self::Draft),
                "bggrey" => Some(Self::Historic),
                "bgbrown" => Some(Self::Obsolete),
                "bgorange" => Some(Self::Informational),
                "bgyellow" => Some(Self::Experimental),
                "bgmagenta" => Some(Self::BestCommonPractice),
                "bgblue" => Some(Self::ProposedStandard),
                "bgcyan" => Some(Self::DraftStandard),
                "bggreen" => Some(Self::InternetStandard),
                _ => None,
            };
            if found.is_some() {
                return found;
            }
        }
        None
    }
}

/// The `RfcGraph` type, wrapping all the logics of this crate.
///
/// Use the function [`RfcGraph::get`]
pub struct RfcGraph {
    did_search:
        std::collections::HashMap<i32, (bool, petgraph::prelude::NodeIndex<u32>, RfcStatus)>,
    graph: petgraph::Graph<i32, i32>,
    cache: std::collections::HashMap<i32, Vec<i32>>,
}

/// Will initialize a graph model, and load a `cache.json` file to reduce web query.
impl Default for RfcGraph {
    fn default() -> Self {
        Self {
            did_search: std::collections::HashMap::default(),
            graph: petgraph::Graph::default(),
            cache: std::fs::read("cache.json")
                .map_err(anyhow::Error::msg)
                .and_then(|file| serde_json::from_slice(&file).map_err(anyhow::Error::msg))
                .unwrap_or_default(),
        }
    }
}

/// Will save the `cache.json` file for next usage.
impl Drop for RfcGraph {
    fn drop(&mut self) {
        serde_json::to_string_pretty(&self.cache)
            .map_err(anyhow::Error::msg)
            .and_then(|json| std::fs::write("cache.json", json).map_err(anyhow::Error::msg))
            .unwrap();
    }
}

impl RfcGraph {
    async fn query_list_links_of_rfc(&mut self, number: i32) -> (Vec<i32>, RfcStatus) {
        // if let Some(links) = self.cache.get(&number) {
        //     return links.clone();
        // }

        let html = reqwest::get(format!("{LINK}{number}"))
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        let document = scraper::Html::parse_document(&html);
        let selector = scraper::Selector::parse("a").unwrap();

        let mut links = document
            .select(&selector)
            .filter_map(|i| {
                let href = match i.value().attr("href") {
                    Some(value) => value,
                    None => return None,
                };

                if !href.starts_with("/doc/html/rfc") {
                    return None;
                }

                Some(href)
            })
            .collect::<Vec<_>>();

        links.sort_unstable();
        links.dedup();

        let mut links = links
            .into_iter()
            .filter_map(|i| i.strip_prefix("/doc/html/rfc")?.parse::<i32>().ok())
            .collect::<Vec<_>>();

        links.sort_unstable();
        links.dedup();

        let links = links
            .into_iter()
            .filter(|i| *i != number)
            .collect::<Vec<_>>();

        let selector =
            scraper::Selector::parse(r#"div[title="Click for colour legend."]"#).unwrap();

        let html_color = document.select(&selector).next().unwrap();
        let html_color_classes =
            RfcStatus::from_classes(html_color.value().classes().collect::<Vec<_>>()).unwrap();
        println!("{:?}", html_color_classes);

        self.cache.insert(number, links.clone());
        (links, html_color_classes)
    }
}

impl RfcGraph {
    fn get_or_emplace(
        &mut self,
        number: i32,
        search: bool,
        color: Option<RfcStatus>,
    ) -> petgraph::prelude::NodeIndex<u32> {
        if let Some((_, i, status)) = self.did_search.get_mut(&number) {
            if let Some(color) = color {
                *status = color;
            }
            *i
        } else {
            let node = self.graph.add_node(number);
            self.did_search
                .insert(number, (search, node, color.unwrap_or(RfcStatus::Unknown)));
            node
        }
    }

    async fn add(&mut self, number: i32) -> Option<(Vec<i32>, RfcStatus)> {
        if self.did_search.get(&number).map_or(false, |i| i.0) {
            return None;
        }
        let (linked, color) = self.query_list_links_of_rfc(number).await;
        let number_node = self.get_or_emplace(number, true, Some(color));

        let nodes_linked = linked
            .iter()
            .map(|i| (number_node, self.get_or_emplace(*i, false, None)))
            .collect::<Vec<_>>();

        self.graph.extend_with_edges(nodes_linked);
        self.to_svg();
        Some((linked, color))
    }

    fn to_svg(&self) {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open("input.dot")
            .unwrap();

        std::io::Write::write_fmt(
            &mut file,
            format_args!(
                "{:?}",
                petgraph::dot::Dot::with_attr_getters(
                    &self.graph,
                    &[petgraph::dot::Config::EdgeNoLabel],
                    &|_, _| String::new(),
                    &|_, node| {
                        let (_, _, color) = self.did_search.get(node.1).unwrap();

                        format!(
                            "color=\"{color}\" style=\"filled\"",
                            color = color.as_color()
                        )
                    }
                )
            ),
        )
        .unwrap();

        std::process::Command::new("dot")
            .arg("-Tsvg")
            .arg("input.dot")
            .arg("-o")
            .arg("output.svg")
            .output()
            .unwrap();
    }
}

impl RfcGraph {
    #[async_recursion::async_recursion]
    async fn rec_get_rfc<'a>(&'a mut self, number: i32, rec_max: u32) -> Vec<i32> {
        // NOTE: should be a stream, but stream! can't be recursive...
        let mut output = vec![];
        if rec_max != 0 {
            let (linked, _) = self.add(number).await.unwrap_or_default();
            output.extend(&linked);
            for i in linked {
                output.extend(self.rec_get_rfc(i, rec_max - 1).await);
            }
        }
        output
    }

    /// Initialize a `RfcGraph` object and query the graph around the node `root`.
    ///
    /// The function will iterate in the graph recursively for `recursion_max`.
    pub async fn get(root: i32, recursion_max: u32) -> Vec<i32> {
        Self::default().rec_get_rfc(root, recursion_max).await
    }
}
