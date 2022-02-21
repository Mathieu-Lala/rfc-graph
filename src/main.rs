use rfc_graph::Database;

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

    let mut db = Database::default();

    let rfcs = db.rec_get_rfc(args.root, args.recursion_max).await;
    println!("{:?}", rfcs);

    Ok(())
}
