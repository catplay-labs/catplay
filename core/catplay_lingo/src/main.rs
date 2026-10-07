mod codegen;

fn main() -> anyhow::Result<()> {
    codegen::run_cli()
}
