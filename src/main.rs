use futures::executor::block_on;
use web_v2::course::get_course;

fn main() {
    block_on(get_course(None,None));
}
