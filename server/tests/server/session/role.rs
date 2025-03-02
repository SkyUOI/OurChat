use base::consts::ID;
use client::TestApp;
use migration::m20241229_022701_add_role_for_session::{PredefinedPermissions, PredefinedRoles};
use pb::service::ourchat::session::{add_role::v1::AddRoleRequest, set_role::v1::SetRoleRequest};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use server::process::error_msg::PERMISSION_DENIED;

#[tokio::test]
async fn set_role() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let (session_user, session) = app.new_session_db_level(3, "session1").await.unwrap();
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
    let user = app.new_user().await.unwrap();

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
        })
        .await
        .unwrap()
        .into_inner();

    // Check in the database
    let model = entities::role::Entity::find_by_id(ret.role_id)
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
    let db_permissions: Vec<i64> = role_permissions.iter().map(|p| p.permission_id).collect();

    for permission in permissions {
        assert!(db_permissions.contains(&permission));
    }

    app.async_drop().await;
}
