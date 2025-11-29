use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
pub enum User {
    Table,
    Id,
    Ocid,
    Passwd,
    Name,
    Email,
    Time,
    ResourceUsed,
    FriendLimit,
    FriendsNum,
    PublicUpdateTime,
    UpdateTime,
    Avatar,
    Status,
    AccountStatus,
    DeletedAt,
    PublicKey,
    GithubId,
    OauthProvider,
    EmailVerified,
}

#[derive(DeriveIden)]
pub enum Friend {
    Table,
    UserId,
    #[allow(clippy::enum_variant_names)]
    FriendId,
    SessionId,
}

#[derive(DeriveIden)]
pub enum SessionRelation {
    Table,
    SessionId,
    UserId,
    DisplayName,
}

#[derive(DeriveIden)]
pub enum Session {
    Table,
    #[allow(clippy::enum_variant_names)]
    SessionId,
    Name,
    Size,
    AvatarKey,
    CreatedTime,
    UpdatedTime,
    Description,
    DefaultRole,
    E2EEOn,
    RoomKeyTime,
    LeavingToProcess,
}

#[derive(DeriveIden)]
pub enum UserChatMsg {
    Table,
    ChatMsgId,
    MsgData,
    SenderId,
    SessionId,
    Time,
    IsEncrypted,
    MsgId,
}

#[derive(DeriveIden)]
pub enum Files {
    Table,
    Key,
    Date,
    AutoClean,
    Path,
    UserId,
}

#[derive(DeriveIden)]
pub enum UserStatus {
    Table,
    Name,
}

#[derive(DeriveIden)]
pub enum UserContactInfo {
    Table,
    UserId,
    ContactUserId,
    DisplayName,
}

#[derive(DeriveIden)]
pub enum ServerManagementPermission {
    Table,
    Id,
    Name,
    Description,
}

#[derive(DeriveIden)]
pub enum ServerManagementRole {
    Table,
    Id,
    Name,
    Description,
}

#[derive(DeriveIden)]
pub enum ServerManagementRolePermissions {
    Table,
    RoleId,
    PermissionId,
}

#[derive(DeriveIden)]
pub enum Role {
    Table,
    Id,
    CreatorId,
    Name,
    Description,
    SessionId,
}

#[derive(DeriveIden)]
pub enum Permission {
    Table,
    Id,
    Description,
}

#[derive(DeriveIden)]
pub enum RolePermissions {
    Table,
    RoleId,
    PermissionId,
}

#[derive(DeriveIden)]
pub enum UserRoleRelation {
    Table,
    UserId,
    SessionId,
    RoleId,
}

#[derive(DeriveIden)]
pub enum ManagerRoleRelation {
    Table,
    UserId,
    RoleId,
}

#[derive(DeriveIden)]
pub enum Announcement {
    Table,
    Id,
    Title,
    Content,
    CreatedAt,
    PublisherId,
}

#[derive(DeriveIden)]
pub enum AnnouncementMsg {
    Table,
    AnnouncementId,
    MsgId,
}

#[derive(DeriveIden)]
pub enum WebrtcRoom {
    Table,
    Id,
    Title,
    Description,
}

#[derive(DeriveIden)]
pub enum WebrtcRoomMember {
    Table,
    RoomId,
    UserId,
    Name,
}

#[derive(DeriveIden)]
pub enum RTCRoom {
    Table,
    RoomId,
    Title,
    UsersNum,
}

#[derive(DeriveIden)]
pub enum SessionInvitation {
    Table,
    Id,
    Inviter,
    Invitee,
    SessionId,
    LeaveMessage,
    ExpireAt,
}

#[derive(DeriveIden)]
pub enum MessageRecords {
    Table,
    MsgId,
    MsgData,
    SenderId,
    SessionId,
    Time,
    IsEncrypted,
    IsAllUser,
}
