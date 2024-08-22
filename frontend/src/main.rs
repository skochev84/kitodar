use std::{ rc::Rc, time::Duration};
use yew_hooks::prelude::*;

use common::model::user::{User, VmsVersion};
use patternfly_yew::prelude::*;
use reqwasm::http::Request;
use serde::Serialize;
use yew::{html::ChildrenRenderer, prelude::*};

async fn get_users() -> Vec<User> {
    let url = "/api/user".to_string();
    Request::get(&url)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}

#[derive(Serialize)]
pub struct CreateUserRequest {
    user_name: String,
    vms_version: String,
}
async fn create_user(new_user: CreateUserRequest) -> Vec<User> {
    let url = "/api/user".to_string();
    let body = serde_json::to_string(&new_user).unwrap();
    Request::post(&url)
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}
async fn delete_user(user: &str) -> User {
    let url = format!("/api/user/{user}");
    Request::delete(&url)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}
async fn upgrade_user(user: &str) -> User {
    let url = format!("/api/user/{user}");
    Request::patch(&url)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}

#[derive(Properties, PartialEq)]
pub struct UserProps {
    pub reload: i32,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum ColumnsUserList {
    Name,
    Version,
    Link,
}

impl TableEntryRenderer<ColumnsUserList> for User {
    fn render_cell(&self, ctx: CellContext<ColumnsUserList>) -> Cell {
        match ctx.column {
            ColumnsUserList::Name => html!({ &self.user_name }),
            ColumnsUserList::Version => {
                if self.vms_version == VmsVersion::XProtect2024R1{

                    html!(<b>{ self.vms_version.to_string() }</b>)
                }else{

                    html!({ self.vms_version.to_string() })
                }
            },
            ColumnsUserList::Link => {
                let mut upgrade = html!();

                if self.vms_version != VmsVersion::XProtect2024R1{
                    upgrade = html!(<ContextProvider<String> context={self.user_name.to_string().clone()}>
                                        <Upgrade />
                                    </ContextProvider<String>>);
                }
                
                html!(
                <>
                <ContextProvider<String> context={self.user_name.to_string().clone()}>
                    <Trash />
                </ContextProvider<String>>
                <a href={format!("http://{0}.platform.myenv.cloud", self.user_name)} style="margin-right:10px">{Icon::ExternalLinkAlt.with_classes(classes!("pf-v5-u-ml-sm", "pf-v5-u-color-200"))}</a> 
                {upgrade}
                </>
             )},
        }
        .into()
    }
}

#[function_component(Trash)]
fn trash_button() -> Html {
    let user_name = use_context::<String>().expect("no ctx found");

    let events: yew::UseStateHandle<Option<User>> = use_state_eq(|| None);
    
    let onclick = {
        let user_name = user_name.clone();
        let events = events.clone();
        Callback::from(move |_| {
            let events = events.clone();
            let user_name = user_name.clone();
            wasm_bindgen_futures::spawn_local(async move {
                    let deleted_user = delete_user(&user_name)
                    .await;
                events.set(Some(deleted_user));
            });})
    };

    if let Some(_) = &*events {
        html!(<Button disabled=true variant={ButtonVariant::Plain}> <Spinner size={SpinnerSize::Md} /></Button>)
    }
    else {        
            html!(<Button onclick={onclick} variant={ButtonVariant::Plain} icon={Icon::Trash} />)
        }
}

#[function_component(Upgrade)]
fn upgrade_button() -> Html {
    let user_name = use_context::<String>().expect("no ctx found");

    let events: yew::UseStateHandle<Option<User>> = use_state_eq(|| None);
    
    let onclick = {
        let user_name = user_name.clone();
        let events = events.clone();
        Callback::from(move |_| {
            let events = events.clone();
            let user_name = user_name.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let upgraded_user = upgrade_user(&user_name)
                    .await;
                events.set(Some(upgraded_user));
            });})
    };

    if let Some(events) = &*events {
        html!(<Button disabled=true variant={ButtonVariant::Plain}> <Spinner size={SpinnerSize::Md} /></Button>)
    }
    else {        
            html!(<Button onclick={onclick} variant={ButtonVariant::Plain} icon={Icon::ArrowCircleUp} />)
        }
}

#[function_component(UsersView)]
fn users_view(props: &UserProps) -> Html {
    let reload = props.reload;
    let users = use_state(|| None);
    let users_old = use_state(|| None);
    {
        let users = users.clone();
        let users_old = users_old.clone();
        use_effect_with(reload, move |_| {
            let users = users.clone();
            let users_old = users_old.clone();
            wasm_bindgen_futures::spawn_local(async move {
                users.set(None);
                let fetched_user = get_users().await;
                users.set(Some(fetched_user.clone()));
                users_old.set(Some(fetched_user));
            });
            || ()
        });
    }

    if let Some(users) = &*users {
        if let Some(users_old) = &*users_old {
            html! {
                <ContextProvider<Vec<User>> context={users_old.clone()}>
                    <UserGrid />
                </ContextProvider<Vec<User>>>
            }
        } else {
            html! {
                <ContextProvider<Vec<User>> context={users.clone()}>
                    <UserGrid />
                </ContextProvider<Vec<User>>>
            }
        }
    } else if let Some(users_old) = &*users_old {
        html! {<>
            <ContextProvider<Vec<User>> context={users_old.clone()}>
                <UserGrid />
            </ContextProvider<Vec<User>>>
        </>
        }
    } else {
        html! { <div>{"Loading..."}</div> }
    }
}

