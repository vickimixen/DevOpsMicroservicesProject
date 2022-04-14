use rocket::Rocket;

use crate::submissions::handler;

pub fn create_routes(rocket: Rocket) -> Rocket {
    rocket.mount(
        "/submissions",
        routes![
            handler::all,
            handler::insert,
            handler::get_by_unique,
            handler::all_submissions_for_assignment
        ],
    )
}
