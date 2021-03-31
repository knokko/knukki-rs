mod menu;

use knukki::start;
use menu::*;

fn main() {
    start(create_app(), EXAMPLE_NAME);
}
