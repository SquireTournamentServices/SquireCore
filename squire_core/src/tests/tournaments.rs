use rocket::http::{ContentType, MediaType, Status};

use squire_lib::tournament::TournamentPreset;
use squire_sdk::tournaments::{CreateTournamentRequest, CreateTournamentResponse};

use super::init::get_server;

#[tokio::test]
async fn create_tournament() {
    let client = get_server().await;
    let data = CreateTournamentRequest {
        name: "Test".into(),
        preset: TournamentPreset::Swiss,
        format: "Pioneer".into(),
    };
    let response = client
        .post("/api/v1/tournaments/create")
        .header(ContentType(MediaType::JSON))
        .body(serde_json::to_string(&data).expect("Could not serialize tournament create request"))
        .dispatch()
        .await;
    println!("{:?}", response.status().reason());
    assert_eq!(response.status(), Status::Ok);
    let response: CreateTournamentResponse = response
        .into_json()
        .await
        .expect("malformed response: tournament create");
    let tourn = response.0;
    assert_eq!(tourn.name, data.name);
    assert_eq!(tourn.format, data.format);
}
