use rocket::Rocket;

use crate::files::handler;

pub fn create_routes(rocket: Rocket) -> Rocket {
    rocket.mount(
        "/files",
        routes![
            handler::patch,
            handler::patch_output,
            handler::get,
            handler::get_by_submission_id
        ],
    )
}
