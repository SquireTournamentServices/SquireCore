use std::collections::HashMap;

use yew::prelude::*;

use reqwasm::http::{Request, RequestMode};
use serde::Deserialize;
use serde_json;

use squire_sdk::accounts::{AccountId, UserAccount};

#[derive(Properties)]
struct UserListProps {
    users: HashMap<AccountId, UserAccount>,
}

#[function_component(UserList)]
fn user_list(UserListProps { users }: &UserListProps) -> Html {
    users.iter().map(|(id, user)| html! {
        <p>{format!("{id:?}: {}", serde_json::to_string(user).unwrap())}</p>
    }).collect()
}

#[function_component(App)]
fn app() -> Html {
    let users = use_state(|| HashMap::new());
    {
        let users = users.clone();
        use_effect_with_deps(move |_| {
            let users = users.clone();
            wasm_bindgen_futures::spawn_local(async move {
                println!("Getting ready to fetch");
                let data =
                    Request::get("http://127.0.0.1:8000/accounts/users/all")
                    .mode(RequestMode::NoCors)
                    .send()
                    .await
                    .unwrap()
                    .json()
                    .await
                    .unwrap();
                println!("{data:?}");
                users.set(data);
            });
            || ()
        }, ());
    }

    html! {
        <>
            <h1>{ "View all Users" }</h1>
            <div>
            <h3>{"Users"}</h3>
            <UserList users={(*users).clone()} />
            </div>
            </>
    }
}

fn main() {
    yew::start_app::<App>();
}

impl PartialEq for UserListProps {
    fn eq(&self, other: &Self) -> bool {
        self.users.len().eq(&other.users.len())
    }
}
