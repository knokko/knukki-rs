mod menu;

use knukki::start;
use menu::create_app;

fn main() {
    start(create_app(), "Example name");
}
