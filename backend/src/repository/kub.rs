use common::model::user::{User, VmsVersion};
use k8s_openapi::api::core::v1::Namespace;
use kube::{
    api::{Api, DeleteParams, ListParams, Patch, PatchParams, PostParams, ResourceExt},
    core::{ObjectMeta, PartialObjectMetaExt},
    Client,
};

use std::str::FromStr;

#[derive(Clone)]
pub struct KubeRepository {
    client: Client,
}

pub struct KubeError;
impl KubeRepository {
    pub async fn init() -> std::io::Result<KubeRepository> {
        let client = Client::try_default().await.unwrap();
        Ok(KubeRepository { client })
    }

    pub async fn create_user(&self, user: User) -> Result<User, KubeError> {
        let client = self.client.clone();

        let namespaces: Api<Namespace> = Api::all(client);
        let namespace = Namespace {
            metadata: ObjectMeta {
                name: Some(user.get_global_id()),
                labels: Some(std::collections::BTreeMap::from_iter(vec![
                    (
                        "vms.myenv.cloud/user-namespace".to_string(),
                        "true".to_string(),
                    ),
                    (
                        "vms.myenv.cloud/vms-version".to_string(),
                        user.vms_version.to_string(),
                    ),
                    (
                        "vms.myenv.cloud/server-type".to_string(),
                        user.server_type.clone(),
                    ),
                ])),
                ..Default::default()
            },
            spec: None,
            status: None,
        };

        match namespaces.create(&PostParams::default(), &namespace).await {
            Ok(_) => Ok(user),
            Err(_) => Err(KubeError),
        }
    }

    pub async fn get_users(&self) -> Option<Vec<User>> {
        let mut result: Vec<User> = Vec::new();
        let client = self.client.clone();

        let namespaces: Api<Namespace> = Api::all(client);
        let list_param = ListParams {
            label_selector: Some("vms.myenv.cloud/user-namespace=true".to_owned()),
            ..Default::default()
        };
        for n in namespaces.list(&list_param).await.unwrap() {
            let user_name = n.name_any();
            //let (user_uuid, user_name) = name_any.split_once('_').unwrap_or_default();

            let (_, vms_version) = n
                .labels()
                .get_key_value("vms.myenv.cloud/vms-version")
                .unwrap();
            let (_, server_type) = n
                .labels()
                .get_key_value("vms.myenv.cloud/server-type")
                .unwrap();

            result.push(User {
                user_name: user_name.to_owned(),
                vms_version: VmsVersion::from_str(vms_version).unwrap(),
                server_type: server_type.to_owned(),
            });
        }
        if !result.is_empty() {
            Some(result)
        } else {
            None
        }
    }

    pub async fn upgrade_user(&self, user_global_id: String) -> Option<User> {
        let client = self.client.clone();

        let namespaces: Api<Namespace> = Api::all(client);

        let patch = ObjectMeta {
            labels: Some(
                [(
                    "vms.myenv.cloud/vms-version".to_string(),
                    "XProtect2024R1".to_string(),
                )]
                .into(),
            ),

            ..Default::default()
        }
        .into_request_partial::<Namespace>();

        match namespaces
            .patch_metadata(
                &user_global_id,
                &PatchParams {
                    force: true,
                    field_manager: Some("application/apply-patch".to_string()),
                    ..Default::default()
                },
                &Patch::Apply(patch),
            )
            .await
        {
            Ok(p) => {
                let n = p.clone();
                let user_name = n.name_any();

                let namespace = n.clone();
                let (_, server_type) = namespace
                    .labels()
                    .get_key_value("vms.myenv.cloud/server-type")
                    .unwrap();

                if let Some(x) = n
                    .clone()
                    .labels_mut()
                    .get_mut("vms.myenv.cloud/vms-version")
                {
                    *x = VmsVersion::XProtect2024R1.to_string();
                }

                Some(User {
                    user_name,
                    vms_version: VmsVersion::XProtect2024R1,
                    server_type: server_type.to_string(),
                })
            }
            Err(_) => None,
        }
    }

    pub async fn delete_user(&self, user_global_id: String) -> Option<User> {
        let client = self.client.clone();

        let namespaces: Api<Namespace> = Api::all(client);
        let del_param = DeleteParams {
            grace_period_seconds: Some(0),
            ..Default::default()
        };
        match namespaces.delete(&user_global_id, &del_param).await {
            Ok(e) => {
                let n = e.left().unwrap();
                let user_name = n.name_any();

                let (_, vms_version) = n
                    .labels()
                    .get_key_value("vms.myenv.cloud/vms-version")
                    .unwrap();
                let (_, server_type) = n
                    .labels()
                    .get_key_value("vms.myenv.cloud/server-type")
                    .unwrap();

                Some(User {
                    user_name: user_name.to_owned(),
                    vms_version: VmsVersion::from_str(vms_version).unwrap(),
                    server_type: server_type.to_owned(),
                })
            }
            Error => None,
        }
    }

    pub async fn get_user(&self, user_global_id: String) -> Option<User> {
        let result = self.get_users().await;
        match result {
            Some(users) => users
                .into_iter()
                .find(|user| user.user_name == user_global_id.to_lowercase()),
            None => None,
        }
    }
}
