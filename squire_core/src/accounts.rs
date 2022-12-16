use axum::Json;
use dashmap::DashMap;
use once_cell::sync::OnceCell;

use squire_sdk::{
    accounts::{LoginRequest, LoginResponse},
    model::{
        accounts::{OrganizationAccount, SquireAccount},
        identifiers::{AdminId, OrganizationAccountId as OrgId, SquireAccountId},
    },
};

use squire_sdk::{
    accounts::{
        CreateAccountRequest, CreateAccountResponse, GetAllUsersResponse, GetOrgResponse,
        GetUserPermissionsResponse, GetUserResponse, UpdateOrgAccountRequest,
        UpdateOrgAccountResponse, UpdateSquireAccountRequest, UpdateSquireAccountResponse,
    },
    Action,
};

pub static USERS_MAP: OnceCell<DashMap<SquireAccountId, SquireAccount>> = OnceCell::new();
pub static ORGS_MAP: OnceCell<DashMap<OrgId, OrganizationAccount>> = OnceCell::new();

pub async fn register(Json(data): Json<CreateAccountRequest>) -> CreateAccountResponse {
    let account = SquireAccount::new(data.user_name, data.display_name);
    USERS_MAP.get().unwrap().insert(account.id, account.clone());
    CreateAccountResponse::new(account)
}

pub async fn login(Json(data): Json<LoginRequest>) -> LoginResponse {
    todo!()
}

pub async fn logout(Json(data): Json<CreateAccountRequest>) -> CreateAccountResponse {
    todo!()
}
