use crate::{
    components::{
        delete_user::DeleteUser,
        router::{AppRoute, Link},
    },
    infra::common_component::{CommonComponent, CommonComponentParts},
};
use anyhow::{Error, Result};
use graphql_client::GraphQLQuery;
use yew::prelude::*;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "../schema.graphql",
    query_path = "queries/list_users.graphql",
    response_derives = "Debug",
    custom_scalars_module = "crate::infra::graphql"
)]
pub struct ListUsersQuery;

use list_users_query::{RequestFilter, ResponseData};

type User = list_users_query::ListUsersQueryUsers;

pub struct UserTable {
    common: CommonComponentParts<Self>,
    users: Option<Vec<User>>,
}

pub enum Msg {
    ListUsersResponse(Result<ResponseData>),
    OnUserDeleted(String),
    OnError(Error),
}

impl CommonComponent<UserTable> for UserTable {
    fn handle_msg(&mut self, msg: <Self as Component>::Message) -> Result<bool> {
        match msg {
            Msg::ListUsersResponse(users) => {
                self.users = Some(users?.users.into_iter().collect());
                Ok(true)
            }
            Msg::OnError(e) => Err(e),
            Msg::OnUserDeleted(user_id) => {
                debug_assert!(self.users.is_some());
                self.users.as_mut().unwrap().retain(|u| u.id != user_id);
                Ok(true)
            }
        }
    }

    fn mut_common(&mut self) -> &mut CommonComponentParts<Self> {
        &mut self.common
    }
}

impl UserTable {
    fn get_users(&mut self, req: Option<RequestFilter>) {
        self.common.call_graphql::<ListUsersQuery, _>(
            list_users_query::Variables { filters: req },
            Msg::ListUsersResponse,
            "Error trying to fetch users",
        );
    }
}

impl Component for UserTable {
    type Message = Msg;
    type Properties = ();

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut table = UserTable {
            common: CommonComponentParts::<Self>::create(props, link),
            users: None,
        };
        table.get_users(None);
        table
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        CommonComponentParts::<Self>::update(self, msg)
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <div>
              {self.view_users()}
              {self.view_errors()}
            </div>
        }
    }
}

impl UserTable {
    fn view_users(&self) -> Html {
        let make_table = |users: &Vec<User>| {
            html! {
                <div class="table-responsive">
                  <table class="table table-striped">
                    <thead>
                      <tr>
                        <th>{"User ID"}</th>
                        <th>{"Email"}</th>
                        <th>{"Display name"}</th>
                        <th>{"First name"}</th>
                        <th>{"Last name"}</th>
                        <th>{"Creation date"}</th>
                        <th>{"Delete"}</th>
                      </tr>
                    </thead>
                    <tbody>
                      {users.iter().map(|u| self.view_user(u)).collect::<Vec<_>>()}
                    </tbody>
                  </table>
                </div>
            }
        };
        match &self.users {
            None => html! {{"Loading..."}},
            Some(users) => make_table(users),
        }
    }

    fn view_user(&self, user: &User) -> Html {
        html! {
          <tr key=user.id.clone()>
              <td><Link route=AppRoute::UserDetails(user.id.clone())>{&user.id}</Link></td>
              <td>{&user.email}</td>
              <td>{&user.display_name}</td>
              <td>{&user.first_name}</td>
              <td>{&user.last_name}</td>
              <td>{&user.creation_date.date().naive_local()}</td>
              <td>
                <DeleteUser
                  username=user.id.clone()
                  on_user_deleted=self.common.callback(Msg::OnUserDeleted)
                  on_error=self.common.callback(Msg::OnError)/>
              </td>
          </tr>
        }
    }

    fn view_errors(&self) -> Html {
        match &self.common.error {
            None => html! {},
            Some(e) => html! {<div>{"Error: "}{e.to_string()}</div>},
        }
    }
}
