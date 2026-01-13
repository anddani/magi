use magi::errors::MagiResult;

fn main() -> MagiResult<()> {
    magi::magi::run()?;
    Ok(())
}
