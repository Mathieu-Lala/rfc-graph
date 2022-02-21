use rfc_graph::RfcGraph;

#[derive(clap::Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Number of the first rfc page in the graph (root)
    #[clap(long)]
    root: i32,

    /// Number of recursive iteration max
    #[clap(long, default_value_t = 2)]
    recursion_max: u32,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = <Args as clap::StructOpt>::parse();

    let rfcs = RfcGraph::get(args.root, args.recursion_max).await;

    println!("{:?}", rfcs);

    Ok(())
}
