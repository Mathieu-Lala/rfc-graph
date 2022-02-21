//! Fast and easy queue abstraction.

#![warn(missing_docs)]

const LINK: &str = "https://datatracker.ietf.org/doc/html/rfc";

/// The `RfcGraph` type, wrapping all the logics of this crate.
///
/// Use the function [RfcGraph::get]
pub struct RfcGraph {
    did_search: std::collections::HashMap<i32, (bool, petgraph::prelude::NodeIndex<u32>)>,
    graph: petgraph::Graph<i32, i32>,
    cache: std::collections::HashMap<i32, Vec<i32>>,
}

/// Will initialize a graph model, and load a `cache.json` file to reduce web query.
impl Default for RfcGraph {
    fn default() -> Self {
        Self {
            did_search: Default::default(),
            graph: Default::default(),
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
    async fn query_list_links_of_rfc(&mut self, number: i32) -> Vec<i32> {
        if let Some(links) = self.cache.get(&number) {
            return links.clone();
        }

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

        self.cache.insert(number, links.clone());
        links
    }
}

impl RfcGraph {
    fn get_or_emplace(&mut self, number: i32, search: bool) -> petgraph::prelude::NodeIndex<u32> {
        match self.did_search.get(&number) {
            Some((_, i)) => *i,
            None => {
                let node = self.graph.add_node(number);
                self.did_search.insert(number, (search, node));
                node
            }
        }
    }

    async fn add(&mut self, number: i32) -> Option<Vec<i32>> {
        if !self.did_search.get(&number).map(|i| i.0).unwrap_or(false) {
            let number_node = self.get_or_emplace(number, true);
            let links = self.query_list_links_of_rfc(number).await;

            let nodes_linked = links
                .iter()
                .map(|i| (number_node, self.get_or_emplace(*i, false)))
                .collect::<Vec<_>>();

            self.graph.extend_with_edges(nodes_linked);
            self.to_svg();
            Some(links)
        } else {
            None
        }
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
                petgraph::dot::Dot::with_config(&self.graph, &[petgraph::dot::Config::EdgeNoLabel])
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
            let v = self.add(number).await.unwrap_or_default();
            output.extend(&v);
            for i in v {
                output.extend(self.rec_get_rfc(i, rec_max - 1).await);
            }
        }
        output
    }

    /// Initialize a `RfcGraph` object and query the graph around the node `root`.
    ///
    /// The function will iterate in the graph recursively for `recursion_max`.
    pub async fn get(root: i32, recursion_max: u32) -> Vec<i32> {
        RfcGraph::default().rec_get_rfc(root, recursion_max).await
    }
}
