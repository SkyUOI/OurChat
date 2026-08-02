//! Authorization tests for the ServerManageService hardening (C-1).
//!
//! Every ServerManageService handler must require an appropriate server-
//! management permission. These tests verify that a *regular* (non-admin)
//! authenticated user is rejected with `PermissionDenied` for each previously
//! unguarded handler, and that an admin succeeds where the operation is
//! observable. This guards against the vertical-privilege-escalation regression
//! where a handler forgets to call `check_server_manage_permission`.

use client::TestApp;
use pb::google::protobuf::Timestamp as ProtoTimestamp;
use pb::service::ourchat::msg_delivery::announcement::v1::Announcement;
use pb::service::server_manage::{
    config::v1::GetConfigRequest,
    delete_account::v1::DeleteAccountRequest,
    monitoring::v1::{GetHistoricalMetricsRequest, GetMonitoringMetricsRequest},
    publish_announcement::v1::PublishAnnouncementRequest,
    set_server_status::v1::{ServerStatus, SetServerStatusRequest},
    user_manage::v1::{
        ListServerRolePermissionsRequest, ListServerRolesRequest, ListUserServerRolesRequest,
        RemoveServerRoleRequest,
    },
};
use tonic::{Code, Request};

/// Helper: assert a future yields a `PermissionDenied` gRPC status.
async fn assert_permission_denied<F, T>(f: F)
where
    F: std::future::Future<Output = Result<T, tonic::Status>>,
    T: std::fmt::Debug,
{
    let err = f.await.expect_err("expected PermissionDenied, got Ok");
    assert_eq!(err.code(), Code::PermissionDenied, "wrong error: {err:?}");
}

#[tokio::test]
async fn regular_user_is_denied_on_all_hardened_handlers() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();

    // A regular, non-admin user that will attempt every privileged call.
    let regular = app.new_user().await.unwrap();
    // A second user that serves as a target id (e.g. for delete/remove role).
    let target = app.new_user().await.unwrap();
    let target_id = target.lock().await.id;

    // delete_account (was unguarded -> any user could delete any account)
    {
        let regular = regular.clone();
        assert_permission_denied(async move {
            regular
                .lock()
                .await
                .server_manage()
                .delete_account(Request::new(DeleteAccountRequest {
                    user_id: target_id.into(),
                }))
                .await
        })
        .await;
    }

    // set_server_status (was unguarded -> global DoS via maintenance mode)
    {
        let regular = regular.clone();
        assert_permission_denied(async move {
            regular
                .lock()
                .await
                .server_manage()
                .set_server_status(Request::new(SetServerStatusRequest {
                    server_status: ServerStatus::Maintaining as i32,
                    reason: "evil".to_string(),
                }))
                .await
        })
        .await;
        // Sanity: server was NOT flipped into maintenance.
        assert!(!app.app_shared.get_maintaining());
    }

    // remove_server_role (was unguarded -> could strip admin rights)
    {
        let regular = regular.clone();
        assert_permission_denied(async move {
            regular
                .lock()
                .await
                .server_manage()
                .remove_server_role(Request::new(RemoveServerRoleRequest {
                    user_id: target_id.into(),
                    role_id: 1, // Admin role id
                }))
                .await
        })
        .await;
    }

    // publish_announcement (was unguarded -> broadcast to all users)
    {
        let regular = regular.clone();
        let announcement = Announcement {
            title: "x".to_string(),
            content: "x".to_string(),
            publisher_id: regular.lock().await.id.into(),
        };
        assert_permission_denied(async move {
            regular
                .lock()
                .await
                .server_manage()
                .publish_announcement(Request::new(PublishAnnouncementRequest {
                    announcement: Some(announcement),
                }))
                .await
        })
        .await;
    }

    // list_server_roles
    {
        let regular = regular.clone();
        assert_permission_denied(async move {
            regular
                .lock()
                .await
                .server_manage()
                .list_server_roles(Request::new(ListServerRolesRequest {}))
                .await
        })
        .await;
    }

    // list_user_server_roles
    {
        let regular = regular.clone();
        assert_permission_denied(async move {
            regular
                .lock()
                .await
                .server_manage()
                .list_user_server_roles(Request::new(ListUserServerRolesRequest {
                    user_id: target_id.into(),
                }))
                .await
        })
        .await;
    }

    // list_server_role_permissions
    {
        let regular = regular.clone();
        assert_permission_denied(async move {
            regular
                .lock()
                .await
                .server_manage()
                .list_server_role_permissions(Request::new(ListServerRolePermissionsRequest {
                    role_id: 1,
                }))
                .await
        })
        .await;
    }

    // get_monitoring_metrics
    {
        let regular = regular.clone();
        assert_permission_denied(async move {
            regular
                .lock()
                .await
                .server_manage()
                .get_monitoring_metrics(Request::new(GetMonitoringMetricsRequest {
                    include_system_metrics: false,
                    include_tokio_metrics: false,
                }))
                .await
        })
        .await;
    }

    // get_historical_metrics
    {
        let regular = regular.clone();
        let now = chrono::Utc::now().timestamp();
        assert_permission_denied(async move {
            regular
                .lock()
                .await
                .server_manage()
                .get_historical_metrics(Request::new(GetHistoricalMetricsRequest {
                    start_time: Some(ProtoTimestamp {
                        seconds: now - 60,
                        nanos: 0,
                    }),
                    end_time: Some(ProtoTimestamp {
                        seconds: now,
                        nanos: 0,
                    }),
                    interval: None,
                }))
                .await
        })
        .await;
    }

    // get_config (was guarded already, but verify here too as part of the surface)
    {
        let regular = regular.clone();
        assert_permission_denied(async move {
            regular
                .lock()
                .await
                .server_manage()
                .get_config(Request::new(GetConfigRequest {}))
                .await
        })
        .await;
    }

    app.async_drop().await;
}

#[tokio::test]
async fn admin_can_list_roles_and_metrics() {
    let mut app = TestApp::new_with_launching_instance().await.unwrap();
    let admin = app.new_user().await.unwrap();
    admin
        .lock()
        .await
        .promote_to_admin(app.get_db_connection())
        .await
        .unwrap();

    // Admin can list server roles.
    let resp = admin
        .lock()
        .await
        .server_manage()
        .list_server_roles(Request::new(ListServerRolesRequest {}))
        .await
        .expect("admin should be able to list server roles");
    // The predefined Admin role (id=1) should be present.
    let roles = resp.into_inner().roles;
    assert!(
        roles.iter().any(|r| r.id == 1),
        "predefined admin role should exist: {roles:?}"
    );

    // Admin can list a role's permissions.
    let _perms = admin
        .lock()
        .await
        .server_manage()
        .list_server_role_permissions(Request::new(ListServerRolePermissionsRequest {
            role_id: 1,
        }))
        .await
        .expect("admin should be able to list role permissions");

    // (Monitoring-metrics authorization is covered separately by metrics.rs,
    // which runs with metrics enabled. The default test config disables them.)

    app.async_drop().await;
}
