use base::{constants::ID, types::RoleId};
use client::TestApp;
use migration::predefined::{PredefinedPermissions, PredefinedRoles};
use pb::service::ourchat::session::{
    add_role::v1::AddRoleRequest, get_role::v1::GetRoleRequest, set_role::v1::SetRoleRequest,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use server::process::error_msg::{self, PERMISSION_DENIED, not_found};

#[tokio::test]
async fn set_role() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let (session_user, session) = app
        .new_session_db_level(3, "session1", false)
        .await
        .unwrap();
    let (a, b, c) = (
        session_user[0].clone(),
        session_user[1].clone(),
        session_user[2].clone(),
    );
    let ret = b
        .lock()
        .await
        .oc()
        .set_role(SetRoleRequest {
            session_id: session.session_id.into(),
            role_id: PredefinedRoles::Owner.into(),
            member_id: c.lock().await.id.into(),
        })
        .await
        .unwrap_err();
    assert_eq!(ret.code(), tonic::Code::PermissionDenied);
    assert_eq!(ret.message(), PERMISSION_DENIED);
    a.lock()
        .await
        .oc()
        .set_role(SetRoleRequest {
            session_id: session.session_id.into(),
            role_id: PredefinedRoles::Owner.into(),
            member_id: c.lock().await.id.into(),
        })
        .await
        .unwrap();
    let relation = entities::user_role_relation::Entity::find_by_id((
        session.session_id.into(),
        c.lock().await.id.into(),
    ))
    .one(app.get_db_connection())
    .await
    .unwrap()
    .unwrap();
    assert_eq!(relation.role_id, PredefinedRoles::Owner as i64);
    app.async_drop().await;
}

#[tokio::test]
async fn add_role() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let (users, session) = app.new_session_db_level(2, "test", false).await.unwrap();
    let user = users[0].clone();

    let permissions = vec![
        PredefinedPermissions::BanUser.into(),
        PredefinedPermissions::SetAvatar.into(),
        PredefinedPermissions::SetTitle.into(),
    ];

    let ret = user
        .lock()
        .await
        .oc()
        .add_role(AddRoleRequest {
            description: Some("test add description".to_owned()),
            name: "test add name".to_owned(),
            permissions: permissions.clone(),
            session_id: session.session_id.into(),
        })
        .await
        .unwrap()
        .into_inner();

    let role_id = RoleId(ret.role_id);
    // Check in the database
    let model = entities::role::Entity::find_by_id(ret.role_id as i64)
        .one(app.get_db_connection())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(model.description, Some("test add description".to_owned()));
    assert_eq!(model.name, "test add name");
    let creator_id: ID = model.creator_id.unwrap().into();
    assert_eq!(creator_id, user.lock().await.id);

    // Check permissions in the database
    let role_permissions = entities::role_permissions::Entity::find()
        .filter(entities::role_permissions::Column::RoleId.eq(ret.role_id))
        .all(app.get_db_connection())
        .await
        .unwrap();

    assert_eq!(role_permissions.len(), permissions.len());
    let db_permissions: Vec<_> = role_permissions
        .iter()
        .map(|p| p.permission_id as u64)
        .collect();

    for permission in permissions {
        assert!(db_permissions.contains(&permission));
    }

    // Use get role api to get
    let ret = user
        .lock()
        .await
        .oc()
        .get_role(GetRoleRequest { role_id: role_id.0 })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(role_permissions.len(), ret.permissions.len());
    for permission in ret.permissions {
        assert!(db_permissions.contains(&permission));
    }
    assert_eq!(ret.description, Some("test add description".to_owned()));
    assert_eq!(ret.name, "test add name");
    // get a wrong role
    let ret = user
        .lock()
        .await
        .oc()
        .get_role(GetRoleRequest { role_id: 114514 })
        .await
        .unwrap_err();
    assert_eq!(ret.code(), tonic::Code::NotFound);
    assert_eq!(ret.message(), not_found::ROLE);
    // without permission
    let new_user = app.new_user().await.unwrap();
    let ret = new_user
        .lock()
        .await
        .oc()
        .get_role(GetRoleRequest { role_id: role_id.0 })
        .await
        .unwrap_err();
    assert_eq!(ret.code(), tonic::Code::PermissionDenied);
    assert_eq!(ret.message(), error_msg::NOT_IN_SESSION);

    app.async_drop().await;
}