#[function_component(UserGrid)]
pub fn user_grid() -> Html {
    let entities = use_context::<Vec<User>>().expect("no ctx found");

    let entries: Rc<Vec<User>> = use_memo((), |()| entities);
    let (entries, _) = use_table_data(MemoizedTableModel::new(entries));

    let header = html_nested! {
        <TableHeader<ColumnsUserList>>
            <TableColumn<ColumnsUserList> label="User Name" index={ColumnsUserList::Name}/>
            <TableColumn<ColumnsUserList> label="VMS Version" index={ColumnsUserList::Version} />
            <TableColumn<ColumnsUserList> label=" " index={ColumnsUserList::Link}/>
        </TableHeader<ColumnsUserList>>
    };

    html! {
            <Table<ColumnsUserList, UseTableData<ColumnsUserList, MemoizedTableModel<User>>>
                    mode={TableMode::Compact}
                    header={header}
                    entries={entries}
                    />
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<Application>::new().render();
}

#[function_component(Application)]
pub fn app() -> Html {
    html! {
        <BackdropViewer>
            <ToastViewer>
                <AdminPanel />
            </ToastViewer>
        </BackdropViewer>
    }
}
#[function_component(AdminPanel)]
pub fn login_page_example() -> Html {
    let header = html! {<> {"Header" }</>};
    let footer = html! {<p>{"Some footer text"}</p>};

    let links = ChildrenRenderer::new(vec![
        html_nested! {<LoginMainFooterLink href="#" target="_blank">{"Footer Link #1"}</LoginMainFooterLink>},
        html_nested! {<LoginMainFooterLink href="#" target="_blank">{"Footer Link #2"}</LoginMainFooterLink>},
    ]);

    let title = html_nested! {<Title size={Size::XXLarge}>{"KITODAR admin panel"}</Title>};
    let toaster = use_toaster();

    let reload = use_state_eq(|| 0);
    let username = use_state_eq(String::new);
    let vms_version = use_state_eq(String::new);

    let onchangeusername = {
        let username = username.clone();
        Callback::from(move |value| {
            username.set(value);
        })
    };

    let selected = use_state_eq(|| None);
    let onselect = use_callback(
        (vms_version.clone(), selected.clone()),
        |item: VmsVersion, selected| {
            selected.0.set(item.clone().to_string());
            selected.1.set(Some(item));
        },
    );

    let onsubmit = {
        let toaster = toaster.clone();
        let reload = reload.clone();
        let username = username.clone();
        let vms_version = vms_version.clone();
        let selected = selected.clone();
        Callback::from(move |_| {
            let reload = reload.clone();
            {
                let username = username.clone();
                let vms_version = vms_version.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let fetched_user = create_user(CreateUserRequest {
                        user_name: username.to_string(),
                        vms_version: vms_version.to_string(),
                    })
                    .await;
                });
            }
            if let Some(toaster) = &toaster {
                if !(*username).is_empty() && !(*vms_version).is_empty(){
                    toaster.toast(Toast {
                        title: format!(
                            "Created - Username: {}, XProtect Version: {}",
                            &*username, &*vms_version
                        ),
                        timeout: Some(Duration::from_secs(5)),
                        ..Default::default()
                    });
                }
            }
            username.set("".to_string());
            vms_version.set("".to_string());
            selected.set(None);
            reload.set(*reload + 1);
        })
    };

    let band = ChildrenRenderer::new(vec![
        html! {<UsersView reload={*reload}/>},
        //html! {<a href="#">{"Some link"}</a>},
        //html! {<>{"Some other"}<a href="#">{" link"}</a></>},
    ]);
        {
        let reload = reload.clone();
        use_interval(move || {
            reload.set(*reload + 1);
        }, 15000);
    }
    html! {
        <>
                <Background/>
                <Login
                   // {header}
                  //  {footer}
                >
                            <img alt="Kitodar and friends" src="img/kitodar-and-friends.png" style="margin-left: auto; margin-right: auto; " title="Kitodar and friends"/>
                            <div style="color: white">
                                {"Authors: "}
                                <List>
                                {"Boris Naskov Danailov - BOD@milestone.dk"}
                                {"Stefan Nikolaev Kochev - SKO@milestone.dk"}
                                </List>
                            </div>
                    <LoginMain>
                        <LoginMainHeader
                            {title}
                            description="Create a new user right here."
                        />
                        <LoginMainBody>
                            <Form {onsubmit} method="dialog">
                                <FormGroup label="Username">
                                    <TextInput required=true name="username" onchange={onchangeusername} value={(*username).clone()} />
                                </FormGroup>
                                <FormGroup label="XProtect Version">
                                <SimpleSelect<VmsVersion>
                                    placeholder="Pick a VMS Version"
                                    selected={*selected}
                                    entries={vec![VmsVersion::XProtect2023R1, VmsVersion::XProtect2023R2, VmsVersion::XProtect2023R3, VmsVersion::XProtect2024R1]}
                                    {onselect}
                                />
                                </FormGroup>
                                <ActionGroup>
                                    <Button label="Create!" r#type={ButtonType::Submit} variant={ButtonVariant::Primary}/>
                                </ActionGroup>
                            </Form>
                        </LoginMainBody>
                        <LoginMainFooter
                        //    {links}
                            {band}
                        >
                        </LoginMainFooter>
                    </LoginMain>
                </Login>
        </>
    }
}
