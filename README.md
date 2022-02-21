# RFC Graph

<div align="center">
  <a href="https://crates.io/crates/rfc-graph">
    <img src="https://img.shields.io/crates/v/rfc-graph.svg"
      alt="Crates.io" />
  </a>
  <a href="https://docs.rs/rfc-grap">
    <img src="https://docs.rs/rfc-graph/badge.svg"
      alt="docs" />
  </a>
</div>

How the rfc are referencing each other ?

```rs
let rfcs: Vec<i32> = rfc_graph::RfcGraph::get(5322, 3).await;
println!("{rfcs:?}");
```

```sh
$> cargo run -- -h
rfc-graph 0.1.1

USAGE:
    rfc-graph [OPTIONS] --root <ROOT>

OPTIONS:
    -h, --help                             Print help information
        --recursion-max <RECURSION_MAX>    Number of recursive iteration max [default: 2]
        --root <ROOT>                      Number of the first rfc page in the graph (root)
    -V, --version                          Print version information
```

Output generated:

* `cache.json` : a `HashMap<i32, Vec<i32>>` with key is the rfc source and values are the rfc referenced
* `input.dot` : a representation of the graph generated following the [dot format](https://graphviz.org/doc/info/lang.html) by [graphviz](https://graphviz.org/)
* `output.svg` : a svg version of the dot graph

```sh
$> cargo run -- --root 5322 --recursion-max 1
```

![rfc 5322 one recursion](doc/5322-level-1.svg)

```sh
$> cargo run -- --root 5322 --recursion-max 2
```

![rfc 5322 two recursion](doc/5322-level-2.svg)

The referencing in the rfcs are exponential, so you **might not** want to run with `--recursion-max 4`

## Next features

I would like to add the following features :

* display the title of the rfc
* show the status following this legend :

| Status                            | Color                                                            |
| --------------------------------- | ---------------------------------------------------------------- |
| Unknown                           | <div style="height:20px;width:20px;background-color:#FFF"></div> |
| Draft                             | <div style="height:20px;width:20px;background-color:#F44"></div> |
| Informational                     | <div style="height:20px;width:20px;background-color:#FA0"></div> |
| Experimental                      | <div style="height:20px;width:20px;background-color:#EE0"></div> |
| Best Common Practice              | <div style="height:20px;width:20px;background-color:#F4F"></div> |
| Proposed Standard                 | <div style="height:20px;width:20px;background-color:#66F"></div> |
| Draft Standard (old designation)  | <div style="height:20px;width:20px;background-color:#4DD"></div> |
| Internet Standard                 | <div style="height:20px;width:20px;background-color:#4F4"></div> |
| Historic                          | <div style="height:20px;width:20px;background-color:#666"></div> |
| Obsolete                          | <div style="height:20px;width:20px;background-color:#840"></div> |
