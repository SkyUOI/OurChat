use base::consts::ID;
use client::TestApp;
use migration::m20241229_022701_add_role_for_session::{PreDefinedPermissions, PreDefinedRoles};
use pb::service::ourchat::session::{add_role::v1::AddRoleRequest, set_role::v1::SetRoleRequest};
use sea_orm::EntityTrait;
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
            role_id: PreDefinedRoles::Owner.into(),
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
            role_id: PreDefinedRoles::Owner.into(),
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
    assert_eq!(relation.role_id, PreDefinedRoles::Owner as i64);
    app.async_drop().await;
}

#[tokio::test]
async fn add_role() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let user = app.new_user().await.unwrap();
    let ret = user
        .lock()
        .await
        .oc()
        .add_role(AddRoleRequest {
            description: "test add description".to_owned(),
            permissions: vec![
                PreDefinedPermissions::BanUser.into(),
                PreDefinedPermissions::SetAvatar.into(),
                PreDefinedRoles::Owner.into(),
            ],
        })
        .await
        .unwrap()
        .into_inner();
    // check in the database
    let model = entities::role::Entity::find_by_id(ret.role_id as i64)
        .one(app.get_db_connection())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(model.description, "test add description");
    let creator_id: ID = model.creator_id.unwrap().into();
    assert_eq!(creator_id, user.lock().await.id);
    app.async_drop().await;
}
