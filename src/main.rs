use errors::MagiResult;

mod config;
mod errors;
mod git;
mod magi;
mod model;
mod msg;
mod view;

fn main() -> MagiResult<()> {
    magi::run()?;
    Ok(())
}
