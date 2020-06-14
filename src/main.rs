use web_v2::database::Database;
use futures::executor::block_on;

fn main() {
    print!("{:?}", block_on(block_on(Database::new(None)).unwrap().connect()));
}
