use rocket::Rocket;

use crate::assignments::handler;

pub fn create_routes(rocket: Rocket) -> Rocket {
    rocket.mount(
        "/assignments",
        routes![handler::insert, handler::get, handler::update],
    )
}
